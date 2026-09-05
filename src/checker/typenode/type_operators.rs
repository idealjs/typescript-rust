use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    Node, Symbol, SymbolFlags,
    SyntaxKind,
};

use crate::checker::checker::Checker;


use super::*;


impl Checker {
    pub(crate) fn get_type_from_type_operator_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = match &node.data {
            NodeData::TypeOperatorNode(data) => match data.operator {
                SyntaxKind::KeyOfKeyword => {
                    let arg_type = self.get_type_from_type_node(&data.type_node);
                    self.get_index_type(&arg_type)
                }
                SyntaxKind::UniqueKeyword => {
                    if data.type_node.kind == SyntaxKind::SymbolKeyword {
                        self.es_symbol_type()
                    } else {
                        self.error_type()
                    }
                }
                SyntaxKind::ReadonlyKeyword => {
                    let inner = self.get_type_from_type_node(&data.type_node);

                    if let TypeData::Tuple(tuple) = &inner.data {
                        if !tuple.readonly {
                            return Arc::new(Type {
                                flags: inner.flags,
                                object_flags: inner.object_flags,
                                id: crate::checker::types::next_type_id(),
                                symbol: None,
                                alias: None,
                                data: TypeData::Tuple(TupleTypeData {
                                    interface_data: InterfaceTypeData::default(),
                                    element_infos: tuple.element_infos.clone(),
                                    min_length: tuple.min_length,
                                    fixed_length: tuple.fixed_length,
                                    combined_flags: tuple.combined_flags,
                                    readonly: true,
                                }),
                            });
                        }
                    }
                    inner
                }
                _ => self.error_type(),
            },
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_indexed_access_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = {
            let (object_type_node, index_type_node) = match &node.data {
                NodeData::IndexedAccessTypeNode(data) => {
                    (Arc::clone(&data.object_type), Arc::clone(&data.index_type))
                }
                _ => return self.error_type(),
            };
            let object_type = self.get_type_from_type_node(&object_type_node);
            let index_type = self.get_type_from_type_node(&index_type_node);

            if self.should_defer_indexed_access_type(&object_type, &index_type) {
                Arc::new(Type::new(
                    TypeFlags::IndexedAccess,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(Arc::clone(&object_type)),
                        index_type: Some(Arc::clone(&index_type)),
                        access_flags: AccessFlags::None,
                    }),
                ))
            } else {

                if !self.index_type_is_kind_usable(&index_type)
                    && self
                        .indexed_access_2538_reported
                        .insert(Arc::as_ptr(&index_type_node) as *const crate::ast::Node)
                {

                    let degraded = self.degraded_type_ptrs.contains(&index_type.id)
                        || self.degraded_type_ptrs.contains(&object_type.id);
                    if !degraded {
                        let type_str = if index_type_node.kind == SyntaxKind::BigIntLiteral {
                            "bigint".to_string()
                        } else {
                            self.type_to_string(&index_type)
                        };
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            index_type_node.loc,
                            crate::diagnostics::messages_generated::
                                TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                            vec![type_str],
                        ));
                    }
                }

                self.get_indexed_access_type(&object_type, &index_type)
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn should_defer_indexed_access_type(
        &self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> bool {
        if self.type_flags_is_generic_index_type(index_type) {
            return true;
        }
        if self.type_flags_is_generic_object_type(object_type) {
            if let TypeData::Tuple(tup) = &object_type.data {
                if index_type_less_than_fixed(index_type, tup.fixed_length) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub(crate) fn index_type_is_kind_usable(&mut self, t: &Arc<Type>) -> bool {
        let primitive_index_kinds = TypeFlags::from_bits_truncate(
            TypeFlags::Any.bits()
                | TypeFlags::Unknown.bits()
                | TypeFlags::Never.bits()
                | TypeFlags::String.bits()
                | TypeFlags::StringLiteral.bits()
                | TypeFlags::StringMapping.bits()
                | TypeFlags::TemplateLiteral.bits()
                | TypeFlags::Number.bits()
                | TypeFlags::NumberLiteral.bits()
                | TypeFlags::ESSymbol.bits()
                | TypeFlags::UniqueESSymbol.bits()
                | TypeFlags::Enum.bits()
                | TypeFlags::EnumLiteral.bits(),
        );
        let constituents: Vec<Arc<Type>> = if t.flags.contains(TypeFlags::Union) {
            t.types().map(|ts| ts.to_vec()).unwrap_or_default()
        } else {
            vec![Arc::clone(t)]
        };
        if constituents.is_empty() {
            return true;
        }
        for c in &constituents {
            if c.flags.intersects(primitive_index_kinds) {
                continue;
            }
            let ok = self.is_type_assignable_to(c, &self.string_type())
                || self.is_type_assignable_to(c, &self.number_type())
                || self.is_type_assignable_to(c, &self.es_symbol_type());
            if !ok {
                return false;
            }
        }
        true
    }

    pub(crate) fn get_type_from_template_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_template_literal_type(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_mapped_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_mapped_type(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_conditional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_conditional_type(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_infer_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        {
            let mut in_extends_clause = false;
            let mut cur = Some(Arc::clone(node));
            while let Some(n) = cur {
                if let Some(p) = n.parent.clone()
                    && let NodeData::ConditionalTypeNode(cd) = &p.data
                    && Arc::ptr_eq(&cd.extends_type, &n)
                {
                    in_extends_clause = true;
                    break;
                }
                cur = n.parent.clone();
            }
            if !in_extends_clause {
                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 1338 && d.loc.pos() == node.loc.pos());
                if !already {
                    self.grammar_error_on_node(
                        node,
                        &crate::diagnostics::messages_generated::
                            X_INFER_DECLARATIONS_ARE_ONLY_PERMITTED_IN_THE_EXTENDS_CLAUSE_OF_A_CONDITIONAL_TYPE,
                    );
                }
            }
        }

        let result = {
            let tp_node = match &node.data {
                NodeData::InferTypeNode(data) => &data.type_parameter,
                _ => return self.error_type(),
            };
            let symbol = self.program.symbol_map().symbol_of(tp_node).map(Arc::clone);
            match symbol {
                Some(sym) => self.get_type_parameter_from_symbol(&sym),
                None => self.error_type(),
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn build_conditional_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (check_type_node, extends_type_node) = match &node.data {
            NodeData::ConditionalTypeNode(data) => {
                (Arc::clone(&data.check_type), Arc::clone(&data.extends_type))
            }
            _ => return self.error_type(),
        };

        let check_type = self.get_type_from_type_node(&check_type_node);
        let extends_type = self.get_type_from_type_node(&extends_type_node);

        let infer_type_parameters = self.collect_infer_type_parameters(node);

        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let saved_name_frames = std::mem::take(&mut self.type_argument_name_frames);
        let unmapped_check_type = self.get_type_from_type_node(&check_type_node);
        self.type_argument_stack = saved_stack;
        self.type_argument_name_frames = saved_name_frames;
        let is_distributive = unmapped_check_type.flags.contains(TypeFlags::TypeParameter);
        let check_type_parameter_symbol = if is_distributive {
            unmapped_check_type.symbol.clone()
        } else {
            None
        };

        let root = Box::new(ConditionalRoot {
            node: Some(Arc::clone(node)),
            check_type: Some(Arc::clone(&check_type)),
            extends_type: Some(Arc::clone(&extends_type)),
            is_distributive,
            check_type_parameter_symbol,
            infer_type_parameters: infer_type_parameters.clone(),
            outer_type_parameters: Vec::new(),
            alias: None,
            creation_scopes: self.scope_stack.clone(),
        });

        let cond_type = Arc::new(Type::new(
            TypeFlags::Conditional,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: Some(root),
                check_type: Some(Arc::clone(&check_type)),
                extends_type: Some(Arc::clone(&extends_type)),
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: None,
                combined_mapper: None,
                creation_type_argument_stack: self
                    .type_argument_stack
                    .iter()
                    .map(|frame| {
                        frame
                            .iter()
                            .map(|(k, v)| (*k as usize, Arc::clone(v)))
                            .collect::<HashMap<_, _>>()
                    })
                    .collect(),
            }),
        ));

        if let Some(resolved) = self.resolve_conditional_type(&cond_type) {
            resolved
        } else {
            cond_type
        }
    }

    pub(crate) fn collect_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> {

        let symbols: Vec<Arc<Symbol>> = self
            .program
            .symbol_map()
            .locals_of(node)
            .map(|locals| {
                locals
                    .iter()
                    .filter(|(_, sym)| sym.flags.contains(SymbolFlags::TypeParameter))
                    .map(|(_, sym)| Arc::clone(sym))
                    .collect()
            })
            .unwrap_or_default();
        symbols
            .into_iter()
            .map(|sym| self.get_type_parameter_from_symbol(&sym))
            .collect()
    }

    pub(crate) fn get_type_from_import_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        if let NodeData::ImportTypeNode(d) = &node.data
            && let Some(attrs) = &d.attributes
        {
            let attrs = Arc::clone(attrs);
            let _ = self.get_resolution_mode_override(&attrs, true);
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn cross_product_union_size(types: &[Arc<Type>]) -> u64 {
        let mut size: u64 = 1;
        for t in types {
            if let TypeData::Union(u) = &t.data {
                size = size.saturating_mul(u.union_or_intersection.types.len() as u64);
            } else if t.flags.contains(TypeFlags::Never) {
                return 0;
            }
        }
        size
    }
}
