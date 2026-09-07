#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn instantiate_signature_in_context_of(
        &mut self,
        source: &Arc<Signature>,
        contextual: &Arc<Signature>,
    ) -> Arc<Signature> {
        if source.type_parameters.is_empty() {
            return Arc::clone(source);
        }
        let inferences: Vec<crate::checker::inference::InferenceInfo> = source
            .type_parameters
            .iter()
            .map(|p| crate::checker::inference::InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = crate::checker::inference::InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(source));

        if let (Some(contextual_this), Some(source_this)) = (
            self.get_this_type_of_signature(contextual),
            self.get_this_type_of_signature(source),
        ) {
            self.infer_types(
                &mut context.inferences,
                Some(contextual_this),
                Some(source_this),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }

        let contextual_count = self.get_parameter_count(contextual);
        let generic_count = self.get_parameter_count(source);
        let contextual_rest = self.get_effective_rest_type(contextual);
        let generic_rest = self.get_effective_rest_type(source);
        let generic_non_rest = generic_count.saturating_sub(usize::from(generic_rest.is_some()));
        let param_count = if contextual_rest.is_none() {
            contextual_count.min(generic_non_rest)
        } else {
            generic_non_rest
        };
        for i in 0..param_count {
            let s = self.get_type_at_position(contextual, i);
            let t = self.get_type_at_position(source, i);
            self.infer_types(
                &mut context.inferences,
                Some(s),
                Some(t),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }
        if let Some(generic_rest) = generic_rest {
            let s = self.get_type_at_position(contextual, param_count);
            self.infer_types(
                &mut context.inferences,
                Some(s),
                Some(generic_rest),
                crate::checker::inference::InferencePriority::None,
                false,
            );
        }

        if let Some(source_return) = self.get_return_type_of_signature(source) {
            if type_contains_type_parameter(&source_return) {
                if let Some(contextual_return) = self.get_return_type_of_signature(contextual) {
                    self.infer_types(
                        &mut context.inferences,
                        Some(contextual_return),
                        Some(source_return),
                        crate::checker::inference::InferencePriority::ReturnType,
                        false,
                    );
                }
            }
        }
        let inferred = self.get_inferred_types(&mut context);
        self.get_signature_instantiation(source, &inferred)
    }

    pub fn get_canonical_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let type_arguments: Vec<Arc<Type>> = sig
            .type_parameters
            .iter()
            .map(|tp| match &tp.data {
                TypeData::TypeParameter(tpd) => match tpd.target.as_ref() {
                    Some(original) if self.get_constraint_of_type_parameter(original).is_none() => {
                        Arc::clone(original)
                    }
                    _ => Arc::clone(tp),
                },
                _ => Arc::clone(tp),
            })
            .collect();

        if type_arguments
            .iter()
            .zip(sig.type_parameters.iter())
            .all(|(arg, param)| Arc::ptr_eq(arg, param))
        {
            return Arc::clone(sig);
        }
        self.get_signature_instantiation(sig, &type_arguments)
    }

    pub fn get_base_constraint_or_type(&self, t: &Arc<Type>) -> Arc<Type> {
        self.get_base_constraint_of_type(t)
            .or_else(|| self.get_constraint_of_type_parameter(t))
            .unwrap_or_else(|| Arc::clone(t))
    }

    pub fn type_flags_is_generic_object_type(&self, t: &Arc<Type>) -> bool {
        if t.flags
            .intersects(TYPE_FLAGS_UNION_OR_INTERSECTION | TypeFlags::Substitution)
        {
            return t
                .types()
                .map(|ts| ts.iter().any(|u| self.type_flags_is_generic_object_type(u)))
                .unwrap_or(false);
        }
        if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE) {
            return true;
        }

        match &t.data {
            TypeData::Mapped(m) => m
                .constraint_type
                .as_ref()
                .map(|c| self.type_flags_is_generic_index_type(c))
                .unwrap_or(false),
            TypeData::Tuple(tup) => tup.element_infos.iter().any(|ei| {
                ei.type_
                    .as_ref()
                    .map(type_contains_type_parameter)
                    .unwrap_or(false)
            }),
            _ => false,
        }
    }

    pub fn type_flags_is_generic_index_type(&self, t: &Arc<Type>) -> bool {
        if t.flags
            .intersects(TYPE_FLAGS_UNION_OR_INTERSECTION | TypeFlags::Substitution)
        {
            return t
                .types()
                .map(|ts| ts.iter().any(|u| self.type_flags_is_generic_index_type(u)))
                .unwrap_or(false);
        }
        t.flags.intersects(
            TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE | TypeFlags::Index | TypeFlags::TemplateLiteral,
        )
    }

    pub fn get_single_call_signature(&self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        let sigs = self.get_signatures_of_type(t, SignatureKind::Call);
        if sigs.len() == 1 {
            sigs.into_iter().next()
        } else {
            None
        }
    }

    pub fn get_non_nullable_type_of(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(constituents) = t.types()
        {
            let kept: Vec<Arc<Type>> = constituents
                .iter()
                .filter(|c| !c.flags.intersects(TypeFlags::Null | TypeFlags::Undefined))
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != constituents.len() {
                return self.get_union_type(kept);
            }
        }
        Arc::clone(t)
    }

    pub fn type_is_undefined_or_null(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Any | TypeFlags::Unknown,
        ) {
            return true;
        }
        match &t.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.type_is_undefined_or_null(c)),
            _ => false,
        }
    }

    pub fn is_instantiated_generic_parameter(&mut self, sig: &Arc<Signature>, pos: usize) -> bool {
        let Some(target) = &sig.target else {
            return false;
        };
        match self.try_get_type_at_position(target, pos) {
            Some(t) => self.is_generic_type(&t),
            None => false,
        }
    }

    pub fn is_generic_type(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::TypeParameter) {
            return true;
        }
        t.types()
            .map(|ts| ts.iter().any(type_contains_type_parameter))
            .is_some()
    }

    pub fn try_get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        access_flags: AccessFlags,
    ) -> Option<Arc<Type>> {
        if object_type.flags.contains(TypeFlags::Any) || index_type.flags.contains(TypeFlags::Any) {
            return Some(self.any_type());
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return Some(self.unknown_type());
        }

        if index_type.flags.contains(TypeFlags::Union) {
            let constituents = index_type.types()?.to_vec();
            let mut resolved = Vec::with_capacity(constituents.len());
            for c in &constituents {
                resolved.push(self.try_get_indexed_access_type(object_type, c, access_flags)?);
            }
            return Some(self.get_union_type(resolved));
        }

        if object_type.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(object_type)?;
            return self.try_get_indexed_access_type(&constraint, index_type, access_flags);
        }
        if let Some(structured) = object_type.as_structured() {
            if index_type.flags.contains(TypeFlags::StringLiteral)
                && let TypeData::Literal(lit) = &index_type.data
                && let LiteralValue::String(name) = &lit.value
            {
                if let Some(sym) = structured.members.get(name) {
                    return Some(self.get_type_of_symbol(sym));
                }
                if !access_flags.contains(AccessFlags::NoIndexSignatures) {
                    if let Some(value_type) =
                        self.lookup_index_signature_value(structured, index_type)
                    {
                        return Some(value_type);
                    }
                }
                return None;
            }

            if index_type
                .flags
                .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
            {
                if let Some(elem) = self.get_array_element_type_of(object_type) {
                    return Some(elem);
                }
                if self.is_tuple_type(object_type) {
                    let elem_types: Vec<Arc<Type>> = structured
                        .properties
                        .iter()
                        .map(|p| self.get_type_of_symbol(p))
                        .collect();
                    if !elem_types.is_empty() {
                        return Some(self.get_union_type(elem_types));
                    }
                }
                return None;
            }

            if index_type
                .flags
                .intersects(TypeFlags::String | TypeFlags::StringLiteral)
                && !access_flags.contains(AccessFlags::NoIndexSignatures)
            {
                return self.lookup_index_signature_value(structured, index_type);
            }
        }
        None
    }
}
