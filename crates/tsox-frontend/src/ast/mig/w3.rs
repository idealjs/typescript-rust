#![allow(unused_imports)]

use crate::ast::mig::m3f::get_property_name_for_property_name_node;
use crate::ast::mig::m3f_2::has_dynamic_name;
use crate::ast::node_data_generated::NodeData;
use crate::ast::{self, Node, SourceFile, SyntaxKind};
use std::sync::Arc;

pub struct AllAccessorDeclarations {
    pub first_accessor: Option<Arc<Node>>,
    pub get_accessor: Option<Arc<Node>>,
    pub second_accessor: Option<Arc<Node>>,
    pub set_accessor: Option<Arc<Node>>,
}

pub fn for_each_child_and_js_doc(
    node: &Arc<Node>,
    source_file: &SourceFile,
    v: &mut dyn FnMut(&Arc<Node>),
) {
    for jsdoc in node.jsdoc(source_file) {
        v(&jsdoc);
    }
    crate::ast::for_each_child(node, |child| {
        v(child);
        false
    });
}

pub fn for_each_child_js_doc_parameter_or_property_tag(
    node: &Arc<Node>,
    v: &mut dyn FnMut(&Arc<Node>),
) {
    let NodeData::JSDocParameterOrPropertyTag(data) = &node.data else {
        return;
    };
    v(&data.tag_name);
    if data.is_name_first {
        v(&data.name);
        if let Some(type_expression) = &data.type_expression {
            v(type_expression);
        }
    } else {
        if let Some(type_expression) = &data.type_expression {
            v(type_expression);
        }
        v(&data.name);
    }
    if let Some(comment) = &data.comment {
        for c in &comment.nodes {
            v(c);
        }
    }
}

pub fn get_all_accessor_declarations_for_declaration(
    accessor: &Arc<Node>,
    declarations_of_symbol: &[Arc<Node>],
) -> AllAccessorDeclarations {
    let other_kind = match accessor.kind {
        SyntaxKind::SetAccessor => SyntaxKind::GetAccessor,
        SyntaxKind::GetAccessor => SyntaxKind::SetAccessor,
        _ => panic!("Unexpected node kind {:?}", accessor.kind),
    };
    let mut other_accessor: Option<Arc<Node>> = None;
    for d in declarations_of_symbol {
        if d.kind == other_kind {
            other_accessor = Some(Arc::clone(d));
            break;
        }
    }
    let (first_accessor, second_accessor) = match &other_accessor {
        Some(other) if other.pos() < accessor.pos() => (Some(Arc::clone(other)), Some(Arc::clone(accessor))),
        _ => (Some(Arc::clone(accessor)), other_accessor.clone()),
    };
    AllAccessorDeclarations {
        first_accessor,
        second_accessor,
        set_accessor: None,
        get_accessor: None,
    }
}

pub fn get_all_accessor_declarations(
    parent_declarations: &[Arc<Node>],
    accessor: &Arc<Node>,
) -> AllAccessorDeclarations {
    if has_dynamic_name(accessor) {
        return get_all_accessor_declarations_for_declaration(accessor, std::slice::from_ref(accessor));
    }
    let accessor_name = accessor
        .name()
        .map(get_property_name_for_property_name_node)
        .unwrap_or_default();
    let accessor_static = ast::is_static(accessor);
    let mut matches: Vec<Arc<Node>> = Vec::new();
    for member in parent_declarations {
        if !ast::is_accessor(member) || ast::is_static(member) != accessor_static {
            continue;
        }
        let member_name = member
            .name()
            .map(get_property_name_for_property_name_node)
            .unwrap_or_default();
        if member_name == accessor_name {
            matches.push(Arc::clone(member));
        }
    }
    get_all_accessor_declarations_for_declaration(accessor, &matches)
}
