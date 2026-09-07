#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_type_from_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let key = (node.id() as usize, self.type_argument_stack_hash());
        if let Some(t) = self.type_node_subst_cache.get(&key) {
            return Arc::clone(t);
        }

        let degraded_epoch = self.heritage_degraded_events;

        if !self.type_node_resolving.insert(key) {
            return self.error_type();
        }
        self.type_node_query_epochs.push(degraded_epoch);

        let over_budget = !self.type_argument_stack.is_empty() && {
            self.type_instantiation_count += 1;
            self.type_instantiation_count >= 5_000_000
        };
        let result = if self.type_resolution_depth >= 500 || over_budget {
            if !self.type_instantiation_limit_reported {
                self.type_instantiation_limit_reported = true;
                let file = self.current_file.clone();
                let loc = self
                    .current_node
                    .as_ref()
                    .map(|n| n.loc)
                    .unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_INSTANTIATION_IS_EXCESSIVELY_DEEP_AND_POSSIBLY_INFINITE,
                    Vec::new(),
                ));
            }
            self.error_type()
        } else {
            self.type_resolution_depth += 1;
            let r = self.get_type_from_type_node_worker(node);
            self.type_resolution_depth -= 1;
            r
        };
        self.type_node_resolving.remove(&key);
        self.type_node_query_epochs.pop();
        if self.heritage_degraded_events == degraded_epoch {
            if self.type_node_subst_cache.len() >= self.type_node_subst_cache_limit {
                self.type_node_subst_cache.clear();
            }
            self.type_node_subst_cache.insert(key, Arc::clone(&result));
        }
        result
    }

    pub(crate) fn type_argument_stack_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        if self.type_argument_stack.is_empty() {
            return 0;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for map in &self.type_argument_stack {
            let mut entries: Vec<(usize, usize)> = map
                .iter()
                .map(|(k, v)| (*k as usize, v.id as usize))
                .collect();
            entries.sort_unstable();
            entries.len().hash(&mut h);
            for e in entries {
                e.hash(&mut h);
            }
        }
        h.finish()
    }

    pub(crate) fn get_type_from_type_node_worker(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            SyntaxKind::AnyKeyword | SyntaxKind::JSDocAllType => self.any_type(),
            SyntaxKind::JSDocNonNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNonNullableType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::JSDocNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNullableType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                if self.strict_null_checks {
                    self.get_nullable_type(&t, TypeFlags::Null)
                } else {
                    t
                }
            }
            SyntaxKind::JSDocVariadicType => {
                let inner = node
                    .type_node()
                    .expect("JSDocVariadicType has type")
                    .clone();
                let elem_type = self.get_type_from_type_node(&inner);
                self.create_array_type(elem_type)
            }
            SyntaxKind::JSDocOptionalType => {
                let inner = node
                    .type_node()
                    .expect("JSDocOptionalType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                self.add_optionality(&t)
            }
            SyntaxKind::UnknownKeyword => self.unknown_type(),
            SyntaxKind::StringKeyword => self.string_type(),
            SyntaxKind::NumberKeyword => self.number_type(),
            SyntaxKind::BigIntKeyword => self.bigint_type(),
            SyntaxKind::BooleanKeyword => self.boolean_type(),
            SyntaxKind::SymbolKeyword => self.es_symbol_type(),
            SyntaxKind::VoidKeyword => self.void_type(),
            SyntaxKind::UndefinedKeyword => self.undefined_type(),
            SyntaxKind::NullKeyword => self.null_type(),
            SyntaxKind::NeverKeyword => self.never_type(),
            SyntaxKind::ObjectKeyword => self.non_primitive_type(),

            SyntaxKind::ConstKeyword => self.any_type(),
            SyntaxKind::ThisType | SyntaxKind::ThisKeyword => {
                self.get_type_from_this_type_node(node)
            }
            SyntaxKind::LiteralType => self.get_type_from_literal_type_node(node),
            SyntaxKind::TypeReference | SyntaxKind::ExpressionWithTypeArguments => {
                self.get_type_from_type_reference(node)
            }
            SyntaxKind::TypePredicate => {
                if let NodeData::TypePredicateNode(data) = &node.data {
                    if data.asserts_modifier.is_some() {
                        return self.void_type();
                    }
                }
                self.boolean_type()
            }
            SyntaxKind::TypeQuery => self.get_type_from_type_query_node(node),
            SyntaxKind::ArrayType | SyntaxKind::TupleType => {
                self.get_type_from_array_or_tuple_type_node(node)
            }
            SyntaxKind::OptionalType => self.get_type_from_optional_type_node(node),
            SyntaxKind::UnionType => self.get_type_from_union_type_node(node),
            SyntaxKind::IntersectionType => self.get_type_from_intersection_type_node(node),
            SyntaxKind::NamedTupleMember => self.get_type_from_named_tuple_type_node(node),
            SyntaxKind::ParenthesizedType => {
                let inner = node
                    .type_node()
                    .expect("ParenthesizedType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::RestType => self.get_type_from_rest_type_node(node),
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType | SyntaxKind::TypeLiteral => {
                self.get_type_from_type_literal_or_function_or_constructor_type_node(node)
            }
            SyntaxKind::TypeOperator => self.get_type_from_type_operator_node(node),
            SyntaxKind::IndexedAccessType => self.get_type_from_indexed_access_type_node(node),
            SyntaxKind::TemplateLiteralType => self.get_type_from_template_type_node(node),
            SyntaxKind::MappedType => self.get_type_from_mapped_type_node(node),
            SyntaxKind::ConditionalType => self.get_type_from_conditional_type_node(node),
            SyntaxKind::InferType => self.get_type_from_infer_type_node(node),
            SyntaxKind::ImportType => self.get_type_from_import_type_node(node),
            _ => self.error_type(),
        }
    }

    pub(crate) fn get_cached_type(&self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if !self.type_argument_stack.is_empty() {
            return None;
        }
        self.type_node_links
            .get(node)
            .and_then(|l| l.resolved_type.clone())
    }

    pub(crate) fn cache_type(&mut self, node: &Arc<Node>, t: Arc<Type>) {
        if !self.type_argument_stack.is_empty() {
            return;
        }
        if let Some(epoch) = self.type_node_query_epochs.last()
            && *epoch != self.heritage_degraded_events
        {
            return;
        }
        self.type_node_links.get_or_default(node).resolved_type = Some(t);
    }
}
