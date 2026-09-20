#![allow(unused_imports)]

use crate::checker::typenode_type_operators::*;

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
                        let target = node
                            .parent()
                            .map(|p| {
                                crate::checker::checker_es_symbol::walk_up_parenthesized_types(&p)
                            })
                            .unwrap_or_else(|| Arc::clone(node));
                        self.get_es_symbol_like_type_for_node(&target)
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
                    // readonly T[] 的数组形态：带 IsReadonlyArray 标志的数组实例。
                    // 不能经 ReadonlyArray 接口实例化承载：lib 自身成员
                    // （flatMap 的 U | readonly U[] 等）会在接口解析期自引用重入，
                    // 触发降级使实例化永不上缓存
                    if self.is_array_type(&inner)
                        && let Some(element) = inner
                            .as_object()
                            .and_then(|o| o.type_arguments.first().cloned())
                    {
                        return self.create_array_type_ex(element, true);
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

            let defer = self.should_defer_indexed_access_type(&object_type, &index_type);
            if defer {
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
                        .insert(Arc::as_ptr(&index_type_node) as *const tsox_frontend::ast::Node)
                {
                    let degraded = self.degraded_type_ptrs.contains(&index_type.id)
                        || self.degraded_type_ptrs.contains(&object_type.id);
                    if !degraded {
                        let type_str = if index_type_node.kind == SyntaxKind::BigIntLiteral {
                            "bigint".to_string()
                        } else {
                            self.type_to_string(&index_type)
                        };
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            index_type_node.loc,
                            tsox_core::diagnostics::messages_generated::
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
        // mapped 实例随类型实参语境变化（接口成员在声明期解析为裸类型参数
        // 版本，实例化期必须重解析），缓存按 (节点, 栈哈希) 区分；无栈语境
        // 的结果才写入免哈希节点缓存
        let key = (node.id() as usize, self.type_argument_stack_hash());
        if let Some(t) = self.type_node_subst_cache.get(&key) {
            return Arc::clone(t);
        }
        let result = self.build_mapped_type(node);
        self.attach_alias_for_type_node(node, &result);
        if self.type_argument_stack.is_empty() {
            self.cache_type(node, result.clone());
        }
        if self.type_node_subst_cache.len() >= self.type_node_subst_cache_limit {
            self.type_node_subst_cache.clear();
        }
        self.type_node_subst_cache.insert(key, Arc::clone(&result));
        result
    }

    pub(crate) fn get_type_from_conditional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_conditional_type(node);
        self.attach_alias_for_type_node(node, &result);
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
                if let Some(p) = n.parent()
                    && let NodeData::ConditionalTypeNode(cd) = &p.data
                    && Arc::ptr_eq(&cd.extends_type, &n)
                {
                    in_extends_clause = true;
                    break;
                }
                cur = n.parent();
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
                        &tsox_core::diagnostics::messages_generated::
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
}
