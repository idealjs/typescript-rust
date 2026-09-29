use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4h::r39k13_defs;
use crate::mig::m4k_2::Transformer;
use crate::mig::m4p_4::r37k15_defs::{R37k15NodeVisitorExt, R37k15PlaceholderExt};
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::visitor::NodeVisitor;

/// Go: map[*ast.Node]*memberInfo(以节点指针为键);Rust Node 未实现 Eq/Hash,
/// 以 Arc 指针等价比较的小型有序映射替代。
#[derive(Default)]
pub struct MemberInfoMap {
    entries: Vec<(Arc<Node>, MemberInfo)>,
}

impl MemberInfoMap {
    pub fn set(&mut self, key: Arc<Node>, value: MemberInfo) {
        if let Some(slot) = self.get_mut(&key) {
            *slot = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn get_mut(&mut self, key: &Arc<Node>) -> Option<&mut MemberInfo> {
        self.entries
            .iter_mut()
            .find(|(k, _)| Arc::ptr_eq(k, key))
            .map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Arc<Node>, &MemberInfo)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexicalEntryKind {
    Class,
    ClassElement,
    Name,
    Other,
}

pub struct LexicalEntry {
    pub kind: LexicalEntryKind,
    pub next: Option<Box<LexicalEntry>>,
    pub class_info_data: Option<Arc<ClassInfo>>,
    pub saved_pending_expressions: Vec<Arc<Node>>,
    pub class_this_data: Option<Arc<Node>>,
    pub class_super_data: Option<Arc<Node>>,
    pub depth: i32,
}

pub struct MemberInfo {
    pub member_decorators_name: Option<Arc<Node>>,
    pub member_initializers_name: Option<Arc<Node>>,
    pub member_extra_initializers_name: Option<Arc<Node>>,
    pub member_descriptor_name: Option<Arc<Node>>,
}

pub struct ClassInfo {
    pub class: Arc<Node>,
    pub class_decorators_name: Option<Arc<Node>>,
    pub class_descriptor_name: Option<Arc<Node>>,
    pub class_extra_initializers_name: Option<Arc<Node>>,
    pub class_this: Option<Arc<Node>>,
    pub class_super: Option<Arc<Node>>,
    pub metadata_reference: Arc<Node>,
    pub member_infos: MemberInfoMap,
    pub instance_method_extra_initializers_name: Option<Arc<Node>>,
    pub static_method_extra_initializers_name: Option<Arc<Node>>,
    pub static_non_field_decoration_statements: Vec<Arc<Node>>,
    pub non_static_non_field_decoration_statements: Vec<Arc<Node>>,
    pub static_field_decoration_statements: Vec<Arc<Node>>,
    pub non_static_field_decoration_statements: Vec<Arc<Node>>,
    pub has_static_initializers: bool,
    pub has_non_ambient_instance_fields: bool,
    pub has_static_private_class_elements: bool,
    pub pending_static_initializers: Vec<Arc<Node>>,
    pub pending_instance_initializers: Vec<Arc<Node>>,
}

pub struct EsDecoratorTransformer {
    pub transformer: Transformer,
    pub compiler_options: CompilerOptions,
    pub top: Option<Box<LexicalEntry>>,
    pub class_info_stack: Option<Arc<ClassInfo>>,
    pub class_this: Option<Arc<Node>>,
    pub class_super: Option<Arc<Node>>,
    pub pending_expressions: Vec<Arc<Node>>,
    pub outer_this: Option<Arc<Node>>,
    pub should_transform_private_static_elements_in_file: bool,
    pub outer_this_visitor: NodeVisitor,
    pub discarded_visitor: NodeVisitor,
    pub modifier_visitor: NodeVisitor,
    pub export_stripping_modifier_visitor: NodeVisitor,
    pub class_element_visitor: NodeVisitor,
    pub non_constructor_class_element_visitor: NodeVisitor,
    pub constructor_class_element_visitor: NodeVisitor,
    pub array_assignment_visitor: NodeVisitor,
    pub object_assignment_visitor: NodeVisitor,
    pub static_only_modifier_visitor: NodeVisitor,
    pub async_only_modifier_visitor: NodeVisitor,
    pub accessor_stripping_modifier_visitor: NodeVisitor,
}

pub fn new_es_decorator_transformer(opts: &TransformOptions) -> Option<Transformer> {
    if opts.compiler_options.experimental_decorators.is_true()
        || (opts.compiler_options.get_emit_script_target() >= ScriptTarget::ESNext
            && opts.compiler_options.get_use_define_for_class_fields())
    {
        return None;
    }
    let mut tx = EsDecoratorTransformer {
        transformer: r39k13_defs::placeholder_transformer(),
        compiler_options: opts.compiler_options.clone(),
        top: None,
        class_info_stack: None,
        class_this: None,
        class_super: None,
        pending_expressions: Vec::new(),
        outer_this: None,
        should_transform_private_static_elements_in_file: false,
        outer_this_visitor: NodeVisitor::placeholder(),
        discarded_visitor: NodeVisitor::placeholder(),
        modifier_visitor: NodeVisitor::placeholder(),
        export_stripping_modifier_visitor: NodeVisitor::placeholder(),
        class_element_visitor: NodeVisitor::placeholder(),
        non_constructor_class_element_visitor: NodeVisitor::placeholder(),
        constructor_class_element_visitor: NodeVisitor::placeholder(),
        array_assignment_visitor: NodeVisitor::placeholder(),
        object_assignment_visitor: NodeVisitor::placeholder(),
        static_only_modifier_visitor: NodeVisitor::placeholder(),
        async_only_modifier_visitor: NodeVisitor::placeholder(),
        accessor_stripping_modifier_visitor: NodeVisitor::placeholder(),
    };
    fn es_decorator_visit_fn(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> {
        Some(node)
    }
    let result = Transformer::new(es_decorator_visit_fn, Some(opts.context.clone()));
    let ec = tx.transformer.emit_context();
    tx.outer_this_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            Some(tx.outer_this_visit(&n))
        });
    tx.discarded_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            Some(tx.discarded_value_visit(&n))
        });
    tx.modifier_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            tx.modifier_visitor_visit(&n)
        });
    tx.export_stripping_modifier_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            tx.export_stripping_modifier_visit(&n)
        });
    tx.class_element_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            tx.class_element_visitor_visit(&n)
        });
    tx.non_constructor_class_element_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            Some(tx.non_constructor_class_element_visit(&n))
        });
    tx.constructor_class_element_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            tx.constructor_class_element_visit(&n)
        });
    tx.array_assignment_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            Some(tx.visit_array_assignment_element(&n))
        });
    tx.object_assignment_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            Some(tx.visit_object_assignment_element(&n))
        });
    tx.static_only_modifier_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            if n.kind == SyntaxKind::StaticKeyword {
                Some(n)
            } else {
                None
            }
        });
    tx.async_only_modifier_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            if n.kind == SyntaxKind::AsyncKeyword {
                Some(n)
            } else {
                None
            }
        });
    tx.accessor_stripping_modifier_visitor =
        ec.new_node_visitor(|tx: &mut EsDecoratorTransformer, n: Arc<Node>| {
            if n.kind == SyntaxKind::AccessorKeyword {
                None
            } else {
                Some(n)
            }
        });
    Some(result)
}

impl EsDecoratorTransformer {
    pub fn update_state(&mut self) {
        self.class_info_stack = None;
        self.class_this = None;
        self.class_super = None;
        let top = match &self.top {
            Some(top) => top,
            None => return,
        };
        match top.kind {
            LexicalEntryKind::Class => {
                self.class_info_stack = top.class_info_data.clone();
            }
            LexicalEntryKind::ClassElement => {
                self.class_info_stack = top.next.as_ref().unwrap().class_info_data.clone();
                self.class_this = top.class_this_data.clone();
                self.class_super = top.class_super_data.clone();
            }
            LexicalEntryKind::Name => {
                let grandparent = top
                    .next
                    .as_ref()
                    .and_then(|n| n.next.as_ref())
                    .and_then(|n| n.next.as_deref());
                if let Some(grandparent) = grandparent {
                    if grandparent.kind == LexicalEntryKind::ClassElement {
                        self.class_info_stack = grandparent.next.as_ref().unwrap().class_info_data.clone();
                        self.class_this = grandparent.class_this_data.clone();
                        self.class_super = grandparent.class_super_data.clone();
                    }
                }
            }
            LexicalEntryKind::Other => {}
        }
    }

    pub fn visit_source_file(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.top = None;
        self.should_transform_private_static_elements_in_file = false;
        let visited = self.transformer.visitor().visit_each_child(node);
        self.transformer
            .emit_context()
            .add_emit_helper(&visited, &self.transformer.emit_context().read_emit_helpers());
        if self.should_transform_private_static_elements_in_file {
            self.transformer
                .emit_context()
                .add_emit_flags(&visited, EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
            self.should_transform_private_static_elements_in_file = false;
        }
        visited
    }

    pub fn outer_this_visit(&mut self, n: &Arc<Node>) -> Arc<Node> {
        if !n.subtree_facts().intersects(SubtreeFacts::LexicalThis)
            && n.kind != SyntaxKind::ThisKeyword
        {
            return n.clone();
        }
        if n.kind == SyntaxKind::ThisKeyword {
            if self.outer_this.is_none() {
                self.outer_this = Some(self.transformer.factory().generated_name_node(
                    &self.transformer.factory().new_unique_name_ex(
                        "_outerThis",
                        AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC,
                            prefix: String::new(),
                            suffix: String::new(),
                        },
                    ),
                ));
            }
            return self.outer_this.clone().unwrap();
        }
        self.outer_this_visitor.visit_each_child(n)
    }

    pub fn should_visit_node(&self, node: &Arc<Node>) -> bool {
        node.subtree_facts().intersects(SubtreeFacts::Decorators)
            || (self.class_this.is_some()
                && node
                    .subtree_facts()
                    .intersects(SubtreeFacts::LexicalThis))
            || (self.class_this.is_some()
                && self.class_super.is_some()
                && node
                    .subtree_facts()
                    .intersects(SubtreeFacts::LexicalSuper))
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if node.kind == SyntaxKind::SourceFile {
            return self.visit_source_file(node);
        }
        if !self.should_visit_node(node) {
            return node.clone();
        }
        match node.kind {
            SyntaxKind::Decorator => Arc::new(Node::new(
                SyntaxKind::Unknown,
                NodeData::MissingDeclaration(tsox_frontend::ast::node_data_generated::MissingDeclarationData { modifiers: None }),
            )),
            SyntaxKind::ClassDeclaration => self.visit_class_declaration(node),
            SyntaxKind::ClassExpression => self.visit_class_expression(node),
            SyntaxKind::Constructor | SyntaxKind::PropertyDeclaration | SyntaxKind::ClassStaticBlockDeclaration => {
                panic!("Not supported outside of a class. Use 'classElementVisitor' instead.")
            }
            SyntaxKind::Parameter => self.visit_parameter_declaration(node),
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, false),
            SyntaxKind::PropertyAssignment | SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {
                self.visit_named_evaluation_site(node, node.initializer())
            }
            SyntaxKind::ExportAssignment => self.visit_export_assignment(node),
            SyntaxKind::ThisKeyword => self.visit_this_expression(node),
            SyntaxKind::ForStatement => self.visit_for_statement(node),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::ParenthesizedExpression => self.visit_parenthesized_expression(node, false),
            SyntaxKind::PartiallyEmittedExpression => self.visit_partially_emitted_expression(node, false),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            SyntaxKind::PrefixUnaryExpression | SyntaxKind::PostfixUnaryExpression => {
                self.visit_pre_or_postfix_unary_expression(node, false)
            }
            SyntaxKind::PropertyAccessExpression => self.visit_property_access_expression(node),
            SyntaxKind::ElementAccessExpression => self.visit_element_access_expression(node),
            SyntaxKind::ComputedPropertyName => self.visit_computed_property_name(node),
            SyntaxKind::MethodDeclaration
            | SyntaxKind::SetAccessor
            | SyntaxKind::GetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration => {
                self.enter_other();
                let result = self.transformer.visitor().visit_each_child(node);
                self.exit_other();
                result
            }
            _ => self.transformer.visitor().visit_each_child(node),
        }
    }

    pub fn modifier_visitor_visit(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::Decorator {
            return None;
        }
        Some(node.clone())
    }

    pub fn non_constructor_class_element_visit(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if is_constructor_declaration(node) {
            return node.clone();
        }
        self.class_element_visitor_visit(node).unwrap_or_else(|| node.clone())
    }
}
