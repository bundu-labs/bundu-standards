//! What answers a request: the static files, then the site, then the file endpoint.
//!
//! The order is the one the site has always had: a file of `public/` wins, then a page, the
//! JSON or the stylesheet, then the file endpoint ([`core::route`]), whose successful GETs are
//! kept in a small in-process cache for as long as their own `cache-control` says. Nothing
//! here knows about sockets, so it is tested natively with a fake GitHub.

use crate::core::{self, Fetch, Reply};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The most answers kept. A commit's files are immutable and `latest` is one key, so the
/// working set is small; the bound stops a stream of distinct commits from growing the heap.
const MAX_ENTRIES: usize = 512;
/// No answer is kept longer than a day, even an immutable one.
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// The file endpoint's successful answers, by [`core::cache_key`].
#[derive(Default)]
pub struct Cache {
    entries: Mutex<HashMap<String, (Instant, Reply)>>,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, key: &str) -> Option<Reply> {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        match entries.get(key) {
            Some((expires, reply)) if *expires > Instant::now() => Some(reply.clone()),
            Some(_) => {
                entries.remove(key);
                None
            }
            None => None,
        }
    }

    fn put(&self, key: String, reply: &Reply) {
        let Some(ttl) = max_age(reply).filter(|t| !t.is_zero()) else {
            return;
        };
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() >= MAX_ENTRIES {
            entries.retain(|_, (expires, _)| *expires > now);
        }
        if entries.len() >= MAX_ENTRIES {
            // Still full of live answers: drop the one that expires first.
            if let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, (expires, _))| *expires)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest);
            }
        }
        entries.insert(key, (now + ttl.min(MAX_AGE), reply.clone()));
    }
}

/// The `max-age` of a reply's `cache-control`, unless it must not be stored.
fn max_age(reply: &Reply) -> Option<Duration> {
    let value = reply.header("cache-control")?;
    let mut age = None;
    for part in value.split(',').map(|p| p.trim().to_ascii_lowercase()) {
        if part == "no-store" || part == "no-cache" || part == "private" {
            return None;
        }
        if let Some(n) = part.strip_prefix("max-age=") {
            age = n.parse::<u64>().ok().map(Duration::from_secs);
        }
    }
    age
}

/// Answer one request. `origin` is the public origin the answers name themselves by (the URLs
/// in `/register/latest`), `if_none_match` the request's validator for a static file.
pub async fn handle(
    method: &str,
    origin: &str,
    path: &str,
    if_none_match: Option<&str>,
    fetch: &impl Fetch,
    cache: &Cache,
) -> Reply {
    if let Some(reply) = crate::assets::reply(method, path, if_none_match) {
        return reply;
    }
    if let Some(reply) = crate::site::reply(method, path) {
        return reply;
    }
    let key = (method == "GET").then(|| core::cache_key(origin, path));
    if let Some(hit) = key.as_deref().and_then(|k| cache.get(k)) {
        return hit;
    }
    let reply = core::route(method, origin, path, fetch).await;
    if let Some(key) = key {
        if reply.status == 200 {
            cache.put(key, &reply);
        }
    }
    reply
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Upstream;
    use pollster::block_on;
    use std::cell::Cell;

    const SHA: &str = "e15535a4fbaab894167cc87644baa4168b5f8150";
    const ORIGIN: &str = "https://standards.bundu.org";

    struct Counting(Cell<u32>);
    impl Fetch for Counting {
        async fn get(&self, _url: &str, _accept: Option<&str>) -> Result<Upstream, ()> {
            self.0.set(self.0.get() + 1);
            Ok(Upstream {
                status: 200,
                body: b"hello".to_vec(),
            })
        }
    }

    fn get(path: &str, f: &Counting, c: &Cache) -> Reply {
        block_on(handle("GET", ORIGIN, path, None, f, c))
    }

    #[test]
    fn files_then_site_then_endpoint() {
        let f = Counting(Cell::new(0));
        let c = Cache::new();
        assert_eq!(get("/robots.txt", &f, &c).status, 200);
        assert_eq!(get("/", &f, &c).status, 200);
        assert_eq!(get("/healthz", &f, &c).status, 200);
        assert_eq!(get("/nope", &f, &c).status, 404);
        assert_eq!(f.0.get(), 0, "none of those reach GitHub");
        let r = block_on(handle("POST", ORIGIN, "/", None, &f, &c));
        assert_eq!(r.status, 405);
    }

    #[test]
    fn a_commit_file_is_fetched_once_and_then_served_from_memory() {
        let f = Counting(Cell::new(0));
        let c = Cache::new();
        let path = format!("/{SHA}/register/register.yaml");
        let first = get(&path, &f, &c);
        let second = get(&path, &f, &c);
        assert_eq!(f.0.get(), 1);
        assert_eq!(first.body, second.body);
        assert_eq!(
            second.header("cache-control"),
            Some("public, max-age=31536000, immutable")
        );
        // HEAD is never cached, as before.
        block_on(handle("HEAD", ORIGIN, &path, None, &f, &c));
        assert_eq!(f.0.get(), 2);
    }

    #[test]
    fn errors_and_no_store_answers_are_not_kept() {
        let f = Counting(Cell::new(0));
        let c = Cache::new();
        get("/healthz", &f, &c);
        get(&format!("/{SHA}/register/other.txt"), &f, &c);
        assert!(c.entries.lock().unwrap().is_empty());
    }

    #[test]
    fn the_cache_is_bounded() {
        let c = Cache::new();
        let reply = Reply {
            status: 200,
            headers: vec![("cache-control", "public, max-age=60".to_owned())],
            body: None,
        };
        for i in 0..MAX_ENTRIES + 10 {
            c.put(format!("k{i}"), &reply);
        }
        assert_eq!(c.entries.lock().unwrap().len(), MAX_ENTRIES);
    }
}
