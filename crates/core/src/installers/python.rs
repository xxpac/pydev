use std::collections::BTreeSet;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::installers::uv;
use crate::progress::Reporter;
use crate::runner;

/// Curated fallback list of Python minor versions offered in the UI when uv is
/// not yet installed (or its listing can't be parsed).
pub fn fallback_versions() -> Vec<String> {
    ["3.14", "3.13", "3.12", "3.11", "3.10"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Return selectable Python versions. Always starts with "latest". If uv is
/// installed, augments the list with the major.minor versions uv knows about.
pub fn list_versions(cfg: &Config) -> Vec<String> {
    let mut minors: BTreeSet<String> = BTreeSet::new();

    if uv::is_installed() {
        if let Ok(uv_path) = uv::uv_binary_path() {
            if let Ok(envs) = uv::tool_envs(cfg) {
                if let Ok(out) = runner::capture(
                    &uv_path.to_string_lossy(),
                    &["python", "list", "--all-versions"],
                    &envs,
                ) {
                    for line in out.lines() {
                        // Rows look like: `cpython-3.13.1-linux-x86_64-gnu   <path|download>`
                        let token = line.split_whitespace().next().unwrap_or("");
                        if let Some(rest) = token.strip_prefix("cpython-") {
                            let ver = rest.split('-').next().unwrap_or("");
                            let mut parts = ver.split('.');
                            if let (Some(maj), Some(min)) = (parts.next(), parts.next()) {
                                if maj.parse::<u32>().is_ok() && min.parse::<u32>().is_ok() {
                                    minors.insert(format!("{maj}.{min}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut versions: Vec<String> = if minors.is_empty() {
        fallback_versions()
    } else {
        // Sort descending by numeric (major, minor).
        let mut v: Vec<String> = minors.into_iter().collect();
        v.sort_by(|a, b| numeric_key(b).cmp(&numeric_key(a)));
        v
    };

    versions.insert(0, "latest".to_string());
    versions
}

fn numeric_key(v: &str) -> (u32, u32) {
    let mut it = v.split('.');
    let maj = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let min = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (maj, min)
}

/// Install the configured Python version through uv.
pub fn install(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    if !uv::is_installed() {
        return Err(Error::msg("uv is not installed yet; install uv first"));
    }
    let uv_path = uv::uv_binary_path()?;
    let uv_str = uv_path.to_string_lossy().to_string();
    let envs = uv::tool_envs(cfg)?;

    let version = cfg.python.version.trim();
    let mut args: Vec<&str> = vec!["python", "install"];
    if !version.is_empty() && version != "latest" {
        args.push(version);
    }
    if cfg.python.set_default {
        args.push("--default");
    }

    reporter.info(&format!(
        "Installing Python ({}) via uv...",
        if version.is_empty() { "latest" } else { version }
    ));
    runner::run(&uv_str, &args, &envs, reporter)?;
    reporter.success("Python installed");
    Ok(())
}
