use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::TokenFlags;
use crate::ast::node_data_generated::{
    is_binary_expression, is_call_expression, is_computed_property_name, is_labeled_statement,
    is_parenthesized_expression, NodeData,
};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::{ModifierList, NodeList};
use crate::ast::mig::m3b::label;
use crate::ast::mig::m3c::{statement, type_argument_list};
use crate::ast::mig::m3g_3::{
    is_outer_expression, try_get_property_name_of_binding_or_assignment_element,
    OuterExpressionKinds,
};
use crate::ast::node_flags::NodeFlags;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_navigation::get_name_of_declaration;
use crate::ast::utilities_predicates::is_member_name;
use crate::ast::utilities_statements::is_prologue_directive;
use crate::ast::utilities_synthesized::{node_is_synthesized, range_is_synthesized};
use tsox_core::core::text::TextRange;

use super::m4o_2::EmitHelper;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EmitFlags(pub u32);

impl EmitFlags {
    pub const NONE: EmitFlags = EmitFlags(0);
    pub const SINGLE_LINE: EmitFlags = EmitFlags(1 << 0);
    pub const MULTI_LINE: EmitFlags = EmitFlags(1 << 1);
    pub const NO_LEADING_SOURCE_MAP: EmitFlags = EmitFlags(1 << 2);
    pub const NO_TRAILING_SOURCE_MAP: EmitFlags = EmitFlags(1 << 3);
    pub const NO_NESTED_SOURCE_MAPS: EmitFlags = EmitFlags(1 << 4);
    pub const NO_TOKEN_LEADING_SOURCE_MAPS: EmitFlags = EmitFlags(1 << 5);
    pub const NO_TOKEN_TRAILING_SOURCE_MAPS: EmitFlags = EmitFlags(1 << 6);
    pub const NO_LEADING_COMMENTS: EmitFlags = EmitFlags(1 << 7);
    pub const NO_TRAILING_COMMENTS: EmitFlags = EmitFlags(1 << 8);
    pub const NO_NESTED_COMMENTS: EmitFlags = EmitFlags(1 << 9);
    pub const HELPER_NAME: EmitFlags = EmitFlags(1 << 10);
    pub const EXPORT_NAME: EmitFlags = EmitFlags(1 << 11);
    pub const LOCAL_NAME: EmitFlags = EmitFlags(1 << 12);
    pub const INDENTED: EmitFlags = EmitFlags(1 << 13);
    pub const NO_INDENTATION: EmitFlags = EmitFlags(1 << 14);
    pub const REUSE_TEMP_VARIABLE_SCOPE: EmitFlags = EmitFlags(1 << 15);
    pub const CUSTOM_PROLOGUE: EmitFlags = EmitFlags(1 << 16);
    pub const NO_ASCII_ESCAPING: EmitFlags = EmitFlags(1 << 17);
    pub const EXTERNAL_HELPERS: EmitFlags = EmitFlags(1 << 18);
    pub const START_ON_NEW_LINE: EmitFlags = EmitFlags(1 << 19);
    pub const INDIRECT_CALL: EmitFlags = EmitFlags(1 << 20);
    pub const ASYNC_FUNCTION_BODY: EmitFlags = EmitFlags(1 << 21);
    pub const NO_LEXICAL_ARGUMENTS: EmitFlags = EmitFlags(1 << 22);
    pub const TRANSFORM_PRIVATE_STATIC_ELEMENTS: EmitFlags = EmitFlags(1 << 23);
    pub const NO_LEXICAL_THIS: EmitFlags = EmitFlags(1 << 24);

    pub const NO_SOURCE_MAP: EmitFlags =
        EmitFlags(Self::NO_LEADING_SOURCE_MAP.0 | Self::NO_TRAILING_SOURCE_MAP.0);
    pub const NO_TOKEN_SOURCE_MAPS: EmitFlags =
        EmitFlags(Self::NO_TOKEN_LEADING_SOURCE_MAPS.0 | Self::NO_TOKEN_TRAILING_SOURCE_MAPS.0);
    pub const NO_COMMENTS: EmitFlags =
        EmitFlags(Self::NO_LEADING_COMMENTS.0 | Self::NO_TRAILING_COMMENTS.0);

    pub fn contains(self, other: EmitFlags) -> bool { ::tsox_core::fntrace::enter("contains"); 
        self.0 & other.0 == other.0
    }

    pub fn intersects(self, other: EmitFlags) -> bool { ::tsox_core::fntrace::enter("intersects"); 
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for EmitFlags {
    type Output = EmitFlags;
    fn bitor(self, rhs: EmitFlags) -> EmitFlags { ::tsox_core::fntrace::enter("bitor"); 
        EmitFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for EmitFlags {
    fn bitor_assign(&mut self, rhs: EmitFlags) { ::tsox_core::fntrace::enter("bitor_assign"); 
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for EmitFlags {
    type Output = EmitFlags;
    fn bitand(self, rhs: EmitFlags) -> EmitFlags { ::tsox_core::fntrace::enter("bitand"); 
        EmitFlags(self.0 & rhs.0)
    }
}

impl std::ops::Not for EmitFlags {
    type Output = EmitFlags;
    fn not(self) -> EmitFlags { ::tsox_core::fntrace::enter("not"); 
        EmitFlags(!self.0)
    }
}

pub const EF_NONE: EmitFlags = EmitFlags::NONE;
pub const EF_NO_COMMENTS: EmitFlags = EmitFlags::NO_COMMENTS;
pub const EF_NO_SOURCE_MAP: EmitFlags = EmitFlags::NO_SOURCE_MAP;
pub const EF_NO_NESTED_COMMENTS: EmitFlags = EmitFlags::NO_NESTED_COMMENTS;
pub const EF_NO_TRAILING_COMMENTS: EmitFlags = EmitFlags::NO_TRAILING_COMMENTS;
pub const EF_HELPER_NAME: EmitFlags = EmitFlags::HELPER_NAME;
pub const EF_CUSTOM_PROLOGUE: EmitFlags = EmitFlags::CUSTOM_PROLOGUE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenEmitFlags(pub u32);

impl TokenEmitFlags {
    pub const NONE: TokenEmitFlags = TokenEmitFlags(0);
    pub const NO_COMMENTS: TokenEmitFlags = TokenEmitFlags(1 << 0);
    pub const INDENT_LEADING_COMMENTS: TokenEmitFlags = TokenEmitFlags(1 << 1);
    pub const NO_SOURCE_MAPS: TokenEmitFlags = TokenEmitFlags(1 << 2);

    pub fn contains(self, other: TokenEmitFlags) -> bool { ::tsox_core::fntrace::enter("contains"); 
        self.0 & other.0 == other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListFormat(pub i32);

impl ListFormat {
    pub const NONE: ListFormat = ListFormat(0);
    pub const MULTI_LINE: ListFormat = ListFormat(1 << 0);
    pub const PRESERVE_LINES: ListFormat = ListFormat(1 << 1);
    pub const LINES_MASK: ListFormat =
        ListFormat(Self::NONE.0 | Self::MULTI_LINE.0 | Self::PRESERVE_LINES.0);
    pub const BAR_DELIMITED: ListFormat = ListFormat(1 << 2);
    pub const AMPERSAND_DELIMITED: ListFormat = ListFormat(1 << 3);
    pub const COMMA_DELIMITED: ListFormat = ListFormat(1 << 4);
    pub const ASTERISK_DELIMITED: ListFormat = ListFormat(1 << 5);
    pub const DELIMITERS_MASK: ListFormat = ListFormat(
        Self::BAR_DELIMITED.0
            | Self::AMPERSAND_DELIMITED.0
            | Self::COMMA_DELIMITED.0
            | Self::ASTERISK_DELIMITED.0,
    );
    pub const ALLOW_TRAILING_COMMA: ListFormat = ListFormat(1 << 6);
    pub const INDENTED: ListFormat = ListFormat(1 << 7);
    pub const SPACE_BETWEEN_BRACES: ListFormat = ListFormat(1 << 8);
    pub const SPACE_BETWEEN_SIBLINGS: ListFormat = ListFormat(1 << 9);
    pub const BRACES: ListFormat = ListFormat(1 << 10);
    pub const PARENTHESIS: ListFormat = ListFormat(1 << 11);
    pub const ANGLE_BRACKETS: ListFormat = ListFormat(1 << 12);
    pub const SQUARE_BRACKETS: ListFormat = ListFormat(1 << 13);
    pub const BRACKETS_MASK: ListFormat = ListFormat(
        Self::BRACES.0 | Self::PARENTHESIS.0 | Self::ANGLE_BRACKETS.0 | Self::SQUARE_BRACKETS.0,
    );
    pub const OPTIONAL_IF_NIL: ListFormat = ListFormat(1 << 14);
    pub const OPTIONAL_IF_EMPTY: ListFormat = ListFormat(1 << 15);
    pub const OPTIONAL: ListFormat =
        ListFormat(Self::OPTIONAL_IF_NIL.0 | Self::OPTIONAL_IF_EMPTY.0);
    pub const PREFER_NEW_LINE: ListFormat = ListFormat(1 << 16);
    pub const NO_TRAILING_NEW_LINE: ListFormat = ListFormat(1 << 17);
    pub const NO_INTERVENING_COMMENTS: ListFormat = ListFormat(1 << 18);
    pub const NO_SPACE_IF_EMPTY: ListFormat = ListFormat(1 << 19);
    pub const SINGLE_ELEMENT: ListFormat = ListFormat(1 << 20);
    pub const SPACE_AFTER_LIST: ListFormat = ListFormat(1 << 21);

    pub const SINGLE_LINE_BLOCK_STATEMENTS: ListFormat = ListFormat(
        Self::SPACE_BETWEEN_BRACES.0 | Self::SPACE_BETWEEN_SIBLINGS.0 | Self::NONE.0,
    );
    pub const MULTI_LINE_BLOCK_STATEMENTS: ListFormat =
        ListFormat(Self::INDENTED.0 | Self::MULTI_LINE.0);
    pub const CLASS_HERITAGE_CLAUSES: ListFormat = ListFormat(Self::NONE.0);
    pub const CLASS_MEMBERS: ListFormat = ListFormat(Self::INDENTED.0 | Self::MULTI_LINE.0);
    pub const CASE_BLOCK_CLAUSES: ListFormat = ListFormat(Self::INDENTED.0 | Self::MULTI_LINE.0);
    pub const CASE_OR_DEFAULT_CLAUSE_STATEMENTS: ListFormat = ListFormat(
        Self::INDENTED.0
            | Self::MULTI_LINE.0
            | Self::NO_TRAILING_NEW_LINE.0
            | Self::OPTIONAL_IF_EMPTY.0,
    );

    pub fn contains(self, other: ListFormat) -> bool { ::tsox_core::fntrace::enter("contains"); 
        self.0 & other.0 == other.0
    }

    pub fn intersects(self, other: ListFormat) -> bool { ::tsox_core::fntrace::enter("intersects"); 
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for ListFormat {
    type Output = ListFormat;
    fn bitor(self, rhs: ListFormat) -> ListFormat { ::tsox_core::fntrace::enter("bitor"); 
        ListFormat(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ListFormat {
    type Output = ListFormat;
    fn bitand(self, rhs: ListFormat) -> ListFormat { ::tsox_core::fntrace::enter("bitand"); 
        ListFormat(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ListFormat {
    fn bitand_assign(&mut self, rhs: ListFormat) { ::tsox_core::fntrace::enter("bitand_assign"); 
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for ListFormat {
    type Output = ListFormat;
    fn not(self) -> ListFormat { ::tsox_core::fntrace::enter("not"); 
        ListFormat(!self.0 & 0x3F_FFFF)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteKind {
    #[default]
    None,
    Keyword,
    Operator,
    Punctuation,
    StringLiteral,
    Parameter,
    Property,
    Comment,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommentSeparator {
    #[default]
    None,
    Before,
    After,
}

#[derive(Default, Clone)]
pub struct EmitContext {
    next_auto_id: std::cell::Cell<u32>,
    node_emit_flags: RefCell<HashMap<u64, EmitFlags>>,
    node_comment_ranges: RefCell<HashMap<u64, TextRange>>,
    node_source_map_ranges: RefCell<HashMap<u64, TextRange>>,
    node_text_sources: RefCell<HashMap<u64, Arc<Node>>>,
    node_type_nodes: RefCell<HashMap<u64, Arc<Node>>>,
    parsed_nodes: RefCell<HashMap<u64, Arc<Node>>>,
    requested_emit_helpers: RefCell<HashMap<String, Arc<EmitHelper>>>,
}

impl EmitContext {
    pub fn next_auto_generate_id(&self) -> u32 { ::tsox_core::fntrace::enter("next_auto_generate_id"); 
        let id = self.next_auto_id.get() + 1;
        self.next_auto_id.set(id);
        id
    }

    pub fn emit_flags(&self, node: &Arc<Node>) -> EmitFlags { ::tsox_core::fntrace::enter("emit_flags"); 
        self.node_emit_flags
            .borrow()
            .get(&node.id())
            .copied()
            .unwrap_or(EmitFlags::NONE)
    }

    pub fn add_emit_flags(&self, node: &Arc<Node>, flags: EmitFlags) { ::tsox_core::fntrace::enter("add_emit_flags"); 
        let mut map = self.node_emit_flags.borrow_mut();
        let entry = map.entry(node.id()).or_insert(EmitFlags::NONE);
        *entry = *entry | flags;
    }

    pub fn set_emit_flags(&self, node: &Arc<Node>, flags: EmitFlags) { ::tsox_core::fntrace::enter("set_emit_flags"); 
        self.node_emit_flags.borrow_mut().insert(node.id(), flags);
    }

    pub fn comment_range(&self, node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("comment_range"); 
        self.node_comment_ranges
            .borrow()
            .get(&node.id())
            .copied()
            .unwrap_or(node.loc)
    }

    pub fn set_comment_range(&self, node: &Arc<Node>, range: TextRange) { ::tsox_core::fntrace::enter("set_comment_range"); 
        self.node_comment_ranges.borrow_mut().insert(node.id(), range);
    }

    pub fn source_map_range(&self, node: &Arc<Node>) -> TextRange { ::tsox_core::fntrace::enter("source_map_range"); 
        self.node_source_map_ranges
            .borrow()
            .get(&node.id())
            .copied()
            .unwrap_or(node.loc)
    }

    pub fn set_source_map_range(&self, node: &Arc<Node>, range: TextRange) { ::tsox_core::fntrace::enter("set_source_map_range"); 
        self.node_source_map_ranges
            .borrow_mut()
            .insert(node.id(), range);
    }

    pub fn set_text_source(&self, node: &Arc<Node>, source: &Arc<Node>) { ::tsox_core::fntrace::enter("set_text_source"); 
        self.node_text_sources
            .borrow_mut()
            .insert(node.id(), source.clone());
    }

    pub fn get_type_node(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_node"); 
        self.node_type_nodes.borrow().get(&node.id()).cloned()
    }

    pub fn set_type_node(&self, node: &Arc<Node>, type_node: &Arc<Node>) { ::tsox_core::fntrace::enter("set_type_node"); 
        self.node_type_nodes
            .borrow_mut()
            .insert(node.id(), type_node.clone());
    }

    pub fn parse_node(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parse_node"); 
        self.parsed_nodes.borrow().get(&node.id()).cloned()
    }

    pub fn assign_comment_and_source_map_ranges(
        &self,
        node: &Arc<Node>,
        source: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("assign_comment_and_source_map_ranges"); 
        let comment_range = self.comment_range(source);
        let source_map_range = self.source_map_range(source);
        self.set_comment_range(node, comment_range);
        self.set_source_map_range(node, source_map_range);
    }

    pub fn request_emit_helper(&self, helper: EmitHelper) { ::tsox_core::fntrace::enter("request_emit_helper"); 
        let mut map = self.requested_emit_helpers.borrow_mut();
        map.entry(helper.name.clone())
            .or_insert_with(|| Arc::new(helper));
    }

    pub fn is_file_level_unique_name(
        &self,
        _source_file: &crate::ast::node_source_file::SourceFile,
        name: &str,
        has_global_name: Option<fn(&str) -> bool>,
    ) -> bool { ::tsox_core::fntrace::enter("is_file_level_unique_name"); 
        if let Some(has_global_name) = has_global_name {
            if has_global_name(name) {
                return false;
            }
        }
        true
    }
}

pub struct NodeFactory {
    pub emit_context: EmitContext,
}

pub fn new_node_factory(context: EmitContext) -> NodeFactory { ::tsox_core::fntrace::enter("new_node_factory"); 
    NodeFactory {
        emit_context: context,
    }
}

pub struct NameOptions {
    pub allow_comments: bool,
    pub allow_source_maps: bool,
}

pub struct AssignedNameOptions {
    pub allow_comments: bool,
    pub allow_source_maps: bool,
    pub ignore_assigned_name: bool,
}

fn simple_helper(name: &str, import_name: &str) -> EmitHelper { ::tsox_core::fntrace::enter("simple_helper"); 
    EmitHelper {
        name: name.to_string(),
        scoped: false,
        text: String::new(),
        text_callback: None,
        priority: None,
        dependencies: Vec::new(),
        import_name: import_name.to_string(),
    }
}

pub fn metadata_helper() -> EmitHelper { ::tsox_core::fntrace::enter("metadata_helper"); 
    simple_helper("typescript:metadata", "__metadata")
}

pub fn param_helper() -> EmitHelper { ::tsox_core::fntrace::enter("param_helper"); 
    simple_helper("typescript:param", "__param")
}

pub fn rest_helper() -> EmitHelper { ::tsox_core::fntrace::enter("rest_helper"); 
    simple_helper("typescript:rest", "__rest")
}

pub fn run_initializers_helper() -> EmitHelper { ::tsox_core::fntrace::enter("run_initializers_helper"); 
    simple_helper("typescript:runInitializers", "__runInitializers")
}

pub fn make_template_object_helper() -> EmitHelper { ::tsox_core::fntrace::enter("make_template_object_helper"); 
    simple_helper("typescript:makeTemplateObject", "__makeTemplateObject")
}

pub fn prop_key_helper() -> EmitHelper { ::tsox_core::fntrace::enter("prop_key_helper"); 
    simple_helper("typescript:propKey", "__propKey")
}

pub fn set_function_name_helper() -> EmitHelper { ::tsox_core::fntrace::enter("set_function_name_helper"); 
    simple_helper("typescript:setFunctionName", "__setFunctionName")
}

pub fn import_default_helper() -> EmitHelper { ::tsox_core::fntrace::enter("import_default_helper"); 
    simple_helper("typescript:importDefault", "__importDefault")
}

pub fn import_star_helper() -> EmitHelper { ::tsox_core::fntrace::enter("import_star_helper"); 
    simple_helper("typescript:importStar", "__importStar")
}

pub fn rewrite_relative_import_extensions_helper() -> EmitHelper { ::tsox_core::fntrace::enter("rewrite_relative_import_extensions_helper"); 
    simple_helper(
        "typescript:rewriteRelativeImportExtensions",
        "__rewriteRelativeImportExtension",
    )
}

impl NodeFactory {
    pub fn new_token(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_token"); 
        Arc::new(Node::new(kind, NodeData::Token))
    }

    pub fn new_node_list(&self, nodes: &[Arc<Node>]) -> Arc<NodeList> { ::tsox_core::fntrace::enter("new_node_list"); 
        Arc::new(NodeList::new(nodes.to_vec()))
    }

    pub fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(crate::ast::node_data_generated::IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    pub fn new_string_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal"); 
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(crate::ast::node_data_generated::StringLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_numeric_literal(&self, text: &str, token_flags: TokenFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("new_numeric_literal"); 
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(crate::ast::node_data_generated::NumericLiteralData {
                text: text.to_string(),
                token_flags,
            }),
        ))
    }

    pub fn new_keyword_expression(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_expression"); 
        self.new_token(kind)
    }

    pub fn new_binary_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        left: &Arc<Node>,
        type_node: Option<Arc<Node>>,
        operator_token: Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_binary_expression"); 
        Arc::new(Node::new(
            SyntaxKind::BinaryExpression,
            NodeData::BinaryExpression(crate::ast::node_data_generated::BinaryExpressionData {
                modifiers,
                left: left.clone(),
                type_node,
                operator_token,
                right: right.clone(),
            }),
        ))
    }

    pub fn new_void_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_void_expression"); 
        Arc::new(Node::new(
            SyntaxKind::VoidExpression,
            NodeData::VoidExpression(crate::ast::node_data_generated::VoidExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_property_access_expression"); 
        let mut node = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(
                crate::ast::node_data_generated::PropertyAccessExpressionData {
                    expression: expression.clone(),
                    question_dot_token,
                    name: name.clone(),
                },
            ),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_call_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: Arc<NodeList>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_call_expression"); 
        let mut node = Node::new(
            SyntaxKind::CallExpression,
            NodeData::CallExpression(crate::ast::node_data_generated::CallExpressionData {
                expression: expression.clone(),
                question_dot_token,
                type_arguments,
                arguments,
            }),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub fn new_parenthesized_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_parenthesized_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(
                crate::ast::node_data_generated::ParenthesizedExpressionData {
                    expression: expression.clone(),
                },
            ),
        ))
    }

    pub fn new_conditional_expression(
        &self,
        condition: &Arc<Node>,
        question_token: Arc<Node>,
        when_true: &Arc<Node>,
        colon_token: Arc<Node>,
        when_false: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_conditional_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ConditionalExpression,
            NodeData::ConditionalExpression(
                crate::ast::node_data_generated::ConditionalExpressionData {
                    condition: condition.clone(),
                    question_token,
                    when_true: when_true.clone(),
                    colon_token,
                    when_false: when_false.clone(),
                },
            ),
        ))
    }

    pub fn new_array_literal_expression(
        &self,
        elements: Arc<NodeList>,
        multi_line: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_array_literal_expression"); 
        Arc::new(Node::new(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(
                crate::ast::node_data_generated::ArrayLiteralExpressionData {
                    elements,
                    multi_line,
                },
            ),
        ))
    }

    pub fn new_arrow_function(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        equals_greater_than_token: Arc<Node>,
        body: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_arrow_function"); 
        Arc::new(Node::new(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(crate::ast::node_data_generated::ArrowFunctionData {
                modifiers,
                type_parameters,
                parameters,
                type_node,
                full_signature,
                equals_greater_than_token,
                body,
            }),
        ))
    }

    pub fn new_block(&self, statements: Arc<NodeList>, multi_line: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("new_block"); 
        Arc::new(Node::new(
            SyntaxKind::Block,
            NodeData::Block(crate::ast::node_data_generated::BlockData {
                statements,
                multi_line,
            }),
        ))
    }

    pub fn new_generated_name_for_node(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_generated_name_for_node"); 
        let _ = self.emit_context.next_auto_generate_id();
        self.new_identifier(&node.text())
    }

    pub fn new_string_literal_from_node(&self, text_source_node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal_from_node"); 
        let mut text = String::new();
        match text_source_node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::JsxNamespacedName
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail
            | SyntaxKind::RegularExpressionLiteral => {
                text = text_source_node.text().to_string();
            }
            _ => {}
        }
        let node = self.new_string_literal(&text, TokenFlags::default());
        self.emit_context.set_text_source(&node, text_source_node);
        node
    }

    pub fn new_this_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_this_expression"); 
        self.new_keyword_expression(SyntaxKind::ThisKeyword)
    }

    pub fn new_true_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_true_expression"); 
        self.new_keyword_expression(SyntaxKind::TrueKeyword)
    }

    pub fn new_logical_or_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_logical_or_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            self.new_token(SyntaxKind::BarBarToken),
            right,
        )
    }

    pub fn new_logical_and_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_logical_and_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            self.new_token(SyntaxKind::AmpersandAmpersandToken),
            right,
        )
    }

    pub fn new_strict_equality_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_strict_equality_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            self.new_token(SyntaxKind::EqualsEqualsEqualsToken),
            right,
        )
    }

    pub fn new_strict_inequality_expression(
        &self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_strict_inequality_expression"); 
        self.new_binary_expression(
            None,
            left,
            None,
            self.new_token(SyntaxKind::ExclamationEqualsEqualsToken),
            right,
        )
    }

    pub fn new_void_zero_expression(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_void_zero_expression"); 
        self.new_void_expression(&self.new_numeric_literal("0", TokenFlags::default()))
    }

    pub fn restore_enclosing_label(
        &self,
        node: &Arc<Node>,
        outermost_labeled_statement: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("restore_enclosing_label"); 
        let Some(outermost) = outermost_labeled_statement else {
            return node.clone();
        };
        let mut inner_label = node.clone();
        let statement_node = statement(outermost).clone();
        if is_labeled_statement(&statement_node) {
            inner_label = self.restore_enclosing_label(node, Some(&statement_node));
        }
        let label_node = label(outermost)
            .expect("labeled statement requires label")
            .clone();
        self.update_labeled_statement(outermost, &label_node, &inner_label)
    }

    pub fn new_type_check(&self, value: &Arc<Node>, tag: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_check"); 
        if tag == "null" {
            self.new_strict_equality_expression(
                value,
                &self.new_keyword_expression(SyntaxKind::NullKeyword),
            )
        } else if tag == "undefined" {
            self.new_strict_equality_expression(value, &self.new_void_zero_expression())
        } else {
            self.new_strict_equality_expression(
                &self.new_type_of_expression(value),
                &self.new_string_literal(tag, TokenFlags::default()),
            )
        }
    }

    pub fn new_method_call(
        &self,
        object: &Arc<Node>,
        method_name: &Arc<Node>,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_method_call"); 
        let property_access =
            self.new_property_access_expression(object, None, method_name, NodeFlags::empty());
        if is_call_expression(object) && object.flags.contains(NodeFlags::OptionalChain) {
            return self.new_call_expression(
                &property_access,
                None,
                None,
                self.new_node_list(arguments_list),
                NodeFlags::OptionalChain,
            );
        }
        self.new_call_expression(
            &property_access,
            None,
            None,
            self.new_node_list(arguments_list),
            NodeFlags::empty(),
        )
    }

    pub fn new_global_method_call(
        &self,
        global_object_name: &str,
        method_name: &str,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_global_method_call"); 
        self.new_method_call(
            &self.new_identifier(global_object_name),
            &self.new_identifier(method_name),
            arguments_list,
        )
    }

    pub fn new_function_call_call(
        &self,
        target: &Arc<Node>,
        this_arg: &Arc<Node>,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_call_call"); 
        let mut args = Vec::with_capacity(1 + arguments_list.len());
        args.push(this_arg.clone());
        args.extend(arguments_list.iter().cloned());
        self.new_method_call(target, &self.new_identifier("call"), &args)
    }

    pub fn is_ignorable_paren(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_ignorable_paren"); 
        is_parenthesized_expression(node)
            && node_is_synthesized(node)
            && range_is_synthesized(self.emit_context.source_map_range(node))
            && range_is_synthesized(self.emit_context.comment_range(node))
    }

    pub fn update_outer_expression(
        &self,
        outer_expression: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_outer_expression"); 
        match outer_expression.kind {
            SyntaxKind::ParenthesizedExpression => {
                self.update_parenthesized_expression(outer_expression, expression)
            }
            SyntaxKind::TypeAssertionExpression => {
                let type_node = outer_expression
                    .type_node()
                    .expect("type assertion requires type node")
                    .clone();
                self.update_type_assertion(outer_expression, &type_node, expression)
            }
            SyntaxKind::AsExpression => {
                let type_node = outer_expression
                    .type_node()
                    .expect("as expression requires type node")
                    .clone();
                self.update_as_expression(outer_expression, expression, &type_node)
            }
            SyntaxKind::SatisfiesExpression => {
                let type_node = outer_expression
                    .type_node()
                    .expect("satisfies expression requires type node")
                    .clone();
                self.update_satisfies_expression(outer_expression, expression, &type_node)
            }
            SyntaxKind::NonNullExpression => self.update_non_null_expression(
                outer_expression,
                expression,
                outer_expression.flags,
            ),
            SyntaxKind::ExpressionWithTypeArguments => {
                let type_arguments = type_argument_list(outer_expression).cloned();
                self.update_expression_with_type_arguments(
                    outer_expression,
                    expression,
                    type_arguments,
                )
            }
            SyntaxKind::PartiallyEmittedExpression => {
                self.update_partially_emitted_expression(outer_expression, expression)
            }
            _ => panic!(
                "Unexpected outer expression kind: {:?}",
                outer_expression.kind
            ),
        }
    }

    pub fn restore_outer_expressions(
        &self,
        outer_expression: Option<&Arc<Node>>,
        inner_expression: &Arc<Node>,
        kinds: OuterExpressionKinds,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("restore_outer_expressions"); 
        if let Some(outer) = outer_expression {
            if is_outer_expression(outer, kinds) && !self.is_ignorable_paren(outer) {
                let outer_inner = outer
                    .expression()
                    .expect("outer expression requires inner expression")
                    .clone();
                return self.update_outer_expression(
                    outer,
                    &self.restore_outer_expressions(
                        Some(&outer_inner),
                        inner_expression,
                        OuterExpressionKinds::ALL,
                    ),
                );
            }
        }
        inner_expression.clone()
    }

    pub fn split_standard_prologue<'a>(
        &self,
        source: &'a [Arc<Node>],
    ) -> (&'a [Arc<Node>], &'a [Arc<Node>]) { ::tsox_core::fntrace::enter("split_standard_prologue"); 
        for (i, statement) in source.iter().enumerate() {
            if !is_prologue_directive(statement) {
                return (&source[..i], &source[i..]);
            }
        }
        (source, &[])
    }

    pub fn split_custom_prologue<'a>(
        &self,
        source: &'a [Arc<Node>],
    ) -> (&'a [Arc<Node>], &'a [Arc<Node>]) { ::tsox_core::fntrace::enter("split_custom_prologue"); 
        for (i, statement) in source.iter().enumerate() {
            if is_prologue_directive(statement)
                || !self
                    .emit_context
                    .emit_flags(statement)
                    .intersects(EF_CUSTOM_PROLOGUE)
            {
                return (&source[..i], &source[i..]);
            }
        }
        (&[], source)
    }

    pub fn get_name(
        &self,
        node: &Arc<Node>,
        emit_flags: EmitFlags,
        opts: AssignedNameOptions,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("get_name"); 
        let mut node_name: Option<Arc<Node>> = None;
        if opts.ignore_assigned_name {
            node_name = get_non_assigned_name_of_declaration(node);
        } else {
            node_name = get_name_of_declaration(node);
        }

        if let Some(node_name) = node_name {
            let name = node_name.clone();
            let mut emit_flags = emit_flags;
            if !opts.allow_comments {
                emit_flags |= EF_NO_COMMENTS;
            }
            if !opts.allow_source_maps {
                emit_flags |= EF_NO_SOURCE_MAP;
            }
            self.emit_context.add_emit_flags(&name, emit_flags);
            return name;
        }

        self.new_generated_name_for_node(node)
    }

    pub fn new_unscoped_helper_name(&self, name: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_unscoped_helper_name"); 
        let node = self.new_identifier(name);
        self.emit_context.set_emit_flags(&node, EF_HELPER_NAME);
        node
    }

    pub fn new_metadata_helper(&self, metadata_key: &str, metadata_value: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_metadata_helper"); 
        self.emit_context.request_emit_helper(metadata_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__metadata"),
            None,
            None,
            self.new_node_list(&[
                self.new_string_literal(metadata_key, TokenFlags::default()),
                metadata_value.clone(),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_param_helper(
        &self,
        expression: &Arc<Node>,
        parameter_offset: usize,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_param_helper"); 
        self.emit_context.request_emit_helper(param_helper());
        let mut helper = self.new_call_expression(
            &self.new_unscoped_helper_name("__param"),
            None,
            None,
            self.new_node_list(&[
                self.new_numeric_literal(&parameter_offset.to_string(), TokenFlags::default()),
                expression.clone(),
            ]),
            NodeFlags::empty(),
        );
        if let Some(node) = Arc::get_mut(&mut helper) {
            node.loc = location;
        }
        helper
    }

    pub fn new_object_define_property_call(
        &self,
        target: &Arc<Node>,
        name: &Arc<Node>,
        descriptor: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_object_define_property_call"); 
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Object"),
                None,
                &self.new_identifier("defineProperty"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(&[target.clone(), name.clone(), descriptor.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_reflect_get_call(
        &self,
        target: &Arc<Node>,
        property_key: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_reflect_get_call"); 
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Reflect"),
                None,
                &self.new_identifier("get"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(&[target.clone(), property_key.clone(), receiver.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_reflect_set_call(
        &self,
        target: &Arc<Node>,
        property_key: &Arc<Node>,
        value: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_reflect_set_call"); 
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Reflect"),
                None,
                &self.new_identifier("set"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(&[
                target.clone(),
                property_key.clone(),
                value.clone(),
                receiver.clone(),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_immediately_invoked_arrow_function(&self, statements: &[Arc<Node>]) -> Arc<Node> { ::tsox_core::fntrace::enter("new_immediately_invoked_arrow_function"); 
        let arrow = self.new_arrow_function(
            None,
            None,
            self.new_node_list(&[]),
            None,
            None,
            self.new_token(SyntaxKind::EqualsGreaterThanToken),
            self.new_block(self.new_node_list(statements), true),
        );
        self.new_call_expression(
            &self.new_parenthesized_expression(&arrow),
            None,
            None,
            self.new_node_list(&[]),
            NodeFlags::empty(),
        )
    }

    pub fn new_rest_helper(
        &self,
        value: &Arc<Node>,
        elements: &[Arc<Node>],
        computed_temp_variables: &[Arc<Node>],
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_rest_helper"); 
        self.emit_context.request_emit_helper(rest_helper());
        let mut property_names: Vec<Arc<Node>> = Vec::new();
        let mut computed_temp_variable_offset = 0;
        for (i, element) in elements.iter().enumerate() {
            if i == elements.len() - 1 {
                break;
            }
            if let Some(property_name) =
                try_get_property_name_of_binding_or_assignment_element(element)
            {
                if is_computed_property_name(&property_name) {
                    let temp = &computed_temp_variables[computed_temp_variable_offset];
                    computed_temp_variable_offset += 1;
                    property_names.push(self.new_conditional_expression(
                        &self.new_type_check(temp, "symbol"),
                        self.new_token(SyntaxKind::QuestionToken),
                        temp,
                        self.new_token(SyntaxKind::ColonToken),
                        &self.new_binary_expression(
                            None,
                            temp,
                            None,
                            self.new_token(SyntaxKind::PlusToken),
                            &self.new_string_literal("", TokenFlags::default()),
                        ),
                    ));
                } else {
                    property_names.push(self.new_string_literal_from_node(&property_name));
                }
            }
        }
        let mut prop_names = self.new_array_literal_expression(
            self.new_node_list(&property_names),
            false,
        );
        if let Some(node) = Arc::get_mut(&mut prop_names) {
            node.loc = location;
        }
        self.new_call_expression(
            &self.new_unscoped_helper_name("__rest"),
            None,
            None,
            self.new_node_list(&[value.clone(), prop_names]),
            NodeFlags::empty(),
        )
    }

    pub fn new_run_initializers_helper(
        &self,
        this_arg: &Arc<Node>,
        initializers: &Arc<Node>,
        value: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_run_initializers_helper"); 
        self.emit_context.request_emit_helper(run_initializers_helper());
        let arguments: Vec<Arc<Node>> = match value {
            Some(value) => vec![this_arg.clone(), initializers.clone(), value.clone()],
            None => vec![this_arg.clone(), initializers.clone()],
        };
        self.new_call_expression(
            &self.new_unscoped_helper_name("__runInitializers"),
            None,
            None,
            self.new_node_list(&arguments),
            NodeFlags::empty(),
        )
    }

    pub fn new_template_object_helper(
        &self,
        cooked_array: &Arc<Node>,
        raw_array: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_template_object_helper"); 
        self.emit_context.request_emit_helper(make_template_object_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__makeTemplateObject"),
            None,
            None,
            self.new_node_list(&[cooked_array.clone(), raw_array.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_prop_key_helper(&self, expr: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_prop_key_helper"); 
        self.emit_context.request_emit_helper(prop_key_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__propKey"),
            None,
            None,
            self.new_node_list(&[expr.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_set_function_name_helper(
        &self,
        fn_: &Arc<Node>,
        name: &Arc<Node>,
        prefix: &str,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_set_function_name_helper"); 
        self.emit_context.request_emit_helper(set_function_name_helper());
        let arguments: Vec<Arc<Node>> = if !prefix.is_empty() {
            vec![
                fn_.clone(),
                name.clone(),
                self.new_string_literal(prefix, TokenFlags::default()),
            ]
        } else {
            vec![fn_.clone(), name.clone()]
        };
        self.new_call_expression(
            &self.new_unscoped_helper_name("__setFunctionName"),
            None,
            None,
            self.new_node_list(&arguments),
            NodeFlags::empty(),
        )
    }

    pub fn new_import_default_helper(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_import_default_helper"); 
        self.emit_context.request_emit_helper(import_default_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__importDefault"),
            None,
            None,
            self.new_node_list(&[expression.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_import_star_helper(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_import_star_helper"); 
        self.emit_context.request_emit_helper(import_star_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__importStar"),
            None,
            None,
            self.new_node_list(&[expression.clone()]),
            NodeFlags::empty(),
        )
    }

    pub fn new_rewrite_relative_import_extensions_helper(
        &self,
        first_argument: &Arc<Node>,
        preserve_jsx: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_rewrite_relative_import_extensions_helper"); 
        self.emit_context
            .request_emit_helper(rewrite_relative_import_extensions_helper());
        let arguments: Vec<Arc<Node>> = if preserve_jsx {
            vec![
                first_argument.clone(),
                self.new_token(SyntaxKind::TrueKeyword),
            ]
        } else {
            vec![first_argument.clone()]
        };
        self.new_call_expression(
            &self.new_unscoped_helper_name("__rewriteRelativeImportExtension"),
            None,
            None,
            self.new_node_list(&arguments),
            NodeFlags::empty(),
        )
    }

    pub fn new_type_of_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_of_expression"); 
        Arc::new(Node::new(
            SyntaxKind::TypeOfExpression,
            NodeData::TypeOfExpression(crate::ast::node_data_generated::TypeOfExpressionData {
                expression: expression.clone(),
            }),
        ))
    }

    pub fn update_labeled_statement(
        &self,
        node: &Arc<Node>,
        label: &Arc<Node>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_labeled_statement"); 
        let data = match &node.data {
            NodeData::LabeledStatement(d) => d,
            _ => panic!("update_labeled_statement on non-labeled statement"),
        };
        if Arc::ptr_eq(&data.label, label) && Arc::ptr_eq(&data.statement, statement) {
            return node.clone();
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::LabeledStatement(crate::ast::node_data_generated::LabeledStatementData {
                label: label.clone(),
                statement: statement.clone(),
            }),
        ))
    }

    pub fn update_parenthesized_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_parenthesized_expression"); 
        let existing = match &node.data {
            NodeData::ParenthesizedExpression(d) => &d.expression,
            _ => panic!("update_parenthesized_expression on non-parenthesized expression"),
        };
        if Arc::ptr_eq(existing, expression) {
            return node.clone();
        }
        self.new_parenthesized_expression(expression)
    }

    pub fn update_type_assertion(
        &self,
        node: &Arc<Node>,
        type_node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_type_assertion"); 
        if let NodeData::TypeAssertion(d) = &node.data {
            if Arc::ptr_eq(&d.type_node, type_node) && Arc::ptr_eq(&d.expression, expression) {
                return node.clone();
            }
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::TypeAssertion(crate::ast::node_data_generated::TypeAssertionData {
                type_node: type_node.clone(),
                expression: expression.clone(),
            }),
        ))
    }

    pub fn update_as_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        type_node: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_as_expression"); 
        if let NodeData::AsExpression(d) = &node.data {
            if Arc::ptr_eq(&d.expression, expression) && Arc::ptr_eq(&d.type_node, type_node) {
                return node.clone();
            }
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::AsExpression(crate::ast::node_data_generated::AsExpressionData {
                expression: expression.clone(),
                type_node: type_node.clone(),
            }),
        ))
    }

    pub fn update_satisfies_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        type_node: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_satisfies_expression"); 
        if let NodeData::SatisfiesExpression(d) = &node.data {
            if Arc::ptr_eq(&d.expression, expression) && Arc::ptr_eq(&d.type_node, type_node) {
                return node.clone();
            }
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::SatisfiesExpression(crate::ast::node_data_generated::SatisfiesExpressionData {
                expression: expression.clone(),
                type_node: type_node.clone(),
            }),
        ))
    }

    pub fn update_non_null_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_non_null_expression"); 
        let unchanged = match &node.data {
            NodeData::NonNullExpression(d) => Arc::ptr_eq(&d.expression, expression),
            _ => false,
        };
        if unchanged && node.flags == flags {
            return node.clone();
        }
        let mut new_node = Node::new(
            node.kind,
            NodeData::NonNullExpression(crate::ast::node_data_generated::NonNullExpressionData {
                expression: expression.clone(),
            }),
        );
        new_node.flags = flags;
        Arc::new(new_node)
    }

    pub fn update_expression_with_type_arguments(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_expression_with_type_arguments"); 
        if let NodeData::ExpressionWithTypeArguments(d) = &node.data {
            let type_arguments_unchanged = match (&d.type_arguments, &type_arguments) {
                (None, None) => true,
                (Some(existing), Some(incoming)) => Arc::ptr_eq(existing, incoming),
                _ => false,
            };
            if Arc::ptr_eq(&d.expression, expression) && type_arguments_unchanged {
                return node.clone();
            }
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::ExpressionWithTypeArguments(
                crate::ast::node_data_generated::ExpressionWithTypeArgumentsData {
                    expression: expression.clone(),
                    type_arguments,
                },
            ),
        ))
    }

    pub fn update_partially_emitted_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_partially_emitted_expression"); 
        let unchanged = match &node.data {
            NodeData::PartiallyEmittedExpression(d) => Arc::ptr_eq(&d.expression, expression),
            _ => false,
        };
        if unchanged {
            return node.clone();
        }
        Arc::new(Node::new(
            node.kind,
            NodeData::PartiallyEmittedExpression(
                crate::ast::node_data_generated::PartiallyEmittedExpressionData {
                    expression: expression.clone(),
                },
            ),
        ))
    }
}

use crate::ast::utilities_navigation::get_non_assigned_name_of_declaration;

pub fn flatten_comma_element(node: &Arc<Node>, expressions: &mut Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("flatten_comma_element"); 
    if is_binary_expression(node)
        && node_is_synthesized(node)
        && matches!(
            &node.data,
            NodeData::BinaryExpression(binary)
                if binary.operator_token.kind == SyntaxKind::CommaToken
        )
    {
        if let NodeData::BinaryExpression(binary) = &node.data {
            flatten_comma_element(&binary.left, expressions);
            flatten_comma_element(&binary.right, expressions);
            return;
        }
    }
    expressions.push(node.clone());
}

pub fn flatten_comma_elements(expressions: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("flatten_comma_elements"); 
    let mut result = Vec::new();
    for expression in expressions {
        flatten_comma_element(expression, &mut result);
    }
    result
}
