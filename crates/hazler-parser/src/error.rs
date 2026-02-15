use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
