use tsox_frontend::ast::mig::m3c::type_parameter_list;
use tsox_frontend::ast::NodeList;
use tsox_frontend::format::mig::m4o::{EmitFlags as GoEmitFlags, ListFormat as GoListFormat};

pub use tsox_frontend::ast::mig::m3e_3::{
    get_expression_precedence, get_type_node_precedence, OperatorPrecedence, TypePrecedence,
    OPERATOR_PRECEDENCE_COALESCE, OPERATOR_PRECEDENCE_DISALLOW_COMMA, OPERATOR_PRECEDENCE_HIGHEST,
    OPERATOR_PRECEDENCE_LOWEST, TYPE_PRECEDENCE_HIGHEST, TYPE_PRECEDENCE_LOWEST,
};
pub use tsox_frontend::format::mig::m4o_2::{
    CommentState, DetachedCommentsInfo, EmitContext, PrinterOptions, SourceMapState, WriteKind,
    Generator as SourceMapGenerator, SourceIndex,
};
pub use tsox_frontend::format::mig::m4t_2::{
    escape_jsx_attribute_string, escape_non_ascii_string, escape_string, GetLiteralTextFlags,
    QuoteChar, GET_LITERAL_TEXT_FLAGS_ALLOW_NUMERIC_SEPARATOR, GET_LITERAL_TEXT_FLAGS_NONE,
    GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE,
};
pub use tsox_frontend::format::mig::m4t_3::{
    new_line_character_cache, Source, get_ecma_line_starts, get_lines_between_position_and_next_non_whitespace_character,
    get_lines_between_position_and_preceding_non_whitespace_character,
    get_lines_between_range_end_and_range_start, range_end_is_on_same_line_as_range_start,
    range_end_positions_are_on_same_line, range_is_on_single_line,
    range_start_positions_are_on_same_line,
};
pub use tsox_frontend::format::mig::m4t_4::{
    greatest_end, is_binary_operation, original_nodes_have_same_parent,
    sibling_node_positions_are_comparable, skip_synthesized_parentheses,
};
pub use tsox_frontend::format::mig::m4t_5::{
    is_jsdoc_like_text, is_pinned_comment, is_recognized_triple_slash_comment,
};
pub use tsox_frontend::format::mig::m4t_6::get_literal_text;
pub use crate::printer::NameGenerator;
pub use tsox_core::core::compiler_options_kinds::ScriptTarget;
pub use tsox_core::core::text::TextRange;
pub use tsox_core::core::tristate::Tristate;
pub use tsox_core::tspath::EXTENSION_JSON;

pub use tsox_frontend::ast::node_source_file::ScriptKind;
pub use tsox_frontend::ast::utilities::
    skip_partially_emitted_expressions_arc as skip_partially_emitted_expressions;
pub use crate::printer::mig::m4m_2::SynthesizedComment;

pub const SCRIPT_TARGET_ES2021: ScriptTarget = ScriptTarget::ES2021;
pub const SCRIPT_KIND_JSON: ScriptKind = ScriptKind::Json;
pub const QUOTE_CHAR_DOUBLE_QUOTE: QuoteChar = QuoteChar::DoubleQuote;

pub const OPERATOR_PRECEDENCE_CONDITIONAL: OperatorPrecedence = OperatorPrecedence::Conditional;
pub const OPERATOR_PRECEDENCE_YIELD: OperatorPrecedence = OperatorPrecedence::Yield;
pub const OPERATOR_PRECEDENCE_LOGICAL_AND: OperatorPrecedence = OperatorPrecedence::LogicalAnd;
pub const OPERATOR_PRECEDENCE_BITWISE_OR: OperatorPrecedence = OperatorPrecedence::BitwiseOr;
pub const OPERATOR_PRECEDENCE_RELATIONAL: OperatorPrecedence = OperatorPrecedence::Relational;
pub const OPERATOR_PRECEDENCE_SHIFT: OperatorPrecedence = OperatorPrecedence::Shift;
pub const OPERATOR_PRECEDENCE_ADDITIVE: OperatorPrecedence = OperatorPrecedence::Additive;
pub const OPERATOR_PRECEDENCE_MULTIPLICATIVE: OperatorPrecedence =
    OperatorPrecedence::Multiplicative;
pub const OPERATOR_PRECEDENCE_EXPONENTIATION: OperatorPrecedence =
    OperatorPrecedence::Exponentiation;
pub const OPERATOR_PRECEDENCE_UPDATE: OperatorPrecedence = OperatorPrecedence::Update;
pub const OPERATOR_PRECEDENCE_UNARY: OperatorPrecedence = OperatorPrecedence::Unary;

pub const TYPE_PRECEDENCE_TYPE_OPERATOR: TypePrecedence = TypePrecedence::TypeOperator;
pub const TYPE_PRECEDENCE_POSTFIX: TypePrecedence = TypePrecedence::Postfix;
pub const TYPE_PRECEDENCE_CONDITIONAL: TypePrecedence = TypePrecedence::Conditional;
pub const TYPE_PRECEDENCE_FUNCTION: TypePrecedence = TypePrecedence::Function;

pub type ListFormat = i32;

pub const LF_NONE: ListFormat = GoListFormat::NONE.0;
pub const LF_MULTI_LINE: ListFormat = GoListFormat::MULTI_LINE.0;
pub const LF_PRESERVE_LINES: ListFormat = GoListFormat::PRESERVE_LINES.0;
pub const LF_COMMA_DELIMITED: ListFormat = GoListFormat::COMMA_DELIMITED.0;
pub const LF_BAR_DELIMITED: ListFormat = GoListFormat::BAR_DELIMITED.0;
pub const LF_ALLOW_TRAILING_COMMA: ListFormat = GoListFormat::ALLOW_TRAILING_COMMA.0;
pub const LF_INDENTED: ListFormat = GoListFormat::INDENTED.0;
pub const LF_SPACE_BETWEEN_BRACES: ListFormat = GoListFormat::SPACE_BETWEEN_BRACES.0;
pub const LF_SPACE_BETWEEN_SIBLINGS: ListFormat = GoListFormat::SPACE_BETWEEN_SIBLINGS.0;
pub const LF_ANGLE_BRACKETS: ListFormat = GoListFormat::ANGLE_BRACKETS.0;
pub const LF_BRACKETS_MASK: ListFormat = GoListFormat::BRACKETS_MASK.0;
pub const LF_OPTIONAL_IF_EMPTY: ListFormat = GoListFormat::OPTIONAL_IF_EMPTY.0;
pub const LF_OPTIONAL: ListFormat = GoListFormat::OPTIONAL.0;
pub const LF_PREFER_NEW_LINE: ListFormat = GoListFormat::PREFER_NEW_LINE.0;
pub const LF_NO_TRAILING_NEW_LINE: ListFormat = GoListFormat::NO_TRAILING_NEW_LINE.0;
pub const LF_NO_SPACE_IF_EMPTY: ListFormat = GoListFormat::NO_SPACE_IF_EMPTY.0;
pub const LF_SINGLE_LINE: ListFormat = 0;
pub const LF_SINGLE_LINE_TYPE_LITERAL_MEMBERS: ListFormat =
    LF_SINGLE_LINE | LF_SPACE_BETWEEN_BRACES | LF_SPACE_BETWEEN_SIBLINGS;
pub const LF_MULTI_LINE_TYPE_LITERAL_MEMBERS: ListFormat =
    LF_MULTI_LINE | LF_INDENTED | LF_OPTIONAL_IF_EMPTY;
pub const LF_SINGLE_LINE_TUPLE_TYPE_ELEMENTS: ListFormat =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_MULTI_LINE_TUPLE_TYPE_ELEMENTS: ListFormat =
    LF_COMMA_DELIMITED | LF_INDENTED | LF_SPACE_BETWEEN_SIBLINGS | LF_MULTI_LINE;
pub const LF_UNION_TYPE_CONSTITUENTS: ListFormat =
    LF_BAR_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_VARIABLE_DECLARATION_LIST: ListFormat =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE;
pub const LF_TYPE_PARAMETERS: ListFormat =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE | LF_ANGLE_BRACKETS | LF_OPTIONAL;
pub const LF_TYPE_ARGUMENTS: ListFormat =
    LF_COMMA_DELIMITED | LF_SPACE_BETWEEN_SIBLINGS | LF_SINGLE_LINE | LF_ANGLE_BRACKETS | LF_OPTIONAL;

pub type EmitFlags = i32;

pub const EF_NONE: EmitFlags = GoEmitFlags::NONE.0 as i32;
pub const EF_NO_ASCII_ESCAPING: EmitFlags = GoEmitFlags::NO_ASCII_ESCAPING.0 as i32;
pub const EF_INDENTED: EmitFlags = GoEmitFlags::INDENTED.0 as i32;
pub const EF_NO_INDENTATION: EmitFlags = GoEmitFlags::NO_INDENTATION.0 as i32;
pub const EF_SINGLE_LINE: EmitFlags = GoEmitFlags::SINGLE_LINE.0 as i32;
pub const EF_MULTI_LINE: EmitFlags = GoEmitFlags::MULTI_LINE.0 as i32;
pub const EF_START_ON_NEW_LINE: EmitFlags = GoEmitFlags::START_ON_NEW_LINE.0 as i32;
pub const EF_NO_LEADING_COMMENTS: EmitFlags = GoEmitFlags::NO_LEADING_COMMENTS.0 as i32;
pub const EF_NO_TRAILING_COMMENTS: EmitFlags = GoEmitFlags::NO_TRAILING_COMMENTS.0 as i32;
pub const EF_NO_NESTED_COMMENTS: EmitFlags = GoEmitFlags::NO_NESTED_COMMENTS.0 as i32;
pub const EF_INDIRECT_CALL: EmitFlags = GoEmitFlags::INDIRECT_CALL.0 as i32;
pub const EF_EXTERNAL_HELPERS: EmitFlags = GoEmitFlags::EXTERNAL_HELPERS.0 as i32;
pub const EF_REUSE_TEMP_VARIABLE_SCOPE: EmitFlags = GoEmitFlags::REUSE_TEMP_VARIABLE_SCOPE.0 as i32;

pub type TokenEmitFlags = i32;

pub const TEF_NONE: TokenEmitFlags =
    tsox_frontend::format::mig::m4o::TokenEmitFlags::NONE.0 as i32;
pub const TEF_NO_COMMENTS: TokenEmitFlags =
    tsox_frontend::format::mig::m4o::TokenEmitFlags::NO_COMMENTS.0 as i32;
pub const TEF_NO_SOURCE_MAPS: TokenEmitFlags =
    tsox_frontend::format::mig::m4o::TokenEmitFlags::NO_SOURCE_MAPS.0 as i32;

pub use crate::printer::AutoGenerateOptions;
pub use crate::printer::GeneratedIdentifierFlags;

pub const GENERATED_IDENTIFIER_FLAGS_FILE_LEVEL: GeneratedIdentifierFlags =
    GeneratedIdentifierFlags::FILE_LEVEL;
pub const GENERATED_IDENTIFIER_FLAGS_OPTIMISTIC: GeneratedIdentifierFlags =
    GeneratedIdentifierFlags::OPTIMISTIC;

#[derive(Default)]
pub struct PrinterState {
    pub comment_state: Option<CommentState>,
}

pub trait EmitTextWriter {
    fn write(&mut self, s: &str);
    fn write_trailing_semicolon(&mut self, text: &str);
    fn write_comment(&mut self, text: &str);
    fn write_keyword(&mut self, text: &str);
    fn write_operator(&mut self, text: &str);
    fn write_punctuation(&mut self, text: &str);
    fn write_space(&mut self, text: &str);
    fn write_string_literal(&mut self, text: &str);
    fn write_parameter(&mut self, text: &str);
    fn write_property(&mut self, text: &str);
    fn write_symbol(&mut self, text: &str, symbol: &Arc<tsox_frontend::ast::symbol::Symbol>);
    fn write_line(&mut self);
    fn write_line_force(&mut self, force: bool);
    fn increase_indent(&mut self);
    fn decrease_indent(&mut self);
    fn clear(&mut self);
    fn string(&self) -> String;
    fn raw_write(&mut self, s: &str);
    fn write_literal(&mut self, s: &str);
    fn get_text_pos(&self) -> i32;
    fn get_line(&self) -> i32;
    fn get_column(&self) -> i32;
    fn get_indent(&self) -> i32;
    fn is_at_start_of_line(&self) -> bool;
    fn has_trailing_comment(&self) -> bool;
    fn has_trailing_whitespace(&self) -> bool;
}

pub fn should_allow_trailing_comma_worker(node: &tsox_frontend::ast::Node, list: &NodeList) -> bool { ::tsox_core::fntrace::enter("should_allow_trailing_comma_worker"); 
    use SyntaxKind as K;
    match node.kind {
        K::ObjectLiteralExpression => true,
        K::ArrayLiteralExpression
        | K::ArrowFunction
        | K::Constructor
        | K::GetAccessor
        | K::SetAccessor
        | K::TypeAliasDeclaration
        | K::JSTypeAliasDeclaration
        | K::FunctionType
        | K::ConstructorType
        | K::CallSignature
        | K::ConstructSignature
        | K::TaggedTemplateExpression
        | K::ObjectBindingPattern
        | K::ArrayBindingPattern
        | K::NamedImports
        | K::NamedExports
        | K::ImportAttributes => true,
        K::ClassExpression | K::ClassDeclaration | K::InterfaceDeclaration => {
            type_parameter_list(node).is_some_and(|t| std::ptr::eq(t.as_ref(), list))
        }
        K::FunctionDeclaration | K::FunctionExpression | K::MethodDeclaration => true,
        K::CallExpression => true,
        K::NewExpression => true,
        _ => false,
    }
}
