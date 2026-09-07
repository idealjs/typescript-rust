#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn substituted_member_type_of(
        &mut self,
        owner: &Arc<Type>,
        prop: &Arc<Symbol>,
    ) -> Arc<Type> {
        let Some(obj) = owner.as_object() else {
            return self.get_type_of_symbol(prop);
        };
        if obj.type_arguments.is_empty() {
            return self.get_type_of_symbol(prop);
        }
        let Some(owner_sym) = owner.symbol.clone() else {
            return self.get_type_of_symbol(prop);
        };
        let key = (
            Arc::as_ptr(owner) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(prop) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.instantiated_member_type_cache.get(&key) {
            return Arc::clone(&cached.1);
        }

        let result = if owner_sym.flags.contains(SymbolFlags::Interface) {
            let proper =
                self.resolve_interface_type_ex(&owner_sym, Some(obj.type_arguments.clone()));
            let prop_sym = proper
                .as_structured()
                .and_then(|s| s.members.get(&prop.name).cloned());
            match prop_sym {
                Some(ps) => self.get_type_of_symbol(&ps),
                None => self.get_type_of_symbol(prop),
            }
        } else {
            self.substitute_member_type_fallback(&owner_sym, prop, &obj.type_arguments)
        };

        if self.instantiated_member_type_cache.len() >= self.instantiated_member_type_cache_limit {
            self.instantiated_member_type_cache.clear();
        }

        self.instantiated_member_type_cache
            .insert(key, (Arc::clone(owner), Arc::clone(&result)));
        result
    }

    pub(crate) fn substitute_member_type_fallback(
        &mut self,
        owner_sym: &Arc<Symbol>,
        prop: &Arc<Symbol>,
        args: &[Arc<Type>],
    ) -> Arc<Type> {
        let decl_tps = self.declared_type_parameter_types(owner_sym);
        if decl_tps.len() == args.len() && !decl_tps.is_empty() {
            let raw = self.get_type_of_symbol(prop);
            let substitutions = args.to_vec();
            let r = self.substitute_infer_type_parameters(&raw, &decl_tps, &substitutions);
            r
        } else {
            self.get_type_of_symbol(prop)
        }
    }

    pub(crate) fn declared_type_parameter_types(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> {
        let decl = symbol.declarations.iter().find(|d| {
            matches!(
                d.data,
                NodeData::InterfaceDeclaration(_) | NodeData::ClassDeclaration(_)
            )
        });
        let Some(decl) = decl else {
            return Vec::new();
        };
        let tps = match &decl.data {
            NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(tps) = tps else {
            return Vec::new();
        };
        let tp_syms: Vec<Arc<Symbol>> = {
            let sym_map = self.program.symbol_map();
            tps.iter()
                .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                .collect()
        };

        self.push_ts2304_suppression();
        let types = tp_syms
            .iter()
            .map(|tp_sym| self.get_type_parameter_from_symbol(tp_sym))
            .collect();
        self.pop_ts2304_suppression();
        types
    }

    pub(crate) fn collect_free_type_parameters_deep(
        &mut self,
        t: &Arc<Type>,
        out: &mut Vec<Arc<Type>>,
    ) {
        match &t.data {
            TypeData::TypeParameter(_) => {
                if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                    out.push(Arc::clone(t));
                }
            }
            TypeData::Union(u) => {
                for ty in &u.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Intersection(i) => {
                for ty in &i.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Object(o) => {
                for ty in &o.type_arguments {
                    self.collect_free_type_parameters_deep(ty, out);
                }

                for sig in o.structured.signatures.clone() {
                    for param in sig.parameters.iter() {
                        let pt = self.get_type_of_symbol(param);
                        self.collect_free_type_parameters_deep(&pt, out);
                    }
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let rt = Arc::clone(rt);
                        self.collect_free_type_parameters_deep(&rt, out);
                    }
                }
            }
            TypeData::Tuple(tu) => {
                for ei in &tu.element_infos {
                    if let Some(ty) = &ei.type_ {
                        self.collect_free_type_parameters_deep(ty, out);
                    }
                }
            }
            TypeData::IndexedAccess(ia) => {
                if let Some(obj) = &ia.object_type {
                    self.collect_free_type_parameters_deep(obj, out);
                }
                if let Some(idx) = &ia.index_type {
                    self.collect_free_type_parameters_deep(idx, out);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn create_tuple_type(&mut self, element_types: Vec<Arc<Type>>) -> Arc<Type> {
        let element_infos: Vec<TupleElementInfo> = element_types
            .iter()
            .map(|t| TupleElementInfo {
                flags: ElementFlags::Required,
                labeled_declaration: None,
                type_: Some(Arc::clone(t)),
            })
            .collect();
        let fixed_length = element_types.len();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Tuple(TupleTypeData {
                interface_data: InterfaceTypeData::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::Required,
                readonly: false,
            }),
        })
    }

    pub(crate) fn get_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Never) {
            return self.never_type();
        }
        if t.flags.contains(TypeFlags::Any) {
            return self.string_type();
        }

        if t.flags.contains(TypeFlags::Union) {
            let types = match &t.data {
                TypeData::Union(u) => &u.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let mut common: Option<Vec<String>> = None;
            for constituent in types {
                let k = self.get_index_type(constituent);
                let names = self.string_literal_values(&k);
                common = Some(match common.take() {
                    None => names,
                    Some(acc) => acc.into_iter().filter(|n| names.contains(n)).collect(),
                });
            }
            let names = common.unwrap_or_default();
            if names.is_empty() {
                return self.never_type();
            }
            let literals: Vec<Arc<Type>> = names
                .into_iter()
                .map(|n| self.get_string_literal_type(&n))
                .collect();
            return self.get_union_type(literals);
        }

        if t.flags.contains(TypeFlags::Intersection) {
            let types = match &t.data {
                TypeData::Intersection(i) => &i.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let keys: Vec<Arc<Type>> = types.iter().map(|c| self.get_index_type(c)).collect();
            return self.get_union_type(keys);
        }

        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.get_index_type(&constraint);
            }

            return self.string_type();
        }

        if let TypeData::Mapped(m) = &t.data
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
            if domain.flags.contains(TypeFlags::String) {
                let keys = vec![domain, self.number_type()];
                return self.get_union_type(keys);
            }
            return domain;
        }

        if let Some(structured) = t.as_structured() {
            let mut keys: Vec<Arc<Type>> = structured
                .properties
                .iter()
                .filter(|p| !p.name.starts_with('#'))
                .map(|p| self.get_string_literal_type(&p.name))
                .collect();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    keys.push(Arc::clone(key));

                    if key.flags.contains(TypeFlags::String) {
                        keys.push(self.number_type());
                    }
                }
            }
            if keys.is_empty() {
                return self.never_type();
            }
            return self.get_union_type(keys);
        }

        self.never_type()
    }

    pub(crate) fn type_node_references_name(node: &Arc<Node>, name: &str) -> bool {
        if node.kind == SyntaxKind::Identifier && node.text() == name {
            return true;
        }
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |c| {
            found = found || Self::type_node_references_name(c, name);
            found
        });
        found
    }
}
