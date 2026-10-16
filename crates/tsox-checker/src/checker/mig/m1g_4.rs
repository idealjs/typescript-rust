#![allow(unused_imports)]
use super::wc3::NodeAccessExt;
use super::m1f_3::get_symbol_path;
use super::m2a::r19k11_defs::R19K11NodeExt;
use super::wc1b::every_type;
use tsox_frontend::ast::get_name_of_declaration;
use tsox_frontend::ast::mig::m3f_2::has_dynamic_name;
use tsox_frontend::ast::node_data_generated::is_binding_element;
use tsox_frontend::ast::node_data_generated::is_property_assignment;
use tsox_frontend::ast::node_data_generated::is_shorthand_property_assignment;
use tsox_frontend::ast::node_data_generated::is_type_parameter_declaration;

use crate::checker::checker::*;
use crate::checker::utilities_is_optional_symbol::is_late_bound_name;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

#[path = "r26k6_defs.rs"]
pub mod r26k6_defs;

use self::r26k6_defs::{unresolved_symbols_get, unresolved_symbols_insert};

impl Checker {
    pub(crate) fn has_default_value(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_default_value"); 
        if is_binding_element(node) {
            return node.initializer().is_some();
        }
        if is_property_assignment(node) {
            if let Some(initializer) = node.initializer() {
                return self.has_default_value(&initializer);
            }
        }
        if is_shorthand_property_assignment(node) {
            if let NodeData::ShorthandPropertyAssignment(d) = &node.data {
                return d.object_assignment_initializer.is_some();
            }
        }
        if node.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(d) = &node.data {
                return d.operator_token.kind == SyntaxKind::EqualsToken;
            }
        }
        false
    }

    pub(crate) fn has_parse_diagnostics(&mut self, source_file: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_parse_diagnostics"); 
        self.get_source_file_of_node(source_file)
            .is_some_and(|f| f.has_parse_diagnostics)
    }

    pub(crate) fn has_signatures(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_signatures"); 
        !self
            .get_signatures_of_structured_type(t, SignatureKind::Call)
            .is_empty()
            || !self
                .get_signatures_of_structured_type(t, SignatureKind::Construct)
                .is_empty()
    }

    pub(crate) fn get_type_with_synthetic_default_only(
        &mut self,
        t: Option<&Arc<Type>>,
        symbol: &Arc<Symbol>,
        original_symbol: &Arc<Symbol>,
        module_specifier: &Arc<Node>,
        import_attributes_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_with_synthetic_default_only"); 
        let has_default_only =
            self.is_only_importable_as_default(module_specifier, None, import_attributes_type);
        if has_default_only && t.is_some_and(|t| !self.is_error_type(t)) {
            let t = t.unwrap();
            let key = CachedTypeKey {
                kind: CachedTypeKind::DefaultOnlyType,
                type_id: t.id,
            };
            if let Some(cached) = self.cached_types.get(&key) {
                return Some(Arc::clone(cached));
            }
            let result =
                self.create_default_property_wrapper_for_module(symbol, original_symbol, None);
            self.cached_types.insert(key, Arc::clone(&result));
            return Some(result);
        }
        None
    }

    pub(crate) fn has_bindable_name(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_bindable_name"); 
        !has_dynamic_name(node) || self.has_late_bindable_name(node)
    }

    pub(crate) fn has_late_bindable_name(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_late_bindable_name"); 
        let Some(name) = get_name_of_declaration(node) else {
            return false;
        };
        if !super::wc3_3::is_late_bindable_ast(&name) {
            return false;
        }
        let t = if name.kind == SyntaxKind::ComputedPropertyName {
            self.check_computed_property_name_type(&name)
        } else {
            self.check_expression_cached(&name.as_element_access_expression().argument_expression)
        };
        crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&t)
    }

    pub(crate) fn has_late_bindable_index_signature(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_late_bindable_index_signature"); 
        get_name_of_declaration(node)
            .is_some_and(|name| self.is_late_bindable_index_signature(&name))
    }

    pub(crate) fn has_type_parameter_default(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_type_parameter_default"); 
        let Some(symbol) = &t.symbol else {
            return false;
        };
        symbol.declarations.iter().any(|d| {
            is_type_parameter_declaration(d)
                && d.as_type_parameter_declaration()
                    .default_type
                    .is_some()
        })
    }

    pub(crate) fn has_array_or_type_type_constraint(
        &mut self,
        type_variable: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("has_array_or_type_type_constraint"); 
        let Some(constraint) = self.get_constraint_of_type_parameter(type_variable) else {
            return false;
        };
        every_type(&constraint, &|t| super::wc1b::is_array_or_tuple_type(t))
    }

    pub(crate) fn get_unresolved_symbol_for_entity_name(
        &mut self,
        name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_unresolved_symbol_for_entity_name"); 
        let identifier = match name.kind {
            SyntaxKind::QualifiedName => match &name.data {
                NodeData::QualifiedName(q) => Arc::clone(&q.right),
                _ => Arc::clone(name),
            },
            SyntaxKind::PropertyAccessExpression => Arc::clone(name.name()?),
            _ => Arc::clone(name),
        };
        let text = identifier.text();
        if !text.is_empty() {
            let parent_symbol = match name.kind {
                SyntaxKind::QualifiedName => match &name.data {
                    NodeData::QualifiedName(q) => {
                        let left = Arc::clone(&q.left);
                        self.get_unresolved_symbol_for_entity_name(&left)
                    }
                    _ => None,
                },
                SyntaxKind::PropertyAccessExpression => match &name.data {
                    NodeData::PropertyAccessExpression(d) => {
                        self.get_unresolved_symbol_for_entity_name(&d.expression)
                    }
                    _ => None,
                },
                _ => None,
            };
            let path = match &parent_symbol {
                Some(parent_symbol) => format!("{}.{}", get_symbol_path(parent_symbol), text),
                None => text.to_string(),
            };
            if let Some(existing) = unresolved_symbols_get(&path) {
                return Some(existing);
            }
            let result =
                self.new_symbol_ex(SymbolFlags::TypeAlias, &text, CheckFlags::Unresolved);
            if let Some(parent_symbol) = &parent_symbol {
                result.set_parent(parent_symbol);
            }
            unresolved_symbols_insert(path, Arc::clone(&result));
            self.type_alias_links
                .get_or_default(&result)
                .declared_type = Some(self.unresolved_type());
            return Some(result);
        }
        Some(self.unknown_symbol())
    }

    pub(crate) fn has_signature_with_arity_greater_than(
        &mut self,
        symbol: &Arc<Symbol>,
        arity: usize,
    ) -> bool { ::tsox_core::fntrace::enter("has_signature_with_arity_greater_than"); 
        for signature in self.get_signatures_of_symbol(Some(symbol)) {
            if self.get_parameter_count(&signature) > arity {
                return true;
            }
        }
        false
    }

    pub(crate) fn get_type_of_first_parameter_of_signature(
        &mut self,
        signature: &Arc<Signature>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_first_parameter_of_signature"); 
        let fallback = self.never_type();
        self.get_type_of_first_parameter_of_signature_with_fallback(signature, &fallback)
    }

    pub(crate) fn get_type_of_first_parameter_of_signature_with_fallback(
        &mut self,
        signature: &Arc<Signature>,
        fallback_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_first_parameter_of_signature_with_fallback"); 
        if !signature.parameters.is_empty() {
            return self.get_type_at_position(signature, 0);
        }
        Arc::clone(fallback_type)
    }

    pub(crate) fn has_type_facts(&mut self, t: &Arc<Type>, mask: TypeFacts) -> bool { ::tsox_core::fntrace::enter("has_type_facts"); 
        !self.get_type_facts(t, mask).is_empty()
    }
}
