use serde::Serialize;

/// Severity / kind of a streamed log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Success,
    /// A command that is about to run (echoed for transparency).
    Cmd,
}

/// A single line of streamed output.
#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub level: LogLevel,
    pub message: String,
}

/// High-level installation step, used to drive a progress indicator.
#[derive(Debug, Clone, Serialize)]
pub struct Stage {
    /// Stable machine key, e.g. "uv", "python", "vscode", "path".
    pub key: String,
    /// 1-based index of the current step.
    pub index: u32,
    /// Total number of steps in the running sequence.
    pub total: u32,
}

/// Sink for log lines and progress updates.
///
/// The GUI implements this by emitting Tauri events; the CLI implements it by
/// printing to the terminal. Kept object-safe so `&dyn Reporter` can be passed
/// down into the (blocking) core routines, including across scoped threads.
pub trait Reporter: Send + Sync {
    fn emit_log(&self, level: LogLevel, message: &str);
    fn emit_stage(&self, stage: Stage);

    fn info(&self, message: &str) {
        self.emit_log(LogLevel::Info, message);
    }
    fn warn(&self, message: &str) {
        self.emit_log(LogLevel::Warn, message);
    }
    fn error(&self, message: &str) {
        self.emit_log(LogLevel::Error, message);
    }
    fn success(&self, message: &str) {
        self.emit_log(LogLevel::Success, message);
    }
    fn cmd(&self, message: &str) {
        self.emit_log(LogLevel::Cmd, message);
    }
    fn stage(&self, key: &str, index: u32, total: u32) {
        self.emit_stage(Stage {
            key: key.to_string(),
            index,
            total,
        });
    }
}

/// A reporter that discards everything. Handy for tests / non-interactive calls.
pub struct NullReporter;

impl Reporter for NullReporter {
    fn emit_log(&self, _level: LogLevel, _message: &str) {}
    fn emit_stage(&self, _stage: Stage) {}
}
