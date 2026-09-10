#![allow(unused_imports)]

use crate::checker::typenode_constructors::*;

impl Checker {
    pub(crate) fn get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Arc<Type> {
        if object_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return self.unknown_type();
        }
        if index_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }

        if index_type.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &index_type.data {
                let prop_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.get_indexed_access_type(object_type, c))
                    .collect();
                if prop_types.is_empty() {
                    return self.any_type();
                }
                return self.get_union_type(prop_types);
            }
        }

        if object_type.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(object_type) {
                return self.get_indexed_access_type(&constraint, index_type);
            }
            // Go createIndexedAccessType：泛型对象的索引访问保持延迟（驻留保恒等）
            return self.deferred_indexed_access(object_type, index_type);
        }

        if let TypeData::Mapped(m) = &object_type.data
            && let Some(constraint) = &m.constraint_type
        {
            let generic = constraint
                .flags
                .intersects(TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index)
                || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if self.is_type_assignable_to(index_type, &domain) {
                let substituted = m
                    .declaration
                    .as_ref()
                    .and_then(|decl| match &decl.data {
                        tsox_frontend::ast::NodeData::MappedTypeNode(d) => {
                            d.type_node.as_ref().map(|tn| {
                                (
                                    Arc::clone(&d.type_parameter),
                                    Arc::clone(tn),
                                    Arc::clone(decl),
                                )
                            })
                        }
                        _ => None,
                    })
                    .and_then(|(tp_node, template_node, decl)| {
                        let tp_sym = self.program.symbol_map().symbol_of(&tp_node).cloned()?;
                        let chain = m
                            .template_subst
                            .as_ref()
                            .map(|c| c.as_ref().clone())
                            .unwrap_or_default();

                        if !Self::type_node_references_name(&template_node, &tp_sym.name) {
                            return None;
                        }
                        let mut mapping = std::collections::HashMap::new();
                        mapping.insert(
                            Arc::as_ptr(&tp_sym) as *const tsox_frontend::ast::Symbol,
                            Arc::clone(index_type),
                        );
                        self.push_scope(&decl);
                        self.type_argument_stack.push(mapping);
                        let t = self.get_type_from_type_node(&template_node);
                        self.type_argument_stack.pop();
                        self.pop_scope();
                        let t = if chain.is_empty() {
                            t
                        } else {
                            self.apply_template_subst_chain(&t, &chain)
                        };
                        Some(t)
                    });
                if let Some(t) = substituted {
                    return t;
                }
                // 实例化路径已预替换的模板优先
                if let Some(t) = &m.template_type {
                    return Arc::clone(t);
                }
                // 模板惰性：等价 get_template_type_from_mapped_type（此处已被
                // &t.data 借用，直接解析模板节点）
                let Some(template_node) = m.template_node.clone() else {
                    return self.get_any_type();
                };
                let chain = m
                    .template_subst
                    .as_ref()
                    .map(|c| c.as_ref().clone())
                    .unwrap_or_default();
                let saved_stack = std::mem::take(&mut self.type_argument_stack);
                let t = self.get_type_from_type_node(&template_node);
                self.type_argument_stack = saved_stack;
                let t = if chain.is_empty() {
                    t
                } else {
                    self.apply_template_subst_chain(&t, &chain)
                };
                let ptr = Arc::as_ptr(object_type) as *mut crate::checker::types::Type;
                unsafe {
                    if let TypeData::Mapped(m) = &mut (*ptr).data {
                        m.template_type = Some(Arc::clone(&t));
                    }
                }
                return t;
            }
        }

        if index_type.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &index_type.data {
                if let LiteralValue::String(name) = &lit.value {
                    if let Some(structured) = object_type.as_structured() {
                        if let Some(sym) = structured.members.get(name) {
                            return self.get_type_of_symbol(sym);
                        }

                        if let Some(value_type) =
                            self.lookup_index_signature_value(structured, index_type)
                        {
                            return value_type;
                        }
                    }
                    return self.any_type();
                }
            }
        }

        if index_type.flags.contains(TypeFlags::Number)
            || index_type.flags.contains(TypeFlags::NumberLiteral)
        {
            if self.is_array_type(object_type) {
                return self.get_array_element_type(object_type);
            }

            if self.is_tuple_type(object_type) {
                if let TypeData::Tuple(tup) = &object_type.data {
                    // 数字字面量按位取元素，number 取全体并集
                    if index_type.flags.contains(TypeFlags::NumberLiteral)
                        && let Some(n) = index_type.literal_value().and_then(|v| match v {
                            LiteralValue::Number(n) => Some(n),
                            _ => None,
                        })
                    {
                        let i = n.0 as usize;
                        if i < tup.element_infos.len()
                            && let Some(t) = &tup.element_infos[i].type_
                        {
                            return Arc::clone(t);
                        }
                    }
                    let elem_types: Vec<Arc<Type>> = tup
                        .element_infos
                        .iter()
                        .filter_map(|ei| ei.type_.clone())
                        .collect();
                    if !elem_types.is_empty() {
                        return self.get_union_type(elem_types);
                    }
                }
            }
        }

        if let Some(structured) = object_type.as_structured() {
            if let Some(value_type) = self.lookup_index_signature_value(structured, index_type) {
                return value_type;
            }
        }
        self.any_type()
    }

    pub(crate) fn lookup_index_signature_value(
        &mut self,
        structured: &StructuredTypeData,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        for info in &structured.index_infos {
            let key_matches = match info.key_type.as_ref() {
                Some(key) => {
                    if key.flags.contains(TypeFlags::String) {
                        index_type.flags.contains(TypeFlags::String)
                            || index_type.flags.contains(TypeFlags::StringLiteral)
                    } else if key.flags.contains(TypeFlags::Number) {
                        index_type.flags.contains(TypeFlags::Number)
                            || index_type.flags.contains(TypeFlags::NumberLiteral)
                    } else {
                        false
                    }
                }
                None => true,
            };
            if key_matches {
                return info.value_type.clone();
            }
        }
        None
    }
}
