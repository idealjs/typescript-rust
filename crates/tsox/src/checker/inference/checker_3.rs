#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_inferred_types(&mut self, context: &InferenceContext) -> Vec<Arc<Type>> {
        let count = context.inferences.len();
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            result.push(self.get_inferred_type(context, i));
        }
        result
    }

    pub fn get_inferred_type(&mut self, context: &InferenceContext, index: usize) -> Arc<Type> {
        let inference = &context.inferences[index];
        if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
            eprintln!(
                "[get-inferred] tp={} cands={} contra={}",
                self.type_to_string(&inference.type_parameter),
                inference
                    .candidates
                    .iter()
                    .map(|c| self.type_to_string(c))
                    .collect::<Vec<_>>()
                    .join(","),
                inference
                    .contra_candidates
                    .iter()
                    .map(|c| self.type_to_string(c))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        if let Some(ref inferred) = inference.inferred_type {
            return Arc::clone(inferred);
        }

        if inference.type_parameter.flags.contains(TypeFlags::Any)
            && inference.type_parameter.intrinsic_name() == Some("error")
        {
            return Arc::clone(&inference.type_parameter);
        }

        let mut inferred_type: Option<Arc<Type>> = None;
        let mut fallback_type: Option<Arc<Type>> = None;

        if let Some(ref signature) = context.signature {
            let inferred_covariant = if !inference.candidates.is_empty() {
                self.get_covariant_inference(inference, signature)
            } else {
                None
            };

            let inferred_contravariant = if !inference.contra_candidates.is_empty() {
                self.get_contravariant_inference(inference)
            } else {
                None
            };

            if inferred_covariant.is_some() || inferred_contravariant.is_some() {
                let prefer_covariant = match (&inferred_covariant, &inferred_contravariant) {
                    (Some(_cov), None) => true,
                    (None, Some(_)) => false,
                    (Some(cov), Some(_contra)) => {
                        let cov_not_never_or_any =
                            !cov.flags.intersects(TypeFlags::Never | TypeFlags::Any);
                        let cov_assignable_to_contra = inference
                            .contra_candidates
                            .iter()
                            .any(|t| self.is_type_assignable_to(cov, t));
                        let no_conflicting_constraints = context.inferences.iter().all(|other| {
                            let other_tp = &other.type_parameter;
                            let constraint = self.get_constraint_of_type_parameter(other_tp);
                            let is_constrained = constraint.as_ref().map_or(false, |c| {
                                if let Some(c_cons) = c.as_union_or_intersection() {
                                    c_cons.types.iter().any(|ct| {
                                        crate::checker::utilities::type_parameters_match(
                                            ct,
                                            &inference.type_parameter,
                                        )
                                    })
                                } else {
                                    false
                                }
                            });
                            !is_constrained
                                || other
                                    .candidates
                                    .iter()
                                    .all(|t| self.is_type_assignable_to(t, cov))
                        });
                        cov_not_never_or_any
                            && cov_assignable_to_contra
                            && no_conflicting_constraints
                    }
                    (None, None) => false,
                };

                if prefer_covariant {
                    inferred_type = inferred_covariant;
                    fallback_type = inferred_contravariant;
                } else {
                    inferred_type = inferred_contravariant;
                    fallback_type = inferred_covariant;
                }
            } else if context.flags.contains(InferenceFlags::NoDefault) {
                inferred_type = Some(self.never_type());
            } else {
                let default_type = self.get_default_from_type_parameter(&inference.type_parameter);
                if let Some(dt) = default_type {
                    inferred_type = Some(dt);
                }
            }
        } else {
            inferred_type = self.get_type_from_inference(inference);
        }

        if inferred_type.is_some() {
            let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
            if let Some(constraint) = constraint {
                if !self.is_type_assignable_to(inferred_type.as_ref().unwrap(), &constraint) {
                    if inference.priority.contains(InferencePriority::ReturnType) {
                        let inferred = inferred_type.as_ref().unwrap();
                        let filtered = if inferred.flags.contains(TypeFlags::Union) {
                            if let Some(types) = inferred.types() {
                                let filtered: Vec<Arc<Type>> = types
                                    .iter()
                                    .filter(|u| self.is_type_assignable_to(u, &constraint))
                                    .cloned()
                                    .collect();
                                if filtered.is_empty() {
                                    self.never_type()
                                } else if filtered.len() == 1 {
                                    filtered[0].clone()
                                } else {
                                    self.get_union_type(filtered)
                                }
                            } else {
                                self.never_type()
                            }
                        } else if self.is_type_assignable_to(inferred, &constraint) {
                            (*inferred).clone()
                        } else {
                            self.never_type()
                        };
                        if filtered.flags.contains(TypeFlags::Never) {
                            inferred_type = None;
                        } else {
                            inferred_type = Some(filtered);
                        }
                    } else {
                        inferred_type = None;
                    }
                }
            }
        }

        if inferred_type.is_none() {
            if let Some(fallback) = fallback_type {
                let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
                if let Some(constraint) = constraint {
                    if self.is_type_assignable_to(&fallback, &constraint) {
                        inferred_type = Some(fallback);
                    } else {
                        inferred_type = Some(constraint);
                    }
                } else {
                    inferred_type = Some(fallback);
                }
            } else {
                let constraint = self.get_constraint_of_type_parameter(&inference.type_parameter);
                inferred_type = constraint;
            }
        }

        inferred_type.unwrap_or_else(|| {
            if context.flags.contains(InferenceFlags::AnyDefault) {
                self.any_type()
            } else {
                self.unknown_type()
            }
        })
    }

}
