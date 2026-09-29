#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

pub fn is_type_reference(node: &Arc<Node>) -> bool {
    let mut node = Arc::clone(node);
    if ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access(&node) {
        let Some(parent) = node.parent() else { return false };
        node = parent;
    }
    match node.kind {
        SyntaxKind::ThisKeyword => !ast::is_expression(&node),
        SyntaxKind::Identifier => {
            let text = ast::node_text(&node);
            if text != "undefined" {
                false
            } else {
                node.parent()
                    .map_or(false, |parent| !ast::is_parameter_declaration(&parent))
            }
        }
        _ => crate::ls::mig::m5x_4::is_namespace_reference(&node),
    }
}

pub fn get_meaning_from_location(node: &Arc<Node>) -> ast::mig::m3e_4::SemanticMeaning {
    let node = get_adjusted_location(node, false, None);
    let Some(parent) = node.parent() else {
        return ast::mig::m3e_4::SemanticMeaning::VALUE;
    };
    if ast::is_source_file(&node) {
        return ast::mig::m3e_4::SemanticMeaning::VALUE;
    }
    if matches!(
        parent.kind,
        SyntaxKind::ExportAssignment
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ExternalModuleReference
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
    ) || (parent.kind == SyntaxKind::ImportEqualsDeclaration
        && crate::ls::mig::m5x_3::import_equals_declaration_name(&parent)
            .map_or(false, |name| Arc::ptr_eq(&name, &node)))
    {
        return ast::mig::m3e_4::SemanticMeaning::ALL;
    }
    if crate::ls::mig::m5x_4::is_in_right_side_of_internal_import_equals_declaration(&node) {
        let mut name: Option<Arc<Node>> = None;
        if node.kind != SyntaxKind::QualifiedName {
            if parent.kind == SyntaxKind::QualifiedName {
                if let Some(right) = crate::ls::mig::m5u_2::qualified_name_right(&parent) {
                    if Arc::ptr_eq(&right, &node) {
                        name = Some(parent.clone());
                    }
                }
            }
        } else {
            name = Some(node.clone());
        }
        if let Some(name) = name {
            if name.parent().map_or(false, |p| p.kind == SyntaxKind::ImportEqualsDeclaration) {
                return ast::mig::m3e_4::SemanticMeaning::ALL;
            }
        }
        return ast::mig::m3e_4::SemanticMeaning::NAMESPACE;
    }
    if ast::mig::m3f_4::is_declaration_name(&node) {
        return crate::ls::mig::m5x_5::get_meaning_from_declaration(&parent);
    }
    if ast::is_entity_name(&node) && ast::mig::w7a::is_jsdoc_name_reference_context(&node) {
        return ast::mig::m3e_4::SemanticMeaning::ALL;
    }
    if is_type_reference(&node) {
        return ast::mig::m3e_4::SemanticMeaning::TYPE;
    }
    if crate::ls::mig::m5x_4::is_namespace_reference(&node) {
        return ast::mig::m3e_4::SemanticMeaning::NAMESPACE;
    }
    if ast::is_type_parameter_declaration(&parent) {
        return ast::mig::m3e_4::SemanticMeaning::TYPE;
    }
    if ast::is_literal_type_node(&parent) {
        return ast::mig::m3e_4::SemanticMeaning::TYPE.union(ast::mig::m3e_4::SemanticMeaning::VALUE);
    }
    ast::mig::m3e_4::SemanticMeaning::VALUE
}

pub fn is_export_specifier_alias(reference_location: &Arc<Node>, export_specifier: &Arc<Node>) -> bool {
    let property_name = crate::ls::mig::m5x_3::export_specifier_property_name(export_specifier);
    if let Some(property_name) = property_name {
        Arc::ptr_eq(&property_name, reference_location)
    } else {
        export_specifier
            .parent()
            .and_then(|p| p.parent())
            .map_or(false, |grand| {
                crate::ls::mig::m5x_3::import_or_export_declaration_module_specifier(&grand).is_none()
            })
    }
}

pub fn get_adjusted_location(
    node: &Arc<Node>,
    for_rename: bool,
    source_file: Option<Arc<SourceFile>>,
) -> Arc<Node> {
    let parent = node.parent();
    let is_modifier = |node: &Arc<Node>, parent: &Option<Arc<Node>>| -> bool {
        if let Some(parent) = parent {
            if ast::mig::m3g::is_modifier(node) && (for_rename || node.kind != SyntaxKind::DefaultKeyword) {
                return ast::can_have_modifiers(parent)
                    && parent.modifier_nodes().iter().any(|m| Arc::ptr_eq(m, node));
            }
            match node.kind {
                SyntaxKind::ClassKeyword => {
                    ast::is_class_declaration(parent) || ast::is_class_expression(node)
                }
                SyntaxKind::FunctionKeyword => {
                    ast::is_function_declaration(parent) || ast::is_function_expression(node)
                }
                SyntaxKind::InterfaceKeyword => ast::is_interface_declaration(parent),
                SyntaxKind::EnumKeyword => ast::is_enum_declaration(parent),
                SyntaxKind::TypeKeyword => ast::is_type_alias_declaration(parent),
                SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => {
                    ast::is_module_declaration(parent)
                }
                SyntaxKind::ImportKeyword => ast::is_import_equals_declaration(parent),
                SyntaxKind::GetKeyword => ast::is_get_accessor_declaration(parent),
                SyntaxKind::SetKeyword => ast::is_set_accessor_declaration(parent),
                _ => false,
            }
        } else {
            false
        }
    };
    if is_modifier(node, &parent) {
        if let Some(parent) = parent.as_ref() {
            let has_source_file =
                source_file.is_some() || ast::get_source_file_of_node(node).is_some();
            if let Some(location) =
                get_adjusted_location_for_declaration(parent, for_rename, has_source_file)
            {
                return location;
            }
        }
    }
    if let Some(parent) = parent.as_ref() {
        if (node.kind == SyntaxKind::VarKeyword
            || node.kind == SyntaxKind::ConstKeyword
            || node.kind == SyntaxKind::LetKeyword)
            && ast::is_variable_declaration_list(parent)
            && crate::ls::mig::m5x_3::variable_declaration_list_declarations(parent).map_or(false, |d| d.len() == 1)
        {
            if let Some(declaration) =
                crate::ls::mig::m5x_3::variable_declaration_list_declarations(parent).and_then(|d| d.into_iter().next())
            {
                if let Some(name) = crate::ls::mig::m5x_3::variable_declaration_name(&declaration) {
                    if ast::is_identifier(&name) {
                        return name;
                    }
                }
            }
        }
        if node.kind == SyntaxKind::TypeKeyword {
            if ast::is_import_clause(parent) && ast::mig::m3b::is_type_only(parent) {
                if let Some(grand) = parent.parent() {
                    if let Some(location) = get_adjusted_location_for_import_declaration(
                        &grand,
                        for_rename,
                    ) {
                        return location;
                    }
                }
            }
            if ast::is_export_declaration(parent) && ast::mig::m3b::is_type_only(parent) {
                if let Some(location) =
                    get_adjusted_location_for_export_declaration(parent, for_rename)
                {
                    return location;
                }
            }
        }
        if node.kind == SyntaxKind::AsKeyword {
            if (parent.kind == SyntaxKind::ImportSpecifier
                && crate::ls::mig::m5x_3::import_specifier_property_name(parent).is_some())
                || (parent.kind == SyntaxKind::ExportSpecifier
                    && crate::ls::mig::m5x_3::export_specifier_property_name(parent).is_some())
                || parent.kind == SyntaxKind::NamespaceImport
                || parent.kind == SyntaxKind::NamespaceExport
            {
                if let Some(name) = crate::ls::mig::m5x_3::import_or_export_specifier_name(parent) {
                    return name;
                }
            }
        }
    }
    Arc::clone(node)
}

pub fn get_adjusted_location_for_declaration(
    node: &Arc<Node>,
    for_rename: bool,
    has_source_file: bool,
) -> Option<Arc<Node>> {
    if let Some(name) = ast::get_name_of_declaration(node) {
        return Some(name);
    }
    if for_rename {
        return None;
    }
    match node.kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration => node
            .modifier_nodes()
            .into_iter()
            .find(|m| m.kind == SyntaxKind::DefaultKeyword)
            .cloned(),
        SyntaxKind::ClassExpression if has_source_file => {
            tsox_frontend::astnav::find_child_of_kind(node, SyntaxKind::ClassKeyword)
        }
        SyntaxKind::FunctionExpression if has_source_file => {
            tsox_frontend::astnav::find_child_of_kind(node, SyntaxKind::FunctionKeyword)
        }
        SyntaxKind::Constructor => Some(Arc::clone(node)),
        _ => None,
    }
}

pub fn get_adjusted_location_for_import_declaration(
    node: &Arc<Node>,
    for_rename: bool,
) -> Option<Arc<Node>> {
    let import_clause = crate::ls::mig::m5x_3::import_declaration_import_clause(node)?;
    if let Some(name) = crate::ls::mig::m5x_3::import_clause_name(&import_clause) {
        if crate::ls::mig::m5x_3::import_clause_named_bindings(&import_clause).is_some() {
            return None;
        }
        return Some(name);
    }
    if let Some(named_bindings) = crate::ls::mig::m5x_3::import_clause_named_bindings(&import_clause) {
        match named_bindings.kind {
            SyntaxKind::NamedImports => {
                let elements = crate::ls::mig::m5x_3::named_imports_elements(&named_bindings)?;
                if elements.len() != 1 {
                    return None;
                }
                return crate::ls::mig::m5x_3::import_specifier_name(&elements[0]);
            }
            SyntaxKind::NamespaceImport => {
                return crate::ls::mig::m5x_3::namespace_import_name(&named_bindings);
            }
            _ => {}
        }
    }
    if !for_rename {
        return crate::ls::mig::m5x_3::import_declaration_module_specifier(node);
    }
    None
}

pub fn get_adjusted_location_for_export_declaration(
    node: &Arc<Node>,
    for_rename: bool,
) -> Option<Arc<Node>> {
    let export_clause = crate::ls::mig::m5x_3::export_declaration_export_clause(node)?;
    match export_clause.kind {
        SyntaxKind::NamedExports => {
            let elements = crate::ls::mig::m5x_3::named_exports_elements(&export_clause)?;
            if elements.len() != 1 {
                return None;
            }
            return crate::ls::mig::m5x_3::export_specifier_name(&elements[0]);
        }
        SyntaxKind::NamespaceExport => {
            return crate::ls::mig::m5x_3::namespace_export_name(&export_clause);
        }
        _ => {}
    }
    if !for_rename {
        return crate::ls::mig::m5x_3::export_declaration_module_specifier(node);
    }
    None
}

pub fn get_contextual_type_from_parent(
    node: &Arc<Node>,
    type_checker: &mut tsox_checker::checker::Checker,
    context_flags: tsox_checker::checker::ContextFlags,
) -> Option<Arc<tsox_checker::checker::types::Type>> {
    let parent = node
        .parent()
        .map(|p| crate::ls::mig::m5x::walk_up_parenthesized_expressions(&p))?;
    match parent.kind {
        SyntaxKind::NewExpression => {
            type_checker.get_contextual_type(&parent, context_flags)
        }
        SyntaxKind::BinaryExpression => {
            if let Some(operator_token) = crate::ls::mig::m5x_3::binary_expression_operator_token(&parent) {
                if crate::ls::mig::m5x_7::is_equality_operator_kind(operator_token.kind) {
                    let right = crate::ls::mig::m5x_3::binary_expression_right(&parent);
                    let left = crate::ls::mig::m5x_3::binary_expression_left(&parent);
                    let target = if right.as_ref().is_some_and(|r| Arc::ptr_eq(r, node)) { left } else { right };
                    if let Some(target) = target {
                        return Some(type_checker.get_type_at_location(&target));
                    }
                    return None;
                }
            }
            type_checker.get_contextual_type(node, context_flags)
        }
        SyntaxKind::CaseClause => {
            crate::ls::mig::m5x_7::get_switched_type(&parent, type_checker)
        }
        _ => type_checker.get_contextual_type(node, context_flags),
    }
}

pub fn get_contextual_type_from_parent_or_ancestor_type_node(
    node: &Arc<Node>,
    type_checker: &mut tsox_checker::checker::Checker,
) -> Option<Arc<tsox_checker::checker::types::Type>> {
    if node.flags.contains(ast::NodeFlags::JSDoc)
        && !node.flags.contains(ast::NodeFlags::JavaScriptFile)
    {
        return None;
    }
    if let Some(contextual_type) =
        get_contextual_type_from_parent(node, type_checker, tsox_checker::checker::ContextFlags::None)
    {
        return Some(contextual_type);
    }
    if let Some(ancestor_type_node) = get_ancestor_type_node(node) {
        return Some(type_checker.get_type_at_location(&ancestor_type_node));
    }
    None
}

pub fn get_ancestor_type_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut last_type_node: Option<Arc<Node>> = None;
    let mut current = node.parent();
    while let Some(n) = current {
        if ast::is_type_node(&n) {
            last_type_node = Some(Arc::clone(&n));
        }
        let stop = n.parent().map_or(true, |p| {
            !ast::is_qualified_name(&p) && !ast::is_type_node(&p) && !ast::is_type_element(&p)
        });
        if stop {
            break;
        }
        current = n.parent();
    }
    last_type_node
}

pub fn is_source_file_with_global_exports(node: &Arc<Node>) -> bool {
    ast::is_source_file(node)
        && matches!(
            &node.data,
            tsox_frontend::ast::NodeData::SourceFile(d) if d.global_exports.is_some()
        )
}
