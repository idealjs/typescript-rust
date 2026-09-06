#![allow(dead_code)]

use std::sync::Arc;

use crate::checker::is_tuple_type;

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::*;

impl Checker {

    pub fn type_arguments_related_to(
        &mut self,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        variances: &[VarianceFlags],
        relation: RelationKind,
    ) -> Ternary {

        if sources.len() != targets.len() && relation == RelationKind::Identity {
            return Ternary::False;
        }
        let length = sources.len().min(targets.len());
        let mut result = Ternary::True;
        for i in 0..length {

            let variance_flags = variances
                .get(i)
                .copied()
                .unwrap_or(VarianceFlags::Covariant);
            let variance = variance_flags & VARIANCE_FLAGS_VARIANCE_MASK;

            if variance == VarianceFlags::Independent {
                continue;
            }

            let s = &sources[i];
            let t = &targets[i];
            let related = if variance_flags.intersects(VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK)
                && !variance_flags
                    .intersects(VarianceFlags::Unmeasurable | VarianceFlags::Unreliable)
            {

                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
            } else if variance_flags.intersects(VarianceFlags::Unmeasurable) {

                if relation == RelationKind::Identity {
                    if self.is_type_related_to(s, t, relation) {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                } else if self.is_type_identical_to(s, t) {
                    Ternary::True
                } else {
                    Ternary::False
                }
            } else {
                match variance {
                    VarianceFlags::Covariant => {
                        self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                    }
                    VarianceFlags::Contravariant => {

                        self.compare_types(Arc::clone(t), Arc::clone(s), relation, false)
                    }
                    VarianceFlags::Independent => {

                        Ternary::True
                    }
                    _ => {

                        let is_bivariant = variance_flags.intersects(VARIANCE_FLAGS_BIVARIANT)
                            && variance != VarianceFlags::None;
                        let contra =
                            self.compare_types(Arc::clone(t), Arc::clone(s), relation, false);
                        if is_bivariant {
                            if !contra.is_false() {
                                contra
                            } else {
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                            }
                        } else {

                            let co =
                                self.compare_types(Arc::clone(s), Arc::clone(t), relation, false);
                            if co.is_false() {
                                Ternary::False
                            } else {
                                co.and(contra)
                            }
                        }
                    }
                }
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    pub fn compare_type_parameters_identical(
        &mut self,
        source_params: &[Arc<Type>],
        target_params: &[Arc<Type>],
    ) -> bool {
        if source_params.len() != target_params.len() {
            return false;
        }
        for (source, target) in source_params.iter().zip(target_params.iter()) {
            if Arc::ptr_eq(source, target) {
                continue;
            }
            let source_constraint = self
                .get_constraint_of_type_parameter(source)
                .unwrap_or_else(|| self.unknown_type());
            let target_constraint = self
                .get_constraint_of_type_parameter(target)
                .unwrap_or_else(|| self.unknown_type());

            if !self.is_type_identical_to(&source_constraint, &target_constraint) {
                return false;
            }
        }
        true
    }

    pub fn generic_type_reference_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {

        if !source.flags.contains(TypeFlags::Object) || !target.flags.contains(TypeFlags::Object) {
            return None;
        }
        if !source.object_flags.contains(ObjectFlags::Reference)
            || !target.object_flags.contains(ObjectFlags::Reference)
        {
            return None;
        }

        if is_tuple_type(source) || is_tuple_type(target) {
            return None;
        }
        let source_target = source.target()?;
        let target_target = target.target()?;
        let same_target = Arc::ptr_eq(source_target, target_target)
            || match (&source_target.symbol, &target_target.symbol) {
                (Some(ss), Some(ts)) => ss.id() == ts.id(),
                _ => false,
            };
        if !same_target {
            return None;
        }

        if self.is_marker_type(source) || self.is_marker_type(target) {
            return None;
        }

        if self.is_empty_array_literal_type(source) {
            return Some(Ternary::True);
        }

        let variances = self.get_variances(source_target);
        if variances.is_empty() {
            return Some(Ternary::Maybe);
        }
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);
        Some(self.type_arguments_related_to(&source_args, &target_args, &variances, relation))
    }

    pub fn get_variances(&self, _target: &Arc<Type>) -> Vec<VarianceFlags> {

        match &_target.data {
            TypeData::Object(o) => {
                if let Some(t) = o.target.as_ref() {
                    if let TypeData::Interface(i) = &t.data {
                        let n = i.all_type_parameters.len();
                        return vec![VarianceFlags::Covariant; n];
                    }
                }
                Vec::new()
            }
            TypeData::Interface(i) => {
                let n = i.all_type_parameters.len();
                vec![VarianceFlags::Covariant; n]
            }
            _ => Vec::new(),
        }
    }

    pub fn is_marker_type(&self, _t: &Arc<Type>) -> bool {
        false
    }

    pub fn is_empty_array_literal_type(&self, t: &Arc<Type>) -> bool {

        t.object_flags.contains(ObjectFlags::FreshLiteral) && self.is_array_type(t)
    }
}
