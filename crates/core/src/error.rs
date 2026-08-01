use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("config parse error: {0}")]
    ConfigParse(String),

    #[error("config write error: {0}")]
    ConfigWrite(String),

    #[error("command `{cmd}` exited with code {code}")]
    Command { cmd: String, code: i32 },

    #[error("unsupported platform: {0}")]
    Unsupported(String),

    #[error("{0}")]
    Msg(String),
}

impl Error {
    pub fn msg(m: impl Into<String>) -> Self {
        Error::Msg(m.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
