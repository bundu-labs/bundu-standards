//! The HTTPS client the file endpoint reads GitHub with: rustls (no OpenSSL), a timeout on
//! the connection and on the whole request, a few redirects, and a cap on the body, so a slow
//! or huge answer cannot hold the server. It follows no instruction of the caller beyond the
//! URL [`crate::core`] built from its four fixed files.

use crate::core::{Fetch, Upstream, USER_AGENT};
use std::time::Duration;

/// The largest upstream body read. The four register files are well under a megabyte.
const MAX_BODY: usize = 1024 * 1024;

pub struct GitHub {
    client: reqwest::Client,
}

impl GitHub {
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()?;
        Ok(Self { client })
    }
}

impl Fetch for GitHub {
    async fn get(&self, url: &str, accept: Option<&str>) -> Result<Upstream, ()> {
        let mut req = self.client.get(url);
        if let Some(a) = accept {
            req = req.header(reqwest::header::ACCEPT, a);
        }
        let mut res = req.send().await.map_err(|_| ())?;
        let status = res.status().as_u16();
        let mut body = Vec::new();
        while let Some(chunk) = res.chunk().await.map_err(|_| ())? {
            if body.len() + chunk.len() > MAX_BODY {
                return Err(());
            }
            body.extend_from_slice(&chunk);
        }
        Ok(Upstream { status, body })
    }
}
