mod error;
mod framework;
mod parser;
mod sourcemap;

pub use error::{Error, Result};
pub use framework::{detect_framework, get_framework_patterns, Framework};
pub use parser::{FrameFileParser, JavaScriptParser};
pub use sourcemap::{
    SourceMapParser, SourceMap, SourceMapReference, SourceMapAnalysis,
    InterestingPath, PathCategory, Priority,
};
