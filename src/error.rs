use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("Win32 API error: {0}")]
    Win32(#[from] windows::core::Error),

    #[error("hotkey error: {0}")]
    Hotkey(String),

    #[error("window error: {0}")]
    Window(String),

    #[error("monitor error: {0}")]
    Monitor(String),

    #[error("layout error: {0}")]
    Layout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown action: {0}")]
    UnknownAction(String),

    #[error("service error: {0}")]
    Service(String),

    #[error("{0}")]
    General(String),
}

pub type AppResult<T> = Result<T, AppError>;
