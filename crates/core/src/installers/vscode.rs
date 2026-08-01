use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::download;
use crate::error::{Error, Result};
use crate::fsutil;
use crate::platform::{self, Os};
use crate::progress::Reporter;
use crate::runner;

/// Best-effort lookup of an already-installed `code` launcher, used when
/// (re)installing extensions without reinstalling the editor.
pub fn locate_code() -> Option<PathBuf> {
    match platform::current_os() {
        Os::Windows => {
            let p = dirs::data_local_dir()?
                .join("Programs")
                .join("Microsoft VS Code")
                .join("bin")
                .join("code.cmd");
            p.exists().then_some(p)
        }
        _ => {
            let p = platform::user_bin_dir().ok()?.join("code");
            p.exists().then_some(p)
        }
    }
}

/// Capture output of the `code` CLI, going through cmd.exe on Windows where the
/// launcher is a `.cmd` shim.
#[cfg(windows)]
pub fn capture_code(code_bin: &Path, args: &[&str]) -> Result<String> {
    let code = code_bin.to_string_lossy().to_string();
    let mut full: Vec<&str> = vec!["/C", &code];
    full.extend_from_slice(args);
    runner::capture("cmd", &full, &[])
}

#[cfg(not(windows))]
pub fn capture_code(code_bin: &Path, args: &[&str]) -> Result<String> {
    runner::capture(&code_bin.to_string_lossy(), args, &[])
}

/// Install the latest stable VSCode for the current platform and return the
/// path to its `code` command-line launcher.
pub fn install(cfg: &Config, reporter: &dyn Reporter) -> Result<PathBuf> {
    match platform::current_os() {
        Os::Windows => install_windows(cfg, reporter),
        Os::MacOS => install_macos(cfg, reporter),
        Os::Linux => install_linux(cfg, reporter),
    }
}

fn download_payload(cfg: &Config, reporter: &dyn Reporter) -> Result<PathBuf> {
    let url = platform::vscode_download_url()?;
    let dest = std::env::temp_dir().join(format!("pydev-vscode.{}", platform::vscode_archive_ext()));
    let client = download::build_client(&cfg.proxy, 300)?;
    reporter.info("Downloading the latest stable VSCode...");
    download::download_file(&client, &url, &dest, reporter)?;
    Ok(dest)
}

fn install_windows(cfg: &Config, reporter: &dyn Reporter) -> Result<PathBuf> {
    let installer = download_payload(cfg, reporter)?;
    reporter.info("Running the VSCode installer silently...");
    runner::run(
        &installer.to_string_lossy(),
        &["/VERYSILENT", "/NORESTART", "/MERGETASKS=!runcode"],
        &[],
        reporter,
    )?;

    let base = dirs::data_local_dir()
        .ok_or_else(|| Error::msg("could not resolve %LOCALAPPDATA%"))?;
    let code = base
        .join("Programs")
        .join("Microsoft VS Code")
        .join("bin")
        .join("code.cmd");
    reporter.success("VSCode installed");
    Ok(code)
}

fn install_macos(cfg: &Config, reporter: &dyn Reporter) -> Result<PathBuf> {
    let archive = download_payload(cfg, reporter)?;
    let apps = platform::home_dir()?.join("Applications");
    std::fs::create_dir_all(&apps)?;

    reporter.info("Extracting VSCode into ~/Applications ...");
    // `ditto` preserves the app bundle's symlinks and permissions.
    runner::run(
        "ditto",
        &[
            "-x",
            "-k",
            &archive.to_string_lossy(),
            &apps.to_string_lossy(),
        ],
        &[],
        reporter,
    )?;

    let code_real = apps
        .join("Visual Studio Code.app")
        .join("Contents/Resources/app/bin/code");
    let link = platform::user_bin_dir()?.join("code");
    fsutil::symlink_or_copy(&code_real, &link)?;
    reporter.success("VSCode installed");
    Ok(link)
}

fn install_linux(cfg: &Config, reporter: &dyn Reporter) -> Result<PathBuf> {
    let archive = download_payload(cfg, reporter)?;
    let opt = platform::user_opt_dir()?;
    std::fs::create_dir_all(&opt)?;

    reporter.info("Extracting the VSCode tarball...");
    runner::run(
        "tar",
        &[
            "-xzf",
            &archive.to_string_lossy(),
            "-C",
            &opt.to_string_lossy(),
        ],
        &[],
        reporter,
    )?;

    // The tarball unpacks to a `VSCode-linux-<arch>` directory.
    let dir = std::fs::read_dir(&opt)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("VSCode-linux"))
                .unwrap_or(false)
        })
        .ok_or_else(|| Error::msg("could not find the extracted VSCode directory"))?;

    let code_real = dir.join("bin").join("code");
    let link = platform::user_bin_dir()?.join("code");
    fsutil::symlink_or_copy(&code_real, &link)?;
    reporter.success("VSCode installed");
    Ok(link)
}
