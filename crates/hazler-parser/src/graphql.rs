use crate::error::Result;
use serde::{Deserialize, Serialize};
use url::Url;

/// GraphQL endpoint detection and introspection
#[derive(Clone)]
pub struct GraphQLParser {
    /// Maximum schema size to process (in bytes)
    max_schema_size: usize,
}

/// GraphQL introspection query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchema {
    pub endpoint: String,
    pub types: Vec<GraphQLType>,
    pub queries: Vec<GraphQLField>,
    pub mutations: Vec<GraphQLField>,
    pub subscriptions: Vec<GraphQLField>,
}

/// GraphQL type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLType {
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    pub fields: Vec<GraphQLField>,
}

/// GraphQL field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLField {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<GraphQLArgument>,
    pub type_info: String,
}

/// GraphQL field argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLArgument {
    pub name: String,
    pub type_info: String,
    pub description: Option<String>,
}

/// GraphQL endpoint detection result
#[derive(Debug, Clone)]
pub struct GraphQLEndpoint {
    pub url: Url,
    pub confidence: f32,
    pub indicators: Vec<String>,
}

impl GraphQLParser {
    /// Create a new GraphQL parser
    pub fn new() -> Self {
        Self {
            max_schema_size: 10 * 1024 * 1024, // 10 MB default
        }
    }

    /// Create a new GraphQL parser with custom max schema size
    pub fn with_max_schema_size(max_size: usize) -> Self {
        Self {
            max_schema_size: max_size,
        }
    }

    /// Detect potential GraphQL endpoints from URL or content
    pub fn detect_graphql_endpoint(&self, url: &Url, content: &str) -> Option<GraphQLEndpoint> {
        let mut confidence = 0.0;
        let mut indicators = Vec::new();

        // Check URL path
        let path = url.path().to_lowercase();
        if path.contains("graphql") || path.contains("gql") {
            confidence += 0.5;
            indicators.push("URL contains 'graphql' or 'gql'".to_string());
        }

        // Check content for GraphQL indicators
        if content.contains("__schema") || content.contains("__type") {
            confidence += 0.4;
            indicators.push("Content contains introspection keywords".to_string());
        }

        if content.contains("query") && content.contains("mutation") {
            confidence += 0.2;
            indicators.push("Content contains GraphQL operation keywords".to_string());
        }

        // Check for GraphQL error patterns
        if content.contains("GraphQL") || content.contains("graphql-go") || content.contains("apollo") {
            confidence += 0.2;
            indicators.push("Content mentions GraphQL libraries".to_string());
        }

        // Check for typical GraphQL response structure
        if content.contains(r#""data":"#) || (content.contains(r#""data""#) && content.contains(r#""errors""#)) {
            confidence += 0.15;
            indicators.push("Content has GraphQL response structure".to_string());
        }

        if confidence >= 0.5 {
            Some(GraphQLEndpoint {
                url: url.clone(),
                confidence,
                indicators,
            })
        } else {
            None
        }
    }

    /// Generate a GraphQL introspection query
    pub fn generate_introspection_query(&self) -> String {
        r#"{
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types {
      ...FullType
    }
  }
}

fragment FullType on __Type {
  kind
  name
  description
  fields(includeDeprecated: true) {
    name
    description
    args {
      ...InputValue
    }
    type {
      ...TypeRef
    }
  }
  inputFields {
    ...InputValue
  }
  interfaces {
    ...TypeRef
  }
  enumValues(includeDeprecated: true) {
    name
    description
  }
  possibleTypes {
    ...TypeRef
  }
}

fragment InputValue on __InputValue {
  name
  description
  type {
    ...TypeRef
  }
  defaultValue
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
              }
            }
          }
        }
      }
    }
  }
}"#
        .to_string()
    }

    /// Parse GraphQL introspection response
    pub fn parse_introspection_response(&self, response: &str) -> Result<GraphQLSchema> {
        // Parse the JSON response
        let json: serde_json::Value = serde_json::from_str(response)?;

        // Extract schema data
        let schema = json
            .get("data")
            .and_then(|d| d.get("__schema"))
            .ok_or_else(|| crate::error::Error::ParseError("Invalid introspection response".into()))?;

        // Extract types
        let types: Vec<GraphQLType> = schema
            .get("types")
            .and_then(|t| t.as_array())
            .map(|types_array| {
                types_array
                    .iter()
                    .filter_map(|type_obj| self.parse_type(type_obj))
                    .collect()
            })
            .unwrap_or_default();

        // Extract root operation types
        let query_type_name = schema
            .get("queryType")
            .and_then(|q| q.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Query");

        let mutation_type_name = schema
            .get("mutationType")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str());

        let subscription_type_name = schema
            .get("subscriptionType")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str());

        // Find and extract queries
        let queries = types
            .iter()
            .find(|t| t.name == query_type_name)
            .map(|t| t.fields.clone())
            .unwrap_or_default();

        // Find and extract mutations
        let mutations = mutation_type_name
            .and_then(|name| types.iter().find(|t| t.name == name))
            .map(|t| t.fields.clone())
            .unwrap_or_default();

        // Find and extract subscriptions
        let subscriptions = subscription_type_name
            .and_then(|name| types.iter().find(|t| t.name == name))
            .map(|t| t.fields.clone())
            .unwrap_or_default();

        Ok(GraphQLSchema {
            endpoint: String::new(),
            types,
            queries,
            mutations,
            subscriptions,
        })
    }

    /// Parse a GraphQL type from JSON
    fn parse_type(&self, type_obj: &serde_json::Value) -> Option<GraphQLType> {
        let name = type_obj.get("name")?.as_str()?.to_string();
        let kind = type_obj.get("kind")?.as_str()?.to_string();
        let description = type_obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let fields = type_obj
            .get("fields")
            .and_then(|f| f.as_array())
            .map(|fields_array| {
                fields_array
                    .iter()
                    .filter_map(|field_obj| self.parse_field(field_obj))
                    .collect()
            })
            .unwrap_or_default();

        Some(GraphQLType {
            name,
            kind,
            description,
            fields,
        })
    }

    /// Parse a GraphQL field from JSON
    fn parse_field(&self, field_obj: &serde_json::Value) -> Option<GraphQLField> {
        let name = field_obj.get("name")?.as_str()?.to_string();
        let description = field_obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let type_info = self.extract_type_info(field_obj.get("type")?);

        let args = field_obj
            .get("args")
            .and_then(|a| a.as_array())
            .map(|args_array| {
                args_array
                    .iter()
                    .filter_map(|arg_obj| self.parse_argument(arg_obj))
                    .collect()
            })
            .unwrap_or_default();

        Some(GraphQLField {
            name,
            description,
            args,
            type_info,
        })
    }

    /// Parse a GraphQL argument from JSON
    fn parse_argument(&self, arg_obj: &serde_json::Value) -> Option<GraphQLArgument> {
        let name = arg_obj.get("name")?.as_str()?.to_string();
        let description = arg_obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let type_info = self.extract_type_info(arg_obj.get("type")?);

        Some(GraphQLArgument {
            name,
            description,
            type_info,
        })
    }

    /// Extract type information recursively
    fn extract_type_info(&self, type_obj: &serde_json::Value) -> String {
        let kind = type_obj.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let name = type_obj.get("name").and_then(|n| n.as_str());

        match kind {
            "NON_NULL" => {
                if let Some(of_type) = type_obj.get("ofType") {
                    format!("{}!", self.extract_type_info(of_type))
                } else {
                    "Unknown!".to_string()
                }
            }
            "LIST" => {
                if let Some(of_type) = type_obj.get("ofType") {
                    format!("[{}]", self.extract_type_info(of_type))
                } else {
                    "[Unknown]".to_string()
                }
            }
            _ => name.unwrap_or("Unknown").to_string(),
        }
    }

    /// Generate sample queries from schema
    pub fn generate_sample_queries(&self, schema: &GraphQLSchema) -> Vec<String> {
        let mut samples = Vec::new();

        // Generate samples for queries
        for query in &schema.queries {
            if let Some(sample) = self.generate_sample_operation("query", &query.name, &query.args) {
                samples.push(sample);
            }
        }

        samples
    }

    /// Generate sample mutations from schema
    pub fn generate_sample_mutations(&self, schema: &GraphQLSchema) -> Vec<String> {
        let mut samples = Vec::new();

        // Generate samples for mutations
        for mutation in &schema.mutations {
            if let Some(sample) = self.generate_sample_operation("mutation", &mutation.name, &mutation.args) {
                samples.push(sample);
            }
        }

        samples
    }

    /// Generate a sample operation (query or mutation)
    fn generate_sample_operation(&self, op_type: &str, name: &str, args: &[GraphQLArgument]) -> Option<String> {
        if args.is_empty() {
            Some(format!("{} {{\n  {}\n}}", op_type, name))
        } else {
            let args_str = args
                .iter()
                .map(|arg| {
                    let example_value = self.generate_example_value(&arg.type_info);
                    format!("{}: {}", arg.name, example_value)
                })
                .collect::<Vec<_>>()
                .join(", ");

            Some(format!("{} {{\n  {}({})\n}}", op_type, name, args_str))
        }
    }

    /// Generate example value for a GraphQL type
    fn generate_example_value(&self, type_info: &str) -> String {
        if type_info.ends_with('!') {
            let inner = &type_info[..type_info.len() - 1];
            return self.generate_example_value(inner);
        }

        if type_info.starts_with('[') && type_info.ends_with(']') {
            return "[]".to_string();
        }

        match type_info {
            "String" => r#""example""#.to_string(),
            "Int" => "1".to_string(),
            "Float" => "1.0".to_string(),
            "Boolean" => "true".to_string(),
            "ID" => r#""1""#.to_string(),
            _ => "null".to_string(),
        }
    }
}

impl Default for GraphQLParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_graphql_endpoint_by_url() {
        let parser = GraphQLParser::new();
        let url = Url::parse("https://api.example.com/graphql").unwrap();
        let result = parser.detect_graphql_endpoint(&url, "");

        assert!(result.is_some());
        let endpoint = result.unwrap();
        assert!(endpoint.confidence >= 0.5);
        assert!(!endpoint.indicators.is_empty());
    }

    #[test]
    fn test_detect_graphql_endpoint_by_content() {
        let parser = GraphQLParser::new();
        let url = Url::parse("https://api.example.com/api").unwrap();
        let content = r#"{"data": {"__schema": {}}}"#;
        let result = parser.detect_graphql_endpoint(&url, content);

        assert!(result.is_some());
    }

    #[test]
    fn test_generate_introspection_query() {
        let parser = GraphQLParser::new();
        let query = parser.generate_introspection_query();

        assert!(query.contains("__schema"));
        assert!(query.contains("queryType"));
        assert!(query.contains("mutationType"));
    }

    #[test]
    fn test_parse_introspection_response() {
        let parser = GraphQLParser::new();
        let response = r#"{
            "data": {
                "__schema": {
                    "queryType": { "name": "Query" },
                    "mutationType": { "name": "Mutation" },
                    "subscriptionType": null,
                    "types": [
                        {
                            "kind": "OBJECT",
                            "name": "Query",
                            "description": "Root query type",
                            "fields": [
                                {
                                    "name": "user",
                                    "description": "Get user by ID",
                                    "args": [
                                        {
                                            "name": "id",
                                            "description": "User ID",
                                            "type": {
                                                "kind": "NON_NULL",
                                                "name": null,
                                                "ofType": {
                                                    "kind": "SCALAR",
                                                    "name": "ID"
                                                }
                                            }
                                        }
                                    ],
                                    "type": {
                                        "kind": "OBJECT",
                                        "name": "User"
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let result = parser.parse_introspection_response(response);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.queries.len(), 1);
        assert_eq!(schema.queries[0].name, "user");
        assert_eq!(schema.queries[0].args.len(), 1);
        assert_eq!(schema.queries[0].args[0].name, "id");
        assert_eq!(schema.queries[0].args[0].type_info, "ID!");
    }

    #[test]
    fn test_generate_sample_queries() {
        let parser = GraphQLParser::new();
        let schema = GraphQLSchema {
            endpoint: String::new(),
            types: vec![],
            queries: vec![
                GraphQLField {
                    name: "users".to_string(),
                    description: None,
                    args: vec![],
                    type_info: "[User]".to_string(),
                },
                GraphQLField {
                    name: "user".to_string(),
                    description: None,
                    args: vec![GraphQLArgument {
                        name: "id".to_string(),
                        type_info: "ID!".to_string(),
                        description: None,
                    }],
                    type_info: "User".to_string(),
                },
            ],
            mutations: vec![],
            subscriptions: vec![],
        };

        let samples = parser.generate_sample_queries(&schema);
        assert_eq!(samples.len(), 2);
        assert!(samples[0].contains("users"));
        assert!(samples[1].contains("user"));
        assert!(samples[1].contains("id:"));
    }

    #[test]
    fn test_extract_type_info_non_null() {
        let parser = GraphQLParser::new();
        let type_obj = serde_json::json!({
            "kind": "NON_NULL",
            "ofType": {
                "kind": "SCALAR",
                "name": "String"
            }
        });

        let type_info = parser.extract_type_info(&type_obj);
        assert_eq!(type_info, "String!");
    }

    #[test]
    fn test_extract_type_info_list() {
        let parser = GraphQLParser::new();
        let type_obj = serde_json::json!({
            "kind": "LIST",
            "ofType": {
                "kind": "SCALAR",
                "name": "Int"
            }
        });

        let type_info = parser.extract_type_info(&type_obj);
        assert_eq!(type_info, "[Int]");
    }
}
