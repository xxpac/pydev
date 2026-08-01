use crate::config::Config;
use crate::error::{Error, Result};
use crate::installers::{extensions, python, uv, vscode};
use crate::pathenv;
use crate::progress::Reporter;
use crate::status;

/// The one-click flow: uv -> Python -> (VSCode + extensions) -> PATH.
///
/// Already-satisfied components are skipped unless `cfg.reinstall_existing` is
/// set, in which case they are reinstalled/updated to the latest version.
pub fn one_click(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    let reinstall = cfg.reinstall_existing;
    let st = status::detect(cfg, false);

    let run_uv = reinstall || !st.uv.installed;
    let run_py = reinstall || !st.python.satisfied;

    let do_vscode = cfg.vscode.install;
    let run_editor = do_vscode && (reinstall || !st.vscode.installed);
    let ext_to_install: Vec<String> = if !do_vscode {
        Vec::new()
    } else if reinstall {
        extensions::effective_list(cfg)
    } else {
        st.extensions
            .iter()
            .filter(|e| !e.installed)
            .map(|e| e.id.clone())
            .collect()
    };
    let run_vscode_step = run_editor || !ext_to_install.is_empty();
    let run_path = cfg.path.update && (reinstall || !st.path.configured);

    let total = [run_uv, run_py, run_vscode_step, run_path]
        .iter()
        .filter(|b| **b)
        .count() as u32;

    if total == 0 {
        reporter.success("Everything is already installed and configured.");
        return Ok(());
    }

    let mut idx = 0u32;

    if run_uv {
        idx += 1;
        reporter.stage("uv", idx, total);
        uv::install(cfg, reporter)?;
    } else {
        reporter.info(&format!(
            "uv {} already installed, skipping",
            st.uv.current.as_deref().unwrap_or("")
        ));
    }

    if run_py {
        idx += 1;
        reporter.stage("python", idx, total);
        python::install(cfg, reporter)?;
    } else {
        reporter.info(&format!(
            "Python {} already installed, skipping",
            st.python.requested
        ));
    }

    if run_vscode_step {
        idx += 1;
        reporter.stage("vscode", idx, total);
        let code = if run_editor {
            vscode::install(cfg, reporter)?
        } else {
            reporter.info("VSCode already installed; installing missing extensions only");
            vscode::locate_code().ok_or_else(|| Error::msg("VSCode not found"))?
        };
        if !ext_to_install.is_empty() {
            extensions::install_list(cfg, &code, &ext_to_install, reporter)?;
        }
    } else if do_vscode {
        reporter.info("VSCode and extensions already installed, skipping");
    }

    if run_path {
        idx += 1;
        reporter.stage("path", idx, total);
        pathenv::apply(cfg, reporter)?;
    } else if cfg.path.update {
        reporter.info("PATH already configured, skipping");
    } else {
        reporter.info("PATH update skipped by configuration");
    }

    reporter.success("All done! Your Python development environment is ready.");
    Ok(())
}

pub fn install_uv(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.stage("uv", 1, 1);
    uv::install(cfg, reporter)
}

pub fn install_python(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.stage("python", 1, 1);
    python::install(cfg, reporter)
}

pub fn install_vscode(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.stage("vscode", 1, 1);
    let code = vscode::install(cfg, reporter)?;
    extensions::install(cfg, &code, reporter)
}

pub fn install_extensions(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.stage("extensions", 1, 1);
    let code = vscode::locate_code()
        .ok_or_else(|| Error::msg("VSCode not found; install VSCode first"))?;
    extensions::install(cfg, &code, reporter)
}

pub fn apply_path(cfg: &Config, reporter: &dyn Reporter) -> Result<()> {
    reporter.stage("path", 1, 1);
    pathenv::apply(cfg, reporter).map(|_| ())
}
