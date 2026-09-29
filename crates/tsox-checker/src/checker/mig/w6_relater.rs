#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{ModifierFlags, Node, Symbol, SymbolFlags};
use tsox_frontend::ast::CheckFlags;

const INVARIANT: VarianceFlags = VarianceFlags::empty();
const BIVARIANT: VarianceFlags = VarianceFlags::Covariant.union(VarianceFlags::Contravariant);
const CHECK_FLAGS_PARTIAL: CheckFlags = CheckFlags::from_bits_retain((1 << 4) | (1 << 5));

use super::m1a::r19k2_defs::is_static_private_identifier_property;
use crate::checker::relater_compare_checker_4::{RELATION_REPORTS_UNMEASURABLE, RELATION_REPORTS_UNRELIABLE};

thread_local! {
    static VARIANCE_STACK_ENTRIES: std::cell::RefCell<Vec<(usize, Arc<Symbol>, Vec<Arc<Type>>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl Checker {
    pub fn get_unmatched_properties_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        require_optional_properties: bool,
        match_discriminant_properties: bool,
        mut props_out: Option<&mut Vec<Arc<Symbol>>>,
    ) -> Option<Arc<Symbol>> {
        let properties = self.get_properties_of_type(target);
        for target_prop in properties {
            if is_static_private_identifier_property(&target_prop) {
                continue;
            }
            if require_optional_properties
                || (!target_prop.flags.intersects(SymbolFlags::Optional)
                    && !target_prop.check_flags.intersects(CHECK_FLAGS_PARTIAL))
            {
                let source_prop = self.get_property_of_type(source, &target_prop.name);
                match source_prop {
                    None => {
                        if let Some(props_out) = props_out.as_deref_mut() {
                            props_out.push(Arc::clone(&target_prop));
                        } else {
                            return Some(target_prop);
                        }
                    }
                    Some(source_prop) if match_discriminant_properties => {
                        let target_type = self.get_type_of_symbol(&target_prop);
                        if target_type.flags.intersects(type_flags_unit()) {
                            let source_type = self.get_type_of_symbol(&source_prop);
                            if !(source_type.flags.intersects(TypeFlags::Any)
                                || Arc::ptr_eq(
                                    &self.get_regular_type_of_literal_type(&source_type),
                                    &self.get_regular_type_of_literal_type(&target_type),
                                ))
                            {
                                if let Some(props_out) = props_out.as_deref_mut() {
                                    props_out.push(Arc::clone(&target_prop));
                                } else {
                                    return Some(target_prop);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn get_variances_worker(&mut self, symbol: &Arc<Symbol>, type_parameters: &[Arc<Type>]) -> Vec<VarianceFlags> {
        let variances_unset = self
            .variance_links
            .get(symbol)
            .map(|l| l.variances.is_empty())
            .unwrap_or(true);
        if variances_unset {
            let stack_index = self.get_variance_stack_index(symbol);
            if stack_index < 0 {
                let save_resolution_start = self.resolution_start;
                if self.variance_stack.is_empty() {
                    self.resolution_start = self.type_resolutions.len();
                }
                self.variance_stack.push(Arc::as_ptr(symbol) as usize);
                VARIANCE_STACK_ENTRIES.with(|s| {
                    s.borrow_mut().push((Arc::as_ptr(symbol) as usize, Arc::clone(symbol), type_parameters.to_vec()))
                });
                let mut variances = vec![VarianceFlags::empty(); type_parameters.len()];
                for (i, tp) in type_parameters.iter().enumerate() {
                    let modifiers = self.get_type_parameter_modifiers(tp);
                    let mut variance = VarianceFlags::empty();
                    if modifiers.intersects(ModifierFlags::Out) {
                        if modifiers.intersects(ModifierFlags::In) {
                            variance = INVARIANT;
                        } else {
                            variance = VarianceFlags::Covariant;
                        }
                    } else if modifiers.intersects(ModifierFlags::In) {
                        variance = VarianceFlags::Contravariant;
                    } else {
                        let save_reliability_flags = self.reliability_flags;
                        self.reliability_flags = 0;
                        let marker_super = self.marker_super();
                        let marker_sub = self.marker_sub();
                        let type_with_super = self.create_marker_type(symbol, tp, &marker_super);
                        let type_with_sub = self.create_marker_type(symbol, tp, &marker_sub);
                        variance = if self.is_marker_type_assignable_to(&type_with_sub, &type_with_super) {
                            VarianceFlags::Covariant
                        } else {
                            VarianceFlags::empty()
                        } | if self.is_marker_type_assignable_to(&type_with_super, &type_with_sub) {
                            VarianceFlags::Contravariant
                        } else {
                            VarianceFlags::empty()
                        };
                        if variance == BIVARIANT {
                            let marker_other = self.marker_other();
                            let type_with_other = self.create_marker_type(symbol, tp, &marker_other);
                            if self.is_marker_type_assignable_to(&type_with_other, &type_with_super) {
                                variance = VarianceFlags::Independent;
                            }
                        }
                        if self.reliability_flags & RELATION_REPORTS_UNMEASURABLE != 0 {
                            variance |= VarianceFlags::Unmeasurable;
                        }
                        if self.reliability_flags & RELATION_REPORTS_UNRELIABLE != 0 {
                            variance |= VarianceFlags::Unreliable;
                        }
                        self.reliability_flags = save_reliability_flags;
                    }
                    let links_computed = self
                        .variance_links
                        .get(symbol)
                        .map(|l| !l.variances.is_empty())
                        .unwrap_or(false);
                    if links_computed {
                        break;
                    }
                    variances[i] = variance;
                }
                let links = self.variance_links.get_or_default(symbol);
                if links.variances.is_empty() {
                    links.variances = variances;
                }
                self.variance_stack.pop();
                VARIANCE_STACK_ENTRIES.with(|s| {
                    s.borrow_mut().pop();
                });
                if self.variance_stack.is_empty() {
                    self.resolution_start = save_resolution_start;
                }
            } else {
                let stack_index_us = stack_index as usize;
                let mut min_index = stack_index_us;
                let lookup_symbol = |key: usize| -> Option<Arc<Symbol>> {
                    VARIANCE_STACK_ENTRIES.with(|s| s.borrow().iter().find(|e| e.0 == key).map(|e| Arc::clone(&e.1)))
                };
                for i in (stack_index_us + 1)..self.variance_stack.len() {
                    let i_symbol = match lookup_symbol(self.variance_stack[i]) {
                        Some(s) => s,
                        None => continue,
                    };
                    let min_symbol = match lookup_symbol(self.variance_stack[min_index]) {
                        Some(s) => s,
                        None => continue,
                    };
                    if self.compare_symbols(&i_symbol, &min_symbol) < 0 {
                        min_index = i;
                    }
                }
                if min_index > stack_index_us {
                    let save_variance_stack = std::mem::take(&mut self.variance_stack);
                    let (sym, tps) = VARIANCE_STACK_ENTRIES
                        .with(|s| {
                            s.borrow().get(min_index).map(|e| (Arc::clone(&e.1), e.2.clone()))
                        })
                        .unwrap_or_else(|| (Arc::clone(symbol), Vec::new()));
                    self.get_variances_worker(&sym, &tps);
                    self.variance_stack = save_variance_stack;
                }
                let links = self.variance_links.get_or_default(symbol);
                if links.variances.is_empty() {
                    links.variances = Vec::new();
                }
            }
        }
        self.variance_links
            .get(symbol)
            .map(|l| l.variances.clone())
            .unwrap_or_default()
    }

    pub fn get_variance_stack_index(&self, symbol: &Arc<Symbol>) -> isize {
        let key = Arc::as_ptr(symbol) as usize;
        match self.variance_stack.iter().position(|k| *k == key) {
            Some(i) => i as isize,
            None => -1,
        }
    }

    fn is_marker_type_assignable_to(&mut self, source: &Option<Arc<Type>>, target: &Option<Arc<Type>>) -> bool {
        match (source, target) {
            (Some(s), Some(t)) => self.is_type_assignable_to(s, t),
            _ => false,
        }
    }
}

fn type_flags_unit() -> TypeFlags {
    TypeFlags::StringLiteral
        .union(TypeFlags::NumberLiteral)
        .union(TypeFlags::BigIntLiteral)
        .union(TypeFlags::BooleanLiteral)
        .union(TypeFlags::String)
        .union(TypeFlags::Number)
}

impl Checker {
    pub fn get_undefined_stripped_target_if_needed(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Arc<Type> {
        if source.flags.intersects(TypeFlags::Union)
            && target.flags.intersects(TypeFlags::Union)
            && let (Some(sts), Some(tts)) = (source.types(), target.types())
            && !sts[0].flags.intersects(TypeFlags::Undefined)
            && !tts[0].flags.intersects(TypeFlags::Undefined)
        {
            return self.extract_types_of_kind(target, !TypeFlags::Undefined);
        }
        Arc::clone(target)
    }
}

pub fn is_conversion_or_interface_implementation_message(message: &tsox_core::diagnostics::Message) -> bool {
    use tsox_core::diagnostics::*;
    message == &CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1
        || message == &CLASS_0_INCORRECTLY_IMPLEMENTS_CLASS_1_DID_YOU_MEAN_TO_EXTEND_1_AND_INHERIT_ITS_MEMBERS_AS_A_SUBCLASS
        || message == &CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST
        || message == &ITS_INSTANCE_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT
        || message == &ITS_RETURN_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT
        || message == &ITS_ELEMENT_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT
}
