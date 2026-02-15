mod error;
mod framework;
mod parser;
mod sourcemap;

pub use error::{Error, Result};
pub use framework::{detect_framework, get_framework_patterns, Framework};
pub use parser::{FrameFileParser, JavaScriptParser};
pub use sourcemap::{
    InterestingPath, PathCategory, Priority, SourceMap, SourceMapAnalysis, SourceMapParser,
    SourceMapReference,
};
