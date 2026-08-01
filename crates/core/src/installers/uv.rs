use std::path::PathBuf;

use crate::config::Config;
use crate::download::{self, proxy_envs};
use crate::error::{Error, Result};
use crate::platform::{self, Os};
use crate::progress::Reporter;
use crate::runner;

/// Absolute path where the uv binary is (or will be) installed.
pub fn uv_binary_path() -> Result<PathBuf> {
    Ok(platform::user_bin_dir()?.join(platform::uv_exe_name()))
}

pub fn is_installed() -> bool {
    uv_binary_path().map(|p| p.exists()).unwrap_or(false)
}

/// Environment shared by the uv installer and `uv python install`:
/// proxy variables plus deterministic install / executable directories so the
/// binaries land where we later add to PATH.
pub fn tool_envs(cfg: &Config) -> Result<Vec<(String, String)>> {
    let mut envs = proxy_envs(&cfg.proxy);
    let bin = platform::user_bin_dir()?;
    let bin_str = bin.to_string_lossy().to_string();

    envs.push(("UV_INSTALL_DIR".to_string(), bin_str.clone()));
    // We manage PATH ourselves for consistency across shells.
    envs.push(("UV_NO_MODIFY_PATH".to_string(), "1".to_string()));
    // Force uv's "executable directory" (where `--default` python shims and
    // tools go) to the same bin dir on Unix so our single PATH entry covers it.
    if platform::current_os() != Os::Windows {
        envs.push(("XDG_BIN_HOME".to_string(), bin_str));
    }
    Ok(envs)
}

/// Install (or reinstall) uv using the official standalone installer.
pub fn install(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.info("Installing the uv package manager...");
    let bin_dir = platform::user_bin_dir()?;
    std::fs::create_dir_all(&bin_dir)?;

    let version = cfg.uv.version.trim();
    let client = download::build_client(&cfg.proxy, 120)?;
    let envs = tool_envs(cfg)?;

    match platform::current_os() {
        Os::Windows => {
            let url = if version.is_empty() || version == "latest" {
                "https://astral.sh/uv/install.ps1".to_string()
            } else {
                format!("https://astral.sh/uv/{version}/install.ps1")
            };
            let script = std::env::temp_dir().join("pydev-uv-install.ps1");
            download::download_file(&client, &url, &script, reporter)?;
            runner::run(
                "powershell",
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    &script.to_string_lossy(),
                ],
                &envs,
                reporter,
            )?;
        }
        _ => {
            let url = if version.is_empty() || version == "latest" {
                "https://astral.sh/uv/install.sh".to_string()
            } else {
                format!("https://astral.sh/uv/{version}/install.sh")
            };
            let script = std::env::temp_dir().join("pydev-uv-install.sh");
            download::download_file(&client, &url, &script, reporter)?;
            runner::run("sh", &[&script.to_string_lossy()], &envs, reporter)?;
        }
    }

    let uv = uv_binary_path()?;
    if !uv.exists() {
        return Err(Error::msg(format!(
            "uv did not appear at {} after installation",
            uv.display()
        )));
    }
    reporter.success(&format!("uv installed at {}", uv.display()));
    Ok(())
}
