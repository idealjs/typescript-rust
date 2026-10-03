#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::mig::m3f_3::is_block_scope;
use tsox_frontend::ast::node_data_generated::is_parameter_declaration;
use tsox_frontend::ast::utilities::{
    find_ancestor, get_root_declaration, is_function_like_or_class_static_block_declaration,
};
use tsox_frontend::ast::{Node, SyntaxKind, Symbol};
use tsox_core::core::core::Pattern;

use crate::checker::checker::*;
use crate::checker::mig::m2a::r20k6_defs::PatternAmbientModule;
use crate::checker::mig::m2b::r22k6_defs;
use crate::checker::utilities_is_private_within_ambient::is_private_within_ambient;

impl TypeSystemEntity {
    pub fn as_symbol(&self) -> Option<&Arc<Symbol>> { ::tsox_core::fntrace::enter("as_symbol"); 
        match self {
            TypeSystemEntity::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("as_type"); 
        match self {
            TypeSystemEntity::Type(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_signature(&self) -> Option<&Arc<Signature>> { ::tsox_core::fntrace::enter("as_signature"); 
        match self {
            TypeSystemEntity::Signature(s) => Some(s),
            _ => None,
        }
    }
}

pub(crate) fn get_enclosing_block_scope_container(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_enclosing_block_scope_container"); 
    let start = node.parent()?;
    find_ancestor(&start, |current| {
        if current.kind == SyntaxKind::Block {
            return current
                .parent()
                .is_some_and(|p| !is_function_like_or_class_static_block_declaration(&p));
        }
        is_block_scope(current, current)
    })
}

pub(crate) fn declaration_belongs_to_private_ambient_member(declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("declaration_belongs_to_private_ambient_member"); 
    let root = get_root_declaration(declaration);
    let member_declaration = if is_parameter_declaration(&root) {
        root.parent()
    } else {
        Some(root)
    };
    member_declaration
        .map(|member| is_private_within_ambient(&member))
        .unwrap_or(false)
}

pub(crate) fn find_best_pattern_match<'a, T>(
    values: &[&'a T],
    get_pattern: impl Fn(&T) -> &Pattern,
    candidate: &str,
) -> Option<&'a T> { ::tsox_core::fntrace::enter("find_best_pattern_match"); 
    let mut best: Option<&'a T> = None;
    let mut longest_match_prefix_length: isize = -1;
    for value in values {
        let pattern = get_pattern(value);
        if (pattern.star_index == -1 || pattern.star_index > longest_match_prefix_length)
            && pattern.matches(candidate)
        {
            best = Some(*value);
            longest_match_prefix_length = pattern.star_index;
        }
    }
    best
}

pub(crate) fn pattern_ambient_modules() -> Vec<PatternAmbientModule> { ::tsox_core::fntrace::enter("pattern_ambient_modules"); 
    r22k6_defs::PATTERN_AMBIENT_MODULES.with(|m| {
        m.borrow()
            .iter()
            .map(|v| PatternAmbientModule {
                pattern: v.pattern.clone(),
                symbol: Arc::clone(&v.symbol),
            })
            .collect()
    })
}

thread_local! {
    static PATTERN_AMBIENT_MODULE_AUGMENTATIONS: RefCell<HashMap<String, Arc<Symbol>>> =
        RefCell::new(HashMap::new());
    static PATTERN_AMBIENT_MODULE_AUGMENTATION_TARGETS: RefCell<HashMap<String, Arc<Symbol>>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn get_pattern_ambient_module_augmentation(name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_pattern_ambient_module_augmentation"); 
    PATTERN_AMBIENT_MODULE_AUGMENTATIONS.with(|m| m.borrow().get(name).cloned())
}

pub(crate) fn get_pattern_ambient_module_augmentation_target(name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_pattern_ambient_module_augmentation_target"); 
    PATTERN_AMBIENT_MODULE_AUGMENTATION_TARGETS.with(|m| m.borrow().get(name).cloned())
}
