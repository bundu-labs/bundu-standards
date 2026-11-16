//! standards.bundu.org's file endpoint as a Rust Cloudflare Worker.
//!
//! The behaviour lives in [`core`], which has no platform dependency and is tested natively
//! (`cargo test`). This file is only the Worker glue: it maps a request to [`core::route`],
//! reaches GitHub with the platform's `fetch`, and keeps successful GETs in the edge cache.

pub mod core;

#[cfg(target_arch = "wasm32")]
mod worker_glue {
    use crate::core::{self, Fetch, Reply, Upstream, USER_AGENT};
    use worker::{
        event, Cache, Context, Env, Fetch as Fetcher, Headers, Method, Request, RequestInit,
        Response, Result,
    };

    struct GitHub;

    impl Fetch for GitHub {
        async fn get(&self, url: &str, accept: Option<&str>) -> std::result::Result<Upstream, ()> {
            let headers = Headers::new();
            headers.set("user-agent", USER_AGENT).map_err(|_| ())?;
            if let Some(a) = accept {
                headers.set("accept", a).map_err(|_| ())?;
            }
            let mut init = RequestInit::new();
            init.with_method(Method::Get).with_headers(headers);
            let req = Request::new_with_init(url, &init).map_err(|_| ())?;
            let mut res = Fetcher::Request(req).send().await.map_err(|_| ())?;
            let status = res.status_code();
            let body = res.bytes().await.map_err(|_| ())?;
            Ok(Upstream { status, body })
        }
    }

    fn to_response(reply: Reply) -> Result<Response> {
        let headers = Headers::new();
        for (k, v) in &reply.headers {
            headers.set(k, v)?;
        }
        let res = match reply.body {
            Some(body) => Response::from_bytes(body)?,
            None => Response::empty()?,
        };
        Ok(res.with_status(reply.status).with_headers(headers))
    }

    #[event(fetch)]
    async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
        let url = req.url()?;
        let origin = url.origin().ascii_serialization();
        let path = url.path().to_owned();
        let method = req.method().to_string();

        let cache = Cache::default();
        let key = (method == "GET").then(|| core::cache_key(&origin, &path));
        if let Some(k) = &key {
            if let Some(hit) = cache.get(k.as_str(), false).await? {
                return Ok(hit);
            }
        }

        let mut res = to_response(core::route(&method, &origin, &path, &GitHub).await)?;
        if let Some(k) = key {
            if res.status_code() == 200 {
                cache.put(k.as_str(), res.cloned()?).await?;
            }
        }
        Ok(res)
    }
}
