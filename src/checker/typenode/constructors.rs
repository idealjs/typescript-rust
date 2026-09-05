use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    Node, Symbol, SymbolFlags,
    SyntaxKind,
};

use crate::checker::checker::Checker;


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
                if t.flags
                    .intersects(TypeFlags::EnumLiteral | TypeFlags::Enum)
                {
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

        let array_symbol = self.globals.get("Array").cloned();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference,
            id: 0,
            symbol: array_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: None,
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

        let is_evolving = obj_type
            .object_flags
            .contains(ObjectFlags::EvolvingArray);
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
            TypeData::EvolvingArray(e) => e
                .element_type
                .clone()
                .unwrap_or_else(|| self.never_type()),
            _ => return None,
        };

        let declared = match self
            .globals
            .get("Array")
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            }) {
            Some(d) => Some(d),

            None => self.globals.get("Array").cloned().map(|sym| {
                self.resolve_interface_type(&sym, None)
            }),
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
        let substituted =
            self.substitute_infer_type_parameters(&raw, &subst_tps, &substitutions);
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

        if self.instantiated_member_type_cache.len() >= self.instantiated_member_type_cache_limit
        {
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

    pub(crate) fn collect_free_type_parameters_deep(&mut self, t: &Arc<Type>, out: &mut Vec<Arc<Type>>) {
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
            id: 0,
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
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
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
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
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
                        crate::ast::NodeData::MappedTypeNode(d) => d.type_node.as_ref().map(
                            |tn| {
                                (
                                    Arc::clone(&d.type_parameter),
                                    Arc::clone(tn),
                                    Arc::clone(decl),
                                )
                            },
                        ),
                        _ => None,
                    })
                    .and_then(|(tp_node, template_node, decl)| {
                        let tp_sym = self
                            .program
                            .symbol_map()
                            .symbol_of(&tp_node)
                            .cloned()?;

                        if !Self::type_node_references_name(
                            &template_node,
                            &tp_sym.name,
                        ) {
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
