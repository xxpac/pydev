use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::platform::{self, Os};
use crate::progress::Reporter;
use crate::runner;

pub const MARKER_BEGIN: &str = "# >>> pydev >>>";
pub const MARKER_END: &str = "# <<< pydev <<<";

/// What the PATH update will change, for display before applying.
#[derive(Debug, Clone, Serialize)]
pub struct PathPreview {
    /// Directories that will be added to PATH.
    pub entries: Vec<String>,
    /// Human-readable targets that will be modified (profile files or registry).
    pub targets: Vec<String>,
}

/// Current state of the PATH configuration.
#[derive(Debug, Clone, Serialize)]
pub struct PathStatus {
    /// True when every required entry/target is already in place.
    pub configured: bool,
    /// Targets (files / registry) that still need updating.
    pub pending_targets: Vec<String>,
}

/// Directories that must be on PATH for the installed tools to be found.
fn path_entries() -> Result<Vec<String>> {
    let mut entries = vec![platform::user_bin_dir()?.to_string_lossy().to_string()];
    if platform::current_os() == Os::Windows {
        if let Some(local) = dirs::data_local_dir() {
            entries.push(
                local
                    .join("Programs")
                    .join("Microsoft VS Code")
                    .join("bin")
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    Ok(entries)
}

#[derive(Clone, Copy)]
enum ShellKind {
    Posix,
    Fish,
}

/// Map a shell key from config to its profile file and syntax.
fn resolve_shell(shell: &str) -> Option<(PathBuf, ShellKind)> {
    let home = dirs::home_dir()?;
    match shell.trim().to_lowercase().as_str() {
        "bash" | "bashrc" => Some((home.join(".bashrc"), ShellKind::Posix)),
        "zsh" | "zshrc" => Some((home.join(".zshrc"), ShellKind::Posix)),
        "profile" => Some((home.join(".profile"), ShellKind::Posix)),
        "fish" => Some((
            home.join(".config").join("fish").join("config.fish"),
            ShellKind::Fish,
        )),
        _ => None,
    }
}

pub fn preview(cfg: &Config) -> Result<PathPreview> {
    let entries = path_entries()?;
    let targets = match platform::current_os() {
        Os::Windows => vec!["Windows user PATH (registry: HKCU\\Environment)".to_string()],
        _ => cfg
            .path
            .shells
            .iter()
            .filter_map(|s| resolve_shell(s).map(|(p, _)| p.to_string_lossy().to_string()))
            .collect(),
    };
    Ok(PathPreview { entries, targets })
}

/// Probe whether the required PATH entries are already configured.
pub fn status(cfg: &Config) -> Result<PathStatus> {
    let entries = path_entries()?;
    match platform::current_os() {
        Os::Windows => {
            let current = runner::capture(
                "powershell",
                &[
                    "-NoProfile",
                    "-Command",
                    "[Environment]::GetEnvironmentVariable('Path','User')",
                ],
                &[],
            )
            .unwrap_or_default();
            let parts: Vec<String> = current
                .split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let missing = entries
                .iter()
                .any(|e| !parts.iter().any(|p| p.eq_ignore_ascii_case(e)));
            Ok(PathStatus {
                configured: !missing,
                pending_targets: if missing {
                    vec!["Windows user PATH".to_string()]
                } else {
                    vec![]
                },
            })
        }
        _ => {
            let mut pending = Vec::new();
            let mut any = false;
            for shell in &cfg.path.shells {
                if let Some((file, _)) = resolve_shell(shell) {
                    any = true;
                    let existing = std::fs::read_to_string(&file).unwrap_or_default();
                    if !existing.contains(MARKER_BEGIN) {
                        pending.push(file.to_string_lossy().to_string());
                    }
                }
            }
            Ok(PathStatus {
                configured: any && pending.is_empty(),
                pending_targets: pending,
            })
        }
    }
}

/// Apply the PATH updates. Returns the list of modified targets.
pub fn apply(cfg: &Config, reporter: &dyn Reporter) -> Result<Vec<String>> {
    let entries = path_entries()?;
    reporter.info(&format!("Ensuring PATH contains: {}", entries.join(", ")));

    match platform::current_os() {
        Os::Windows => apply_windows(&entries, reporter),
        _ => apply_unix(cfg, &entries, reporter),
    }
}

fn apply_unix(cfg: &Config, entries: &[String], reporter: &dyn Reporter) -> Result<Vec<String>> {
    let mut modified = Vec::new();
    for shell in &cfg.path.shells {
        let Some((file, kind)) = resolve_shell(shell) else {
            reporter.warn(&format!("Unknown shell '{shell}', skipping"));
            continue;
        };

        let existing = std::fs::read_to_string(&file).unwrap_or_default();
        if existing.contains(MARKER_BEGIN) {
            reporter.info(&format!("{} already configured", file.display()));
            continue;
        }

        let mut block = format!("\n{MARKER_BEGIN}\n# Added by pydev: PATH for uv, Python and VSCode\n");
        for dir in entries {
            match kind {
                ShellKind::Posix => block.push_str(&format!("export PATH=\"{dir}:$PATH\"\n")),
                ShellKind::Fish => block.push_str(&format!("set -gx PATH \"{dir}\" $PATH\n")),
            }
        }
        block.push_str(&format!("{MARKER_END}\n"));

        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = OpenOptions::new().create(true).append(true).open(&file)?;
        f.write_all(block.as_bytes())?;
        reporter.success(&format!("Updated {}", file.display()));
        modified.push(file.to_string_lossy().to_string());
    }
    if modified.is_empty() {
        reporter.warn("No shell profiles were updated");
    } else {
        reporter.info("Open a new terminal (or `source` the profile) to use the new PATH");
    }
    Ok(modified)
}

fn apply_windows(entries: &[String], reporter: &dyn Reporter) -> Result<Vec<String>> {
    // Build a PowerShell array literal, escaping single quotes.
    let array = entries
        .iter()
        .map(|e| format!("'{}'", e.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        r#"$dirs = @({array})
$p = [Environment]::GetEnvironmentVariable('Path','User')
if ($null -eq $p) {{ $p = '' }}
foreach ($d in $dirs) {{
  $parts = $p.Split(';') | Where-Object {{ $_ -ne '' }}
  if (-not ($parts -contains $d)) {{
    if ($p -ne '') {{ $p = $p + ';' + $d }} else {{ $p = $d }}
  }}
}}
[Environment]::SetEnvironmentVariable('Path', $p, 'User')
Write-Output 'pydev: user PATH updated (open a new terminal to use it)'
"#
    );

    let script_path = std::env::temp_dir().join("pydev-path.ps1");
    std::fs::write(&script_path, script)?;
    runner::run(
        "powershell",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ],
        &[],
        reporter,
    )
    .map_err(|e| Error::msg(format!("failed to update Windows PATH: {e}")))?;

    reporter.success("Windows user PATH updated");
    Ok(vec!["Windows user PATH".to_string()])
}
