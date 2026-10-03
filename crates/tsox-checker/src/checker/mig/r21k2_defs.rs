#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

pub fn is_entity_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_entity_name"); 
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::QualifiedName
    )
}

pub fn get_class_like_declaration_of_symbol(symbol: &Symbol) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_class_like_declaration_of_symbol"); 
    for declaration in &symbol.declarations {
        if matches!(
            declaration.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return Some(Arc::clone(declaration));
        }
    }
    None
}

pub fn get_host_signature_from_jsdoc(name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_host_signature_from_jsdoc"); 
    let mut current = name.parent();
    while let Some(node) = current {
        match node.kind {
            SyntaxKind::QualifiedName | SyntaxKind::PropertyAccessExpression => {
                current = node.parent();
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature => return Some(node),
            _ => return None,
        }
    }
    None
}

impl Checker {
    pub fn resolve_external_module(
        &mut self,
        location: &Arc<Node>,
        module_name: &str,
        module_not_found_error: Option<&'static tsox_core::diagnostics::Message>,
        error_node: Option<&Arc<Node>>,
        is_for_augmentation: bool,
        _import_attributes_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_external_module"); 
        let specifier = super::r19k8_defs::new_string_literal(module_name);
        let Some(symbol) = self.resolve_external_module_name(&specifier) else {
            if let Some(message) = module_not_found_error {
                let location = error_node.unwrap_or(location);
                self.error_message(location, *message, &[]);
            }
            return None;
        };
        let resolved = self.resolve_external_module_symbol(&symbol, false);
        if is_for_augmentation && Arc::ptr_eq(&resolved, &self.unknown_symbol()) {
            return Some(symbol);
        }
        Some(resolved)
    }

    pub fn silent_never_signature(&mut self) -> Arc<Signature> { ::tsox_core::fntrace::enter("silent_never_signature"); 
        let silent_never = self.silent_never_type();
        self.new_signature(
            SignatureFlags::None,
            None,
            &[],
            None,
            &[],
            &silent_never,
            None,
            0,
        )
    }

    pub fn any_base_type_index_info(&mut self) -> Arc<IndexInfo> { ::tsox_core::fntrace::enter("any_base_type_index_info"); 
        let value_type = self.any_type();
        self.new_index_info(&self.string_type(), &value_type, false, None, &[])
    }
}
