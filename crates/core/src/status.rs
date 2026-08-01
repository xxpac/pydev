//! Detection of already-installed components and their versions, so the UI/CLI
//! can show status and decide whether to skip, reinstall, or upgrade.

use std::collections::HashMap;

use serde::Serialize;

use crate::config::Config;
use crate::download;
use crate::installers::{extensions, uv, vscode};
use crate::pathenv::{self, PathStatus};
use crate::platform;
use crate::runner;

/// Status of a single tool (uv / VSCode).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Tool {
    pub installed: bool,
    pub current: Option<String>,
    pub latest: Option<String>,
    pub upgrade_available: bool,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtStatus {
    pub id: String,
    pub installed: bool,
    pub version: Option<String>,
    pub latest: Option<String>,
    pub upgrade_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PythonStatus {
    pub installed_versions: Vec<String>,
    pub requested: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvStatus {
    pub uv: Tool,
    pub python: PythonStatus,
    pub vscode: Tool,
    pub extensions: Vec<ExtStatus>,
    pub path: PathStatus,
}

/// Detect the state of all components. `check_latest` enables best-effort
/// network lookups (currently the newest uv release) to flag upgrades.
pub fn detect(cfg: &Config, check_latest: bool) -> EnvStatus {
    EnvStatus {
        uv: detect_uv(cfg, check_latest),
        python: detect_python(cfg),
        vscode: detect_vscode(cfg, check_latest),
        extensions: detect_extensions(cfg, check_latest),
        path: pathenv::status(cfg).unwrap_or(PathStatus {
            configured: false,
            pending_targets: vec![],
        }),
    }
}

fn detect_uv(cfg: &Config, check_latest: bool) -> Tool {
    let path = uv::uv_binary_path().ok();
    let installed = path.as_ref().map(|p| p.exists()).unwrap_or(false);
    let mut tool = Tool {
        installed,
        location: path
            .as_ref()
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().to_string()),
        ..Tool::default()
    };

    if installed {
        if let Some(p) = &path {
            if let Ok(out) = runner::capture(&p.to_string_lossy(), &["--version"], &[]) {
                tool.current = first_version_token(&out);
            }
        }
    }
    if check_latest {
        tool.latest = uv_latest(cfg);
    }
    tool.upgrade_available = match (&tool.current, &tool.latest) {
        (Some(c), Some(l)) => version_lt(c, l),
        _ => false,
    };
    tool
}

fn uv_latest(cfg: &Config) -> Option<String> {
    let client = download::build_client(&cfg.proxy, 6).ok()?;
    let resp = client
        .get("https://api.github.com/repos/astral-sh/uv/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    v.get("tag_name")?
        .as_str()
        .map(|s| s.trim_start_matches('v').to_string())
}

fn detect_python(cfg: &Config) -> PythonStatus {
    let mut installed: Vec<String> = Vec::new();
    if uv::is_installed() {
        if let Ok(p) = uv::uv_binary_path() {
            if let Ok(out) = runner::capture(
                &p.to_string_lossy(),
                &["python", "list", "--only-installed"],
                &[],
            ) {
                for line in out.lines() {
                    let token = line.split_whitespace().next().unwrap_or("");
                    if let Some(rest) = token.strip_prefix("cpython-") {
                        if let Some(ver) = rest.split('-').next() {
                            if ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                                installed.push(ver.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    installed.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
    installed.dedup();

    let requested = {
        let r = cfg.python.version.trim();
        if r.is_empty() { "latest".to_string() } else { r.to_string() }
    };
    let satisfied = if requested == "latest" {
        !installed.is_empty()
    } else {
        installed
            .iter()
            .any(|v| v == &requested || v.starts_with(&format!("{requested}.")))
    };

    PythonStatus {
        installed_versions: installed,
        requested,
        satisfied,
    }
}

fn detect_vscode(cfg: &Config, check_latest: bool) -> Tool {
    let mut tool = Tool::default();
    if let Some(code) = vscode::locate_code() {
        tool.installed = true;
        tool.location = Some(code.to_string_lossy().to_string());
        if let Ok(out) = vscode::capture_code(&code, &["--version"]) {
            tool.current = out
                .lines()
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
        }
    }
    if check_latest && tool.installed {
        tool.latest = vscode_latest(cfg);
    }
    tool.upgrade_available = match (&tool.current, &tool.latest) {
        (Some(c), Some(l)) => version_lt(c, l),
        _ => false,
    };
    tool
}

fn vscode_latest(cfg: &Config) -> Option<String> {
    let client = download::build_client(&cfg.proxy, 8).ok()?;
    let url = format!(
        "https://update.code.visualstudio.com/api/update/{}/stable/latest",
        platform::vscode_update_platform()
    );
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    v.get("productVersion")
        .or_else(|| v.get("name"))?
        .as_str()
        .map(|s| s.to_string())
}

fn detect_extensions(cfg: &Config, check_latest: bool) -> Vec<ExtStatus> {
    let wanted = extensions::effective_list(cfg);
    let mut installed_map: HashMap<String, Option<String>> = HashMap::new();

    if let Some(code) = vscode::locate_code() {
        if let Ok(out) = vscode::capture_code(&code, &["--list-extensions", "--show-versions"]) {
            for line in out.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let (id, ver) = match line.split_once('@') {
                    Some((a, b)) => (a.to_string(), Some(b.to_string())),
                    None => (line.to_string(), None),
                };
                installed_map.insert(id.to_lowercase(), ver);
            }
        }
    }

    // Only query the marketplace for the wanted extensions that are installed
    // (there is nothing to compare against otherwise).
    let installed_wanted: Vec<String> = wanted
        .iter()
        .filter(|id| installed_map.contains_key(&id.to_lowercase()))
        .cloned()
        .collect();
    let latest_map = if check_latest && !installed_wanted.is_empty() {
        ext_latest_versions(cfg, &installed_wanted)
    } else {
        HashMap::new()
    };

    wanted
        .into_iter()
        .map(|id| {
            let key = id.to_lowercase();
            let installed = installed_map.contains_key(&key);
            let version = installed_map.get(&key).cloned().flatten();
            let latest = latest_map.get(&key).cloned();
            let upgrade_available = match (&version, &latest) {
                (Some(c), Some(l)) => version_lt(c, l),
                _ => false,
            };
            ExtStatus {
                id,
                installed,
                version,
                latest,
                upgrade_available,
            }
        })
        .collect()
}

/// Batch-query the VSCode Marketplace for the latest version of each extension.
/// Returns a map of lowercase `publisher.name` -> latest version. Best-effort:
/// any failure yields an empty/partial map.
fn ext_latest_versions(cfg: &Config, ids: &[String]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return out;
    }
    let client = match download::build_client(&cfg.proxy, 10) {
        Ok(c) => c,
        Err(_) => return out,
    };

    let mut criteria = vec![serde_json::json!({
        "filterType": 8,
        "value": "Microsoft.VisualStudio.Code"
    })];
    for id in ids {
        criteria.push(serde_json::json!({ "filterType": 7, "value": id }));
    }
    let body = serde_json::json!({
        "filters": [{ "criteria": criteria, "pageSize": ids.len(), "pageNumber": 1 }],
        "flags": 914
    });

    let resp = match client
        .post("https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery")
        .header("Accept", "application/json;api-version=3.0-preview.1")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
    {
        Ok(r) => r,
        Err(_) => return out,
    };
    if !resp.status().is_success() {
        return out;
    }
    let v: serde_json::Value = match resp.json() {
        Ok(j) => j,
        Err(_) => return out,
    };

    if let Some(exts) = v.pointer("/results/0/extensions").and_then(|e| e.as_array()) {
        for e in exts {
            let publisher = e.pointer("/publisher/publisherName").and_then(|x| x.as_str());
            let name = e.get("extensionName").and_then(|x| x.as_str());
            let ver = e.pointer("/versions/0/version").and_then(|x| x.as_str());
            if let (Some(p), Some(n), Some(vv)) = (publisher, name, ver) {
                out.insert(format!("{p}.{n}").to_lowercase(), vv.to_string());
            }
        }
    }
    out
}

/// First whitespace-separated token that looks like a version, e.g. from
/// "uv 0.9.2" -> "0.9.2".
fn first_version_token(s: &str) -> Option<String> {
    s.split_whitespace()
        .find(|t| t.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(|t| t.trim().to_string())
}

/// Split a version into numeric components for comparison ("0.9.10" -> [0,9,10]).
fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| c == '.' || c == '-')
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

fn version_lt(a: &str, b: &str) -> bool {
    version_key(a) < version_key(b)
}
