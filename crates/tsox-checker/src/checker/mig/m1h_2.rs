#![allow(unused_imports)]
#![allow(dead_code)]
use super::wc1b::every_type;
use crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type;
use crate::checker::nodecopy_property_name::is_numeric_literal_name;
use super::m1b::jsnum_from_string;
use crate::checker::mapper::new_simple_type_mapper;
use crate::checker::utilities_has_only_expression_initialization::compare_types;
use crate::checker::string_mapping_checker::string_mapping_kind;
use tsox_core::diagnostics;
use super::m1h::r22k7_defs::*;

use crate::checker::checker::*;
use std::sync::Arc;

pub fn index_type_less_than(index_type: &Arc<Type>, limit: i32) -> bool {
    every_type(index_type, &|t: &Arc<Type>| {
        if t.flags.intersects(TYPE_FLAGS_STRING_OR_NUMBER_LITERAL) {
            let prop_name = get_property_name_from_type(t);
            if is_numeric_literal_name(&prop_name) {
                let index = jsnum_from_string(&prop_name);
                return index.0 >= 0.0 && index.0 < limit as f64;
            }
        }
        false
    })
}

pub fn insert_type(types: &[Arc<Type>], t: &Arc<Type>) -> (Vec<Arc<Type>>, bool) {
    match types.binary_search_by(|probe| compare_types(probe, t)) {
        Err(i) => {
            let mut result = types.to_vec();
            result.insert(i, Arc::clone(t));
            (result, true)
        }
        Ok(_) => (types.to_vec(), false),
    }
}

impl Checker {
    pub fn instantiate_types(
        &mut self,
        types: &[Arc<Type>],
        m: Option<&Arc<TypeMapper>>,
    ) -> Vec<Arc<Type>> {
        self.instantiate_list(types, m, |c, t, m| c.instantiate_type(t, m), |a, b| {
            Arc::ptr_eq(a, b)
        })
    }

    pub fn instantiate_symbols(
        &mut self,
        symbols: &[Arc<Symbol>],
        m: Option<&Arc<TypeMapper>>,
    ) -> Vec<Arc<Symbol>> {
        let mut result: Vec<Arc<Symbol>> = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            if let Some(instantiated) = self.instantiate_symbol(symbol, m) {
                result.push(instantiated);
            }
        }
        result
    }

    pub fn instantiate_signatures(
        &mut self,
        signatures: &[Arc<Signature>],
        m: Option<&Arc<TypeMapper>>,
    ) -> Vec<Arc<Signature>> {
        self.instantiate_list(
            signatures,
            m,
            |c, s, m| c.instantiate_signature(s, m),
            |a: &Arc<Signature>, b: &Arc<Signature>| Arc::ptr_eq(a, b),
        )
    }

    pub fn instantiate_index_infos(
        &mut self,
        index_infos: &[Arc<IndexInfo>],
        m: Option<&Arc<TypeMapper>>,
    ) -> Vec<Arc<IndexInfo>> {
        self.instantiate_list(
            index_infos,
            m,
            |c, info, m| c.instantiate_index_info(info, m),
            |a: &Arc<IndexInfo>, b: &Arc<IndexInfo>| Arc::ptr_eq(a, b),
        )
    }

    fn instantiate_list<T: Clone>(
        &mut self,
        values: &[T],
        m: Option<&Arc<TypeMapper>>,
        mut instantiator: impl FnMut(&mut Checker, &T, Option<&Arc<TypeMapper>>) -> T,
        same: impl Fn(&T, &T) -> bool,
    ) -> Vec<T> {
        for (i, value) in values.iter().enumerate() {
            let mapped = instantiator(self, value, m);
            if !same(&mapped, value) {
                let mut result: Vec<T> = Vec::with_capacity(values.len());
                result.extend_from_slice(&values[..i]);
                result.push(mapped);
                for v in &values[i + 1..] {
                    result.push(instantiator(self, v, m));
                }
                return result;
            }
        }
        values.to_vec()
    }

    pub fn instantiate_type_alias(
        &mut self,
        alias: Option<&TypeAlias>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Option<TypeAlias> {
        let alias = alias?;
        Some(TypeAlias {
            symbol: alias.symbol.clone(),
            type_arguments: self.instantiate_types(&alias.type_arguments, m),
        })
    }

    pub fn instantiate_type_with_alias(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> {
        let alias_has_type_variables = t.alias.as_ref().is_some_and(|a| {
            !a.type_arguments.is_empty()
                && a.type_arguments
                    .iter()
                    .any(|ta| self.could_contain_type_variables(ta))
        });
        let m = match m {
            Some(m) if self.could_contain_type_variables(t) || alias_has_type_variables => m,
            _ => return Arc::clone(t),
        };
        if self.instantiation_depth == 100 || self.instantiation_count >= 5_000_000 {
            if let Some(node) = self.current_node.clone() {
                self.error_message(
                    &node,diagnostics::TYPE_INSTANTIATION_IS_EXCESSIVELY_DEEP_AND_POSSIBLY_INFINITE,
                    &[],
                );
            }
            return Arc::clone(&self.error_type());
        }
        let index = self.find_active_mapper(m);
        let cache_index = match index {
            Some(i) => i,
            None => {
                self.push_active_mapper(Arc::clone(m));
                self.active_type_mappers_caches.len() - 1
            }
        };
        let mut b = R22KeyBuilder::new();
        b.write_type(t);
        b.write_alias(alias);
        let key = b.hash();
        if let Some(cached_type) = self.active_type_mappers_caches[cache_index].get(&key) {
            return Arc::clone(cached_type);
        }
        self.total_instantiation_count += 1;
        self.instantiation_count += 1;
        self.instantiation_depth += 1;
        let result = self.instantiate_type_worker(t, Some(m), alias);
        match index {
            Some(i) => {
                self.active_type_mappers_caches[i].insert(key, Arc::clone(&result));
            }
            None => self.pop_active_mapper(),
        }
        self.instantiation_depth -= 1;
        result
    }

    pub fn instantiate_type_worker(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> {
        let flags = t.flags;
        if flags.intersects(TypeFlags::TypeParameter) {
            return m.unwrap().map(t);
        }
        if flags.intersects(TypeFlags::Object) {
            let object_flags = t.object_flags;
            if object_flags.intersects(
                ObjectFlags::Reference | ObjectFlags::Anonymous | ObjectFlags::Mapped,
            ) {
                if object_flags.intersects(ObjectFlags::Reference) {
                    if let Some(d) = t.as_type_reference() {
                        let resolved_type_arguments = d.type_arguments.clone();
                        let new_type_arguments =
                            self.instantiate_types(&resolved_type_arguments, m);
                        if same_types(&new_type_arguments, &resolved_type_arguments) {
                            return Arc::clone(t);
                        }
                        if let Some(target) = t.target() {
                            return self
                                .create_normalized_type_reference(target, &new_type_arguments);
                        }
                        return Arc::clone(t);
                    }
                }
                if object_flags.intersects(ObjectFlags::ReverseMapped) {
                    return self.instantiate_reverse_mapped_type(t, m);
                }
                return self.get_object_type_instantiation(t, m.map(Arc::as_ref), alias);
            }
            return Arc::clone(t);
        }
        if flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION) {
            let mut source = t;
            if flags.intersects(TypeFlags::Union) {
                if let Some(origin) = t.as_union_type().and_then(|d| d.origin.as_ref()) {
                    if origin.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION) {
                        source = origin;
                    }
                }
            }
            let types: Vec<Arc<Type>> = source.types().map(|ts| ts.to_vec()).unwrap_or_default();
            let new_types = self.instantiate_types(&types, m);
            if same_types(&new_types, &types)
                && type_alias_symbol_eq(alias, t.alias.as_deref())
            {
                return Arc::clone(t);
            }
            let alias = match alias {
                Some(alias) => Some(alias.clone()),
                None => self.instantiate_type_alias(t.alias.as_deref(), m),
            };
            if source.flags.intersects(TypeFlags::Intersection) {
                return self.get_intersection_type_ex(
                    &new_types,
                    IntersectionFlags::None,
                    alias.as_ref(),
                );
            }
            return self.get_union_type_ex(new_types, UnionReduction::Literal);
        }
        if flags.intersects(TypeFlags::Index) {
            if let Some(target) = t.target() {
                let instantiated = self.instantiate_type(target, m);
                return self.get_index_type(&instantiated);
            }
            return Arc::clone(t);
        }
        if flags.intersects(TypeFlags::IndexedAccess) {
            let alias = match alias {
                Some(alias) => Some(alias.clone()),
                None => self.instantiate_type_alias(t.alias.as_deref(), m),
            };
            if let Some(d) = t.as_indexed_access_type() {
                if let (Some(object_type), Some(index_type)) =
                    (d.object_type.clone(), d.index_type.clone())
                {
                    let new_object_type = self.instantiate_type(&object_type, m);
                    let new_index_type = self.instantiate_type(&index_type, m);
                    return self.get_indexed_access_type_ex(
                        &new_object_type,
                        &new_index_type,
                        d.access_flags,
                        None,
                        alias.as_ref(),
                    );
                }
            }
            return Arc::clone(t);
        } else if flags.intersects(TypeFlags::TemplateLiteral) {
            if let Some(d) = t.as_template_literal_type() {
                let new_types = self.instantiate_types(&d.types, m);
                return self.get_template_literal_type(&d.texts, &new_types);
            }
            Arc::clone(t)
        } else if flags.intersects(TypeFlags::StringMapping) {
            if let (Some(symbol), Some(d)) = (t.symbol.clone(), t.as_string_mapping_type()) {
                if let (Some(kind), Some(target)) =
                    (string_mapping_kind(&symbol.name), d.target.clone())
                {
                    let new_target = self.instantiate_type(&target, m);
                    return self.get_string_mapping_type(kind, Some(symbol), &new_target);
                }
            }
            Arc::clone(t)
        } else if flags.intersects(TypeFlags::Conditional) {
            let combined = self
                .combine_type_mappers(
                    t.as_conditional_type().and_then(|d| d.mapper.as_ref()),
                    m,
                )
                .unwrap();
            self.get_conditional_type_instantiation(t, Some(&combined), false, alias)
        } else if flags.intersects(TypeFlags::Substitution) {
            if let Some(d) = t.as_substitution_type() {
                if let (Some(base_type), Some(constraint)) =
                    (d.base_type.clone(), d.constraint.clone())
                {
                    let new_base_type = self.instantiate_type(&base_type, m);
                    if self.is_no_infer_type(t) {
                        return self.get_no_infer_type(&new_base_type);
                    }
                    let new_constraint = self.instantiate_type(&constraint, m);
                    if new_base_type.flags.intersects(TYPE_FLAGS_TYPE_VARIABLE)
                        && self.is_generic_type(&new_constraint)
                    {
                        return self.get_substitution_type(&new_base_type, &new_constraint);
                    }
                    let restrictive_base = self.get_restrictive_instantiation(&new_base_type);
                    let restrictive_constraint = self.get_restrictive_instantiation(&new_constraint);
                    if new_constraint.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN)
                        || self.is_type_assignable_to(&restrictive_base, &restrictive_constraint)
                    {
                        return new_base_type;
                    }
                    if new_base_type.flags.intersects(TYPE_FLAGS_TYPE_VARIABLE) {
                        return self.get_substitution_type(&new_base_type, &new_constraint);
                    }
                    return self.get_intersection_type(vec![new_constraint, new_base_type]);
                }
            }
            Arc::clone(t)
        } else {
            Arc::clone(t)
        }
    }

    pub fn instantiate_anonymous_type(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> {
        let mut object_flags = t.object_flags
            & !(ObjectFlags::CouldContainTypeVariablesComputed
                | ObjectFlags::CouldContainTypeVariables)
            | ObjectFlags::Instantiated;
        let mut m = m.map(Arc::clone);
        let mut result = self.new_object_type(object_flags, t.symbol.clone());
        if t.object_flags.intersects(ObjectFlags::Mapped) {
            self.set_mapped_declaration(&mut result, self.get_mapped_declaration(t));
            if let Some(orig_type_parameter) = self.get_type_parameter_from_mapped_type(t) {
                let mut fresh_type_parameter = self.clone_type_parameter(&orig_type_parameter);
                let combined = self.combine_type_mappers(
                    Some(&Arc::new(new_simple_type_mapper(
                        Arc::clone(&orig_type_parameter),
                        Arc::clone(&fresh_type_parameter),
                    ))),
                    m.as_ref(),
                );
                m = combined;
                if let Some(mm) = m.as_ref() {
                    if let Some(fresh_mut) = Arc::get_mut(&mut fresh_type_parameter) {
                        fresh_mut.set_type_parameter_mapper(mm);
                    }
                }
                self.set_mapped_type_parameter(&mut result, &fresh_type_parameter);
            }
        } else if t.object_flags.intersects(ObjectFlags::InstantiationExpressionType) {
            let node = t
                .as_instantiation_expression_type()
                .and_then(|d| d.node.clone());
            self.set_instantiation_expression_node(&mut result, node);
        }
        let alias = match alias {
            Some(alias) => Some(alias.clone()),
            None => self.instantiate_type_alias(t.alias.as_deref(), m.as_ref()),
        };
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.alias = alias.as_ref().map(|a| Box::new(a.clone()));
        }
        if let Some(alias) = &alias {
            if !alias.type_arguments.is_empty() {
                object_flags |= self
                    .get_propagating_flags_of_types(&alias.type_arguments, TypeFlags::None);
                set_object_flags(&mut result, object_flags);
            }
        }
        self.set_anonymous_target_and_mapper(&mut result, t, m.as_ref());
        result
    }
}
