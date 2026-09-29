#![allow(unused_imports, dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub use tsox_frontend::format::mig::m4o::{
    CommentSeparator, EmitFlags, ListFormat, TokenEmitFlags, WriteKind, flatten_comma_elements,
};
pub use tsox_frontend::format::mig::m4o_2::{
    CommentState, DetachedCommentsInfo, EmitContext, PrinterOptions, SourceMapState,
    can_emit_simple_arrow_head,
};
pub use tsox_frontend::format::mig::m4t_4::{greatest_end, is_binary_operation};
pub use tsox_frontend::format::mig::m4t_5::calculate_indent;

pub use tsox_frontend::ast::mig::m3e_3::{OperatorPrecedence, TypePrecedence};
pub use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
pub use tsox_frontend::ast::mig::m3g::is_modifier;
pub use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
pub use tsox_frontend::ast::mig::m3g_3::{
    FindAncestorResult, skip_parentheses, to_find_ancestor_result,
};
pub use tsox_frontend::ast::node_data_generated::{
    is_decorator, is_keyword_kind, is_module_declaration, is_not_emitted_statement,
    is_partially_emitted_expression, is_punctuation_kind,
};
pub use tsox_frontend::ast::visitor::{NodeVisitor, NodeVisitorHooks};

pub use tsox_core::collections::set::Set;
pub use tsox_core::core::core::{concatenate, every, splice};
pub use tsox_core::core::mig::m3j::{compute_ecma_line_starts, compute_ecma_line_starts_seq};
pub use tsox_core::core::mig::m3j_3::get_new_line_kind;
pub use tsox_core::core::text::{TextPos, TextRange};

pub use crate::printer::NameGenerator;
pub use crate::printer::generated_identifier_flags::NodeFactory;
pub use crate::printer::mig::m4m_2::{SnippetElement, SynthesizedComment};
pub use crate::mig::m4n::SnippetKind;
pub use crate::mig::m4g::r33k7_defs::{PrivateIdentifierKind, has_decorators};
pub use crate::mig::m4h_2::is_class_named_evaluation_helper_block;
pub use tsox_frontend::ast::utilities::skip_partially_emitted_expressions_arc as skip_partially_emitted_expressions;

pub fn position_is_synthesized(pos: usize) -> bool {
    (pos as i32) < 0
}

pub fn is_private_identifier_class_element_declaration(member: &Node) -> bool {
    use tsox_frontend::ast::node_data_generated as g;
    g::is_property_declaration(member)
        || g::is_method_declaration(member)
        || g::is_get_accessor_declaration(member)
        || g::is_set_accessor_declaration(member)
}
pub use crate::mig::m4r_4::format_synthesized_comment;
pub use tsox_frontend::scanner::mig::w1::compute_line_of_position;

pub fn new_text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(start, end)
}

pub fn set_parent_in_children(node: &Arc<Node>) {
    tsox_frontend::ast::mig::m3g_3::set_parent_in_children(node)
}

pub fn append_if_unique_vec<T: Clone + PartialEq>(v: &mut Vec<T>, value: T) {
    if !v.contains(&value) {
        v.push(value);
    }
}

pub struct PrinterState {
    pub comment_state: Option<CommentState>,
    pub source_map_state: Option<SourceMapState>,
}

pub type ListFlags = i32;

pub const LF_NONE: ListFlags = 0;
pub const LF_SINGLE_LINE: ListFlags = 0;
pub const LF_MULTI_LINE: ListFlags = 1 << 0;
pub const LF_PRESERVE_LINES: ListFlags = 1 << 1;
pub const LF_LINES_MASK: ListFlags = LF_SINGLE_LINE | LF_MULTI_LINE | LF_PRESERVE_LINES;
pub const LF_BAR_DELIMITED: ListFlags = 1 << 2;
pub const LF_AMPERSAND_DELIMITED: ListFlags = 1 << 3;
pub const LF_COMMA_DELIMITED: ListFlags = 1 << 4;
pub const LF_ASTERISK_DELIMITED: ListFlags = 1 << 5;
pub const LF_DELIMITERS_MASK: ListFlags =
    LF_BAR_DELIMITED | LF_AMPERSAND_DELIMITED | LF_COMMA_DELIMITED | LF_ASTERISK_DELIMITED;
pub const LF_ALLOW_TRAILING_COMMA: ListFlags = 1 << 6;
pub const LF_INDENTED: ListFlags = 1 << 7;
pub const LF_SPACE_BETWEEN_BRACES: ListFlags = 1 << 8;
pub const LF_SPACE_BETWEEN_SIBLINGS: ListFlags = 1 << 9;
pub const LF_BRACES: ListFlags = 1 << 10;
pub const LF_PARENTHESIS: ListFlags = 1 << 11;
pub const LF_ANGLE_BRACKETS: ListFlags = 1 << 12;
pub const LF_SQUARE_BRACKETS: ListFlags = 1 << 13;
pub const LF_BRACKETS_MASK: ListFlags =
    LF_BRACES | LF_PARENTHESIS | LF_ANGLE_BRACKETS | LF_SQUARE_BRACKETS;
pub const LF_OPTIONAL_IF_NIL: ListFlags = 1 << 14;
pub const LF_OPTIONAL_IF_EMPTY: ListFlags = 1 << 15;
pub const LF_OPTIONAL: ListFlags = LF_OPTIONAL_IF_NIL | LF_OPTIONAL_IF_EMPTY;
pub const LF_PREFER_NEW_LINE: ListFlags = 1 << 16;
pub const LF_NO_TRAILING_NEW_LINE: ListFlags = 1 << 17;
pub const LF_NO_INTERVENING_COMMENTS: ListFlags = 1 << 18;
pub const LF_NO_SPACE_IF_EMPTY: ListFlags = 1 << 19;
pub const LF_SINGLE_ELEMENT: ListFlags = 1 << 20;
pub const LF_SPACE_AFTER_LIST: ListFlags = 1 << 21;

pub const LF_MODIFIERS: ListFlags =
    LF_SINGLE_LINE | LF_SPACE_BETWEEN_SIBLINGS | LF_NO_INTERVENING_COMMENTS | LF_SPACE_AFTER_LIST;
pub const LF_HERITAGE_CLAUSES: ListFlags = LF_SINGLE_LINE | LF_SPACE_BETWEEN_SIBLINGS;
pub const LF_SINGLE_LINE_TYPE_LITERAL_MEMBERS: ListFlags =
    LF_SINGLE_LINE | LF_SPACE_BETWEEN_BRACES | LF_SPACE_BETWEEN_SIBLINGS;
pub const LF_MULTI_LINE_TYPE_LITERAL_MEMBERS: ListFlags =
    LF_MULTI_LINE | LF_INDENTED | LF_OPTIONAL_IF_EMPTY;
pub const LF_SINGLE_LINE_TUPLE_TYPE_ELEMENTS: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_MULTI_LINE_TUPLE_TYPE_ELEMENTS: ListFlags =
    LF_COMMA_DELIMITED | LF_INDENTED | LF_SPACE_BETWEEN_SIBLINGS | LF_MULTI_LINE;
pub const LF_UNION_TYPE_CONSTITUENTS: ListFlags =
    LF_BAR_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_INTERSECTION_TYPE_CONSTITUENTS: ListFlags =
    LF_AMPERSAND_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_OBJECT_BINDING_PATTERN_ELEMENTS: ListFlags = LF_SINGLE_LINE
    | LF_ALLOW_TRAILING_COMMA
    | LF_SPACE_BETWEEN_BRACES
    | LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_NO_SPACE_IF_EMPTY;
pub const LF_ARRAY_BINDING_PATTERN_ELEMENTS: ListFlags = LF_SINGLE_LINE
    | LF_ALLOW_TRAILING_COMMA
    | LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_NO_SPACE_IF_EMPTY;
pub const LF_OBJECT_LITERAL_EXPRESSION_PROPERTIES: ListFlags = LF_PRESERVE_LINES
    | LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_SPACE_BETWEEN_BRACES
    | LF_INDENTED
    | LF_BRACES
    | LF_NO_SPACE_IF_EMPTY;
pub const LF_IMPORT_ATTRIBUTES: ListFlags = LF_OBJECT_LITERAL_EXPRESSION_PROPERTIES;
pub const LF_ARRAY_LITERAL_EXPRESSION_ELEMENTS: ListFlags = LF_PRESERVE_LINES
    | LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_ALLOW_TRAILING_COMMA
    | LF_INDENTED
    | LF_SQUARE_BRACKETS;
pub const LF_COMMA_LIST_ELEMENTS: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_CALL_EXPRESSION_ARGUMENTS: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE | LF_PARENTHESIS;
pub const LF_NEW_EXPRESSION_ARGUMENTS: ListFlags =
    LF_CALL_EXPRESSION_ARGUMENTS | LF_OPTIONAL_IF_NIL;
pub const LF_TEMPLATE_EXPRESSION_SPANS: ListFlags = LF_SINGLE_LINE | LF_NO_INTERVENING_COMMENTS;
pub const LF_SINGLE_LINE_BLOCK_STATEMENTS: ListFlags =
    LF_SPACE_BETWEEN_BRACES | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_MULTI_LINE_BLOCK_STATEMENTS: ListFlags = LF_INDENTED | LF_MULTI_LINE;
pub const LF_VARIABLE_DECLARATION_LIST: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_SINGLE_LINE_FUNCTION_BODY_STATEMENTS: ListFlags =
    LF_SINGLE_LINE | LF_SPACE_BETWEEN_SIBLINGS | LF_SPACE_BETWEEN_BRACES;
pub const LF_MULTI_LINE_FUNCTION_BODY_STATEMENTS: ListFlags = LF_MULTI_LINE;
pub const LF_CLASS_HERITAGE_CLAUSES: ListFlags = LF_SINGLE_LINE;
pub const LF_CLASS_MEMBERS: ListFlags = LF_INDENTED | LF_MULTI_LINE;
pub const LF_INTERFACE_MEMBERS: ListFlags = LF_INDENTED | LF_MULTI_LINE;
pub const LF_ENUM_MEMBERS: ListFlags = LF_COMMA_DELIMITED | LF_INDENTED | LF_MULTI_LINE;
pub const LF_CASE_BLOCK_CLAUSES: ListFlags = LF_INDENTED | LF_MULTI_LINE;
pub const LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS: ListFlags = LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_ALLOW_TRAILING_COMMA
    | LF_SINGLE_LINE
    | LF_SPACE_BETWEEN_BRACES
    | LF_NO_SPACE_IF_EMPTY;
pub const LF_JSX_ELEMENT_OR_FRAGMENT_CHILDREN: ListFlags =
    LF_SINGLE_LINE | LF_NO_INTERVENING_COMMENTS;
pub const LF_JSX_ELEMENT_ATTRIBUTES: ListFlags =
    LF_SINGLE_LINE | LF_SPACE_BETWEEN_SIBLINGS | LF_NO_INTERVENING_COMMENTS;
pub const LF_CASE_OR_DEFAULT_CLAUSE_STATEMENTS: ListFlags =
    LF_INDENTED | LF_MULTI_LINE | LF_NO_TRAILING_NEW_LINE | LF_OPTIONAL_IF_EMPTY;
pub const LF_HERITAGE_CLAUSE_TYPES: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_SOURCE_FILE_STATEMENTS: ListFlags = LF_MULTI_LINE | LF_NO_TRAILING_NEW_LINE;
pub const LF_DECORATORS: ListFlags = LF_MULTI_LINE | LF_OPTIONAL | LF_SPACE_AFTER_LIST;
pub const LF_TYPE_ARGUMENTS: ListFlags = LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_SINGLE_LINE
    | LF_ANGLE_BRACKETS
    | LF_OPTIONAL;
pub const LF_TYPE_PARAMETERS: ListFlags = LF_TYPE_ARGUMENTS;
pub const LF_PARAMETERS: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE | LF_PARENTHESIS;
pub const LF_SINGLE_ARROW_PARAMETER: ListFlags =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_INDEX_SIGNATURE_PARAMETERS: ListFlags = LF_COMMA_DELIMITED
    | LF_SPACE_BETWEEN_SIBLINGS
    | LF_SINGLE_LINE
    | LF_INDENTED
    | LF_SQUARE_BRACKETS;
pub const LF_JSDOC_COMMENT: ListFlags = LF_MULTI_LINE | LF_ASTERISK_DELIMITED;
pub const LF_IMPORT_CLAUSE_ENTRIES: ListFlags = LF_IMPORT_ATTRIBUTES;

pub struct RuntimeSyntaxTransformer {
    pub transformer: crate::mig::m4k_2::Transformer,
    pub compiler_options: Arc<tsox_core::core::compiler_options::CompilerOptions>,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,
    pub current_source_file: Option<Arc<Node>>,
    pub current_scope: Option<Arc<Node>>,
    pub current_scope_first_declarations_of_name: HashMap<String, Arc<Node>>,
    pub current_enum: Option<Arc<Node>>,
    pub current_namespace: Option<Arc<Node>>,
}

impl std::ops::Deref for RuntimeSyntaxTransformer {
    type Target = crate::mig::m4k_2::Transformer;

    fn deref(&self) -> &Self::Target {
        &self.transformer
    }
}
