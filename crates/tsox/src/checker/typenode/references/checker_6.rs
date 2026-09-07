#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn merge_interface_type_with_base(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> = derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        let merged = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });

        let merged_degraded = self.degraded_type_ptrs.contains(&base.id)
            || self.degraded_type_ptrs.contains(&derived.id);
        if merged_degraded {
            self.degraded_type_ptrs.insert(merged.id);
        }
        merged
    }

    pub(crate) fn resolve_enum_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }

        let sym_map = self.program.symbol_map();
        let mut entries: Vec<(Option<Arc<Symbol>>, String, Option<Arc<Node>>)> = Vec::new();
        for decl in symbol.declarations.iter() {
            if let NodeData::EnumDeclaration(data) = &decl.data {
                for member_node in data.members.iter() {
                    let NodeData::EnumMember(member) = &member_node.data else {
                        continue;
                    };
                    let member_name = member.name.text().to_string();
                    let member_sym = sym_map.symbol_of(member_node).map(Arc::clone);
                    entries.push((member_sym, member_name, member.initializer.clone()));
                }
            }
        }
        let result = if entries.is_empty() {
            self.error_type()
        } else {
            let mut member_types: Vec<Arc<Type>> = Vec::new();
            let mut next_value: Option<f64> = Some(0.0);
            for (member_sym, member_name, initializer) in &entries {
                let base = match initializer {
                    Some(init) => {
                        let t = self.get_type_of_node(init);

                        if t.flags.contains(TypeFlags::NumberLiteral) {
                            if let TypeData::Literal(LiteralTypeData {
                                value: LiteralValue::Number(n),
                                ..
                            }) = &t.data
                            {
                                next_value = Some(n.0 + 1.0);
                            }
                        } else if t.flags.contains(TypeFlags::StringLiteral) {
                            next_value = None;
                        }
                        t
                    }
                    None => match next_value {
                        Some(v) => {
                            next_value = Some(v + 1.0);
                            self.get_number_literal_type(jsnum::Number::from(v))
                        }
                        None => self.get_any_type(),
                    },
                };

                let member_type = if base
                    .flags
                    .intersects(TypeFlags::NumberLiteral | TypeFlags::StringLiteral)
                {
                    let value = match &base.data {
                        TypeData::Literal(lit) => lit.value.clone(),
                        _ => LiteralValue::None,
                    };
                    let enum_literal_flags = base.flags | TypeFlags::EnumLiteral;
                    let mut regular_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value: value.clone(),
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::new(),
                        }),
                    );
                    regular_ty.symbol = member_sym.clone();
                    let regular_ty = Arc::new(regular_ty);
                    let mut fresh_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value,
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::from(Arc::clone(&regular_ty)),
                        }),
                    );
                    fresh_ty.symbol = member_sym.clone();
                    let fresh_ty = Arc::new(fresh_ty);

                    if let TypeData::Literal(reg_lit) = &regular_ty.data {
                        let _ = reg_lit.fresh_type.set(Arc::clone(&fresh_ty));
                    }
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(fresh_ty),
                                ..Default::default()
                            },
                        );
                    }
                    regular_ty
                } else {
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(Arc::clone(&base)),
                                ..Default::default()
                            },
                        );
                    }
                    base
                };
                let _ = member_name;
                member_types.push(member_type);
            }
            match member_types.len() {
                0 => self.never_type(),
                1 => member_types.into_iter().next().unwrap(),
                _ => self.get_union_type(member_types),
            }
        };
        self.pop_type_resolution();
        self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        result
    }

    pub(crate) fn get_type_of_prototype_property(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let Some(parent) = symbol.parent.clone() else {
            return self.get_any_type();
        };
        let Some(class_decl) = parent
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
            .cloned()
        else {
            return self.get_any_type();
        };
        let ctor_type = self.get_type_of_class_declaration(&class_decl);
        let instance_type = ctor_type
            .as_structured()
            .and_then(|s| s.construct_signatures().first().cloned())
            .and_then(|sig| self.get_return_type_of_signature(&sig))
            .unwrap_or_else(|| self.get_any_type());
        let tp_types: Vec<Arc<Type>> = match &class_decl.data {
            NodeData::ClassDeclaration(d) => match &d.type_parameters {
                Some(tps) => {
                    let sym_map = self.program.symbol_map();
                    let tp_syms: Vec<Arc<Symbol>> = tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect();
                    tp_syms
                        .iter()
                        .map(|s| self.get_type_parameter_from_symbol(s))
                        .collect()
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        if tp_types.is_empty() {
            return instance_type;
        }
        let any_t = self.get_any_type();
        let anys: Vec<Arc<Type>> = tp_types.iter().map(|_| Arc::clone(&any_t)).collect();
        self.substitute_infer_type_parameters(&instance_type, &tp_types, &anys)
    }
}
