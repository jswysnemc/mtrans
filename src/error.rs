use thiserror::Error;

pub type Result<T> = std::result::Result<T, MtransError>;

#[derive(Error, Debug)]
pub enum MtransError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Prompt error: {0}")]
    Prompt(String),

    #[error("Translation error: {0}")]
    Translation(String),
}

impl From<arboard::Error> for MtransError {
    fn from(err: arboard::Error) -> Self {
        MtransError::Clipboard(err.to_string())
    }
}
