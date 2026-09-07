use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticTokens {
    pub data: Vec<u32>,
}

pub mod semantic_token_type {
    pub const NAMESPACE: &str = "namespace";
    pub const CLASS: &str = "class";
    pub const ENUM: &str = "enum";
    pub const INTERFACE: &str = "interface";
    pub const STRUCT: &str = "struct";
    pub const TYPE_PARAMETER: &str = "typeParameter";
    pub const TYPE: &str = "type";
    pub const PARAMETER: &str = "parameter";
    pub const VARIABLE: &str = "variable";
    pub const PROPERTY: &str = "property";
    pub const ENUM_MEMBER: &str = "enumMember";
    pub const DECORATOR: &str = "decorator";
    pub const EVENT: &str = "event";
    pub const FUNCTION: &str = "function";
    pub const METHOD: &str = "method";
    pub const MACRO: &str = "macro";
    pub const LABEL: &str = "label";
    pub const COMMENT: &str = "comment";
    pub const STRING: &str = "string";
    pub const KEYWORD: &str = "keyword";
    pub const NUMBER: &str = "number";
    pub const REGEXP: &str = "regexp";
    pub const OPERATOR: &str = "operator";
}

pub mod semantic_token_modifier {
    pub const DECLARATION: &str = "declaration";
    pub const DEFINITION: &str = "definition";
    pub const READONLY: &str = "readonly";
    pub const STATIC: &str = "static";
    pub const DEPRECATED: &str = "deprecated";
    pub const ABSTRACT: &str = "abstract";
    pub const ASYNC: &str = "async";
    pub const MODIFICATION: &str = "modification";
    pub const DOCUMENTATION: &str = "documentation";
    pub const DEFAULT_LIBRARY: &str = "defaultLibrary";
    pub const LOCAL: &str = "local";
}
