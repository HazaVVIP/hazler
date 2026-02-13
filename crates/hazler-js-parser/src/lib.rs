mod error;
mod framework;
mod parser;

pub use error::{Error, Result};
pub use framework::{detect_framework, get_framework_patterns, Framework};
pub use parser::{FrameFileParser, JavaScriptParser};
