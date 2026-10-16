use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::mig::m3b::{decorators, members, module_specifier, parameters};
use tsox_frontend::ast::mig::m3g_2::is_this_parameter;
use tsox_frontend::ast::mig::m3g_3::node_is_decorated;
use tsox_frontend::ast::mig::m3h::get_import_type_node_literal;
use tsox_frontend::ast::mig::w3::get_all_accessor_declarations;
use tsox_frontend::ast::mig::x1a::arguments;
use tsox_frontend::ast::{is_accessor, is_method_declaration, is_string_literal};
use tsox_frontend::ast::{Diagnostic, Node, SourceFile, SyntaxKind};

use crate::checker::types::{Type, TypeData, TypeFlags};

pub fn is_union_with_undefined(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_union_with_undefined"); 
    t.flags.intersects(TypeFlags::UNION)
        && t.types()
            .and_then(|ts| ts.first())
            .is_some_and(|first| first.flags.intersects(TypeFlags::UNDEFINED))
}

pub fn is_union_with_null(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_union_with_null"); 
    t.flags.intersects(TypeFlags::UNION)
        && t.types().is_some_and(|ts| {
            ts.first().is_some_and(|t0| t0.flags.intersects(TypeFlags::Null))
                || ts.get(1).is_some_and(|t1| t1.flags.intersects(TypeFlags::Null))
        })
}

pub fn get_constituent_count(t: &Type) -> usize { ::tsox_core::fntrace::enter("get_constituent_count"); 
    if !t.flags.intersects(TypeFlags::UNION | TypeFlags::INTERSECTION) || t.alias.is_some() {
        return 1;
    }
    if t.flags.intersects(TypeFlags::UNION) {
        if let TypeData::Union(u) = &t.data {
            if let Some(origin) = &u.origin {
                return get_constituent_count(origin);
            }
        }
    }
    t.types().map(get_constituent_count_of_types).unwrap_or(1)
}

pub fn get_constituent_count_of_types(types: &[Arc<Type>]) -> usize { ::tsox_core::fntrace::enter("get_constituent_count_of_types"); 
    types.iter().map(|t| get_constituent_count(t)).sum()
}

pub fn get_external_module_name(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_external_module_name"); 
    match node.kind {
        SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration => {
            module_specifier(node).cloned()
        }
        SyntaxKind::ImportEqualsDeclaration => match &node.data {
            tsox_frontend::ast::NodeData::ImportEqualsDeclaration(d) => {
                if d.module_reference.kind == SyntaxKind::ExternalModuleReference {
                    d.module_reference.expression().cloned()
                } else {
                    None
                }
            }
            _ => None,
        },
        SyntaxKind::ImportType => get_import_type_node_literal(node).cloned(),
        SyntaxKind::CallExpression => arguments(node).first().cloned(),
        SyntaxKind::ModuleDeclaration => {
            let name = node.name()?;
            if is_string_literal(name) {
                Some(Arc::clone(name))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn has_decorators(node: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("has_decorators"); 
    node.is_some_and(|n| !decorators(n).is_empty())
}

fn is_entity_name_expression_local(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_entity_name_expression_local"); 
    node.kind == SyntaxKind::Identifier
        || (node.kind == SyntaxKind::PropertyAccessExpression
            && node.expression().is_some_and(|e| is_entity_name_expression_local(&e)))
}

pub fn class_element_or_class_element_parameter_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("class_element_or_class_element_parameter_is_decorated"); 
    let mut parameter_list: Option<&[Arc<Node>]> = None;
    let accessor_declarations;
    if is_accessor(node) {
        accessor_declarations = get_all_accessor_declarations(members(parent), node);
        let declarations = &accessor_declarations;
        let first_accessor_with_decorators = if has_decorators(declarations.first_accessor.as_deref()) {
            declarations.first_accessor.clone()
        } else if has_decorators(declarations.second_accessor.as_deref()) {
            declarations.second_accessor.clone()
        } else {
            None
        };
        if !first_accessor_with_decorators
            .as_ref()
            .is_some_and(|d| Arc::ptr_eq(d, node))
        {
            return false;
        }
        if let Some(set_accessor) = &declarations.set_accessor {
            parameter_list = Some(parameters(set_accessor));
        }
    } else if is_method_declaration(node) {
        parameter_list = Some(parameters(node));
    }
    if node_is_decorated(use_legacy_decorators, node, Some(parent), None) {
        return true;
    }
    if let Some(parameter_list) = parameter_list {
        for parameter in parameter_list {
            if is_this_parameter(parameter) {
                continue;
            }
            if node_is_decorated(use_legacy_decorators, parameter, Some(node), Some(parent)) {
                return true;
            }
        }
    }
    false
}

pub fn new_text_range(start: usize, end: usize) -> TextRange { ::tsox_core::fntrace::enter("new_text_range"); 
    TextRange::new(start, end)
}

pub fn new_diagnostic(
    file: &Arc<SourceFile>,
    loc: TextRange,
    message: Message,
    args: &[String],
) -> Diagnostic { ::tsox_core::fntrace::enter("new_diagnostic"); 
    Diagnostic::new(Some(Arc::clone(file)), loc, message, args.to_vec())
}
