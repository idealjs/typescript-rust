#![allow(unused_imports)]

use crate::checker::relater_index_signatures::*;

impl Checker {
    pub fn index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        source_is_primitive: bool,
        relation: RelationKind,
    ) -> Ternary {
        if relation == RelationKind::Identity {
            return self.index_signatures_identical_to(source, target);
        }
        let target_indexes = self.get_index_infos_of_type(target);
        let target_has_string_index = target_indexes.iter().any(|info| {
            info.key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false)
        });
        let mut result = Ternary::True;
        for target_info in &target_indexes {
            let target_value_any = target_info
                .value_type
                .as_ref()
                .map(|v| v.flags.contains(TypeFlags::Any))
                .unwrap_or(false);
            let target_key_is_string = target_info
                .key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false);
            let related = if relation != RelationKind::StrictSubtype
                && !source_is_primitive
                && target_has_string_index
                && target_key_is_string
                && target_value_any
            {
                Ternary::True
            } else if self.is_generic_mapped_type(source) && target_key_is_string {
                let template = self.get_template_type_from_mapped_type(source);
                match template {
                    Some(template) => {
                        let target_value = target_info
                            .value_type
                            .clone()
                            .unwrap_or_else(|| self.any_type());
                        self.compare_types(template, target_value, relation, false)
                    }
                    None => Ternary::False,
                }
            } else {
                self.type_related_to_index_info(source, target_info, relation)
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    pub fn type_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let target_key = match &target_info.key_type {
            Some(k) => k,
            None => return Ternary::True,
        };
        let source_info = self.get_applicable_index_info(source, target_key);
        if let Some(source_info) = source_info {
            return self.index_info_related_to(&source_info, target_info, relation);
        }

        let is_fresh_literal = source.object_flags.contains(ObjectFlags::FreshLiteral);
        if relation != RelationKind::StrictSubtype || is_fresh_literal {
            if self.is_object_type_with_inferable_index(source) {
                return self.members_related_to_index_info(source, target_info, relation);
            }
        }
        Ternary::False
    }

    pub fn index_info_related_to(
        &mut self,
        source_info: &IndexInfo,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let source_value = source_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        self.compare_types(source_value, target_value, relation, false)
    }

    pub fn index_signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        let source_infos = self.get_index_infos_of_type(source);
        let target_infos = self.get_index_infos_of_type(target);
        if source_infos.len() != target_infos.len() {
            return Ternary::False;
        }
        for target_info in &target_infos {
            let target_key = match &target_info.key_type {
                Some(k) => Arc::clone(k),
                None => continue,
            };
            let source_info = self.get_index_info_of_type(source, &target_key);
            let related = match source_info {
                Some(si) => {
                    let sv = si.value_type.clone().unwrap_or_else(|| self.any_type());
                    let tv = target_info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    let type_related = self.compare_types(sv, tv, RelationKind::Identity, false);
                    let readonly_match = si.is_readonly == target_info.is_readonly;
                    if type_related.is_true() && readonly_match {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                }
                None => Ternary::False,
            };
            if related.is_false() {
                return Ternary::False;
            }
        }
        Ternary::True
    }

    pub fn get_index_infos_of_type(&self, t: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        t.as_structured()
            .map(|s| s.index_infos.clone())
            .unwrap_or_default()
    }

    pub fn get_index_info_of_type(
        &self,
        t: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        for info in self.get_index_infos_of_type(t) {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) || info_key.flags == key_type.flags {
                    return Some(info);
                }
            }
        }

        if key_type.flags.contains(TypeFlags::Number) {
            if let TypeData::Tuple(tuple) = &t.data {
                let elements: Vec<Arc<Type>> = tuple
                    .element_infos
                    .iter()
                    .filter_map(|e| e.type_.clone())
                    .collect();
                if !elements.is_empty() {
                    let value = if elements.len() == 1 {
                        Arc::clone(&elements[0])
                    } else if elements.iter().all(|e| Arc::ptr_eq(e, &elements[0])) {
                        Arc::clone(&elements[0])
                    } else {
                        Arc::new(Type {
                            flags: TypeFlags::Union,
                            object_flags: ObjectFlags::None,
                            id: crate::checker::types::next_type_id(),
                            symbol: None,
                            alias: None,
                            data: TypeData::Union(UnionTypeData {
                                union_or_intersection: UnionOrIntersectionTypeData {
                                    structured: StructuredTypeData::default(),
                                    types: elements,
                                },
                                resolved_reduced_type: std::sync::OnceLock::new(),
                                regular_type: std::sync::OnceLock::new(),
                                origin: None,
                                key_property_name: None,
                                constituent_map: std::collections::HashMap::new(),
                            }),
                        })
                    };
                    return Some(Arc::new(IndexInfo {
                        key_type: Some(self.number_type()),
                        value_type: Some(value),
                        is_readonly: tuple.readonly,
                        declaration: None,
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
            }
        }
        None
    }

    pub fn get_applicable_index_info(
        &self,
        source: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let infos = self.get_index_infos_of_type(source);
        for info in infos {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) {
                    return Some(info);
                }

                if info_key.flags.contains(TypeFlags::Number)
                    && key_type.flags.contains(TypeFlags::String)
                {
                    return Some(info);
                }

                if info_key.flags.contains(TypeFlags::String)
                    && key_type.flags.contains(TypeFlags::Number)
                {
                    return Some(info);
                }
            }
        }
        None
    }

    pub fn is_generic_mapped_type(&self, t: &Arc<Type>) -> bool {
        if let TypeData::Mapped(m) = &t.data {
            m.type_parameter.is_some() && m.template_type.is_some()
        } else {
            false
        }
    }

    pub fn get_template_type_from_mapped_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            // 坏上下文（推断中空作用域）解析出的 error 不得驻留：视为未解析重试
            if let Some(tpl) = &m.template_type
                && tpl.intrinsic_name() != Some("error")
            {
                return Some(Arc::clone(tpl));
            }
            // 惰性解析模板节点并回写（Go getTemplateTypeFromMappedType）。
            // 空栈解析：按节点祖先链构造作用域（模板里的类型参数按声明处解析），
            // 实例差异由替换链承担。解析中途的惰性重入直接让位（外层完成后有缓存）
            let node = m.template_node.clone()?;
            if !self.template_resolving_ids.insert(t.id) {
                return None;
            }
            let chain = m.template_subst.as_ref().map(|c| c.as_ref().clone());
            self.clear_type_node_cache_subtree(&node);
            let saved_stack = std::mem::take(&mut self.type_argument_stack);
            let saved_scopes = std::mem::take(&mut self.scope_stack);
            let mut scope_chain: Vec<u64> = Vec::new();
            let mut cur = node.parent.as_ref();
            while let Some(c) = cur {
                scope_chain.push(c.id());
                cur = c.parent.as_ref();
            }
            scope_chain.reverse();
            self.scope_stack = scope_chain;
            // Go getTemplateTypeFromMappedType（checker.go 23042）：按声明符号直接
            // 实例化别名，不走节点级解析（type_node_resolving 守卫会把外层对同一
            // 模板节点的在途解析误判为循环）
            let resolved = match &node.data {
                tsox_frontend::ast::NodeData::TypeReferenceNode(tr) => {
                    match self.resolve_identifier(&tr.type_name) {
                        Some(symbol) if symbol.flags.intersects(SymbolFlags::TypeAlias) => {
                            let args = tr.type_arguments.clone();
                            self.resolve_type_alias_reference(&symbol, args)
                        }
                        _ => self.get_type_from_type_node(&node),
                    }
                }
                _ => self.get_type_from_type_node(&node),
            };
            self.scope_stack = saved_scopes;
            self.type_argument_stack = saved_stack;
            let resolved = match &chain {
                Some(chain) => self.apply_template_subst_chain(&resolved, chain),
                None => resolved,
            };
            self.template_resolving_ids.remove(&t.id);
            if resolved.intrinsic_name() == Some("error") {
                // 外层对同一别名/节点的解析在途（符号级循环守卫）会产出 error；
                // 让位不驻留：本次不回写，推断层也不缓存，待外层完成后重试
                self.template_resolution_letway = true;
                return None;
            }
            let ptr = Arc::as_ptr(t) as *mut crate::checker::types::Type;
            unsafe {
                if let TypeData::Mapped(m) = &mut (*ptr).data {
                    m.template_type = Some(Arc::clone(&resolved));
                }
            }
            return Some(resolved);
        }
        None
    }

    fn clear_type_node_cache_subtree(&mut self, node: &Arc<Node>) {
        if let Some(links) = self.type_node_links.get_mut(node) {
            links.resolved_type = None;
        }
        let mut children = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        for child in children {
            self.clear_type_node_cache_subtree(&child);
        }
    }
}
