use std::path::Path;

use crate::config::Config;
use crate::download::proxy_envs;
use crate::error::Result;
use crate::progress::Reporter;
use crate::runner;

const ZH_LANGUAGE_PACK: &str = "ms-ceintl.vscode-language-pack-zh-hans";

/// The extensions that should end up installed for the given config: the
/// configured list plus the Simplified Chinese language pack when the UI
/// language is Chinese.
pub fn effective_list(cfg: &Config) -> Vec<String> {
    let mut exts: Vec<String> = cfg.vscode.extensions.clone();
    if cfg.language.to_lowercase().starts_with("zh")
        && !exts.iter().any(|e| e == ZH_LANGUAGE_PACK)
    {
        exts.push(ZH_LANGUAGE_PACK.to_string());
    }
    exts
}

/// Install the full effective extension list.
pub fn install(cfg: &Config, code_bin: &Path, reporter: &dyn Reporter) -> Result<()> {
    install_list(cfg, code_bin, &effective_list(cfg), reporter)
}

/// Install a specific list of extensions using the `code` CLI (`--force` also
/// updates already-installed ones to the latest version).
pub fn install_list(
    cfg: &Config,
    code_bin: &Path,
    list: &[String],
    reporter: &dyn Reporter,
) -> Result<()> {
    if list.is_empty() {
        reporter.info("No extensions to install");
        return Ok(());
    }
    let envs = proxy_envs(&cfg.proxy);
    for ext in list {
        reporter.info(&format!("Installing extension {ext} ..."));
        run_code(
            code_bin,
            &["--install-extension", ext, "--force"],
            &envs,
            reporter,
        )?;
    }
    reporter.success("VSCode extensions installed");
    Ok(())
}

#[cfg(windows)]
fn run_code(
    code_bin: &Path,
    args: &[&str],
    envs: &[(String, String)],
    reporter: &dyn Reporter,
) -> Result<()> {
    // `code` is a .cmd shim on Windows and must be launched through cmd.exe.
    let code = code_bin.to_string_lossy().to_string();
    let mut full: Vec<&str> = vec!["/C", &code];
    full.extend_from_slice(args);
    runner::run("cmd", &full, envs, reporter)
}

#[cfg(not(windows))]
fn run_code(
    code_bin: &Path,
    args: &[&str],
    envs: &[(String, String)],
    reporter: &dyn Reporter,
) -> Result<()> {
    runner::run(&code_bin.to_string_lossy(), args, envs, reporter)
}
