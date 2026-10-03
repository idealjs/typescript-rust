use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    ComputedPropertyNameData, IdentifierData, ImportEqualsDeclarationData, ImportTypeNodeData,
    NodeData, ParameterDeclarationData, QualifiedNameData, TypeParameterDeclarationData,
    TypePredicateNodeData,
};
use tsox_frontend::ast::{ModifierFlags, ModifierList, Node, Symbol, SyntaxKind};

use crate::checker::nodecopy_builder::{EmitContextStub, NodeFactoryStub};
use crate::checker::mig::m2f::VerbosityContext;

pub use crate::checker::symboltracker::NodeBuilderFlags;

pub type EmitContext = EmitContextStub;
pub type IdToSymbolMap = HashMap<u64, Arc<Symbol>>;

pub const EF_NO_ASCII_ESCAPING: u32 = 1 << 6;

pub const TYPE_FORMAT_FLAGS_NODE_BUILDER_FLAGS_MASK: u32 =
    TypeFormatFlagsBits::WRITE_ARRAY_AS_GENERIC
        | TypeFormatFlagsBits::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE
        | TypeFormatFlagsBits::ALLOW_UNIQUE_ES_SYMBOL_TYPE
        | TypeFormatFlagsBits::NO_TRUNCATION
        | TypeFormatFlagsBits::MULTILINE_OBJECT_LITERALS;

pub enum PseudoLiteralValue {
    Str(String),
    Number(f64),
    Bool(bool),
    PseudoBigInt(String),
}

pub fn value_to_string(value: &PseudoLiteralValue) -> String { ::tsox_core::fntrace::enter("value_to_string"); 
    match value {
        PseudoLiteralValue::Str(s) => format!("\"{}\"", s),
        PseudoLiteralValue::Number(n) => n.to_string(),
        PseudoLiteralValue::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        PseudoLiteralValue::PseudoBigInt(v) => format!("{}n", v),
    }
}

pub struct SortedSymbolNamePair {
    pub sym: Arc<Symbol>,
    pub name: String,
}

pub fn count_path_components(s: &str) -> i32 { ::tsox_core::fntrace::enter("count_path_components"); 
    s.matches('/').count() as i32
}

#[derive(Clone, Debug, Default)]
pub struct PrinterOptions {
    pub remove_comments: bool,
    pub omit_trailing_semicolon: bool,
    pub never_ascii_escape: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PrintHandlers;

#[derive(Clone, Debug, Default)]
pub struct TextWriter {
    pieces: Vec<String>,
    new_line: String,
    indent: usize,
}

impl TextWriter {
    pub fn new(new_line: &str, indent: usize) -> Self { ::tsox_core::fntrace::enter("new"); 
        TextWriter {
            pieces: Vec::new(),
            new_line: new_line.to_string(),
            indent,
        }
    }

    pub fn write_piece(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_piece"); 
        self.pieces.push(s.to_string());
    }

    pub fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.pieces.concat()
    }
}

pub fn single_line_string_writer() -> TextWriter { ::tsox_core::fntrace::enter("single_line_string_writer"); 
    TextWriter::new("", 0)
}

#[derive(Default)]
pub struct Printer;

pub fn new_printer(
    _options: PrinterOptions,
    _handlers: PrintHandlers,
    _emit_context: &EmitContext,
) -> Printer { ::tsox_core::fntrace::enter("new_printer"); 
    Printer
}

impl Printer {
    pub fn emit(&self, _node: &Arc<Node>, _source_file: Option<&Node>) -> String { ::tsox_core::fntrace::enter("emit"); 
        String::new()
    }

    pub fn write(&self, _node: &Arc<Node>, _source_file: Option<&Node>, writer: &mut TextWriter) { ::tsox_core::fntrace::enter("write"); 
        writer.write_piece("");
    }
}

pub struct VerbosityContextHandle<'a>(pub &'a mut VerbosityContext);

impl<'a> VerbosityContextHandle<'a> {
    pub fn borrow_mut(vc: &'a mut VerbosityContext) -> Self { ::tsox_core::fntrace::enter("borrow_mut"); 
        VerbosityContextHandle(vc)
    }
}

impl Deref for VerbosityContextHandle<'_> {
    type Target = VerbosityContext;

    fn deref(&self) -> &VerbosityContext { ::tsox_core::fntrace::enter("deref"); 
        self.0
    }
}

impl DerefMut for VerbosityContextHandle<'_> {
    fn deref_mut(&mut self) -> &mut VerbosityContext { ::tsox_core::fntrace::enter("deref_mut"); 
        self.0
    }
}

pub trait EmitContextStubExt {
    fn assign_comment_range(&self, _node: &Arc<Node>, _range: &Arc<Node>) { ::tsox_core::fntrace::enter("assign_comment_range"); }
}

impl EmitContextStubExt for EmitContextStub {}

pub trait NodeFactoryExt21 {
    fn new_token(&self, kind: SyntaxKind) -> Arc<Node>;
    fn new_identifier(&self, text: &str) -> Arc<Node>;
    fn new_computed_property_name(&self, expression: &mut Arc<Node>) -> Arc<Node>;
    fn new_qualified_name(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node>;
    fn create_modifiers_from_modifier_flags(&self, flags: ModifierFlags) -> Vec<Arc<Node>>;
    fn new_modifier_list(&self, nodes: &[Arc<Node>]) -> Option<Arc<ModifierList>>;
    fn new_type_parameter_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        constraint: Option<Arc<Node>>,
        expression: Option<Arc<Node>>,
        default_type: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_this_type_node(&self) -> Arc<Node>;
    fn new_type_predicate_node(
        &self,
        asserts_modifier: Option<Arc<Node>>,
        parameter_name: &Arc<Node>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_keyword_type_node(&self, kind: SyntaxKind) -> Arc<Node>;
    fn clone_node(&self, node: &Arc<Node>) -> Arc<Node>;
    fn new_parameter_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<Arc<Node>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
}

fn token_node(kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("token_node"); 
    Arc::new(Node::new(kind, NodeData::Token))
}

impl NodeFactoryExt21 for NodeFactoryStub {
    fn new_token(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_token"); 
        token_node(kind)
    }

    fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    fn new_computed_property_name(&self, expression: &mut Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_computed_property_name"); 
        Arc::new(Node::new(
            SyntaxKind::ComputedPropertyName,
            NodeData::ComputedPropertyName(ComputedPropertyNameData {
                expression: Arc::clone(expression),
            }),
        ))
    }

    fn new_qualified_name(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_qualified_name"); 
        Arc::new(Node::new(
            SyntaxKind::QualifiedName,
            NodeData::QualifiedName(QualifiedNameData {
                left: Arc::clone(left),
                right: Arc::clone(right),
            }),
        ))
    }

    fn create_modifiers_from_modifier_flags(&self, flags: ModifierFlags) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("create_modifiers_from_modifier_flags"); 
        tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags(flags, token_node)
    }

    fn new_modifier_list(&self, nodes: &[Arc<Node>]) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("new_modifier_list"); 
        let owned: Vec<Arc<Node>> = nodes.to_vec();
        let flags = tsox_frontend::ast::modifiers_to_flags(&owned);
        Some(Arc::new(ModifierList::new(owned, flags)))
    }

    fn new_type_parameter_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        constraint: Option<Arc<Node>>,
        expression: Option<Arc<Node>>,
        default_type: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_parameter_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers,
                name: Arc::clone(name),
                constraint,
                expression,
                default_type,
            }),
        ))
    }

    fn new_this_type_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_this_type_node"); 
        token_node(SyntaxKind::ThisType)
    }

    fn new_type_predicate_node(
        &self,
        asserts_modifier: Option<Arc<Node>>,
        parameter_name: &Arc<Node>,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_type_predicate_node"); 
        Arc::new(Node::new(
            SyntaxKind::TypePredicate,
            NodeData::TypePredicateNode(TypePredicateNodeData {
                asserts_modifier,
                parameter_name: Arc::clone(parameter_name),
                type_node,
            }),
        ))
    }

    fn new_keyword_type_node(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_keyword_type_node"); 
        token_node(kind)
    }

    fn clone_node(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("clone_node"); 
        Arc::clone(node)
    }

    fn new_parameter_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<Arc<Node>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_parameter_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
                initializer,
            }),
        ))
    }
}

pub trait NodeM2gExt {
    fn module_specifier(&self) -> Option<&Arc<Node>>;
    fn as_import_equals_declaration(&self) -> Option<&ImportEqualsDeclarationData>;
    fn as_import_type_node(&self) -> Option<&ImportTypeNodeData>;
}

impl NodeM2gExt for Node {
    fn module_specifier(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("module_specifier"); 
        tsox_frontend::ast::mig::m3b::module_specifier(self)
    }

    fn as_import_equals_declaration(&self) -> Option<&ImportEqualsDeclarationData> { ::tsox_core::fntrace::enter("as_import_equals_declaration"); 
        match &self.data {
            NodeData::ImportEqualsDeclaration(d) => Some(d),
            _ => None,
        }
    }

    fn as_import_type_node(&self) -> Option<&ImportTypeNodeData> { ::tsox_core::fntrace::enter("as_import_type_node"); 
        match &self.data {
            NodeData::ImportTypeNode(d) => Some(d),
            _ => None,
        }
    }
}

pub fn is_literal_import_type_node_m2g(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_literal_import_type_node_m2g"); 
    crate::checker::mig::m1a::r19k2_defs::is_literal_import_type_node(node)
}

struct TypeFormatFlagsBits;

impl TypeFormatFlagsBits {
    const WRITE_ARRAY_AS_GENERIC: u32 = 1 << 1;
    const USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE: u32 = 1 << 2;
    const ALLOW_UNIQUE_ES_SYMBOL_TYPE: u32 = 1 << 3;
    const NO_TRUNCATION: u32 = 1 << 7;
    const MULTILINE_OBJECT_LITERALS: u32 = 1 << 8;
}
