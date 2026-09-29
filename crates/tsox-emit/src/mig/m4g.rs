#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList, SourceFile};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::subtree_facts::{SubtreeContainsClassFields, SubtreeContainsLexicalSuper, SubtreeContainsLexicalThis};
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{is_class_expression, is_identifier, node_name};
use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::format::mig::m4o::EmitFlags;
use crate::mig::m4g::r33k7_defs::{PrivateIdentifierKind, ReferenceResolver, property_access_name};
use crate::mig::m4k::R39K02NodeExt;

#[path = "r33k7_defs.rs"]
pub mod r33k7_defs;

#[path = "r39k15_defs.rs"]
pub mod r39k15_defs;

#[path = "r40k14_defs.rs"]
pub mod r40k14_defs;

use crate::mig::m4g_2::r37k13_defs::{ClassFieldsTransformerR37k13, EmitContextR37k13};
use crate::mig::m4g::r39k15_defs::ClassFieldsTransformerR39k15;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ClassFacts: u32 {
        const None = 0;
        const ClassWasDecorated = 1 << 0;
        const NeedsClassConstructorReference = 1 << 1;
        const NeedsClassSuperReference = 1 << 2;
        const NeedsSubstitutionForThisInClassStaticField = 1 << 3;
        const WillHoistInitializersToConstructor = 1 << 4;
    }
}

pub struct PrivateIdentifierInfo {
    pub kind: PrivateIdentifierKind,
    pub brand_check_identifier: Option<Arc<Node>>,
    pub is_static: bool,
    pub is_valid: bool,
    pub variable_name: Option<Arc<Node>>,
    pub method_name: Option<Arc<Node>>,
    pub getter_name: Option<Arc<Node>>,
    pub setter_name: Option<Arc<Node>>,
}

#[derive(Clone)]
pub struct PrivateEnvironmentData {
    pub class_name: Option<Arc<Node>>,
    pub weak_set_name: Option<Arc<Node>>,
}

#[derive(Clone)]
pub struct PrivateEnvironment {
    pub data: PrivateEnvironmentData,
    pub members: HashMap<String, PrivateIdentifierInfo>,
    pub generated_identifiers: HashMap<*const Node, PrivateIdentifierInfo>,
}

#[derive(Clone)]
pub struct ClassLexicalEnvironment {
    pub facts: ClassFacts,
    pub class_constructor: Option<Arc<Node>>,
    pub class_this: Option<Arc<Node>>,
    pub super_class_reference: Option<Arc<Node>>,
}

#[derive(Clone)]
pub struct ClassLexicalEnv {
    pub previous: Option<Box<ClassLexicalEnv>>,
    pub data: Option<ClassLexicalEnvironment>,
    pub private_env: Option<PrivateEnvironment>,
}

pub struct ClassFieldsTransformer {
    pub compiler_options: Arc<CompilerOptions>,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub should_transform_initializers_using_set: bool,
    pub should_transform_initializers_using_define: bool,
    pub should_transform_initializers: bool,
    pub should_transform_private_elements_or_class_static_blocks: bool,
    pub should_transform_auto_accessors: bool,
    pub should_transform_this_in_static_initializers: bool,
    pub should_transform_super_in_static_initializers: bool,
    pub should_transform_private_static_elements_in_file: bool,
    pub legacy_decorators: bool,
    pub pending_expressions: Vec<Arc<Node>>,
    pub pending_statements: Vec<Arc<Node>>,
    pub lexical_environment: Option<ClassLexicalEnv>,
    pub current_class_container: Option<Arc<Node>>,
    pub current_class_element: Option<Arc<Node>>,
    pub class_aliases: HashMap<usize, Arc<Node>>,
    pub enclosing_class_declarations: HashSet<usize>,
    pub in_iteration_statement: bool,
    pub inside_computed_property_name: bool,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,
    pub is_anonymous_class_needing_assigned_name: fn(&ClassFieldsTransformer, &Arc<Node>) -> bool,
}

fn with_loc(mut node: Arc<Node>, loc: TextRange) -> Arc<Node> {
    if let Some(n) = Arc::get_mut(&mut node) {
        n.loc = loc;
    }
    node
}

impl ClassFieldsTransformer {
    fn push_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent = self.parent_node.take();
        self.parent_node = self.current_node.take();
        self.current_node = Some(node.clone());
        grandparent
    }

    fn pop_node(&mut self, grandparent_node: Option<Arc<Node>>) {
        self.current_node = self.parent_node.take();
        self.parent_node = grandparent_node;
    }

    pub(crate) fn requires_block_scoped_var(&self) -> bool {
        self.in_iteration_statement
            && self.current_class_container.is_some()
            && is_class_expression(self.current_class_container.as_ref().unwrap())
    }

    pub fn visit_source_file(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if node.is_declaration_file_node() {
            return node.clone();
        }
        self.lexical_environment = None;
        self.should_transform_private_static_elements_in_file = self
            .emit_context()
            .emit_flags(node)
            .contains(EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
        self.class_aliases = HashMap::new();
        self.enclosing_class_declarations.clear();
        let visited = self.visitor().visit_each_child(node);
        let helpers = self.emit_context().read_emit_helpers();
        self.emit_context().add_emit_helper(&visited, &helpers);
        self.class_aliases = HashMap::new();
        self.enclosing_class_declarations.clear();
        visited
    }

    pub fn visit_modifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::AccessorKeyword {
            if self.should_transform_auto_accessors_in_current_class() {
                return None;
            }
            return Some(node.clone());
        }
        if is_modifier(node) {
            return Some(node.clone());
        }
        None
    }

    pub fn visit_for_substitution(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if node.kind == SyntaxKind::Identifier {
            return self.visit_identifier(node);
        }
        if node.kind == SyntaxKind::PropertyAccessExpression && is_identifier(property_access_name(node)) {            return self.visit_property_access_expression_for_substitution(node);
        }
        self.substitution_visitor().visit_each_child(node)
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let grandparent_node = self.push_node(node);
        let result = self.visit_worker(node);
        self.pop_node(grandparent_node);
        result
    }

    fn visit_worker(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if !node.subtree_facts().intersects(
            SubtreeContainsClassFields | SubtreeContainsLexicalThis | SubtreeContainsLexicalSuper,
        ) {
            if self.current_class_container.is_some() && !self.class_aliases.is_empty() {
                return self.visit_for_substitution(node);
            }
            return node.clone();
        }

        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ClassDeclaration => self.visit_class_declaration(node),
            SyntaxKind::ClassExpression => self.visit_class_expression(node),
            SyntaxKind::ClassStaticBlockDeclaration | SyntaxKind::PropertyDeclaration => {
                panic!("Use `classElementVisitor` instead.")
            }
            SyntaxKind::PropertyAssignment => self.visit_property_assignment(node),
            SyntaxKind::VariableStatement => self.visit_variable_statement(node),
            SyntaxKind::VariableDeclaration => self.visit_variable_declaration(node),
            SyntaxKind::Parameter => self.visit_parameter_declaration(node),
            SyntaxKind::BindingElement => self.visit_binding_element(node),
            SyntaxKind::ExportAssignment => self.visit_export_assignment(node),
            SyntaxKind::PrivateIdentifier => self.visit_private_identifier(node),
            SyntaxKind::PropertyAccessExpression => self.visit_property_access_expression(node),
            SyntaxKind::ElementAccessExpression => self.visit_element_access_expression(node),
            SyntaxKind::PrefixUnaryExpression | SyntaxKind::PostfixUnaryExpression => {
                self.visit_pre_or_postfix_unary_expression(node, false)
            }
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, false),
            SyntaxKind::ParenthesizedExpression => self.visit_parenthesized_expression(node, false),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            SyntaxKind::ForStatement => self.visit_for_statement(node),
            SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement => {
                let saved = self.in_iteration_statement;
                self.in_iteration_statement = true;
                let result = self.visit_each_child_of_node(node);
                self.in_iteration_statement = saved;
                result
            }
            SyntaxKind::ThisKeyword => self.visit_this_expression_k15(node),
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => {
                let saved = self.in_iteration_statement;
                self.in_iteration_statement = false;
                let result = self.visit_function_expression_or_declaration(node);
                self.in_iteration_statement = saved;
                result
            }
            SyntaxKind::Constructor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                let saved = self.in_iteration_statement;
                self.in_iteration_statement = false;
                let saved_element = self.current_class_element.clone();
                self.current_class_element = Some(node.clone());
                let result = self.visit_each_child_of_node(node);
                self.current_class_element = saved_element;
                self.in_iteration_statement = saved;
                result
            }
            _ => self.visitor().visit_each_child(node),
        }
    }
}
