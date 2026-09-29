#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::node_data_generated::{
    is_binding_pattern, is_identifier, is_omitted_expression, TokenFlags,
};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::{
    get_class_extends_heritage_element, get_combined_modifier_flags, has_syntactic_modifier,
    is_any_import_or_re_export, is_assignment_expression,
    is_ambient_module, is_big_int_literal, is_call_expression, is_class_declaration,
    is_computed_property_name, is_constructor_declaration, is_element_access_expression,
    is_entity_name, is_entity_name_expression,
    is_export_assignment, is_export_declaration, is_expression_with_type_arguments,
    is_function_declaration, is_function_like, is_function_like_declaration, is_get_accessor_declaration,
    is_heritage_clause, is_import_equals_declaration, is_index_signature_declaration,
    is_interface_declaration, is_js_type_alias_declaration, is_literal_expression,
    is_mapped_type_node, is_method_declaration, is_method_signature_declaration, is_module_declaration,
    is_object_literal_expression, is_parameter_declaration, is_property_assignment,
    is_property_access_expression, is_property_declaration, is_property_name_literal,
    is_property_signature_declaration, is_set_accessor_declaration, is_shorthand_property_assignment,
    is_source_file, is_spread_element, is_string_literal, is_string_or_numeric_literal_like,
    is_type_alias_declaration, is_type_literal_node, is_type_parameter_declaration,
    is_variable_declaration, node_is_missing, node_is_synthesized,
};
use tsox_frontend::ast::node_data_generated::{
    is_binary_expression, is_binding_element, is_call_signature_declaration,
    is_construct_signature_declaration,
};
use tsox_core::core::text::TextRange;
use tsox_core::tspath::{get_directory_path, normalize_slashes};
use tsox_frontend::ast::mig::m3e_4::{
    get_assignment_declaration_kind, get_element_or_property_access_name,
    get_heritage_clause_element_name, get_this_container,
};
use tsox_frontend::ast::mig::m3f::{
    get_rest_indicator_of_binding_or_assignment_element, get_target_of_binding_or_assignment_element,
};
use tsox_frontend::ast::mig::m3f_2::has_dynamic_name;
use tsox_frontend::ast::mig::m3f_3::is_assignment_pattern;
use tsox_frontend::ast::mig::m3f_4::{is_declaration_binding_element, is_destructuring_assignment};
use tsox_frontend::ast::mig::m3g::{is_label_name, is_modifier};
use tsox_frontend::ast::mig::m3g_2::is_primitive_literal_value;
use tsox_frontend::ast::mig::m3g_3::{
    is_var_await_using, is_var_using, is_variable_declaration_initialized_to_require,
    try_get_property_name_of_binding_or_assignment_element,
};
use tsox_frontend::ast::mig::w5::get_text_of_property_name;
use tsox_frontend::ast::mig::x4ast::{
    get_elements_of_binding_or_assignment_pattern,
    get_external_module_import_equals_declaration_expression,
};
use tsox_frontend::scanner::mig::m3i::is_identifier_text;

use crate::mig::m4e::{DeclarationEmitHost, DeclarationTransformer, EmitContext, EmitResolver};
use crate::mig::m4e::R38K1NodeExt;
use crate::mig::m4k::r37k2_defs::{R37K2NodeExt};
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4n::r37k19_defs::R37K19ArcNodeExt;
use crate::mig::m4l_7::r38k3_defs::{K3EmitResolverExt, K3NodeAccessExt};
use tsox_frontend::ast::deep_clone_node;
use crate::mig::m4k_2::Transformer;
use crate::mig::m4m_2::{is_simple_copiable_expression, is_simple_inlineable_expression};
use tsox_checker::checker::utilities_has_only_expression_initialization::{
    is_empty_array_literal, is_empty_object_literal,
};

fn update_variable_declaration_node_unique(
    node: &Arc<Node>,
    name: &Arc<Node>,
    initializer: Option<Arc<Node>>,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::VariableDeclaration,
        tsox_frontend::ast::node_data_generated::NodeData::VariableDeclaration(
            tsox_frontend::ast::node_data_generated::VariableDeclarationData {
                name: name.clone(),
                exclamation_token: None,
                type_node: None,
                initializer,
            },
        ),
    );
    updated.loc = node.loc;
    Arc::new(updated)
}

pub fn needs_scope_marker(result: &Arc<Node>) -> bool {
    !is_any_import_or_re_export(result)
        && !is_export_assignment(result)
        && !has_syntactic_modifier(result, ModifierFlags::Export)
        && !is_ambient_module(result)
}

pub fn can_have_literal_initializer(host: &dyn DeclarationEmitHost, node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => host
            .get_effective_declaration_flags(node, ModifierFlags::Private)
            == ModifierFlags::empty(),
        SyntaxKind::Parameter | SyntaxKind::VariableDeclaration => true,
        _ => false,
    }
}

pub fn can_produce_diagnostics(node: &Arc<Node>) -> bool {
    is_variable_declaration(node)
        || is_property_declaration(node)
        || is_property_signature_declaration(node)
        || is_binding_element(node)
        || is_set_accessor_declaration(node)
        || is_get_accessor_declaration(node)
        || is_construct_signature_declaration(node)
        || is_call_signature_declaration(node)
        || is_method_declaration(node)
        || is_method_signature_declaration(node)
        || is_function_declaration(node)
        || is_parameter_declaration(node)
        || is_type_parameter_declaration(node)
        || is_expression_with_type_arguments(node)
        || is_import_equals_declaration(node)
        || is_type_alias_declaration(node)
        || is_js_type_alias_declaration(node)
        || is_constructor_declaration(node)
        || is_index_signature_declaration(node)
        || is_property_access_expression(node)
        || is_element_access_expression(node)
        || is_binary_expression(node)
        || is_call_expression(node)
}

pub fn can_reuse_modifier_nodes(nodes: &[Arc<Node>]) -> bool {
    for node in nodes {
        if is_modifier(node) && node.flags & NodeFlags::Reparsed != NodeFlags::empty() {
            return false;
        }
    }
    true
}

pub fn is_declaration_and_not_visible(
    emit_context: &EmitContext,
    resolver: &EmitResolver,
    node: &Arc<Node>,
) -> bool {
    let Some(node) = emit_context.parse_node(node) else {
        return false;
    };
    match node.kind {
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::EnumDeclaration => !resolver.is_declaration_visible(&node),
        SyntaxKind::VariableDeclaration => !get_binding_name_visible(resolver, &node),
        SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportDeclaration
        | SyntaxKind::JSImportDeclaration
        | SyntaxKind::ExportDeclaration
        | SyntaxKind::ExportAssignment => false,
        SyntaxKind::ClassStaticBlockDeclaration => true,
        _ => false,
    }
}

pub fn get_binding_name_visible(resolver: &EmitResolver, elem: &Arc<Node>) -> bool {
    if is_omitted_expression(elem) {
        return false;
    }
    if elem.name().is_none() {
        return false;
    }
    let name = elem.name().unwrap();
    if is_binding_pattern(&name) {
        let Some(elements) = name.elements() else {
            return false;
        };
        for child_elem in &elements.nodes {
            if get_binding_name_visible(resolver, child_elem) {
                return true;
            }
        }
        false
    } else {
        resolver.is_declaration_visible(elem)
    }
}

pub fn is_enclosing_declaration(node: &Arc<Node>) -> bool {
    is_source_file(node)
        || is_type_alias_declaration(node)
        || is_js_type_alias_declaration(node)
        || is_module_declaration(node)
        || is_class_declaration(node)
        || is_interface_declaration(node)
        || is_function_like(node)
        || is_index_signature_declaration(node)
        || is_mapped_type_node(node)
        || is_variable_declaration(node)
}

pub fn is_always_type(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::InterfaceDeclaration
}

pub fn mask_modifier_flags(
    node: &Arc<Node>,
    modifier_mask: ModifierFlags,
    modifier_additions: ModifierFlags,
) -> ModifierFlags {
    let mut flags = (get_combined_modifier_flags(node) & modifier_mask) | modifier_additions;
    if flags.contains(ModifierFlags::Default) && !flags.contains(ModifierFlags::Export) {
        flags.remove(ModifierFlags::Export);
    }
    if flags.contains(ModifierFlags::Default) && flags.contains(ModifierFlags::Ambient) {
        flags.remove(ModifierFlags::Ambient);
    }
    flags
}

pub fn unwrap_parenthesized_expression(o: &Arc<Node>) -> &Arc<Node> {
    let mut o = o;
    while o.kind == SyntaxKind::ParenthesizedExpression {
        o = o.expression().expect("ParenthesizedExpression expression");
    }
    o
}

pub fn is_private_method_type_parameter(
    host: &dyn DeclarationEmitHost,
    node: &Arc<Node>,
) -> bool {
    let parent = node.parent().unwrap();
    parent.kind == SyntaxKind::MethodDeclaration
        && host.get_effective_declaration_flags(&parent, ModifierFlags::Private)
            != ModifierFlags::empty()
}

pub fn should_emit_function_properties(input: &Arc<Node>) -> bool {
    if input.as_function_declaration().body.is_some() {
        return true;
    }
    let declarations = input.symbol_declarations_k3();
    !declarations
        .iter()
        .all(|decl| !is_function_declaration(decl) || decl.as_function_declaration().body.is_none())
}

pub fn get_effective_base_type_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    get_class_extends_heritage_element(node)
}

pub fn is_scope_marker(node: &Arc<Node>) -> bool {
    is_export_assignment(node) || is_export_declaration(node)
}

pub fn has_scope_marker(statements: &Option<Arc<Node>>) -> bool {
    match statements {
        None => false,
        Some(statements) => statements
            .as_statement_list()
            .children
            .iter()
            .any(is_scope_marker),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlattenLevel {
    All,
    ObjectRest,
}

pub type CreateAssignmentCallback =
    fn(name: Arc<Node>, value: Arc<Node>, location: TextRange) -> Arc<Node>;

pub fn flatten_destructuring_assignment(
    tx: &mut Transformer,
    node: Arc<Node>,
    needs_value: bool,
    level: FlattenLevel,
    create_assignment_callback: Option<CreateAssignmentCallback>,
) -> Option<Arc<Node>> {
    let mut f = new_flattener(tx, level);
    f.create_assignment_callback = create_assignment_callback;
    f.hoist_temp_variables = true;
    f.emit_binding_or_assignment = k3_emit_assignment_shim;
    f.create_array_binding_or_assignment_pattern = k3_create_array_assignment_pattern_shim;
    f.create_object_binding_or_assignment_pattern = k3_create_object_assignment_pattern_shim;
    f.create_array_binding_or_assignment_element = k3_create_array_assignment_element_shim;
    f.flatten_destructuring_assignment(node, needs_value)
}

pub fn flatten_destructuring_binding(
    tx: &mut Transformer,
    node: Arc<Node>,
    rval: Option<Arc<Node>>,
    level: FlattenLevel,
    hoist_temp_variables: bool,
    skip_initializer: bool,
) -> Option<Arc<Node>> {
    let mut f = new_flattener(tx, level);
    f.hoist_temp_variables = hoist_temp_variables;
    f.emit_binding_or_assignment = k3_emit_binding_shim;
    f.create_array_binding_or_assignment_pattern = k3_create_array_binding_pattern_shim;
    f.create_object_binding_or_assignment_pattern = k3_create_object_binding_pattern_shim;
    f.create_array_binding_or_assignment_element = k3_create_array_binding_element_shim;
    f.flatten_destructuring_binding(node, rval, skip_initializer)
}

pub struct PendingDecl {
    pub pending_expressions: Vec<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub value: Option<Arc<Node>>,
    pub location: TextRange,
    pub original: Option<Arc<Node>>,
}

pub struct Flattener<'a> {
    pub tx: &'a mut Transformer,
    pub level: FlattenLevel,
    pub create_assignment_callback: Option<CreateAssignmentCallback>,
    pub expressions: Vec<Arc<Node>>,
    pub declarations: Vec<PendingDecl>,
    pub has_transformed_prior_element: bool,
    pub hoist_temp_variables: bool,
    pub emit_binding_or_assignment: fn(&mut Flattener, Option<Arc<Node>>, Option<Arc<Node>>, TextRange, Option<Arc<Node>>),
    pub create_array_binding_or_assignment_pattern: fn(&mut Flattener, Vec<Arc<Node>>) -> Arc<Node>,
    pub create_object_binding_or_assignment_pattern: fn(&mut Flattener, Vec<Arc<Node>>) -> Arc<Node>,
    pub create_array_binding_or_assignment_element: fn(&mut Flattener, Arc<Node>) -> Arc<Node>,
}

pub fn new_flattener<'a>(tx: &'a mut Transformer, level: FlattenLevel) -> Flattener<'a> {
    Flattener {
        tx,
        level,
        create_assignment_callback: None,
        expressions: Vec::new(),
        declarations: Vec::new(),
        has_transformed_prior_element: false,
        hoist_temp_variables: false,
        emit_binding_or_assignment: k3_emit_assignment_shim,
        create_array_binding_or_assignment_pattern: k3_create_array_assignment_pattern_shim,
        create_object_binding_or_assignment_pattern: k3_create_object_assignment_pattern_shim,
        create_array_binding_or_assignment_element: k3_create_array_assignment_element_shim,
    }
}

fn k3_emit_assignment_shim(
    f: &mut Flattener<'_>,
    target: Option<Arc<Node>>,
    value: Option<Arc<Node>>,
    location: TextRange,
    original: Option<Arc<Node>>,
) {
    f.emit_assignment(target, value, location, original)
}

fn k3_emit_binding_shim(
    f: &mut Flattener<'_>,
    target: Option<Arc<Node>>,
    value: Option<Arc<Node>>,
    location: TextRange,
    original: Option<Arc<Node>>,
) {
    f.emit_binding(target, value, location, original)
}

fn k3_create_array_assignment_pattern_shim(
    f: &mut Flattener<'_>,
    elements: Vec<Arc<Node>>,
) -> Arc<Node> {
    f.create_array_assignment_pattern(elements)
}

fn k3_create_object_assignment_pattern_shim(
    f: &mut Flattener<'_>,
    elements: Vec<Arc<Node>>,
) -> Arc<Node> {
    f.create_object_assignment_pattern(elements)
}

fn k3_create_array_assignment_element_shim(f: &mut Flattener<'_>, expr: Arc<Node>) -> Arc<Node> {
    f.create_array_assignment_element(expr)
}

fn k3_create_array_binding_pattern_shim(f: &mut Flattener<'_>, elements: Vec<Arc<Node>>) -> Arc<Node> {
    f.create_array_binding_pattern(elements)
}

fn k3_create_object_binding_pattern_shim(
    f: &mut Flattener<'_>,
    elements: Vec<Arc<Node>>,
) -> Arc<Node> {
    f.create_object_binding_pattern(elements)
}

fn k3_create_array_binding_element_shim(f: &mut Flattener<'_>, expr: Arc<Node>) -> Arc<Node> {
    f.create_array_binding_element(expr)
}

impl<'a> Flattener<'a> {
    pub fn create_array_assignment_pattern(&mut self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        self.tx
            .factory()
            .new_array_literal_expression(&self.tx.factory().new_node_list(elements), false)
    }

    pub fn create_object_assignment_pattern(&mut self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        self.tx
            .factory()
            .new_object_literal_expression(&self.tx.factory().new_node_list(elements), false)
    }

    pub fn create_array_assignment_element(&mut self, expr: Arc<Node>) -> Arc<Node> {
        expr
    }

    pub fn emit_assignment(
        &mut self,
        target: Option<Arc<Node>>,
        value: Option<Arc<Node>>,
        location: TextRange,
        original: Option<Arc<Node>>,
    ) {
        let target = target.unwrap();
        let value = value.unwrap();
        let expression: Arc<Node> = match self.create_assignment_callback {
            Some(cb) if is_identifier(&target) => cb(target, value, location),
            _ => {
                let visited = self.tx.visitor().visit_node(&target);
                let mut expr = self.tx.factory().new_assignment_expression(&visited, &value);
                expr.set_loc(location);
                expr
            }
        };
        if let Some(original) = original.as_ref() {
            self.tx.emit_context().set_original(&expression, original);
        }
        self.emit_expression(expression);
    }

    pub fn create_array_binding_pattern(&mut self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        self.tx.factory().new_binding_pattern(
            SyntaxKind::ArrayBindingPattern,
            &self.tx.factory().new_node_list(elements),
        )
    }

    pub fn create_object_binding_pattern(&mut self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        self.tx.factory().new_binding_pattern(
            SyntaxKind::ObjectBindingPattern,
            &self.tx.factory().new_node_list(elements),
        )
    }

    pub fn create_array_binding_element(&mut self, expr: Arc<Node>) -> Arc<Node> {
        self.tx
            .factory()
            .new_binding_element(None, None, Some(expr), None)
    }

    pub fn emit_binding(
        &mut self,
        target: Option<Arc<Node>>,
        value: Option<Arc<Node>>,
        location: TextRange,
        original: Option<Arc<Node>>,
    ) {
        let mut value = value;
        if !self.expressions.is_empty() {
            let mut all = std::mem::take(&mut self.expressions);
            all.push(value.unwrap());
            value = self.tx.factory().inline_expressions(all);
        }
        self.declarations.push(PendingDecl {
            pending_expressions: Vec::new(),
            name: target,
            value,
            location,
            original,
        });
    }

    pub fn emit_expression(&mut self, expr: Arc<Node>) {
        self.expressions.push(expr);
    }

    pub fn ensure_identifier(
        &mut self,
        value: Option<Arc<Node>>,
        reuse_identifier_expressions: bool,
        location: TextRange,
    ) -> Option<Arc<Node>> {
        let value = value?;
        if reuse_identifier_expressions && is_identifier(&value) {
            return Some(value);
        }
        let temp = self
            .tx
            .factory()
            .generated_name_node(&self.tx.factory().new_temp_variable());
        if self.hoist_temp_variables {
            self.tx.emit_context().add_variable_declaration(&temp);
            let mut assign = self
                .tx
                .factory()
                .new_assignment_expression(&temp, &value);
            assign.set_loc(location);
            self.emit_expression(assign);
        } else {
            (self.emit_binding_or_assignment)(self, Some(temp.clone()), Some(value), location, None);
        }
        Some(temp)
    }

    pub fn create_default_value_check(
        &mut self,
        value: Arc<Node>,
        default_value: Arc<Node>,
        location: TextRange,
    ) -> Arc<Node> {
        let value = self
            .ensure_identifier(Some(value), true, location)
            .unwrap();
        self.tx.factory().new_conditional_expression(
            &self.tx.factory().new_type_check(&value, "undefined"),
            &self.tx.factory().new_token(SyntaxKind::QuestionToken),
            &default_value,
            &self.tx.factory().new_token(SyntaxKind::ColonToken),
            &value,
        )
    }

    pub fn create_destructuring_property_access(
        &mut self,
        value: Arc<Node>,
        property_name: &Arc<Node>,
    ) -> Arc<Node> {
        if is_computed_property_name(property_name) {
            let visited = self
                .tx
                .visitor()
                .visit_node(property_name.expression().expect("computed property expression"));
            let argument_expression = self
                .ensure_identifier(Some(visited), false, property_name.loc())
                .unwrap();
            self.tx
                .factory()
                .new_element_access_expression(&value, None, &argument_expression, NodeFlags::empty())
        } else if is_string_or_numeric_literal_like(property_name) || is_big_int_literal(property_name) {
            let argument_expression = deep_clone_node(property_name);
            self.tx
                .factory()
                .new_element_access_expression(&value, None, &argument_expression, NodeFlags::empty())
        } else {
            let name = self.tx.factory().new_identifier(property_name.text());
            self.tx
                .factory()
                .new_property_access_expression(&value, None, &name, NodeFlags::empty())
        }
    }

    pub fn flatten_destructuring_assignment(
        &mut self,
        node: Arc<Node>,
        needs_value: bool,
    ) -> Option<Arc<Node>> {
        let mut node = node;
        let mut location = node.loc();
        let mut value: Option<Arc<Node>> = None;
        if is_destructuring_assignment(&node) {
            value = Some(node.as_binary_expression().right.clone());
            while is_empty_array_literal(&node.as_binary_expression().left)
                || is_empty_object_literal(&node.as_binary_expression().left)
            {
                let v = value.clone().unwrap();
                if is_destructuring_assignment(&v) {
                    node = v;
                    location = node.loc();
                    value = Some(node.as_binary_expression().right.clone());
                } else {
                    return Some(self.tx.visitor().visit_node(&v));
                }
            }
        }

        if let Some(v) = value.clone() {
            let v = self.tx.visitor().visit_node(&v);
            value = Some(if (is_identifier(&v)
                && binding_or_assignment_element_assigns_to_name(&node, v.text()))
                || binding_or_assignment_element_contains_non_literal_computed_name(&node)
            {
                self.ensure_identifier(Some(v), false, location).unwrap()
            } else if needs_value {
                self.ensure_identifier(Some(v), true, location).unwrap()
            } else if node_is_synthesized(&node) {
                location = v.loc();
                v
            } else {
                v
            });
        }

        self.flatten_binding_or_assignment_element(
            node.clone(),
            value.clone(),
            location,
            is_destructuring_assignment(&node),
        );

        if let Some(v) = value.clone() {
            if needs_value {
                if self.expressions.is_empty() {
                    return Some(v);
                }
                self.expressions.push(v);
            }
        }

        let res = self.tx.factory().inline_expressions(std::mem::take(&mut self.expressions));
        match res {
            Some(res) => Some(res),
            None => Some(self.tx.factory().new_omitted_expression()),
        }
    }

    pub fn flatten_destructuring_binding(
        &mut self,
        node: Arc<Node>,
        rval: Option<Arc<Node>>,
        skip_initializer: bool,
    ) -> Option<Arc<Node>> {
        let mut node = node;
        if is_variable_declaration(&node) {
            let mut initializer = get_initializer_of_binding_or_assignment_element(Some(&node));
            if let Some(init) = initializer.clone() {
                if (is_identifier(&init)
                    && binding_or_assignment_element_assigns_to_name(&node, init.text()))
                    || binding_or_assignment_element_contains_non_literal_computed_name(&node)
                {
                    let visited = self.tx.visitor().visit_node(&init);
                    let ensured = self
                        .ensure_identifier(Some(visited), false, init.loc())
                        .unwrap();
                    initializer = Some(ensured);
                    let visited_name =
                        self.tx.visitor().visit_node(node.name().expect("VariableDeclaration name"));
                    node = update_variable_declaration_node_unique(&node, &visited_name, initializer);
                }
            }
        }

        let location = node.loc();
        self.flatten_binding_or_assignment_element(node, rval, location, skip_initializer);

        if !self.expressions.is_empty() {
            let temp = self
                .tx
                .factory()
                .generated_name_node(&self.tx.factory().new_temp_variable());
            if self.hoist_temp_variables {
                let value = self
                    .tx
                    .factory()
                    .inline_expressions(std::mem::take(&mut self.expressions));
                (self.emit_binding_or_assignment)(self, Some(temp), value, TextRange::default(), None);
            } else {
                self.tx.emit_context().add_variable_declaration(&temp);
                let last = self.declarations.last_mut().unwrap();
                let last_value = last.value.clone().unwrap();
                last.pending_expressions
                    .push(self.tx.factory().new_assignment_expression(&temp, &last_value));
                last.pending_expressions
                    .extend(std::mem::take(&mut self.expressions));
                last.value = Some(temp);
            }
        }

        let mut decls: Vec<Arc<Node>> = Vec::with_capacity(self.declarations.len());
        for pending in std::mem::take(&mut self.declarations) {
            let mut expr = pending.value.clone();
            if !pending.pending_expressions.is_empty() {
                let mut all = pending.pending_expressions;
                if let Some(v) = pending.value {
                    all.push(v);
                }
                expr = self.tx.factory().inline_expressions(all);
            }
            let mut decl = self
                .tx
                .factory()
                .new_variable_declaration(&pending.name.unwrap(), None, None, expr.as_ref());
            decl.set_loc(pending.location);
            if let Some(original) = &pending.original {
                self.tx.emit_context().set_original(&decl, original);
            }
            decls.push(decl);
        }

        if decls.len() == 1 {
            return Some(decls.into_iter().next().unwrap());
        }
        if decls.is_empty() {
            return None;
        }
        Some(self.tx.factory().new_syntax_list(decls))
    }

    pub fn flatten_binding_or_assignment_element(
        &mut self,
        element: Arc<Node>,
        value: Option<Arc<Node>>,
        location: TextRange,
        skip_initializer: bool,
    ) {
        let binding_target = match get_target_of_binding_or_assignment_element(&element) {
            None => return,
            Some(t) => t,
        };
        let mut value = value;
        if !skip_initializer {
            let initializer = get_initializer_of_binding_or_assignment_element(Some(&element))
                .map(|i| self.tx.visitor().visit_node(&i));
            if let Some(initializer) = initializer {
                if let Some(v) = value {
                    let mut new_value = self.create_default_value_check(v, initializer.clone(), location);
                    if !is_simple_copiable_expression(&initializer)
                        && (is_binding_pattern(&binding_target)
                            || is_assignment_pattern(&binding_target))
                    {
                        new_value = self
                            .ensure_identifier(Some(new_value), true, location)
                            .unwrap();
                    }
                    value = Some(new_value);
                } else {
                    value = Some(initializer);
                }
            } else if value.is_none() {
                value = Some(self.tx.factory().new_void_zero_expression());
            }
        }

        if is_object_binding_or_assignment_pattern(&binding_target) {
            self.flatten_object_binding_or_assignment_pattern(element, binding_target, value, location);
        } else if is_array_binding_or_assignment_pattern(&binding_target) {
            self.flatten_array_binding_or_assignment_pattern(element, binding_target, value, location);
        } else {
            (self.emit_binding_or_assignment)(
                self,
                Some(binding_target),
                value,
                location,
                Some(element),
            );
        }
    }

    pub fn flatten_object_binding_or_assignment_pattern(
        &mut self,
        parent: Arc<Node>,
        pattern: Arc<Node>,
        value: Option<Arc<Node>>,
        location: TextRange,
    ) {
        let elements = get_elements_of_binding_or_assignment_pattern(&pattern);
        let num_elements = elements.len();
        let mut value = value;
        if num_elements != 1 {
            let reuse_identifier_expressions =
                !is_declaration_binding_element(&parent) || num_elements != 0;
            value = self.ensure_identifier(value, reuse_identifier_expressions, location);
        }
        let mut binding_elements: Vec<Arc<Node>> = Vec::new();
        let mut computed_temp_variables: Vec<Arc<Node>> = Vec::new();
        for (i, element) in elements.iter().enumerate() {
            if get_rest_indicator_of_binding_or_assignment_element(element).is_none() {
                let property_name =
                    try_get_property_name_of_binding_or_assignment_element(element);
                if self.level >= FlattenLevel::ObjectRest
                    && element.subtree_facts()
                        & (SubtreeFacts::CONTAINS_REST_OR_SPREAD
                            | SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
                        == SubtreeFacts::empty()
                    && get_target_of_binding_or_assignment_element(element)
                        .map(|t| {
                            t.subtree_facts()
                                & (SubtreeFacts::CONTAINS_REST_OR_SPREAD
                                    | SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD)
                                == SubtreeFacts::empty()
                        })
                        .unwrap_or(false)
                    && !property_name
                        .as_ref()
                        .map(|n| is_computed_property_name(n))
                        .unwrap_or(false)
                {
                    binding_elements.push(self.tx.visitor().visit_node(element));
                } else {
                    if !binding_elements.is_empty() {
                        let pattern_elements = std::mem::take(&mut binding_elements);
                        let created = (self.create_object_binding_or_assignment_pattern)(
                            self,
                            pattern_elements,
                        );
                        (self.emit_binding_or_assignment)(
                            self,
                            Some(created),
                            value.clone(),
                            location,
                            Some(pattern.clone()),
                        );
                    }
                    let rhs_value = self.create_destructuring_property_access(
                        value.clone().unwrap(),
                        property_name.as_ref().unwrap(),
                    );
                    if property_name
                        .as_ref()
                        .map(|n| is_computed_property_name(n))
                        .unwrap_or(false)
                    {
                        computed_temp_variables.push(
                            rhs_value.as_element_access_expression().argument_expression.clone(),
                        );
                    }
                    let elem_loc = element.loc();
                    self.flatten_binding_or_assignment_element(
                        element.clone(),
                        Some(rhs_value),
                        elem_loc,
                        false,
                    );
                }
            } else if i == num_elements - 1 {
                if !binding_elements.is_empty() {
                    let pattern_elements = std::mem::take(&mut binding_elements);
                    let created =
                        (self.create_object_binding_or_assignment_pattern)(self, pattern_elements);
                    (self.emit_binding_or_assignment)(
                        self,
                        Some(created),
                        value.clone(),
                        location,
                        Some(pattern.clone()),
                    );
                }
                let rhs_value = self.tx.factory().new_rest_helper(
                    &value.clone().unwrap(),
                    &elements,
                    &computed_temp_variables,
                    pattern.loc(),
                );
                let elem_loc = element.loc();
                self.flatten_binding_or_assignment_element(element.clone(), Some(rhs_value), elem_loc, false);
            }
        }
        if !binding_elements.is_empty() {
            let created = (self.create_object_binding_or_assignment_pattern)(
                self,
                std::mem::take(&mut binding_elements),
            );
            (self.emit_binding_or_assignment)(
                self,
                Some(created),
                value,
                location,
                Some(pattern),
            );
        }
    }

    pub fn flatten_array_binding_or_assignment_pattern(
        &mut self,
        parent: Arc<Node>,
        pattern: Arc<Node>,
        value: Option<Arc<Node>>,
        location: TextRange,
    ) {
        let elements = get_elements_of_binding_or_assignment_pattern(&pattern);
        let num_elements = elements.len();
        let mut value = value;
        if (num_elements != 1 && (self.level < FlattenLevel::ObjectRest || num_elements == 0))
            || elements.iter().all(|e| is_omitted_expression(e))
        {
            let reuse_identifier_expressions =
                !is_declaration_binding_element(&parent) || num_elements != 0;
            value = self.ensure_identifier(value, reuse_identifier_expressions, location);
        }
        let mut binding_elements: Vec<Arc<Node>> = Vec::new();
        let mut rest_containing_elements: Vec<RestIdElemPair> = Vec::new();
        for (i, element) in elements.iter().enumerate() {
            if self.level >= FlattenLevel::ObjectRest {
                if element.subtree_facts() & SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD
                    != SubtreeFacts::empty()
                    || (self.has_transformed_prior_element
                        && !is_simple_binding_or_assignment_element(element))
                {
                    self.has_transformed_prior_element = true;
                    let temp = self
                        .tx
                        .factory()
                        .generated_name_node(&self.tx.factory().new_temp_variable());
                    if self.hoist_temp_variables {
                        self.tx.emit_context().add_variable_declaration(&temp);
                    }
                    rest_containing_elements.push(RestIdElemPair {
                        id: temp.clone(),
                        element: element.clone(),
                    });
                    let created =
                        (self.create_array_binding_or_assignment_element)(self, temp);
                    binding_elements.push(created);
                } else {
                    binding_elements.push(element.clone());
                }
            } else if is_omitted_expression(element) {
                continue;
            } else if get_rest_indicator_of_binding_or_assignment_element(element).is_none() {
                let rhs_value = self.tx.factory().new_element_access_expression(
                    &value.clone().unwrap(),
                    None,
                    &self.tx.factory().new_numeric_literal(
                        &i.to_string(),
                        0,
                    ),
                    NodeFlags::empty(),
                );
                let elem_loc = element.loc();
                self.flatten_binding_or_assignment_element(
                    element.clone(),
                    Some(rhs_value),
                    elem_loc,
                    false,
                );
            } else if i == num_elements - 1 {
                let rhs_value = self
                    .tx
                    .factory()
                    .new_array_slice_call(&value.clone().unwrap(), i as i32);
                let elem_loc = element.loc();
                self.flatten_binding_or_assignment_element(
                    element.clone(),
                    Some(rhs_value),
                    elem_loc,
                    false,
                );
            }
        }
        if !binding_elements.is_empty() {
            let created = (self.create_array_binding_or_assignment_pattern)(
                self,
                std::mem::take(&mut binding_elements),
            );
            (self.emit_binding_or_assignment)(
                self,
                Some(created),
                value,
                location,
                Some(pattern),
            );
        }
        for pair in rest_containing_elements {
            let elem_loc = pair.element.loc();
            self.flatten_binding_or_assignment_element(pair.element, Some(pair.id), elem_loc, false);
        }
    }
}

pub struct RestIdElemPair {
    pub id: Arc<Node>,
    pub element: Arc<Node>,
}

pub fn binding_or_assignment_element_assigns_to_name(element: &Arc<Node>, name: &str) -> bool {
    let target = match get_target_of_binding_or_assignment_element(element) {
        None => return false,
        Some(t) => t,
    };
    if is_binding_pattern(&target) || is_assignment_pattern(&target) {
        binding_or_assignment_pattern_assigns_to_name(&target, name)
    } else if is_identifier(&target) {
        target.text() == name
    } else {
        false
    }
}

pub fn binding_or_assignment_pattern_assigns_to_name(pattern: &Arc<Node>, name: &str) -> bool {
    let elements = get_elements_of_binding_or_assignment_pattern(pattern);
    elements
        .iter()
        .any(|element| binding_or_assignment_element_assigns_to_name(element, name))
}

pub fn binding_or_assignment_element_contains_non_literal_computed_name(element: &Arc<Node>) -> bool {
    let property_name = try_get_property_name_of_binding_or_assignment_element(element);
    if let Some(property_name) = &property_name {
        if is_computed_property_name(property_name) && !is_literal_expression(property_name.expression().expect("computed property expression").as_ref()) {
            return true;
        }
    }
    match get_target_of_binding_or_assignment_element(element) {
        Some(target) if is_binding_pattern(&target) || is_assignment_pattern(&target) => {
            binding_or_assignment_pattern_contains_non_literal_computed_name(&target)
        }
        _ => false,
    }
}

pub fn binding_or_assignment_pattern_contains_non_literal_computed_name(pattern: &Arc<Node>) -> bool {
    let elements = get_elements_of_binding_or_assignment_pattern(pattern);
    elements
        .iter()
        .any(binding_or_assignment_element_contains_non_literal_computed_name)
}

pub fn get_initializer_of_binding_or_assignment_element(
    binding_element: Option<&Arc<Node>>,
) -> Option<Arc<Node>> {
    let binding_element = binding_element?;
    if is_declaration_binding_element(binding_element) {
        return binding_element.initializer().cloned();
    }
    if is_property_assignment(binding_element) {
        let initializer = binding_element.initializer()?;
        if is_assignment_expression(&initializer, true) {
            return Some(initializer.as_binary_expression().right.clone());
        }
        return None;
    }
    if is_shorthand_property_assignment(binding_element) {
        return binding_element
            .as_shorthand_property_assignment()
            .object_assignment_initializer
            .clone();
    }
    if is_assignment_expression(binding_element, true) {
        return Some(binding_element.as_binary_expression().right.clone());
    }
    if is_spread_element(binding_element) {
        return get_initializer_of_binding_or_assignment_element(binding_element.expression());
    }
    None
}

pub fn is_object_binding_or_assignment_pattern(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::ObjectBindingPattern || node.kind == SyntaxKind::ObjectLiteralExpression
}

pub fn is_array_binding_or_assignment_pattern(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::ArrayBindingPattern || node.kind == SyntaxKind::ArrayLiteralExpression
}

pub fn is_simple_binding_or_assignment_element(element: &Arc<Node>) -> bool {
    let target = match get_target_of_binding_or_assignment_element(element) {
        None => return true,
        Some(t) => t,
    };
    if is_omitted_expression(&target) {
        return true;
    }
    let property_name = try_get_property_name_of_binding_or_assignment_element(element);
    if let Some(property_name) = &property_name {
        if !is_property_name_literal(property_name) {
            return false;
        }
    }
    let initializer = get_initializer_of_binding_or_assignment_element(Some(element));
    if let Some(initializer) = &initializer {
        if !is_simple_inlineable_expression(initializer) {
            return false;
        }
    }
    if is_binding_pattern(&target) || is_assignment_pattern(&target) {
        return get_elements_of_binding_or_assignment_pattern(&target)
            .iter()
            .all(is_simple_binding_or_assignment_element);
    }
    is_identifier(&target)
}
