use std::path::PathBuf;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
}

pub fn current_os() -> Os {
    if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOS
    } else {
        Os::Linux
    }
}

pub fn os_key() -> &'static str {
    match current_os() {
        Os::Windows => "windows",
        Os::MacOS => "macos",
        Os::Linux => "linux",
    }
}

/// Architecture as reported by the compiler: "x86_64", "aarch64", ...
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| Error::msg("could not determine the home directory"))
}

/// Directory where user-level executables live and where we install uv and the
/// Python shims. Added to PATH. Unix: `~/.local/bin`; Windows: `%USERPROFILE%\.local\bin`.
pub fn user_bin_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".local").join("bin"))
}

/// Directory for larger user-level payloads such as the extracted VSCode tarball
/// on Linux: `~/.local/opt`.
pub fn user_opt_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".local").join("opt"))
}

pub fn uv_exe_name() -> &'static str {
    if current_os() == Os::Windows {
        "uv.exe"
    } else {
        "uv"
    }
}

/// The slug used by `https://update.code.visualstudio.com/latest/<slug>/stable`.
pub fn vscode_slug() -> Result<&'static str> {
    let a = arch();
    match current_os() {
        Os::Windows => match a {
            "x86_64" => Ok("win32-x64-user"),
            "aarch64" => Ok("win32-arm64-user"),
            other => Err(Error::Unsupported(format!("windows arch {other}"))),
        },
        // Universal build works on both Intel and Apple Silicon.
        Os::MacOS => Ok("darwin-universal"),
        Os::Linux => match a {
            "x86_64" => Ok("linux-x64"),
            "aarch64" => Ok("linux-arm64"),
            "arm" => Ok("linux-armhf"),
            other => Err(Error::Unsupported(format!("linux arch {other}"))),
        },
    }
}

/// Platform slug for the VSCode update JSON API
/// (`https://update.code.visualstudio.com/api/update/<slug>/stable/latest`).
/// The reported version is the same across platforms, so any valid slug works.
pub fn vscode_update_platform() -> &'static str {
    let a = arch();
    match current_os() {
        Os::Windows => {
            if a == "aarch64" {
                "win32-arm64"
            } else {
                "win32-x64"
            }
        }
        Os::MacOS => "darwin",
        Os::Linux => {
            if a == "aarch64" {
                "linux-arm64"
            } else {
                "linux-x64"
            }
        }
    }
}

pub fn vscode_download_url() -> Result<String> {
    Ok(format!(
        "https://update.code.visualstudio.com/latest/{}/stable",
        vscode_slug()?
    ))
}

/// File extension used when saving the downloaded VSCode payload.
pub fn vscode_archive_ext() -> &'static str {
    match current_os() {
        Os::Windows => "exe",
        Os::MacOS => "zip",
        Os::Linux => "tar.gz",
    }
}
