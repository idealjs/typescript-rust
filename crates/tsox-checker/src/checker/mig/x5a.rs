#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::checker::types_type_id::SignatureKind;
use crate::checker::mig::m2a::r19k11_defs::R19K11CheckerExt;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::types::{TypeFlags, TypeFacts};
use crate::checker::Type;
use crate::checker::Checker;
use crate::checker::flow_narrow_typeof::TYPEOF_NE_HOST_OBJECT;

impl Checker {
    pub(crate) fn get_narrowed_type(
        &mut self,
        t: &Arc<Type>,
        candidate: &Arc<Type>,
        assume_true: bool,
        check_derived: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_narrowed_type"); 
        if !t.flags.intersects(TypeFlags::Union) {
            return self.get_narrowed_type_worker(t, candidate, assume_true, check_derived);
        }
        self.get_narrowed_type_worker(t, candidate, assume_true, check_derived)
    }

    pub(crate) fn get_narrowed_type_worker(
        &mut self,
        t: &Arc<Type>,
        candidate: &Arc<Type>,
        assume_true: bool,
        check_derived: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_narrowed_type_worker"); 
        let checker_ptr: *mut Checker = self;
        if !assume_true {
            if Arc::ptr_eq(t, candidate) {
                return self.never_type();
            }
            if check_derived {
                let candidate = Arc::clone(candidate);
                return self.filter_type(t, &mut |t: &Arc<Type>| {
                    unsafe { !(*checker_ptr).is_type_derived_from(t, &candidate) }
                });
            }
            let t = if t.flags.intersects(TypeFlags::Unknown) {
                self.unknown_union_type()
            } else {
                Arc::clone(t)
            };
            let true_type = self.get_narrowed_type(&t, candidate, true, false);
            let filtered = self.filter_type(&t, &mut |t: &Arc<Type>| {
                unsafe { !(*checker_ptr).is_type_subset_of(t, &true_type) }
            });
            return self.recombine_unknown_type(&filtered);
        }
        if t.flags.intersects(TypeFlags::ANY_OR_UNKNOWN) {
            return Arc::clone(candidate);
        }
        if Arc::ptr_eq(t, candidate) {
            return Arc::clone(candidate);
        }
        let key_property_name = if t.flags.intersects(TypeFlags::Union) {
            self.get_key_property_name(t)
        } else {
            None
        };
        let t2 = Arc::clone(t);
        let check_derived2 = check_derived;
        let narrowed_type = self
            .map_type(candidate, &mut |n: &Arc<Type>| {
                let mut matching = Arc::clone(&t2);
                if let Some(key_property_name) = &key_property_name {
                    if let Some(discriminant) = unsafe { (*checker_ptr).get_type_of_property_of_type(n, key_property_name) } {
                        if let Some(constituent) = unsafe { (*checker_ptr).get_constituent_type_for_key_type(&t2, &discriminant) } {
                            matching = constituent;
                        }
                    }
                }
                let directly_related = unsafe { (*checker_ptr).map_type(&matching, &mut |t: &Arc<Type>| {
                    if check_derived2 {
                        if unsafe { (*checker_ptr).is_type_derived_from(t, n) } {
                            return Some(Arc::clone(t));
                        }
                        if unsafe { (*checker_ptr).is_type_derived_from(n, t) } {
                            return Some(Arc::clone(n));
                        }
                    } else {
                        if unsafe { (*checker_ptr).is_type_strict_subtype_of(t, n) } {
                            return Some(Arc::clone(t));
                        }
                        if unsafe { (*checker_ptr).is_type_strict_subtype_of(n, t) } {
                            return Some(Arc::clone(n));
                        }
                        if unsafe { (*checker_ptr).is_type_subtype_of(t, n) } {
                            return Some(Arc::clone(t));
                        }
                        if unsafe { (*checker_ptr).is_type_subtype_of(n, t) } {
                            return Some(Arc::clone(n));
                        }
                    }
                    Some(unsafe { (*checker_ptr).never_type() })
                }) };
                if let Some(directly_related) = directly_related {
                    if !directly_related.flags.intersects(TypeFlags::Never) {
                        return Some(directly_related);
                    }
                }
                unsafe { (*checker_ptr).map_type(&t2, &mut |t: &Arc<Type>| {
                    if unsafe { (*checker_ptr).maybe_type_of_kind(t, TypeFlags::INSTANTIABLE) } {
                        let constraint = unsafe { (*checker_ptr).get_base_constraint_of_type(t) };
                        let related = match &constraint {
                            None => true,
                            Some(constraint) => {
                                if check_derived2 {
                                    unsafe { (*checker_ptr).is_type_derived_from(n, constraint) }
                                } else {
                                    unsafe { (*checker_ptr).is_type_subtype_of(n, constraint) }
                                }
                            }
                        };
                        if related {
                            return Some(unsafe { (*checker_ptr).get_intersection_type(vec![Arc::clone(t), Arc::clone(n)]) });
                        }
                    }
                    Some(unsafe { (*checker_ptr).never_type() })
                }) }
            })
            .unwrap_or_else(|| self.never_type());
        if !narrowed_type.flags.intersects(TypeFlags::Never) {
            return narrowed_type;
        }
        if self.is_type_subtype_of(candidate, t) {
            return Arc::clone(candidate);
        }
        if self.is_type_assignable_to(t, candidate) {
            return Arc::clone(t);
        }
        if self.is_type_assignable_to(candidate, t) {
            return Arc::clone(candidate);
        }
        self.get_intersection_type(vec![Arc::clone(t), Arc::clone(candidate)])
    }

    pub(crate) fn get_not_equal_facts_from_typeof_switch(
        &self,
        start: usize,
        end: usize,
        witnesses: &[String],
    ) -> TypeFacts { ::tsox_core::fntrace::enter("get_not_equal_facts_from_typeof_switch"); 
        let mut facts = TypeFacts::empty();
        for (i, witness) in witnesses.iter().enumerate() {
            if (i < start || i >= end) && !witness.is_empty() {
                facts |= TypeFacts::from_bits_truncate(Self::typeof_ne_facts_of_witness(witness));
            }
        }
        facts
    }

    pub(crate) fn get_reference_candidate(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_reference_candidate"); 
        match node.kind {
            SyntaxKind::ParenthesizedExpression => {
                return self.get_reference_candidate(node.expression().expect("parenthesized expression has inner expression"));
            }
            SyntaxKind::BinaryExpression => {
                match node.as_binary_expression().operator_token.kind {
                    SyntaxKind::EqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken
                    | SyntaxKind::QuestionQuestionEqualsToken => {
                        return self.get_reference_candidate(&node.as_binary_expression().left);
                    }
                    SyntaxKind::CommaToken => {
                        return self.get_reference_candidate(&node.as_binary_expression().right);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Arc::clone(node)
    }

    pub(crate) fn get_symbol_has_instance_method_of_object_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_symbol_has_instance_method_of_object_type"); 
        let has_instance_property_name = self.get_property_name_for_known_symbol_name("hasInstance");
        if self.all_types_assignable_to_kind(t, TypeFlags::NonPrimitive) {
            if let Some(has_instance_property) = self.get_property_of_type(t, &has_instance_property_name) {
                let has_instance_property_type = self.get_type_of_symbol(&has_instance_property);
                if !self
                    .get_signatures_of_type(&has_instance_property_type, SignatureKind::Call)
                    .is_empty()
                {
                    return Some(has_instance_property_type);
                }
            }
        }
        None
    }
}
