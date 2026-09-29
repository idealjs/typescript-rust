use crate::ast::*;
use std::sync::Arc;
use tsox_core::core::tristate::Tristate;

pub fn is_non_local_alias(symbol: Option<&Symbol>, excludes: SymbolFlags) -> bool {
    let Some(symbol) = symbol else {
        return false;
    };
    let flags = symbol.flags;
    flags.intersection(SymbolFlags::Alias | excludes) == SymbolFlags::Alias
        || flags.intersects(SymbolFlags::Alias) && flags.intersects(SymbolFlags::Assignment)
}

pub fn is_object_binding_or_assignment_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::BindingElement
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
    )
}

pub fn is_object_literal_method(node: &Node) -> bool {
    node.kind == SyntaxKind::MethodDeclaration
        && node
            .parent()
            .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
}

pub fn is_object_literal_or_class_expression_method_or_accessor(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    ) && node.parent().is_some_and(|p| {
        matches!(p.kind, SyntaxKind::ObjectLiteralExpression | SyntaxKind::ClassExpression)
    })
}

pub fn is_object_type_declaration(node: &Node) -> bool {
    is_class_like(node) || is_interface_declaration(node) || is_type_literal_node(node)
}

pub fn is_optional_chain_root(node: &Node) -> bool {
    if !is_optional_chain(node) || is_non_null_expression(node) {
        return false;
    }
    match &node.data {
        NodeData::PropertyAccessExpression(d) => d.question_dot_token.is_some(),
        NodeData::ElementAccessExpression(d) => d.question_dot_token.is_some(),
        NodeData::CallExpression(d) => d.question_dot_token.is_some(),
        _ => false,
    }
}

pub fn is_outermost_optional_chain(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return true;
    };
    !is_optional_chain(&parent)
        || is_optional_chain_root(&parent)
        || !parent.expression().is_some_and(|e| Arc::ptr_eq(e, node))
}

pub fn is_parameter_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Parameter | SyntaxKind::TypeParameter
    )
}

pub fn is_parameter_property_declaration(node: &Node, parent: &Node) -> bool {
    is_parameter_declaration(node)
        && has_syntactic_modifier(node, ModifierFlags::ParameterPropertyModifier)
        && parent.kind == SyntaxKind::Constructor
}

pub fn is_parameter_property_modifier(kind: SyntaxKind) -> bool {
    modifier_to_flag(kind).intersects(ModifierFlags::ParameterPropertyModifier)
}

pub fn is_parse_tree_node(node: &Node) -> bool {
    !node.flags.intersects(NodeFlags::Synthesized)
}

pub fn is_part_of_parameter_declaration(node: &Arc<Node>) -> bool {
    get_root_declaration(node).kind == SyntaxKind::Parameter
}

pub fn is_part_of_type_query(node: &Arc<Node>) -> bool {
    let mut node = node.clone();
    while matches!(node.kind, SyntaxKind::QualifiedName | SyntaxKind::Identifier) {
        let Some(parent) = node.parent() else {
            return false;
        };
        node = parent;
    }
    node.kind == SyntaxKind::TypeQuery
}

pub fn is_plain_js_file(file: Option<&SourceFile>, check_js: Tristate) -> bool {
    match file {
        Some(file) => {
            matches!(file.script_kind, ScriptKind::Js | ScriptKind::Jsx)
                && check_js == Tristate::Unknown
        }
        None => false,
    }
}

pub fn is_primitive_literal_value(node: &Node, include_big_int: bool) -> bool {
    match node.kind {
        SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::NumericLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral => true,
        SyntaxKind::BigIntLiteral => include_big_int,
        SyntaxKind::PrefixUnaryExpression => match &node.data {
            NodeData::PrefixUnaryExpression(d) => match d.operator {
                SyntaxKind::MinusToken => {
                    is_numeric_literal(&d.operand)
                        || (include_big_int && is_big_int_literal(&d.operand))
                }
                SyntaxKind::PlusToken => is_numeric_literal(&d.operand),
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

pub fn is_private_identifier_class_element_declaration(node: &Node) -> bool {
    (is_property_declaration(node) || is_method_or_accessor(node))
        && node.name().is_some_and(|n| is_private_identifier(n))
}

pub fn is_property_access_entity_name_expression(node: &Node, allow_js: bool) -> bool {
    is_property_access_expression(node)
        && node.name().is_some_and(|n| is_identifier(n))
        && node
            .expression()
            .is_some_and(|e| crate::ast::mig::x6a::is_entity_name_expression_ex(e, allow_js))
}

pub fn is_proto_setter(node: &Node) -> bool {
    (is_identifier(node) || is_string_literal(node)) && node.text() == "__proto__"
}

pub fn is_prototype_access(node: &Arc<Node>) -> bool {
    if crate::ast::mig::m3f_3::is_bindable_static_access_expression(node, false) {
        if let Some(name) = crate::ast::mig::m3e_4::get_element_or_property_access_name(node) {
            return name.text() == "prototype";
        }
    }
    false
}

pub fn is_push_or_unshift_identifier(node: &Node) -> bool {
    let text = node.text();
    text == "push" || text == "unshift"
}

pub fn is_require_variable_statement(node: &Node) -> bool {
    if !is_variable_statement(node) {
        return false;
    }
    let NodeData::VariableStatement(d) = &node.data else {
        return false;
    };
    let NodeData::VariableDeclarationList(list) = &d.declaration_list.data else {
        return false;
    };
    !list.declarations.nodes.is_empty()
        && list
            .declarations
            .nodes
            .iter()
            .all(|decl| crate::ast::mig::m3g_3::is_variable_declaration_initialized_to_require(decl))
}

pub fn is_resolution_mode_override_host(node: Option<&Node>) -> bool {
    let Some(node) = node else {
        return false;
    };
    matches!(
        node.kind,
        SyntaxKind::ImportType
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
    )
}

pub fn is_right_side_of_property_access(node: &Arc<Node>) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind == SyntaxKind::PropertyAccessExpression
            && parent.name().is_some_and(|n| Arc::ptr_eq(n, node))
    })
}

pub fn is_right_side_of_qualified_name_or_property_access(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match &parent.data {
        NodeData::QualifiedName(d) => Arc::ptr_eq(&d.right, node),
        NodeData::PropertyAccessExpression(d) => Arc::ptr_eq(&d.name, node),
        NodeData::MetaProperty(d) => Arc::ptr_eq(&d.name, node),
        _ => false,
    }
}

pub fn is_signed_numeric_literal(node: &Node) -> bool {
    if node.kind != SyntaxKind::PrefixUnaryExpression {
        return false;
    }
    match &node.data {
        NodeData::PrefixUnaryExpression(d) => {
            matches!(d.operator, SyntaxKind::PlusToken | SyntaxKind::MinusToken)
                && is_numeric_literal(&d.operand)
        }
        _ => false,
    }
}

pub fn is_string_literal_like_type(node: &Node) -> bool {
    if node.kind != SyntaxKind::LiteralType {
        return false;
    }
    match &node.data {
        NodeData::LiteralTypeNode(d) => is_string_literal_like(&d.literal),
        _ => false,
    }
}

pub fn is_string_text_containing_node(node: &Node) -> bool {
    node.kind == SyntaxKind::StringLiteral || is_template_literal_kind(node.kind)
}

pub fn is_super_property(node: &Node) -> bool {
    (is_property_access_expression(node) || is_element_access_expression(node))
        && node.expression().is_some_and(|e| e.kind == SyntaxKind::SuperKeyword)
}

pub fn is_tag_name(node: &Arc<Node>) -> bool {
    node.parent().is_some_and(|parent| {
        is_jsdoc_tag(&parent) && parent.name().is_some_and(|n| Arc::ptr_eq(n, node))
    })
}

pub fn is_template_literal_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail
    )
}

pub fn is_template_literal_token(node: &Node) -> bool {
    is_template_literal_kind(node.kind)
}

pub fn is_this_in_type_query(node: &Arc<Node>) -> bool {
    if !is_this_identifier(Some(node)) {
        return false;
    }
    let mut current = node.clone();
    while let Some(parent) = current.parent() {
        if !is_qualified_name(&parent) {
            break;
        }
        let is_left = match &parent.data {
            NodeData::QualifiedName(d) => Arc::ptr_eq(&d.left, &current),
            _ => false,
        };
        if !is_left {
            break;
        }
        current = parent;
    }
    current.parent().is_some_and(|p| p.kind == SyntaxKind::TypeQuery)
}

pub fn is_this_parameter(node: &Node) -> bool {
    is_parameter_declaration(node)
        && node
            .name()
            .is_some_and(|n| is_this_identifier(Some(&**n)))
}

pub fn is_type_declaration(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::TypeParameter
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::EnumDeclaration => true,
        SyntaxKind::ImportClause => crate::ast::mig::m3b::is_type_only(node),
        SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier => node
            .parent()
            .and_then(|p| p.parent())
            .is_some_and(|gp| crate::ast::mig::m3b::is_type_only(&gp)),
        _ => false,
    }
}

pub fn is_type_declaration_name(name: &Arc<Node>) -> bool {
    if name.kind != SyntaxKind::Identifier {
        return false;
    }
    name.parent().is_some_and(|parent| {
        is_type_declaration(&parent) && get_name_of_declaration(&parent).is_some_and(|n| Arc::ptr_eq(&n, name))
    })
}

pub fn is_type_keyword_token(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeKeyword
}

pub fn is_type_only_import_declaration(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::ImportSpecifier => {
            crate::ast::mig::m3b::is_type_only(node)
                || node
                    .parent()
                    .and_then(|p| p.parent())
                    .is_some_and(|gp| crate::ast::mig::m3b::is_type_only(&gp))
        }
        SyntaxKind::NamespaceImport => node
            .parent()
            .is_some_and(|p| crate::ast::mig::m3b::is_type_only(&p)),
        SyntaxKind::ImportClause | SyntaxKind::ImportEqualsDeclaration => crate::ast::mig::m3b::is_type_only(node),
        _ => false,
    }
}

fn is_type_only_export_declaration(node: &Node) -> bool {
    match node.kind {
        SyntaxKind::ExportSpecifier => {
            crate::ast::mig::m3b::is_type_only(node)
                || node
                    .parent()
                    .and_then(|p| p.parent())
                    .is_some_and(|gp| crate::ast::mig::m3b::is_type_only(&gp))
        }
        SyntaxKind::ExportDeclaration => match &node.data {
            NodeData::ExportDeclaration(d) => {
                d.is_type_only && d.module_specifier.is_some() && d.export_clause.is_none()
            }
            _ => false,
        },
        SyntaxKind::NamespaceExport => node.parent().is_some_and(|p| crate::ast::mig::m3b::is_type_only(&p)),
        _ => false,
    }
}

pub fn is_type_only_import_or_export_declaration(node: &Node) -> bool {
    is_type_only_import_declaration(node) || is_type_only_export_declaration(node)
}

pub fn is_type_reference_type(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TypeReference | SyntaxKind::ExpressionWithTypeArguments
    )
}
