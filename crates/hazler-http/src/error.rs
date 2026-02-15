use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout")]
    Timeout,

    #[error("Too many redirects")]
    TooManyRedirects,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;
