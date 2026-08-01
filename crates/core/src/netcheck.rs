use std::time::Instant;

use serde::Serialize;

use crate::config::Config;
use crate::download;
use crate::error::Result;
use crate::progress::Reporter;

#[derive(Debug, Clone, Serialize)]
pub struct EndpointResult {
    pub name: String,
    pub url: String,
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Endpoints that must be reachable for a successful install.
const ENDPOINTS: &[(&str, &str)] = &[
    ("Astral (uv)", "https://astral.sh"),
    ("VSCode updates", "https://update.code.visualstudio.com/"),
    ("GitHub (Python builds)", "https://github.com"),
    ("PyPI", "https://pypi.org/simple/"),
];

/// Probe each endpoint (through the configured proxy) and report reachability
/// and latency. Never returns an error for an unreachable host - that is
/// captured per endpoint so the whole test always completes.
pub fn run(cfg: &Config, reporter: &dyn Reporter) -> Result<Vec<EndpointResult>> {
    if cfg.proxy.is_empty() {
        reporter.info("Testing network (direct connection, no proxy)...");
    } else {
        reporter.info("Testing network through the configured proxy...");
    }

    let client = download::build_client(&cfg.proxy, 10)?;
    let mut results = Vec::with_capacity(ENDPOINTS.len());

    for (name, url) in ENDPOINTS {
        let start = Instant::now();
        let result = match client.get(*url).send() {
            Ok(resp) => {
                let status = resp.status();
                let ms = start.elapsed().as_millis() as u64;
                let ok = status.is_success() || status.is_redirection() || status.is_client_error();
                // 4xx still proves connectivity to the host.
                if ok {
                    reporter.success(&format!("{name}: reachable ({} ms)", ms));
                } else {
                    reporter.warn(&format!("{name}: HTTP {}", status.as_u16()));
                }
                EndpointResult {
                    name: name.to_string(),
                    url: url.to_string(),
                    ok,
                    status: Some(status.as_u16()),
                    latency_ms: Some(ms),
                    error: None,
                }
            }
            Err(e) => {
                reporter.error(&format!("{name}: {e}"));
                EndpointResult {
                    name: name.to_string(),
                    url: url.to_string(),
                    ok: false,
                    status: None,
                    latency_ms: None,
                    error: Some(e.to_string()),
                }
            }
        };
        results.push(result);
    }

    Ok(results)
}
