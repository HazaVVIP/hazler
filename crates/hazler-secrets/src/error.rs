use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("Pattern compilation error: {0}")]
    PatternError(String),

    #[error("Scanning error: {0}")]
    ScanError(String),
}
