#![allow(unused_imports)]

use super::*;

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
            return self.any_type();
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
                        crate::ast::NodeData::MappedTypeNode(d) => d.type_node.as_ref().map(|tn| {
                            (
                                Arc::clone(&d.type_parameter),
                                Arc::clone(tn),
                                Arc::clone(decl),
                            )
                        }),
                        _ => None,
                    })
                    .and_then(|(tp_node, template_node, decl)| {
                        let tp_sym = self.program.symbol_map().symbol_of(&tp_node).cloned()?;

                        if !Self::type_node_references_name(&template_node, &tp_sym.name) {
                            return None;
                        }
                        let mut mapping = std::collections::HashMap::new();
                        mapping.insert(
                            Arc::as_ptr(&tp_sym) as *const crate::ast::Symbol,
                            Arc::clone(index_type),
                        );
                        self.push_scope(&decl);
                        self.type_argument_stack.push(mapping);
                        let t = self.get_type_from_type_node(&template_node);
                        self.type_argument_stack.pop();
                        self.pop_scope();
                        Some(t)
                    });
                return substituted.unwrap_or_else(|| {
                    Arc::clone(m.template_type.as_ref().expect("template present"))
                });
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
                if let Some(structured) = object_type.as_structured() {
                    let elem_types: Vec<Arc<Type>> = structured
                        .properties
                        .iter()
                        .map(|p| self.get_type_of_symbol(p))
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
