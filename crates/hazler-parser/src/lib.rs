pub mod error;
pub mod graphql;
pub mod parser;

pub use error::{Error, Result};
pub use graphql::{
    GraphQLArgument, GraphQLEndpoint, GraphQLField, GraphQLParser, GraphQLSchema, GraphQLType,
};
pub use parser::HtmlParser;
