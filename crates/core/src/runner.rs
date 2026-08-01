use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};
use crate::progress::{LogLevel, Reporter};

/// A key/value environment pair for a child process.
pub type EnvPair = (String, String);

/// Run a command, streaming stdout/stderr to the reporter line by line.
///
/// Returns an error if the process cannot be started or exits non-zero.
pub fn run(
    program: &str,
    args: &[&str],
    envs: &[EnvPair],
    reporter: &dyn Reporter,
) -> Result<()> {
    run_in(program, args, envs, None, reporter)
}

pub fn run_in(
    program: &str,
    args: &[&str],
    envs: &[EnvPair],
    cwd: Option<&Path>,
    reporter: &dyn Reporter,
) -> Result<()> {
    reporter.cmd(&format!("$ {} {}", program, args.join(" ")));

    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        command.env(k, v);
    }
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let mut child = command
        .spawn()
        .map_err(|e| Error::msg(format!("failed to start `{program}`: {e}")))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Borrow the reporter across scoped threads (Reporter: Send + Sync).
    std::thread::scope(|scope| {
        if let Some(err) = stderr {
            scope.spawn(move || {
                for line in BufReader::new(err).lines().map_while(std::result::Result::ok) {
                    // Many well-behaved tools log progress to stderr, so treat
                    // these as informational; exit status decides success.
                    reporter.emit_log(LogLevel::Info, &line);
                }
            });
        }
        if let Some(out) = stdout {
            for line in BufReader::new(out).lines().map_while(std::result::Result::ok) {
                reporter.emit_log(LogLevel::Info, &line);
            }
        }
    });

    let status = child.wait()?;
    if !status.success() {
        return Err(Error::Command {
            cmd: program.to_string(),
            code: status.code().unwrap_or(-1),
        });
    }
    Ok(())
}

/// Run a command and capture its stdout as a string (no streaming).
pub fn capture(program: &str, args: &[&str], envs: &[EnvPair]) -> Result<String> {
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    for (k, v) in envs {
        command.env(k, v);
    }
    let output = command
        .output()
        .map_err(|e| Error::msg(format!("failed to start `{program}`: {e}")))?;
    if !output.status.success() {
        return Err(Error::Command {
            cmd: program.to_string(),
            code: output.status.code().unwrap_or(-1),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
