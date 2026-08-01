use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;

use crate::config::ProxyConfig;
use crate::error::{Error, Result};
use crate::progress::Reporter;

/// Build a blocking HTTP client honoring the proxy configuration.
pub fn build_client(proxy: &ProxyConfig, timeout_secs: u64) -> Result<Client> {
    let mut builder = Client::builder()
        .user_agent("pydev-installer")
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(15));

    let no_proxy = proxy
        .no_proxy_opt()
        .and_then(reqwest::NoProxy::from_string);

    if let Some(url) = proxy.http_opt() {
        builder = builder.proxy(reqwest::Proxy::http(url)?.no_proxy(no_proxy.clone()));
    }
    if let Some(url) = proxy.https_opt() {
        builder = builder.proxy(reqwest::Proxy::https(url)?.no_proxy(no_proxy.clone()));
    }

    builder.build().map_err(Error::from)
}

/// Environment variables to pass to child processes so their own downloaders
/// (curl / PowerShell / uv) go through the configured proxy.
pub fn proxy_envs(proxy: &ProxyConfig) -> Vec<(String, String)> {
    let mut envs = Vec::new();
    let mut push = |k: &str, v: &str| envs.push((k.to_string(), v.to_string()));

    if let Some(u) = proxy.http_opt() {
        push("HTTP_PROXY", u);
        push("http_proxy", u);
    }
    if let Some(u) = proxy.https_opt() {
        push("HTTPS_PROXY", u);
        push("https_proxy", u);
        // uv reads ALL_PROXY as a general fallback.
        push("ALL_PROXY", u);
    } else if let Some(u) = proxy.http_opt() {
        push("ALL_PROXY", u);
    }
    if let Some(u) = proxy.no_proxy_opt() {
        push("NO_PROXY", u);
        push("no_proxy", u);
    }
    envs
}

fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

/// Download `url` to `dest`, streaming with periodic progress logs.
pub fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    reporter: &dyn Reporter,
) -> Result<()> {
    reporter.info(&format!("Downloading {url}"));
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut resp = client.get(url).send()?.error_for_status()?;
    let total = resp.content_length();
    let mut file = File::create(dest)?;

    let mut buf = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut last = Instant::now();
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        if last.elapsed() >= Duration::from_millis(500) {
            match total {
                Some(t) if t > 0 => {
                    let pct = (downloaded as f64 / t as f64 * 100.0) as u32;
                    reporter.info(&format!(
                        "  {} / {} ({pct}%)",
                        human_bytes(downloaded),
                        human_bytes(t)
                    ));
                }
                _ => reporter.info(&format!("  {}", human_bytes(downloaded))),
            }
            last = Instant::now();
        }
    }
    file.flush()?;
    reporter.info(&format!("Downloaded {}", human_bytes(downloaded)));
    Ok(())
}
