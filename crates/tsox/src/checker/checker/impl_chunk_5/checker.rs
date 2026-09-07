#![allow(unused_imports)]

use super::*;

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
        if let Some(structured) = t.as_structured() {
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
        t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::Reference)
    }

    pub fn is_tuple_type(&self, t: &Arc<Type>) -> bool {
        crate::checker::utilities::is_tuple_type(t)
    }

    pub fn get_base_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::EnumLiteral) {
            if let Some(sym) = &t.symbol
                && sym.flags.contains(SymbolFlags::EnumMember)
                && let Some(parent) = &sym.parent
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

    pub fn get_widened_type(&self, t: &Arc<Type>) -> Arc<Type> {
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
        Arc::clone(t)
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
