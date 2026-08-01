use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use pydev_core::config::Config;
use pydev_core::progress::{LogLevel, Reporter, Stage};
use pydev_core::{netcheck, orchestrate, pathenv, status};
use pydev_core::installers::python;

/// pydev - set up a Python development environment (uv + Python + VSCode).
#[derive(Parser)]
#[command(name = "pydev-cli", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install everything (or a single component with --only).
    Install {
        /// Path to a config.toml (defaults to built-in defaults if omitted).
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Install just one component instead of the full flow.
        #[arg(long, value_enum)]
        only: Option<Component>,
        /// Reinstall/update components even if they are already present.
        #[arg(long)]
        reinstall: bool,
    },
    /// Show which components are already installed and their versions.
    Status {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Test network reachability (through the configured proxy, if any).
    TestNetwork {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// List the Python versions that can be installed.
    ListPython {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Show which PATH entries and files/registry would be changed.
    PathPreview {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Write a starter config.toml with the default settings.
    Init {
        /// Output path (default: ./config.toml).
        #[arg(short, long, default_value = "config.toml")]
        output: PathBuf,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum Component {
    Uv,
    Python,
    Vscode,
    Extensions,
    Path,
}

/// Reporter that prints streamed output to the terminal.
struct CliReporter;

impl Reporter for CliReporter {
    fn emit_log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => eprintln!("[error] {message}"),
            LogLevel::Warn => eprintln!("[warn ] {message}"),
            LogLevel::Success => println!("[ ok  ] {message}"),
            LogLevel::Cmd => println!("       {message}"),
            LogLevel::Info => println!("        {message}"),
        }
    }

    fn emit_stage(&self, stage: Stage) {
        println!("\n== [{}/{}] {} ==", stage.index, stage.total, stage.key);
    }
}

fn fmt_tool(t: &status::Tool) -> String {
    if !t.installed {
        return "not installed".to_string();
    }
    let cur = t.current.as_deref().unwrap_or("installed");
    if t.upgrade_available {
        if let Some(latest) = &t.latest {
            return format!("{cur}  (latest {latest} - upgrade available)");
        }
    }
    cur.to_string()
}

fn load_config(path: &Option<PathBuf>) -> Result<Config> {
    match path {
        Some(p) => Config::load_from_path(p)
            .with_context(|| format!("failed to load config from {}", p.display())),
        None => Ok(Config::default()),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let reporter = CliReporter;

    match cli.command {
        Command::Install {
            config,
            only,
            reinstall,
        } => {
            let mut cfg = load_config(&config)?;
            if reinstall {
                cfg.reinstall_existing = true;
            }
            match only {
                None => orchestrate::one_click(&cfg, &reporter)?,
                Some(Component::Uv) => orchestrate::install_uv(&cfg, &reporter)?,
                Some(Component::Python) => orchestrate::install_python(&cfg, &reporter)?,
                Some(Component::Vscode) => orchestrate::install_vscode(&cfg, &reporter)?,
                Some(Component::Extensions) => orchestrate::install_extensions(&cfg, &reporter)?,
                Some(Component::Path) => orchestrate::apply_path(&cfg, &reporter)?,
            }
        }
        Command::Status { config } => {
            let cfg = load_config(&config)?;
            let st = status::detect(&cfg, true);
            println!("uv:      {}", fmt_tool(&st.uv));
            println!(
                "Python:  requested {} -> {}",
                st.python.requested,
                if st.python.satisfied { "satisfied" } else { "MISSING" }
            );
            if !st.python.installed_versions.is_empty() {
                println!("         installed: {}", st.python.installed_versions.join(", "));
            }
            println!("VSCode:  {}", fmt_tool(&st.vscode));
            println!("Extensions:");
            for e in &st.extensions {
                let mark = if e.installed { "x" } else { " " };
                let ver = e
                    .version
                    .as_deref()
                    .map(|v| format!(" ({v})"))
                    .unwrap_or_default();
                let up = if e.upgrade_available {
                    format!(" -> {} available", e.latest.as_deref().unwrap_or("latest"))
                } else {
                    String::new()
                };
                println!("  [{mark}] {}{ver}{up}", e.id);
            }
            if st.path.configured {
                println!("PATH:    configured");
            } else {
                println!(
                    "PATH:    pending ({} target(s))",
                    st.path.pending_targets.len()
                );
            }
        }
        Command::TestNetwork { config } => {
            let cfg = load_config(&config)?;
            let results = netcheck::run(&cfg, &reporter)?;
            let reachable = results.iter().filter(|r| r.ok).count();
            println!("\n{reachable}/{} endpoints reachable", results.len());
            if reachable < results.len() {
                println!("If you are behind a firewall, set [proxy] in your config.");
            }
        }
        Command::ListPython { config } => {
            let cfg = load_config(&config)?;
            println!("Installable Python versions:");
            for v in python::list_versions(&cfg) {
                println!("  {v}");
            }
        }
        Command::PathPreview { config } => {
            let cfg = load_config(&config)?;
            let preview = pathenv::preview(&cfg)?;
            println!("PATH entries to add:");
            for e in &preview.entries {
                println!("  {e}");
            }
            println!("Targets to modify:");
            for t in &preview.targets {
                println!("  {t}");
            }
        }
        Command::Init { output } => {
            let cfg = Config::default();
            cfg.save_to_path(&output)?;
            println!("Wrote default config to {}", output.display());
        }
    }

    Ok(())
}
