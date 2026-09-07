#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_optional_type(&mut self, t: Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            self.get_union_type(vec![t, self.undefined_type()])
        } else {
            t
        }
    }

    pub(crate) fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }

        let types: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }

        let mut flattened: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !flattened.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        flattened.push(Arc::clone(inner));
                    }
                }
            } else if !flattened.iter().any(|s| Arc::ptr_eq(s, &t)) {
                flattened.push(t);
            }
        }
        if flattened.is_empty() {
            return self.never_type();
        }
        if flattened.len() == 1 {
            return flattened.into_iter().next().expect("exactly one");
        }

        {
            let rank = |t: &Arc<Type>| -> u32 {
                if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum) {
                    return TypeFlags::Enum.bits();
                }
                let b = t.flags.bits();
                b & b.wrapping_neg()
            };
            flattened.sort_by_key(rank);
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: flattened,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub(crate) fn get_intersection_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            }),
        ))
    }

    pub(crate) fn create_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        let Some(array_symbol) = self.globals.get("Array").cloned() else {
            return self.get_any_type();
        };
        let target = self.get_declared_type_of_symbol(&array_symbol);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference,
            id: crate::checker::types::next_type_id(),
            symbol: Some(array_symbol),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: Some(target),
                mapper: None,
                type_arguments: vec![element_type],
            }),
        })
    }

    pub(crate) fn array_type_parameter_symbols(&mut self) -> Vec<Arc<Symbol>> {
        if let Some(cached) = &self.array_type_parameter_symbols {
            return cached.clone();
        }
        let collected = self
            .globals
            .get("Array")
            .and_then(|sym| {
                let decl = sym
                    .declarations
                    .iter()
                    .find(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))?;
                let NodeData::InterfaceDeclaration(d) = &decl.data else {
                    return None;
                };
                let sym_map = self.program.symbol_map();
                Some(
                    d.type_parameters
                        .as_ref()?
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();
        self.array_type_parameter_symbols = Some(collected.clone());
        collected
    }

    pub(crate) fn instantiate_array_member_type(
        &mut self,
        obj_type: &Arc<Type>,
        member: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let is_evolving = obj_type.object_flags.contains(ObjectFlags::EvolvingArray);
        if !self.is_array_type(obj_type) && !is_evolving {
            return None;
        }
        if let Some(structured) = obj_type.as_structured()
            && structured.members.get(&member.name).is_some()
        {
            return None;
        }
        let element = match &obj_type.data {
            TypeData::Object(o) => match o.type_arguments.first() {
                Some(e) => Arc::clone(e),
                None => return None,
            },
            TypeData::EvolvingArray(e) => {
                e.element_type.clone().unwrap_or_else(|| self.never_type())
            }
            _ => return None,
        };

        let declared = match self.globals.get("Array").and_then(|sym| {
            self.type_alias_links
                .get(sym)
                .and_then(|l| l.declared_type.clone())
        }) {
            Some(d) => Some(d),

            None => self
                .globals
                .get("Array")
                .cloned()
                .map(|sym| self.resolve_interface_type(&sym, None)),
        };
        let raw = declared
            .as_ref()
            .and_then(|d| d.as_structured())
            .and_then(|s| s.members.get(&member.name).cloned())
            .map(|synthetic| self.get_type_of_symbol(&synthetic))?;
        let key = (
            Arc::as_ptr(&element) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(member) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.array_member_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }

        let mut free_tps: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&raw, SignatureKind::Call) {
            for param in &sig.parameters {
                let pt = self.get_type_of_symbol(param);
                self.collect_free_type_parameters_deep(&pt, &mut free_tps);
            }
            if let Some(rt) = self.get_return_type_of_signature(&sig) {
                self.collect_free_type_parameters_deep(&rt, &mut free_tps);
            }
        }

        let array_tps = self.array_type_parameter_symbols();
        let subst_tps: Vec<Arc<Type>> = free_tps
            .iter()
            .filter(|tp| {
                tp.symbol
                    .as_ref()
                    .is_some_and(|s| array_tps.iter().any(|a| Arc::ptr_eq(a, s)))
            })
            .cloned()
            .collect();
        if subst_tps.is_empty() {
            return Some(raw);
        }
        let substitutions: Vec<Arc<Type>> = std::iter::repeat(Arc::clone(&element))
            .take(subst_tps.len())
            .collect();
        let substituted = self.substitute_infer_type_parameters(&raw, &subst_tps, &substitutions);
        self.array_member_type_cache
            .insert(key, Arc::clone(&substituted));
        Some(substituted)
    }

    pub(crate) fn declared_array_member_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            })?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    pub(crate) fn declared_array_member_symbols(&mut self) -> Vec<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            });
        declared
            .and_then(|t| t.as_structured().map(|s| s.properties.clone()))
            .unwrap_or_default()
    }

    pub(crate) fn global_interface_member_symbol(
        &mut self,
        interface_name: &str,
        member: &str,
    ) -> Option<Arc<Symbol>> {
        let sym = self.globals.get(interface_name).cloned()?;
        let declared = self
            .type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .or_else(|| Some(self.resolve_interface_type(&sym, None)))?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(member).cloned())
    }
}
