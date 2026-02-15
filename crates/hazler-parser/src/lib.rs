pub mod error;
pub mod graphql;
pub mod parser;

pub use error::{Error, Result};
pub use graphql::{GraphQLParser, GraphQLSchema, GraphQLEndpoint, GraphQLField, GraphQLType, GraphQLArgument};
pub use parser::HtmlParser;
