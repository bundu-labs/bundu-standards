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

/// The most bytes kept in all. A commit's files are immutable and `latest` is one key, so the
/// working set is a few hundred kilobytes; the bound stops a stream of distinct commits from
/// growing the heap, whatever the size of each answer.
const MAX_BYTES: usize = 32 * 1024 * 1024;
/// The largest single answer kept. The register's files are under a megabyte.
const MAX_ENTRY_BYTES: usize = 1024 * 1024;
/// What every entry costs besides its body (key, headers, bookkeeping), so many tiny entries
/// are bounded too.
const OVERHEAD: usize = 512;
/// No answer is kept longer than a day, even an immutable one.
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// A 404 is kept this long, so a stream of unknown commits does not reach GitHub each time.
const NEGATIVE_TTL: Duration = Duration::from_secs(60);

/// The file endpoint's answers, by [`core::cache_key`]: successes for as long as their own
/// `cache-control` says, and 404s briefly.
#[derive(Default)]
pub struct Cache {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    entries: HashMap<String, (Instant, usize, Reply)>,
    bytes: usize,
}

impl Inner {
    fn remove(&mut self, key: &str) {
        if let Some((_, size, _)) = self.entries.remove(key) {
            self.bytes -= size;
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn get(&self, key: &str) -> Option<Reply> {
        let mut inner = self.lock();
        match inner.entries.get(key) {
            Some((expires, _, reply)) if *expires > Instant::now() => Some(reply.clone()),
            Some(_) => {
                inner.remove(key);
                None
            }
            None => None,
        }
    }

    /// Keep a reply if it may be kept and is small enough, evicting the entries that expire
    /// soonest until it fits.
    fn put(&self, key: String, reply: &Reply) {
        let ttl = match reply.status {
            200 => match max_age(reply).filter(|t| !t.is_zero()) {
                Some(t) => t.min(MAX_AGE),
                None => return,
            },
            404 => NEGATIVE_TTL,
            _ => return,
        };
        let body = reply.body.as_ref().map_or(0, Vec::len);
        if body > MAX_ENTRY_BYTES {
            return;
        }
        let size = body + OVERHEAD + key.len();
        let now = Instant::now();
        let mut inner = self.lock();
        inner.remove(&key);
        if inner.bytes + size > MAX_BYTES {
            let expired: Vec<String> = inner
                .entries
                .iter()
                .filter(|(_, (expires, _, _))| *expires <= now)
                .map(|(k, _)| k.clone())
                .collect();
            for k in expired {
                inner.remove(&k);
            }
        }
        while inner.bytes + size > MAX_BYTES {
            let Some(soonest) = inner
                .entries
                .iter()
                .min_by_key(|(_, (expires, _, _))| *expires)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            inner.remove(&soonest);
        }
        inner.bytes += size;
        inner.entries.insert(key, (now + ttl, size, reply.clone()));
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
        cache.put(key, &reply);
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
    fn no_store_answers_are_not_kept_and_a_404_is_kept_briefly() {
        let f = Counting(Cell::new(0));
        let c = Cache::new();
        get("/healthz", &f, &c);
        assert!(c.lock().entries.is_empty());
        let missing = format!("/{SHA}/register/other.txt");
        get(&missing, &f, &c);
        assert_eq!(c.lock().entries.len(), 1, "the 404 is kept for a minute");
    }

    fn big(len: usize) -> Reply {
        Reply {
            status: 200,
            headers: vec![("cache-control", "public, max-age=60".to_owned())],
            body: Some(vec![0; len]),
        }
    }

    #[test]
    fn the_cache_is_bounded_in_bytes_and_evicts_to_fit() {
        let c = Cache::new();
        // Each is under the per-entry limit; together they far exceed the total.
        for i in 0..200 {
            c.put(format!("k{i}"), &big(MAX_ENTRY_BYTES - 1));
            assert!(c.lock().bytes <= MAX_BYTES, "after {i}");
        }
        let inner = c.lock();
        assert!(inner.entries.len() < 40 && !inner.entries.is_empty());
        assert_eq!(
            inner.bytes,
            inner.entries.values().map(|(_, s, _)| *s).sum::<usize>()
        );
    }

    #[test]
    fn a_body_over_the_limit_is_not_kept_and_tiny_entries_are_bounded_too() {
        let c = Cache::new();
        c.put("huge".into(), &big(MAX_ENTRY_BYTES + 1));
        assert!(c.lock().entries.is_empty());
        for i in 0..(MAX_BYTES / OVERHEAD) + 100 {
            c.put(format!("k{i}"), &big(0));
        }
        assert!(c.lock().bytes <= MAX_BYTES);
    }
}
