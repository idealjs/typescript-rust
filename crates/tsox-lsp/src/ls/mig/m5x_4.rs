#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

pub fn is_name_of_module_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_name_of_module_declaration"); 
    node.parent().map_or(false, |parent| {
        parent.kind == SyntaxKind::ModuleDeclaration
            && crate::ls::mig::m5x_3::module_declaration_name(&parent).map_or(false, |name| Arc::ptr_eq(&name, node))
    })
}

pub fn is_expression_of_external_module_import_equals_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_expression_of_external_module_import_equals_declaration"); 
    node.parent()
        .and_then(|parent| parent.parent())
        .map_or(false, |grand| {
            crate::ls::mig::m5x_3::is_external_module_import_equals_declaration(&grand)
                && crate::ls::mig::m5x_3::get_external_module_import_equals_declaration_expression(&grand)
                    .map_or(false, |expr| Arc::ptr_eq(&expr, node))
        })
}

pub fn is_namespace_reference(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_namespace_reference"); 
    is_qualified_name_namespace_reference(node) || is_property_access_namespace_reference(node)
}

pub fn is_qualified_name_namespace_reference(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_qualified_name_namespace_reference"); 
    let mut root = Arc::clone(node);
    let mut is_last_clause = true;
    if let Some(parent) = root.parent() {
        if parent.kind == SyntaxKind::QualifiedName {
            while let Some(parent) = root.parent() {
                if parent.kind != SyntaxKind::QualifiedName {
                    break;
                }
                root = parent;
            }
            is_last_clause = crate::ls::mig::m5u_2::qualified_name_right(&root)
                .map_or(false, |right| Arc::ptr_eq(&right, node));
        }
    }
    root.parent().map_or(false, |parent| {
        parent.kind == SyntaxKind::TypeReference && !is_last_clause
    })
}

pub fn is_property_access_namespace_reference(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_property_access_namespace_reference"); 
    let mut root = Arc::clone(node);
    let mut is_last_clause = true;
    if let Some(parent) = root.parent() {
        if parent.kind == SyntaxKind::PropertyAccessExpression {
            while let Some(parent) = root.parent() {
                if parent.kind != SyntaxKind::PropertyAccessExpression {
                    break;
                }
                root = parent;
            }
            is_last_clause =
                crate::ls::mig::m5x_3::access_expression_name(&root).map_or(false, |name| Arc::ptr_eq(&name, node));
        }
    }
    if !is_last_clause {
        if let (Some(parent), Some(grand), Some(decl)) =
            (root.parent(), root.parent().and_then(|p| p.parent()), root.parent().and_then(|p| p.parent()).and_then(|g| g.parent()))
        {
            if parent.kind == SyntaxKind::ExpressionWithTypeArguments
                && grand.kind == SyntaxKind::HeritageClause
            {
                return (decl.kind == SyntaxKind::ClassDeclaration
                    && crate::ls::mig::m5x_3::heritage_clause_token(&grand) == SyntaxKind::ImplementsKeyword)
                    || (decl.kind == SyntaxKind::InterfaceDeclaration
                        && crate::ls::mig::m5x_3::heritage_clause_token(&grand) == SyntaxKind::ExtendsKeyword);
            }
        }
    }
    false
}

pub fn is_this(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_this"); 
    match node.kind {
        SyntaxKind::ThisKeyword => true,
        SyntaxKind::Identifier => {
            ast::node_text(node) == "this"
                && node.parent().map_or(false, |p| p.kind == SyntaxKind::Parameter)
        }
        _ => false,
    }
}

pub fn is_in_right_side_of_internal_import_equals_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_in_right_side_of_internal_import_equals_declaration"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    let mut current = Arc::clone(node);
    while current.parent().map_or(false, |p| p.kind == SyntaxKind::QualifiedName) {
        current = current.parent().unwrap();
    }
    current
        .parent()
        .map_or(false, |parent| {
            crate::ls::mig::m5x_3::is_internal_module_import_equals_declaration(&parent)
                && crate::ls::mig::m5x_3::import_equals_declaration_module_reference(&parent)
                    .map_or(false, |mr| Arc::ptr_eq(&mr, &current))
        })
}

pub fn is_no_substitution_template_literal(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_no_substitution_template_literal"); 
    node.kind == SyntaxKind::NoSubstitutionTemplateLiteral
}

pub fn is_tagged_template_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_tagged_template_expression"); 
    node.kind == SyntaxKind::TaggedTemplateExpression
}

pub fn is_inside_template_literal(
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
) -> bool { ::tsox_core::fntrace::enter("is_inside_template_literal"); 
    ast::mig::m3g_2::is_template_literal_kind(node.kind)
        && ((tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, source_file, false) as usize)
            < position
            && position < node.end()
            || (ast::mig::m3g_3::is_unterminated_literal(node) && position == node.end()))
}

pub fn is_template_head(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_template_head"); 
    node.kind == SyntaxKind::TemplateHead
}

pub fn is_template_tail(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_template_tail"); 
    node.kind == SyntaxKind::TemplateTail
}

pub fn get_target_label(reference_node: &Arc<Node>, label_name: &str) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_target_label"); 
    let mut current = Some(Arc::clone(reference_node));
    while let Some(node) = current {
        if node.kind == SyntaxKind::LabeledStatement
            && crate::ls::mig::m5x_3::labeled_statement_label(&node)
                .map_or(false, |label| ast::node_text(&label) == label_name)
        {
            return crate::ls::mig::m5x_3::labeled_statement_label(&node);
        }
        current = node.parent();
    }
    None
}

pub fn node_seen_tracker() -> impl FnMut(&Arc<Node>) -> bool { ::tsox_core::fntrace::enter("node_seen_tracker"); 
    let mut seen: HashSet<(u64, usize, usize)> = HashSet::new();
    move |node: &Arc<Node>| {
        seen.insert((
            crate::ls::mig::m5w_6::node_root_file_id(node),
            node.pos(),
            node.end(),
        ))
    }
}

pub fn is_object_literal_or_jsx_element(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_object_literal_or_jsx_element"); 
    ast::is_object_literal_element(node)
        || ast::is_jsx_attribute(node)
        || ast::is_jsx_spread_attribute(node)
}

pub fn get_containing_object_literal_element_worker(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_containing_object_literal_element_worker"); 
    match node.kind {
        SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::NumericLiteral => {
            let parent = node.parent()?;
            if parent.kind == SyntaxKind::ComputedPropertyName {
                let grand = parent.parent()?;
                if is_object_literal_or_jsx_element(&grand) {
                    return Some(grand);
                }
                return None;
            }
            get_containing_object_literal_element_worker(&parent)
        }
        SyntaxKind::Identifier | SyntaxKind::JsxNamespacedName => {
            let parent = node.parent()?;
            if is_object_literal_or_jsx_element(&parent)
                && parent
                    .parent()
                    .map_or(false, |grand| {
                        grand.kind == SyntaxKind::ObjectLiteralExpression
                            || grand.kind == SyntaxKind::JsxAttributes
                    })
                && crate::ls::mig::m5x_3::object_literal_element_name(&parent)
                    .map_or(false, |name| Arc::ptr_eq(&name, node))
            {
                return Some(parent);
            }
            None
        }
        _ => None,
    }
}

pub fn is_object_binding_element_without_property_name(binding_element: &Node) -> bool { ::tsox_core::fntrace::enter("is_object_binding_element_without_property_name"); 
    binding_element.kind == SyntaxKind::BindingElement
        && binding_element
            .parent()
            .map_or(false, |parent| parent.kind == SyntaxKind::ObjectBindingPattern)
        && binding_element
            .name()
            .map_or(false, |name| name.kind == SyntaxKind::Identifier)
        && tsox_frontend::ast::mig::m3b::property_name(binding_element).is_none()
}
