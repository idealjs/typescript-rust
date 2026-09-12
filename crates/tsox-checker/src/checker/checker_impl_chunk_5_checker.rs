#![allow(unused_imports)]

use crate::checker::checker_impl_chunk_5::*;

impl Checker {
    pub fn get_string_type(&self) -> Arc<Type> {
        self.string_type()
    }
    pub fn get_number_type(&self) -> Arc<Type> {
        self.number_type()
    }
    pub fn get_boolean_type(&self) -> Arc<Type> {
        self.boolean_type()
    }
    pub fn get_void_type(&self) -> Arc<Type> {
        self.void_type()
    }
    pub fn get_undefined_type(&self) -> Arc<Type> {
        self.undefined_type()
    }
    pub fn get_null_type(&self) -> Arc<Type> {
        self.null_type()
    }
    pub fn get_any_type(&self) -> Arc<Type> {
        self.any_type()
    }
    pub fn get_error_type(&self) -> Arc<Type> {
        self.error_type()
    }
    pub fn get_never_type(&self) -> Arc<Type> {
        self.never_type()
    }
    pub fn get_unknown_type(&self) -> Arc<Type> {
        self.unknown_type()
    }
    pub fn get_bigint_type(&self) -> Arc<Type> {
        self.bigint_type()
    }
    pub fn get_es_symbol_type(&self) -> Arc<Type> {
        self.es_symbol_type()
    }

    pub fn get_unknown_symbol(&self) -> Option<Arc<Symbol>> {
        self.unknown_symbol.clone()
    }
    pub fn get_undefined_symbol(&self) -> Option<Arc<Symbol>> {
        self.undefined_symbol.clone()
    }
    pub fn get_arguments_symbol(&self) -> Option<Arc<Symbol>> {
        self.arguments_symbol.clone()
    }

    pub fn get_properties_of_type(&self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        // Go getPropertiesOfUnionOrIntersectionType：联合=各成员共有属性
        // （never 不参与，`never | {x}` 出 x）
        if t.is_union()
            && let Some(members) = t.types()
        {
            let relevant: Vec<&Arc<Type>> = members
                .iter()
                .filter(|m| !m.flags.contains(TypeFlags::Never))
                .collect();
            if relevant.is_empty() {
                return Vec::new();
            }
            let mut props = self.get_properties_of_type(relevant[0]);
            for m in &relevant[1..] {
                if props.is_empty() {
                    break;
                }
                let others = self.get_properties_of_type(m);
                props.retain(|p| others.iter().any(|o| o.name == p.name));
            }
            return props;
        }
        // Go getPropertiesOfType：交集成员属性合并（同名属性取首个声明，
        // 完整交集属性合成走 getUnionOrIntersectionProperty）
        if t.is_intersection()
            && let Some(members) = t.types()
        {
            let mut merged: Vec<Arc<Symbol>> = Vec::new();
            for m in members {
                for p in self.get_properties_of_type(m) {
                    if !merged.iter().any(|x| x.name == p.name) {
                        merged.push(Arc::clone(&p));
                    }
                }
            }
            if !merged.is_empty() {
                return merged;
            }
        }
        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty() {
                return structured.properties.clone();
            }
            // 空壳再水化：递归接口构建窗口内嵌出的 shell 无成员，经符号的
            // 完整声明型回取（Go deferred 引用的查询期等价）
            if let Some(sym) = &t.symbol
                && sym.flags.intersects(
                    SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeLiteral,
                )
                && let Some(declared) = self
                    .type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
                && !Arc::ptr_eq(&declared, t)
                && !crate::checker::utilities::is_type_error(&declared)
                && let Some(ds) = declared.as_structured()
                && !ds.properties.is_empty()
            {
                return ds.properties.clone();
            }
            return structured.properties.clone();
        }
        Vec::new()
    }

    pub fn get_signatures_of_type(
        &self,
        t: &Arc<Type>,
        kind: SignatureKind,
    ) -> Vec<Arc<Signature>> {
        if let Some(structured) = t.as_structured() {
            return match kind {
                SignatureKind::Call => structured.call_signatures().to_vec(),
                SignatureKind::Construct => structured.construct_signatures().to_vec(),
            };
        }
        Vec::new()
    }

    pub fn type_has_call_or_construct_signatures(&self, t: &Arc<Type>) -> bool {
        if let Some(structured) = t.as_structured() {
            return !structured.signatures.is_empty();
        }
        false
    }

    pub fn is_array_like_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Object) {
            if let Some(structured) = t.as_structured() {
                for info in &structured.index_infos {
                    if info
                        .key_type
                        .as_ref()
                        .map(|kt| kt.flags.contains(TypeFlags::Number))
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }

                return t.object_flags.contains(ObjectFlags::Tuple);
            }
        }
        false
    }

    pub fn is_array_type(&self, t: &Arc<Type>) -> bool {
        t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Reference)
            && self
                .globals
                .get("Array")
                .zip(t.symbol.as_ref())
                .is_some_and(|(array_sym, sym)| Arc::ptr_eq(array_sym, sym))
    }

    pub fn is_tuple_type(&self, t: &Arc<Type>) -> bool {
        crate::checker::utilities::is_tuple_type(t)
    }

    pub fn get_base_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::EnumLiteral) {
            if let Some(sym) = &t.symbol
                && sym.flags.contains(SymbolFlags::EnumMember)
                && let Some(parent) = &sym.parent()
                && let Some(cached) = self
                    .type_alias_links
                    .get(parent)
                    .and_then(|l| l.declared_type.clone())
            {
                return cached;
            }
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            return self.string_type();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            return self.number_type();
        }
        if t.flags.contains(TypeFlags::BigIntLiteral) {
            return self.bigint_type();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            return self.boolean_type();
        }

        if let TypeData::Union(u) = &t.data {
            let widened: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .map(|m| self.get_base_type_of_literal_type(m))
                .collect();
            if widened.len() == 1 {
                return Arc::clone(&widened[0]);
            }
            if widened
                .iter()
                .zip(u.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            if let Some(first) = widened.first() {
                if widened.iter().all(|w| Arc::ptr_eq(w, first)) {
                    return Arc::clone(first);
                }
            }

            return Arc::new(Type {
                flags: TypeFlags::Union,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: None,
                alias: None,
                data: TypeData::Union(UnionTypeData {
                    union_or_intersection: UnionOrIntersectionTypeData {
                        structured: StructuredTypeData::default(),
                        types: widened,
                    },
                    resolved_reduced_type: std::sync::OnceLock::new(),
                    regular_type: std::sync::OnceLock::new(),
                    origin: None,
                    key_property_name: None,
                    constituent_map: HashMap::new(),
                }),
            });
        }
        Arc::clone(t)
    }

    pub fn get_widened_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_NULLABLE)
            && t.object_flags
                .intersects(crate::checker::types::OBJECT_FLAGS_REQUIRES_WIDENING)
        {
            return self.get_any_type();
        }

        if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
            return Arc::clone(t);
        }

        if t.flags.intersects(TYPE_FLAGS_LITERAL) {
            if crate::checker::is_fresh_literal_type(t) {
                return self.get_base_type_of_literal_type(t);
            }
            return Arc::clone(t);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            return self.es_symbol_type();
        }

        // Go getWidenedTypeWithContext：对象字面量含 fresh literal / nullable 属性时逐属性 widen
        // （如作为函数返回类型的 { myProp: "test" } → { myProp: string }）
        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::ObjectLiteral)
        {
            if let Some(widened) = self.widen_object_literal_properties(t) {
                return widened;
            }
        }

        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_type(member))
                .collect();

            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }

            return self.build_union_from_types(widened);
        }

        // Go getWidenedTypeWithContext：数组/元组逐类型实参 widen（"s"[] → string[]）
        if t.object_flags.contains(ObjectFlags::Reference)
            && let Some(obj) = t.as_object()
            && !obj.type_arguments.is_empty()
        {
            let widened: Vec<Arc<Type>> = obj
                .type_arguments
                .iter()
                .map(|a| self.get_widened_type(a))
                .collect();
            let unchanged = widened
                .iter()
                .zip(obj.type_arguments.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o));
            if !unchanged {
                return crate::checker::checker_attach_explicit_type_arguments::attach_explicit_type_arguments(t, widened);
            }
        }
        Arc::clone(t)
    }

    // 逐属性 widen 对象字面量；无可 widen 属性时返回 None（保持原类型）
    fn widen_object_literal_properties(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let obj = match &t.data {
            TypeData::Object(o) => o,
            _ => return None,
        };
        let needs_widen = |ty: &Arc<Type>| {
            crate::checker::is_fresh_literal_type(ty)
                || ty.flags.intersects(
                    TypeFlags::Null | TypeFlags::Undefined | crate::checker::types::TYPE_FLAGS_NULLABLE,
                )
        };
        let mut changed = false;
        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(obj.structured.properties.len());
        for prop in &obj.structured.properties {
            let old_type = match self.value_symbol_links.get(prop).and_then(|l| l.resolved_type.clone()) {
                Some(ty) => ty,
                None => {
                    members.insert(prop.name.clone(), Arc::clone(prop));
                    props.push(Arc::clone(prop));
                    continue;
                }
            };
            let new_type = match &old_type.data {
                TypeData::Union(u) => {
                    let parts: Vec<Arc<Type>> = u
                        .union_or_intersection
                        .types
                        .iter()
                        .map(|c| {
                            if needs_widen(c) {
                                if c.flags.intersects(TYPE_FLAGS_NULLABLE)
                                    && !crate::checker::is_fresh_literal_type(c)
                                {
                                    self.get_any_type()
                                } else {
                                    self.get_widened_type(c)
                                }
                            } else {
                                Arc::clone(c)
                            }
                        })
                        .collect();
                    let u2 = self.build_union_from_types(parts);
                    if self.type_to_string(&u2) != self.type_to_string(&old_type) {
                        changed = true;
                    }
                    u2
                }
                _ if needs_widen(&old_type) => {
                    changed = true;
                    if old_type.flags.intersects(TYPE_FLAGS_NULLABLE)
                        && !crate::checker::is_fresh_literal_type(&old_type)
                    {
                        self.get_any_type()
                    } else {
                        self.get_widened_type(&old_type)
                    }
                }
                _ => Arc::clone(&old_type),
            };
            let new_prop = Arc::new(Symbol::new(prop.flags, prop.name.clone()));
            {
                let np = Arc::as_ptr(&new_prop) as *mut Symbol;
                unsafe {
                    (*np).check_flags = prop.check_flags;
                }
            }
            self.value_symbol_links.insert(
                &new_prop,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(new_type),
                    ..Default::default()
                },
            );
            members.insert(new_prop.name.clone(), Arc::clone(&new_prop));
            props.push(new_prop);
        }
        if !changed {
            return None;
        }
        Some(Arc::new(Type {
            flags: t.flags,
            object_flags: t.object_flags,
            id: crate::checker::types::next_type_id(),
            symbol: t.symbol.clone(),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        }))
    }

    pub fn widen_initializer_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if crate::checker::is_object_literal_type(t) {
            return self.widen_object_literal_type(t);
        }

        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return Arc::clone(t);
        }

        if self.is_auto_array_type(t) {
            return self.get_evolving_array_type(self.never_type());
        }

        self.get_widened_type(t)
    }

    pub fn is_auto_array_type(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) || !t.object_flags.contains(ObjectFlags::Reference)
        {
            return false;
        }

        match &t.data {
            TypeData::Object(obj) => obj
                .type_arguments
                .first()
                .map(|elem| elem.object_flags.contains(ObjectFlags::NonInferrableType))
                .unwrap_or(false),
            _ => false,
        }
    }
}
