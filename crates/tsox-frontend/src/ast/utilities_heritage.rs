use crate::ast::*;
use std::sync::Arc;

pub fn get_heritage_clauses(node: &Arc<Node>) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("get_heritage_clauses"); 
    match &node.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
        NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
        NodeData::InterfaceDeclaration(d) => d.heritage_clauses.as_ref(),
        _ => None,
    }
}

pub fn get_heritage_clause(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_heritage_clause"); 
    if let Some(clauses) = get_heritage_clauses(node) {
        for clause in &clauses.nodes {
            if let NodeData::HeritageClause(d) = &clause.data {
                if d.token == kind {
                    return Some(Arc::clone(clause));
                }
            }
        }
    }
    None
}

pub fn get_extends_heritage_clause_elements(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_extends_heritage_clause_elements"); 
    get_heritage_elements(node, SyntaxKind::ExtendsKeyword)
}

pub fn get_implements_heritage_clause_elements(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_implements_heritage_clause_elements"); 
    get_heritage_elements(node, SyntaxKind::ImplementsKeyword)
}

fn get_heritage_elements(node: &Arc<Node>, kind: SyntaxKind) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_heritage_elements"); 
    match get_heritage_clause(node, kind) {
        Some(clause) => {
            if let NodeData::HeritageClause(d) = &clause.data {
                return d.types.nodes.clone();
            }
            Vec::new()
        }
        None => Vec::new(),
    }
}

pub fn get_extends_heritage_clause_element(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_extends_heritage_clause_element"); 
    get_extends_heritage_clause_elements(node)
        .into_iter()
        .next()
}

pub fn get_containing_class(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_containing_class"); 
    let parent = node.parent()?;
    find_ancestor(&parent, is_class_like)
}
