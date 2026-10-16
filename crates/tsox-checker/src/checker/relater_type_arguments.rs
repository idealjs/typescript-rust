#![allow(dead_code)]

use std::sync::Arc;

use crate::checker::is_tuple_type;

use crate::checker::checker::Checker;

use crate::checker::relater::*;

impl Checker {
    pub fn type_arguments_related_to(
        &mut self,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        variances: &[VarianceFlags],
        relation: RelationKind,
    ) -> Ternary { ::tsox_core::fntrace::enter("type_arguments_related_to"); 
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
            let related = if variance_flags.intersects(VarianceFlags::Unmeasurable) {
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
                if variance_flags.intersects(VarianceFlags::Unreliable) {
                    self.report_unreliable_markers(s);
                }
                match variance {
                    VarianceFlags::Covariant => {
                        self.compare_types(Arc::clone(s), Arc::clone(t), relation, false)
                    }
                    VarianceFlags::Contravariant => {
                        self.compare_types(Arc::clone(t), Arc::clone(s), relation, false)
                    }
                    VarianceFlags::Independent => Ternary::True,
                    _ => {
                        let is_bivariant = variance_flags.intersects(VARIANCE_FLAGS_BIVARIANT)
                            && variance != VarianceFlags::None;
                        if is_bivariant {
                            let was_silent = self.silence_relation_chain();
                            let contra =
                                self.compare_types(Arc::clone(t), Arc::clone(s), relation, false);
                            self.restore_relation_chain(was_silent);
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
                                co.and(self.compare_types(
                                    Arc::clone(t),
                                    Arc::clone(s),
                                    relation,
                                    false,
                                ))
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
    ) -> bool { ::tsox_core::fntrace::enter("compare_type_parameters_identical"); 
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
    ) -> Option<Ternary> { ::tsox_core::fntrace::enter("generic_type_reference_related_to"); 
        if !source.flags.contains(TypeFlags::Object) || !target.flags.contains(TypeFlags::Object) {
            return None;
        }

        if is_tuple_type(source) || is_tuple_type(target) {
            return None;
        }
        let same_generic_origin = 'origin: {
            if let (Some(st), Some(tt)) = (source.target(), target.target()) {
                if Arc::ptr_eq(&st, &tt) {
                    break 'origin true;
                }
                if let (Some(ss), Some(ts)) = (&st.symbol, &tt.symbol) {
                    if ss.id() == ts.id() {
                        break 'origin true;
                    }
                }
            }
            match (&source.symbol, &target.symbol) {
                (Some(ss), Some(ts)) => {
                    ss.id() == ts.id()
                        && ss.flags.intersects(
                            tsox_frontend::ast::SymbolFlags::Interface
                                | tsox_frontend::ast::SymbolFlags::Class,
                        )
                        && !self.is_class_ctor_vs_instance_pair(&source, &target, ss)
                }
                _ => false,
            }
        };
        if !same_generic_origin {
            return None;
        }

        if self.is_marker_type(source) || self.is_marker_type(target) {
            return None;
        }

        if self.is_empty_array_literal_type(source) {
            return Some(Ternary::True);
        }

        let source_args = self.get_type_arguments(source);
        let source_args = if source_args.is_empty() {
            self.bare_generic_type_parameters(source)
        } else {
            source_args
        };
        let target_args = self.get_type_arguments(target);
        let target_args = if target_args.is_empty() {
            self.bare_generic_type_parameters(target)
        } else {
            target_args
        };
        if source_args.is_empty() && target_args.is_empty() {
            return Some(Ternary::True);
        }
        if source_args.len() != target_args.len() {
            return None;
        }
        let variances = match source.target() {
            Some(t) => {
                let v = self.get_variances(&t);
                if v.is_empty() {
                    return Some(Ternary::Unknown);
                }
                v
            }
            None => match self.measure_variances_from_symbol(source) {
                Some(v) if v.is_empty() => return Some(Ternary::Unknown),
                Some(v) => v,
                None => vec![VarianceFlags::Covariant; source_args.len()],
            },
        };
        let chain_len = self.relater_error_chain.len();
        if self
            .type_arguments_related_to(&source_args, &target_args, &variances, relation)
            .is_false()
        {
            if variances
                .iter()
                .any(|v| v.intersects(VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK))
            {
                self.relater_error_chain.truncate(chain_len);
                return None;
            }
            if !self.has_covariant_void_argument(&target_args, &variances) {
                let has_invariant = variances
                    .iter()
                    .any(|v| (*v & VARIANCE_FLAGS_VARIANCE_MASK) == VARIANCE_FLAGS_INVARIANT);
                if !(self.relater_chain_active && has_invariant) {
                    return Some(Ternary::False);
                }
                let (variance_chain, prefix_kept) = self.take_variance_chain(chain_len);
                let saved_primitive = std::mem::take(&mut self.relater_pending_primitive_source);
                let structural =
                    self.is_object_type_related_to(source, target, relation, saved_primitive);
                self.relater_pending_primitive_source = saved_primitive;
                if structural {
                    self.restore_variance_chain(chain_len, variance_chain, prefix_kept);
                }
                return Some(Ternary::False);
            }
            return None;
        }
        Some(Ternary::True)
    }

    pub fn is_class_ctor_vs_instance_pair(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        symbol: &Arc<tsox_frontend::ast::Symbol>,
    ) -> bool { ::tsox_core::fntrace::enter("is_class_ctor_vs_instance_pair"); 
        if source.object_flags.contains(ObjectFlags::Reference)
            && target.object_flags.contains(ObjectFlags::Reference)
        {
            return false;
        }
        let declared = self.get_declared_type_of_class_or_interface(symbol);
        Arc::ptr_eq(&declared, source) != Arc::ptr_eq(&declared, target)
    }

    pub fn bare_generic_type_parameters(&mut self, t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("bare_generic_type_parameters"); 
        let Some(symbol) = t.symbol.as_ref() else {
            return Vec::new();
        };
        if !symbol.flags.intersects(
            tsox_frontend::ast::SymbolFlags::Interface
                | tsox_frontend::ast::SymbolFlags::Class,
        ) {
            return Vec::new();
        }
        self.declared_type_parameter_types(symbol)
    }

    pub fn get_variances(&mut self, target: &Arc<Type>) -> Vec<VarianceFlags> { ::tsox_core::fntrace::enter("get_variances"); 
        let Some(symbol) = target.symbol.clone() else {
            return Vec::new();
        };
        let type_parameters = self.declared_type_parameter_types(&symbol);
        self.get_variances_worker(&symbol, &type_parameters)
    }

    pub fn measure_variances_from_symbol(&mut self, t: &Arc<Type>) -> Option<Vec<VarianceFlags>> { ::tsox_core::fntrace::enter("measure_variances_from_symbol"); 
        let symbol = t.symbol.as_ref()?;
        if !symbol.flags.intersects(
            tsox_frontend::ast::SymbolFlags::Interface
                | tsox_frontend::ast::SymbolFlags::Class,
        ) {
            return None;
        }
        let type_parameters = self.declared_type_parameter_types(symbol);
        if type_parameters.is_empty() {
            return None;
        }
        Some(self.get_variances_worker(symbol, &type_parameters))
    }

    pub fn is_marker_type(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_marker_type"); 
        self.marker_types.contains(&t.id)
    }

    /// Go isGenericMappedType（checker.go:25259）：mapped 且惰性约束解析后
    /// 是泛型索引（nameType/as 子句路径不在本域）。实例化壳经
    /// target+mapper 链取约束
    pub fn is_generic_mapped_type_relater(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_mapped_type_relater"); 
        t.object_flags.contains(ObjectFlags::Mapped)
            && self
                .get_constraint_type_from_mapped_type(t)
                .is_some_and(|c| self.is_generic_index_type(&c))
    }

    pub fn relate_alias_variances(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        variances: &[VarianceFlags],
        relation: RelationKind,
    ) -> Option<bool> { ::tsox_core::fntrace::enter("relate_alias_variances"); 
        let chain_len = self.relater_error_chain.len();
        if !self
            .type_arguments_related_to(sources, targets, variances, relation)
            .is_false()
        {
            return Some(true);
        }
        if variances
            .iter()
            .any(|v| v.intersects(VARIANCE_FLAGS_ALLOWS_STRUCTURAL_FALLBACK))
        {
            self.relater_error_chain.truncate(chain_len);
            return None;
        }
        if !variances.is_empty() && !self.has_covariant_void_argument(targets, variances) {
            if self.relater_chain_active
                && variances
                    .iter()
                    .any(|v| (*v & VARIANCE_FLAGS_VARIANCE_MASK) == VARIANCE_FLAGS_INVARIANT)
            {
                let (variance_chain, prefix_kept) = self.take_variance_chain(chain_len);
                let saved_primitive = std::mem::take(&mut self.relater_pending_primitive_source);
                let structural =
                    self.is_object_type_related_to(source, target, relation, saved_primitive);
                self.relater_pending_primitive_source = saved_primitive;
                if structural {
                    self.restore_variance_chain(chain_len, variance_chain, prefix_kept);
                }
            }
            return Some(false);
        }
        None
    }

    pub fn is_empty_array_literal_type(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_empty_array_literal_type"); 
        self.is_array_type(t)
            && self
                .get_element_type_of_array_type(t)
                .is_some_and(|element_type| self.is_empty_literal_type(&element_type))
    }

    fn take_variance_chain(&mut self, chain_len: usize) -> (Vec<RelaterChainEntry>, bool) { ::tsox_core::fntrace::enter("take_variance_chain"); 
        if self.relater_error_chain.len() >= chain_len {
            (self.relater_error_chain.split_off(chain_len), true)
        } else {
            (std::mem::take(&mut self.relater_error_chain), false)
        }
    }

    fn restore_variance_chain(
        &mut self,
        chain_len: usize,
        variance_chain: Vec<RelaterChainEntry>,
        prefix_kept: bool,
    ) { ::tsox_core::fntrace::enter("restore_variance_chain"); 
        if prefix_kept {
            let keep = chain_len.min(self.relater_error_chain.len());
            self.relater_error_chain.truncate(keep);
        } else {
            self.relater_error_chain.clear();
        }
        self.relater_error_chain.extend(variance_chain);
    }
}
