use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SyntaxKind};

pub(crate) fn type_node_references_names(node: &Arc<Node>, names: &[String]) -> bool {
    let mut found = false;
    NodeWalker {
        names,
        found: &mut found,
    }
    .walk(node);
    found
}

pub(crate) struct NodeWalker<'a> {
    pub(crate) names: &'a [String],
    pub(crate) found: &'a mut bool,
}

impl<'a> NodeWalker<'a> {
    pub(crate) fn walk(&mut self, node: &Arc<Node>) {
        if *self.found {
            return;
        }
        if node.kind == SyntaxKind::Identifier && names_contain(self.names, node.text()) {
            *self.found = true;
            return;
        }
        crate::ast::node_data_generated::for_each_child(node, |c| {
            self.walk(c);
            *self.found
        });
    }
}

pub(crate) fn names_contain(names: &[String], text: &str) -> bool {
    names.iter().any(|n| n == text)
}

pub(crate) fn type_name_inside_conditional_branch(node: &Arc<Node>) -> bool {
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if matches!(&a.data, NodeData::ConditionalTypeNode(_)) {
            if let NodeData::ConditionalTypeNode(c) = &a.data {
                if node_inside(node, &c.check_type) || node_inside(node, &c.extends_type) {
                    cur = a.parent.as_ref();
                    continue;
                }
            }
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

pub(crate) fn node_inside(node: &Arc<Node>, root: &Arc<Node>) -> bool {
    if Arc::ptr_eq(node, root) {
        return true;
    }
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if Arc::ptr_eq(a, root) {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

pub(crate) fn type_name_shadowed_by_type_parameter(type_name: &Arc<Node>) -> bool {
    let name = type_name.text();
    let mut cur = type_name.parent.as_ref();
    while let Some(a) = cur {
        let tps = match &a.data {
            NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
            NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
            NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
            NodeData::MethodDeclaration(m) => m.type_parameters.as_ref(),
            NodeData::FunctionDeclaration(f) => f.type_parameters.as_ref(),
            _ => None,
        };
        if let Some(list) = tps
            && list
                .iter()
                .any(|p| p.name().is_some_and(|n| n.text() == name))
        {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}
