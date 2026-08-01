use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Full configuration shared by the GUI and CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub language: String,
    /// When true, the one-click flow reinstalls/updates components that are
    /// already present; when false it skips satisfied components.
    pub reinstall_existing: bool,
    pub proxy: ProxyConfig,
    pub uv: UvConfig,
    pub python: PythonConfig,
    pub vscode: VscodeConfig,
    pub path: PathConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: "zh-CN".to_string(),
            reinstall_existing: false,
            proxy: ProxyConfig::default(),
            uv: UvConfig::default(),
            python: PythonConfig::default(),
            vscode: VscodeConfig::default(),
            path: PathConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    pub http: String,
    pub https: String,
    pub no_proxy: String,
}

impl ProxyConfig {
    pub fn is_empty(&self) -> bool {
        self.http.trim().is_empty() && self.https.trim().is_empty()
    }

    /// Prefer an explicit https proxy, else fall back to the http one.
    pub fn https_or_http(&self) -> Option<&str> {
        let h = self.https.trim();
        if !h.is_empty() {
            return Some(h);
        }
        let h = self.http.trim();
        if !h.is_empty() {
            Some(h)
        } else {
            None
        }
    }

    pub fn http_opt(&self) -> Option<&str> {
        let h = self.http.trim();
        if h.is_empty() {
            None
        } else {
            Some(h)
        }
    }

    pub fn https_opt(&self) -> Option<&str> {
        let h = self.https.trim();
        if h.is_empty() {
            None
        } else {
            Some(h)
        }
    }

    pub fn no_proxy_opt(&self) -> Option<&str> {
        let h = self.no_proxy.trim();
        if h.is_empty() {
            None
        } else {
            Some(h)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UvConfig {
    /// "latest" or a pinned version like "0.9.2".
    pub version: String,
}

impl Default for UvConfig {
    fn default() -> Self {
        UvConfig {
            version: "latest".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PythonConfig {
    /// "latest" or a version like "3.13".
    pub version: String,
    /// Also install bare `python` / `python3` shims (uv `--default`).
    pub set_default: bool,
}

impl Default for PythonConfig {
    fn default() -> Self {
        PythonConfig {
            version: "latest".to_string(),
            set_default: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VscodeConfig {
    pub install: bool,
    pub extensions: Vec<String>,
}

impl Default for VscodeConfig {
    fn default() -> Self {
        VscodeConfig {
            install: true,
            extensions: default_extensions(),
        }
    }
}

pub fn default_extensions() -> Vec<String> {
    [
        "ms-python.python",
        "ms-python.vscode-pylance",
        "ms-python.debugpy",
        "charliermarsh.ruff",
        "ms-toolsai.jupyter",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathConfig {
    pub update: bool,
    /// Unix shell profiles to update. Any of: "bashrc", "zshrc", "fish", "profile".
    /// Ignored on Windows (the user registry PATH is always used).
    pub shells: Vec<String>,
}

impl Default for PathConfig {
    fn default() -> Self {
        PathConfig {
            update: true,
            shells: vec!["bashrc".to_string()],
        }
    }
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Config> {
        let text = std::fs::read_to_string(path)?;
        toml::from_str(&text).map_err(|e| Error::ConfigParse(e.to_string()))
    }

    /// Load from the given path if it exists, otherwise return defaults.
    pub fn load_or_default(path: &Path) -> Config {
        if path.exists() {
            Config::load_from_path(path).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(|e| Error::ConfigWrite(e.to_string()))?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

/// Default location for the persisted GUI config: `<config-dir>/pydev/config.toml`.
pub fn default_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("pydev").join("config.toml")
}
