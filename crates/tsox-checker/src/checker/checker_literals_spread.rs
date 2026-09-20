#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags};

impl Checker {
    pub(crate) fn get_spread_type(
        &mut self,
        left: &Arc<Type>,
        right: &Arc<Type>,
        symbol: Option<Arc<Symbol>>,
        object_flags: ObjectFlags,
        readonly: bool,
    ) -> Arc<Type> {
        if left.flags.contains(TypeFlags::Any) || right.flags.contains(TypeFlags::Any) {
            return self.get_any_type();
        }
        if left.flags.contains(TypeFlags::Unknown) || right.flags.contains(TypeFlags::Unknown) {
            return self.unknown_type();
        }
        if left.flags.contains(TypeFlags::Never) {
            return Arc::clone(right);
        }
        if right.flags.contains(TypeFlags::Never) {
            return Arc::clone(left);
        }
        let left = self.try_merge_union_of_object_type_and_empty_object(left);
        if left.flags.contains(TypeFlags::Union) {
            if self.spread_cross_product_ok(&left, right) {
                let members = self.union_members(&left).unwrap_or_default();
                let mapped = members
                    .iter()
                    .map(|t| {
                        self.get_spread_type(t, right, symbol.clone(), object_flags, readonly)
                    })
                    .collect();
                return self.get_union_type(mapped);
            }
            return self.error_type();
        }
        let right = self.try_merge_union_of_object_type_and_empty_object(right);
        if right.flags.contains(TypeFlags::Union) {
            if self.spread_cross_product_ok(&left, &right) {
                let members = self.union_members(&right).unwrap_or_default();
                let mapped = members
                    .iter()
                    .map(|t| {
                        self.get_spread_type(&left, t, symbol.clone(), object_flags, readonly)
                    })
                    .collect();
                return self.get_union_type(mapped);
            }
            return self.error_type();
        }
        if right.flags.intersects(
            TYPE_FLAGS_BOOLEAN_LIKE
                | TYPE_FLAGS_NUMBER_LIKE
                | TYPE_FLAGS_BIG_INT_LIKE
                | TYPE_FLAGS_STRING_LIKE
                | TYPE_FLAGS_ENUM_LIKE
                | TypeFlags::NonPrimitive
                | TypeFlags::Index,
        ) {
            return left;
        }
        if self.type_flags_is_generic_object_type(&left)
            || self.type_flags_is_generic_object_type(&right)
        {
            if self.type_is_empty_object(&left) {
                return right;
            }
            return self.get_intersection_type(vec![left, right]);
        }

        let mut members = SymbolTable::new();
        let mut skipped_private: Vec<String> = Vec::new();
        let index_infos = if self.type_is_empty_object(&left) {
            self.get_index_infos_of_type(&right)
        } else {
            self.get_union_index_infos(&left, &right)
        };
        for right_prop in self.get_properties_of_type(&right) {
            let mods = crate::checker::exports::get_declaration_modifier_flags_from_symbol(
                &right_prop,
            );
            if mods.intersects(
                tsox_frontend::ast::ModifierFlags::Private
                    | tsox_frontend::ast::ModifierFlags::Protected,
            ) {
                skipped_private.push(right_prop.name.clone());
            } else if self.is_spreadable_property(&right_prop) {
                let spread_symbol = self.get_spread_symbol(&right_prop, readonly);
                members.insert(right_prop.name.clone(), spread_symbol);
            }
        }
        for left_prop in self.get_properties_of_type(&left) {
            if skipped_private.contains(&left_prop.name)
                || !self.is_spreadable_property(&left_prop)
            {
                continue;
            }
            if let Some(right_prop) = members.get(&left_prop.name).cloned() {
                if right_prop.flags.contains(SymbolFlags::Optional) {
                    let left_type = self.get_type_of_symbol(&left_prop);
                    let right_type = self.get_type_of_symbol(&right_prop);
                    let left_wo = self.remove_missing_or_undefined_type(&left_type);
                    let right_wo = self.remove_missing_or_undefined_type(&right_type);
                    let resolved = if Arc::ptr_eq(&left_wo, &right_wo) {
                        left_type
                    } else {
                        self.get_union_type_ex(
                            vec![left_type, right_wo],
                            crate::checker::exports_union_reduction::UnionReduction::Subtype,
                        )
                    };
                    let mut sym = Symbol::new(
                        SymbolFlags::Property.union(left_prop.flags.intersection(SymbolFlags::Optional)),
                        left_prop.name.clone(),
                    );
                    sym.declarations = left_prop
                        .declarations
                        .iter()
                        .chain(right_prop.declarations.iter())
                        .cloned()
                        .collect();
                    let symbol = Arc::new(sym);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(resolved),
                            name_type: self
                                .value_symbol_links
                                .get(&left_prop)
                                .and_then(|l| l.name_type.clone()),
                            ..Default::default()
                        },
                    );
                    members.insert(left_prop.name.clone(), symbol);
                }
            } else {
                let spread_symbol = self.get_spread_symbol(&left_prop, readonly);
                members.insert(left_prop.name.clone(), spread_symbol);
            }
        }
        let spread_index_infos = index_infos
            .iter()
            .map(|info| self.get_index_info_with_readonly(info, readonly))
            .collect();
        let mut props: Vec<Arc<Symbol>> = members.entries.values().cloned().collect();
        props.sort_by(|a, b| a.name.cmp(&b.name));
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral
                | ObjectFlags::ContainsSpread
                | object_flags,
            id: next_type_id(),
            symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    index_infos: spread_index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn fresh_empty_object_type(&mut self) -> Arc<Type> {
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::None,
            id: next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                ..Default::default()
            }),
        })
    }

    pub(crate) fn fold_object_literal_spread(
        &mut self,
        prop_pairs: &mut Vec<(String, Arc<Type>, Vec<Arc<Node>>)>,
        spread_acc: &mut Option<Arc<Type>>,
        expression: &Arc<Node>,
        prop: &Arc<Node>,
        literal_symbol: Option<Arc<Symbol>>,
    ) -> bool {
        if !prop_pairs.is_empty() {
            let segment =
                self.object_literal_type_from_pairs(std::mem::take(prop_pairs), literal_symbol.clone());
            let base = spread_acc
                .clone()
                .unwrap_or_else(|| self.fresh_empty_object_type());
            *spread_acc = Some(self.get_spread_type(
                &base,
                &segment,
                literal_symbol.clone(),
                ObjectFlags::None,
                false,
            ));
        }
        let expr_type = self.get_type_of_node(expression);
        if !self.is_valid_spread_type(&expr_type) {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                prop.loc,
                tsox_core::diagnostics::messages_generated::
                    SPREAD_TYPES_MAY_ONLY_BE_CREATED_FROM_OBJECT_TYPES,
                vec![],
            ));
            return false;
        }
        let base = spread_acc
            .clone()
            .unwrap_or_else(|| self.fresh_empty_object_type());
        if crate::checker::utilities::is_type_error(&base) {
            return true;
        }
        *spread_acc = Some(self.get_spread_type(
            &base,
            &expr_type,
            literal_symbol,
            ObjectFlags::None,
            false,
        ));
        true
    }

    fn union_members(&self, t: &Arc<Type>) -> Option<Vec<Arc<Type>>> {
        t.as_union_or_intersection()
            .filter(|_| t.flags.contains(TypeFlags::Union))
            .map(|ui| ui.types.clone())
    }

    fn spread_cross_product_ok(&self, left: &Arc<Type>, right: &Arc<Type>) -> bool {
        let mut size: usize = 1;
        for t in [left, right] {
            if t.flags.contains(TypeFlags::Union)
                && let Some(members) = self.union_members(t)
            {
                size = size.saturating_mul(members.len());
            } else if t.flags.contains(TypeFlags::Never) {
                return true;
            }
            if size >= 100_000 {
                return false;
            }
        }
        true
    }

    fn try_merge_union_of_object_type_and_empty_object(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if !t.flags.contains(TypeFlags::Union) {
            return Arc::clone(t);
        }
        let Some(members) = self.union_members(t) else {
            return Arc::clone(t);
        };
        if members
            .iter()
            .all(|m| self.is_empty_object_type_or_spreads_into_empty_object(m))
        {
            return match members.iter().find(|m| self.type_is_empty_object(m)) {
                Some(empty) => Arc::clone(empty),
                None => Arc::clone(t),
            };
        }
        let first = members
            .iter()
            .find(|m| !self.is_empty_object_type_or_spreads_into_empty_object(m));
        let Some(first) = first else {
            return Arc::clone(t);
        };
        let second = members
            .iter()
            .find(|m| !Arc::ptr_eq(m, first) && !self.is_empty_object_type_or_spreads_into_empty_object(m));
        if second.is_some() {
            return Arc::clone(t);
        }
        let mut spread_members = SymbolTable::new();
        for prop in self.get_properties_of_type(first) {
            let mut sym = Symbol::new(
                SymbolFlags::Property
                    .union(prop.flags.intersection(SymbolFlags::Optional)),
                prop.name.clone(),
            );
            sym.declarations = prop.declarations.clone();
            let symbol = Arc::new(sym);
            let prop_type = self.get_type_of_symbol(&prop);
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            spread_members.insert(prop.name.clone(), symbol);
        }
        let mut props: Vec<Arc<Symbol>> = spread_members.entries.values().cloned().collect();
        props.sort_by(|a, b| a.name.cmp(&b.name));
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: spread_members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn is_empty_object_type_or_spreads_into_empty_object(&self, t: &Arc<Type>) -> bool {
        self.type_is_empty_object(t)
            || t.flags.intersects(
                TypeFlags::Null
                    | TypeFlags::Undefined
                    | TypeFlags::Boolean
                    | TypeFlags::BooleanLiteral
                    | TypeFlags::Number
                    | TypeFlags::NumberLiteral
                    | TypeFlags::BigInt
                    | TypeFlags::BigIntLiteral
                    | TypeFlags::String
                    | TypeFlags::StringLiteral
                    | TYPE_FLAGS_ENUM_LIKE
                    | TypeFlags::NonPrimitive
                    | TypeFlags::Index,
            )
    }

    pub(crate) fn type_is_empty_object(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) {
            return false;
        }
        t.as_structured().is_some_and(|s| {
            s.properties.is_empty()
                && s.call_signatures().is_empty()
                && s.construct_signatures().is_empty()
                && s.index_infos.is_empty()
        })
    }


    fn get_spread_symbol(&mut self, prop: &Arc<Symbol>, readonly: bool) -> Arc<Symbol> {
        let is_setonly_accessor = prop.flags.contains(SymbolFlags::SetAccessor)
            && !prop.flags.contains(SymbolFlags::GetAccessor);
        if !is_setonly_accessor && readonly == self.is_readonly_symbol_for_identity(prop) {
            return Arc::clone(prop);
        }
        let mut sym = Symbol::new(
            SymbolFlags::Property.union(prop.flags.intersection(SymbolFlags::Optional)),
            prop.name.clone(),
        );
        if readonly {
            sym.check_flags |= tsox_frontend::ast::CheckFlags::Readonly;
        }
        sym.declarations = prop.declarations.clone();
        let symbol = Arc::new(sym);
        let resolved = if is_setonly_accessor {
            self.undefined_type()
        } else {
            self.get_type_of_symbol(prop)
        };
        self.value_symbol_links.insert(
            &symbol,
            ValueSymbolLinks {
                resolved_type: Some(resolved),
                name_type: self
                    .value_symbol_links
                    .get(prop)
                    .and_then(|l| l.name_type.clone()),
                ..Default::default()
            },
        );
        symbol
    }

    fn get_index_info_with_readonly(
        &mut self,
        info: &Arc<IndexInfo>,
        readonly: bool,
    ) -> Arc<IndexInfo> {
        if info.is_readonly != readonly {
            return Arc::new(IndexInfo {
                key_type: info.key_type.clone(),
                value_type: info.value_type.clone(),
                is_readonly: readonly,
                declaration: info.declaration.clone(),
                index_symbol: info.index_symbol.clone(),
                components: info.components.clone(),
            });
        }
        Arc::clone(info)
    }

    fn get_union_index_infos(
        &mut self,
        left: &Arc<Type>,
        right: &Arc<Type>,
    ) -> Vec<Arc<IndexInfo>> {
        let source_infos = self.get_index_infos_of_type(left);
        let mut result = Vec::new();
        for info in &source_infos {
            let Some(key_type) = &info.key_type else {
                continue;
            };
            let right_info = self.get_index_info_of_type(right, key_type);
            if right_info.is_none() {
                continue;
            }
            let left_value = self.get_index_info_of_type(left, key_type);
            let mut values = Vec::new();
            for side in [left_value, right_info] {
                if let Some(i) = side
                    && let Some(v) = &i.value_type
                {
                    values.push(Arc::clone(v));
                }
            }
            let value_type = if values.is_empty() {
                None
            } else {
                Some(self.get_union_type(values))
            };
            let is_readonly = [left, right].iter().any(|t| {
                self.get_index_info_of_type(t, key_type)
                    .is_some_and(|i| i.is_readonly)
            });
            result.push(Arc::new(IndexInfo {
                key_type: Some(Arc::clone(key_type)),
                value_type,
                is_readonly,
                declaration: None,
                index_symbol: None,
                components: Vec::new(),
            }));
        }
        result
    }

    pub(crate) fn is_valid_spread_type(&mut self, t: &Arc<Type>) -> bool {
        let base = self.get_base_constraint_or_type(t);
        if base.flags.contains(TypeFlags::Union) {
            let Some(ui) = base.as_union_or_intersection() else {
                return false;
            };
            let non_falsy: Vec<Arc<Type>> = ui
                .types
                .iter()
                .filter(|m| !type_is_definitely_falsy(m))
                .cloned()
                .collect();
            if non_falsy.is_empty() {
                return self.spread_type_flags_ok(&self.never_type());
            }
            for m in &non_falsy {
                let mapped = self.get_base_constraint_or_type(m);
                if !self.spread_type_flags_ok(&mapped)
                    && !(mapped.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                        && self.every_member_valid_spread(&mapped))
                {
                    return false;
                }
            }
            return true;
        }
        self.spread_type_flags_ok(&base)
            || (base.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                && self.every_member_valid_spread(&base))
    }

    fn every_member_valid_spread(&mut self, t: &Arc<Type>) -> bool {
        t.as_union_or_intersection()
            .map(|ui| {
                ui.types
                    .iter()
                    .all(|m| self.is_valid_spread_type(m))
            })
            .unwrap_or(false)
    }

    fn spread_type_flags_ok(&self, t: &Arc<Type>) -> bool {
        t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::NonPrimitive
                | TypeFlags::Object
                | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE,
        )
    }

    pub(crate) fn object_literal_type_from_pairs(
        &mut self,
        prop_pairs: Vec<(String, Arc<Type>, Vec<Arc<Node>>)>,
        literal_symbol: Option<Arc<Symbol>>,
    ) -> Arc<Type> {
        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t, decls) in prop_pairs {
            let mut sym = Symbol::new(SymbolFlags::Property, name.clone());
            sym.value_declaration = decls.first().cloned();
            sym.declarations.extend(decls);
            let symbol = Arc::new(sym);
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            members.insert(name, Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::FreshLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral,
            id: next_type_id(),
            symbol: literal_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }
}

fn type_is_definitely_falsy(t: &Arc<Type>) -> bool {
    t.flags.intersects(
        TypeFlags::BooleanLiteral
            | TypeFlags::StringLiteral
            | TypeFlags::NumberLiteral
            | TypeFlags::BigIntLiteral
            | TypeFlags::Void
            | TypeFlags::Undefined
            | TypeFlags::Null,
    )
}
