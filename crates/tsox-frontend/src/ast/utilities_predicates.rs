use crate::ast::*;

pub fn is_jsx_child(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsx_child"); 
    matches!(
        node.kind,
        SyntaxKind::JsxElement
            | SyntaxKind::JsxExpression
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxText
            | SyntaxKind::JsxFragment
    )
}

pub fn is_jsx_attribute_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsx_attribute_like"); 
    matches!(
        node.kind,
        SyntaxKind::JsxAttribute | SyntaxKind::JsxSpreadAttribute
    )
}

pub fn is_import_or_export_specifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_import_or_export_specifier"); 
    matches!(
        node.kind,
        SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier
    )
}

pub fn is_break_or_continue_statement(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_break_or_continue_statement"); 
    matches!(
        node.kind,
        SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement
    )
}

pub fn is_property_access_or_qualified_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_property_access_or_qualified_name"); 
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::QualifiedName
    )
}

pub fn is_property_name_literal(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_property_name_literal"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
    )
}

pub fn is_member_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_member_name"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier
    )
}

pub fn is_entity_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_entity_name"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::QualifiedName
    )
}

pub fn is_property_name_node(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_property_name_node"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ComputedPropertyName
    )
}

pub fn is_entity_name_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_entity_name_expression"); 
    is_identifier(node)
        || node.kind == SyntaxKind::UndefinedKeyword
        || (is_property_access_expression(node) && {
            if let NodeData::PropertyAccessExpression(d) = &node.data {
                is_identifier(&d.name) && is_entity_name_expression(&d.expression)
            } else {
                false
            }
        })
}

pub fn is_modifier_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_modifier_like"); 
    is_modifier_kind(node.kind) || is_decorator(node)
}
