#![allow(unused_imports)]

use crate::checker::checker_union_signatures::*;

impl Checker {
    pub fn get_union_signatures(
        &mut self,
        signature_lists: &[Vec<Arc<Signature>>],
    ) -> Vec<Arc<Signature>> {
        if signature_lists.is_empty() || signature_lists.iter().any(|l| l.is_empty()) {
            return Vec::new();
        }
        let mut index_with_length_over_one = 0;
        let mut count_length_over_one = 0;
        for (i, list) in signature_lists.iter().enumerate() {
            if list.len() > 1 {
                index_with_length_over_one = i;
                count_length_over_one += 1;
            }
        }
        if count_length_over_one > 1 {
            return Vec::new();
        }
        let mut results = signature_lists[index_with_length_over_one].clone();
        for (i, list) in signature_lists.iter().enumerate() {
            if i == index_with_length_over_one {
                continue;
            }
            let signature = Arc::clone(&list[0]);
            results = results
                .iter()
                .map(|sig| self.combine_union_member_signature(sig, &signature, true))
                .collect();
        }
        results
    }

    pub(crate) fn combine_union_member_signature(
        &mut self,
        left: &Arc<Signature>,
        right: &Arc<Signature>,
        is_union: bool,
    ) -> Arc<Signature> {
        let (params, overrides) = self.combine_union_parameters(left, right, is_union);
        let mut flags = left.flags | right.flags;
        flags.remove(SignatureFlags::HasRestParameter);
        if params.last().is_some_and(|_| self.combined_has_rest_tail(left, right)) {
            flags.insert(SignatureFlags::HasRestParameter);
        }
        let min_arg = left.min_argument_count.max(right.min_argument_count);
        let mut s = Signature::new();
        s.flags = flags;
        s.min_argument_count = min_arg;
        s.resolved_min_argument_count = -1;
        s.declaration = left.declaration.clone().or_else(|| right.declaration.clone());
        s.type_parameters = if left.type_parameters.is_empty() {
            right.type_parameters.clone()
        } else {
            left.type_parameters.clone()
        };
        s.parameters = params;
        s.instantiated_parameter_types = Some(overrides);
        let left_ret = self
            .get_return_type_of_signature(left)
            .unwrap_or_else(|| self.any_type());
        let right_ret = self
            .get_return_type_of_signature(right)
            .unwrap_or_else(|| self.any_type());
        if is_union {
            let _ = s
                .resolved_return_type
                .set(self.get_union_type(vec![left_ret, right_ret]));
        } else {
            let _ = s
                .resolved_return_type
                .set(self.get_intersection_type(vec![left_ret, right_ret]));
        }
        Arc::new(s)
    }

    fn combined_has_rest_tail(&mut self, left: &Arc<Signature>, right: &Arc<Signature>) -> bool {
        self.has_effective_rest_parameter(left) || self.has_effective_rest_parameter(right)
    }

    pub(crate) fn get_intersected_signatures(
        &mut self,
        signatures: &[Arc<Signature>],
    ) -> Option<Arc<Signature>> {
        if !self.no_implicit_any {
            return None;
        }
        let mut combined: Option<Arc<Signature>> = None;
        for sig in signatures {
            match &combined {
                None => combined = Some(Arc::clone(sig)),
                Some(c) if Arc::ptr_eq(c, sig) => {}
                Some(c) => {
                    if self.compare_type_parameters_identical(
                        &c.type_parameters,
                        &sig.type_parameters,
                    ) {
                        combined = Some(self.combine_union_member_signature(c, sig, false));
                    } else {
                        return None;
                    }
                }
            }
        }
        combined
    }

    fn combine_union_parameters(
        &mut self,
        left: &Arc<Signature>,
        right: &Arc<Signature>,
        is_union: bool,
    ) -> (Vec<Arc<Symbol>>, Vec<Arc<Type>>) {
        let left_count = self.get_parameter_count(left);
        let right_count = self.get_parameter_count(right);
        let (longest, shorter, longest_count) = if left_count >= right_count {
            (left, right, left_count)
        } else {
            (right, left, right_count)
        };
        let either_has_rest =
            self.has_effective_rest_parameter(left) || self.has_effective_rest_parameter(right);
        let longest_has_rest = self.has_effective_rest_parameter(longest);
        let needs_extra_rest = either_has_rest && !longest_has_rest;
        let total = longest_count + usize::from(needs_extra_rest);
        let left_min = self.get_min_argument_count(longest);
        let right_min = self.get_min_argument_count(shorter);
        let mut params: Vec<Arc<Symbol>> = Vec::with_capacity(total);
        let mut overrides: Vec<Arc<Type>> = Vec::with_capacity(total);
        for i in 0..longest_count {
            let longest_ty = self.combined_param_type_at(longest, i);
            let shorter_ty = self.combined_param_type_at(shorter, i);
            let stripped_longest = self.strip_optional_undefined(&longest_ty);
            let stripped_shorter = self.strip_optional_undefined(&shorter_ty);
            let members: Vec<Arc<Type>> = vec![stripped_longest, stripped_shorter]
                .into_iter()
                .filter(|m| !m.flags.contains(TypeFlags::Unknown))
                .collect();
            let combined = if members.is_empty() {
                self.unknown_type()
            } else if is_union {
                self.get_intersection_type(members)
            } else {
                self.get_union_type(members)
            };
            let is_rest = either_has_rest && !needs_extra_rest && i == longest_count - 1;
            let is_optional = i >= left_min && i >= right_min;
            let left_name = self.parameter_name_at_position(left, i);
            let right_name = self.parameter_name_at_position(right, i);
            let name = if left_name == right_name {
                left_name
            } else if left_name.is_empty() {
                right_name
            } else if right_name.is_empty() {
                left_name
            } else {
                format!("arg{i}")
            };
            let mut sym_flags = SymbolFlags::FunctionScopedVariable;
            if is_optional && !is_rest {
                sym_flags.insert(SymbolFlags::Optional);
            }
            let stored = if is_rest {
                self.create_array_type(combined)
            } else {
                combined
            };
            let sym = Arc::new(Symbol::new(sym_flags, name));
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(Arc::clone(&stored)),
                    ..Default::default()
                },
            );
            params.push(sym);
            overrides.push(stored);
        }
        if needs_extra_rest {
            let elem = self.combined_param_type_at(shorter, longest_count);
            let arr = self.create_array_type(elem);
            let sym = Arc::new(Symbol::new(
                SymbolFlags::FunctionScopedVariable,
                "args".to_string(),
            ));
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(Arc::clone(&arr)),
                    ..Default::default()
                },
            );
            params.push(sym);
            overrides.push(arr);
        }
        (params, overrides)
    }

    fn combined_param_type_at(&mut self, sig: &Arc<Signature>, i: usize) -> Arc<Type> {
        self.try_get_type_at_position(sig, i)
            .unwrap_or_else(|| self.unknown_type())
    }

    fn parameter_name_at_position(&mut self, sig: &Arc<Signature>, i: usize) -> String {
        sig.parameters
            .get(i)
            .map(|p| p.name.clone())
            .unwrap_or_default()
    }
}
