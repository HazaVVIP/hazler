use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Regex compilation error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Source map too large: {0} bytes")]
    SourceMapTooLarge(usize),
}

pub type Result<T> = std::result::Result<T, Error>;
