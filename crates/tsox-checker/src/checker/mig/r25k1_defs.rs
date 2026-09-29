use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};

pub(crate) fn is_in_property_initializer_or_class_static_block_ex(
    node: &Arc<Node>,
    ignore_arrow_functions: bool,
) -> bool {
    let mut cur = node.parent();
    while let Some(n) = cur {
        match n.kind {
            SyntaxKind::PropertyDeclaration | SyntaxKind::ClassStaticBlockDeclaration => {
                return true
            }
            SyntaxKind::TypeQuery | SyntaxKind::JsxClosingElement => return false,
            SyntaxKind::ArrowFunction => {
                if !ignore_arrow_functions {
                    return false;
                }
            }
            SyntaxKind::Block => {
                let parent_is_fn_like = n.parent().is_some_and(|p| {
                    matches!(
                        p.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::Constructor
                            | SyntaxKind::GetAccessor
                            | SyntaxKind::SetAccessor
                            | SyntaxKind::FunctionExpression
                    )
                });
                if parent_is_fn_like {
                    return false;
                }
            }
            _ => {}
        }
        cur = n.parent();
    }
    false
}

thread_local! {
    static SOURCE_FILE_DEFERRED_NODES: RefCell<HashMap<usize, Vec<Arc<Node>>>> =
        RefCell::new(HashMap::new());
}

fn source_file_key(source_file: &SourceFile) -> usize {
    Arc::as_ptr(&source_file.node) as *const () as usize
}

pub(crate) fn source_file_deferred_nodes_insert(source_file: &SourceFile, node: Arc<Node>) {
    SOURCE_FILE_DEFERRED_NODES.with(|map| {
        let mut map = map.borrow_mut();
        let nodes = map.entry(source_file_key(source_file)).or_default();
        let key = Arc::as_ptr(&node) as *const () as usize;
        if nodes.iter().any(|n| Arc::as_ptr(n) as *const () as usize == key) {
            return;
        }
        nodes.push(node);
    });
}

pub(crate) fn take_source_file_deferred_nodes(source_file: &SourceFile) -> Vec<Arc<Node>> {
    SOURCE_FILE_DEFERRED_NODES.with(|map| {
        map.borrow_mut()
            .remove(&source_file_key(source_file))
            .unwrap_or_default()
    })
}
