use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchError(String),

    #[error("Failed to create new page: {0}")]
    PageCreationError(String),

    #[error("Navigation failed: {0}")]
    NavigationError(String),

    #[error("Request interception error: {0}")]
    InterceptionError(String),

    #[error("Screenshot error: {0}")]
    ScreenshotError(String),

    #[error("Cookie error: {0}")]
    CookieError(String),

    #[error("JavaScript execution error: {0}")]
    JsExecutionError(String),

    #[error("Browser timeout: {0}")]
    TimeoutError(String),

    #[error("Browser error: {0}")]
    BrowserError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, BrowserError>;
