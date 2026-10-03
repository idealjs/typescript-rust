#![allow(unused_imports)]

use super::m2d::EmitResolver;
use crate::checker::checker_checker::*;
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use crate::checker::types::{SymbolAccessibility, SymbolAccessibilityResult};
use crate::checker::mig::m2c_5::TypeReferenceSerializationKind;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags};

impl EmitResolver {
    pub(crate) fn checker_mut(&self) -> &mut Checker { ::tsox_core::fntrace::enter("checker_mut"); 
        unsafe { &mut *self.checker }
    }

    pub fn is_symbol_accessible(
        &self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        meaning: SymbolFlags,
        should_compute_alias_to_mark_visible: bool,
    ) -> SymbolAccessibilityResult { ::tsox_core::fntrace::enter("is_symbol_accessible"); 
        let checker = self.checker_mut();
        checker.is_symbol_accessible(
            symbol,
            Some(enclosing_declaration),
            meaning,
            should_compute_alias_to_mark_visible,
        )
    }

    pub fn is_entity_name_visible(
        &self,
        entity_name: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> SymbolAccessibilityResult { ::tsox_core::fntrace::enter("is_entity_name_visible"); 
        if !is_parse_tree_node(entity_name) {
            return SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                ..Default::default()
            };
        }
        let checker = self.checker_mut();
        checker.is_entity_name_visible(
            entity_name,
            enclosing_declaration.expect("enclosing declaration"),
        )
    }

    pub fn get_referenced_value_declaration_unsafe(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_value_declaration_unsafe"); 
        self.checker_mut()
            .get_referenced_value_declaration_unsafe(node)
    }

    pub fn get_enum_member_value(&self, node: &Arc<Node>) -> tsox_frontend::evaluator::EvalResult { ::tsox_core::fntrace::enter("get_enum_member_value"); 
        if !is_parse_tree_node(node) {
            return tsox_frontend::evaluator::EvalResult::none();
        }
        self.checker_mut().get_enum_member_value(node)
    }

    pub fn is_expando_function_declaration_unsafe(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_expando_function_declaration_unsafe"); 
        self.checker_mut()
            .is_expando_function_declaration_unsafe(node)
    }

    pub fn requires_adding_implicit_undefined_unsafe(
        &self,
        declaration: &Arc<Node>,
        symbol: Option<&Arc<Symbol>>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("requires_adding_implicit_undefined_unsafe"); 
        if !is_parse_tree_node(declaration) {
            return false;
        }
        let checker = self.checker_mut();
        self.requires_adding_implicit_undefined(checker, declaration, symbol, enclosing_declaration)
    }

    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_literal_const_declaration"); 
        self.checker_mut().is_literal_const_declaration(node)
    }

    pub fn create_late_bound_index_signatures<T>(
        &self,
        container: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        _tracker: &T,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("create_late_bound_index_signatures"); 
        let checker = self.checker_mut();
        checker.create_late_bound_index_signatures(
            container,
            enclosing_declaration.expect("enclosing declaration"),
            flags,
            internal_flags,
            None,
        )
    }

    pub fn get_properties_of_container_function_unsafe(
        &self,
        node: Option<&Arc<Node>>,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_properties_of_container_function_unsafe"); 
        self.checker_mut().get_properties_of_container_function(node)
    }

    pub fn get_type_reference_serialization_kind_unsafe(
        &self,
        type_name: &Arc<Node>,
        location: &Arc<Node>,
    ) -> TypeReferenceSerializationKind { ::tsox_core::fntrace::enter("get_type_reference_serialization_kind_unsafe"); 
        self.checker_mut()
            .get_type_reference_serialization_kind(type_name, location)
    }

    pub fn try_js_type_node_to_type_node<T>(
        &self,
        type_node: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        _tracker: &T,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_js_type_node_to_type_node"); 
        let checker = self.checker_mut();
        checker.try_js_type_node_to_type_node(
            type_node,
            enclosing_declaration.expect("enclosing declaration"),
            flags,
            internal_flags,
            None,
        )
    }

    pub fn create_type_of_declaration<T>(
        &self,
        declaration: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        _tracker: &T,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_type_of_declaration"); 
        let checker = self.checker_mut();
        Some(checker.create_type_of_declaration(
            declaration,
            enclosing_declaration.expect("enclosing declaration"),
            flags,
            internal_flags,
            None,
        ))
    }

    pub fn create_return_type_of_signature_declaration<T>(
        &self,
        signature_declaration: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        _tracker: &T,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_return_type_of_signature_declaration"); 
        let checker = self.checker_mut();
        Some(checker.create_return_type_of_signature_declaration(
            signature_declaration,
            enclosing_declaration.expect("enclosing declaration"),
            flags,
            internal_flags,
            None,
        ))
    }

    pub fn create_type_parameters_of_signature_declaration<T>(
        &self,
        signature_declaration: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        _tracker: &T,
    ) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("create_type_parameters_of_signature_declaration"); 
        let checker = self.checker_mut();
        Some(checker.create_type_parameters_of_signature_declaration(
            signature_declaration,
            enclosing_declaration.expect("enclosing declaration"),
            flags,
            internal_flags,
            None,
        ))
    }

    pub fn create_literal_const_value<T>(
        &self,
        node: &Option<Arc<Node>>,
        _tracker: &T,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("create_literal_const_value"); 
        let node = node.as_ref()?;
        let checker = self.checker_mut();
        checker.create_literal_const_value(node, None)
    }
}
