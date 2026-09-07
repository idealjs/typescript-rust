#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_type_of_property_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;

        if t.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.get_type_of_property_of_contextual_type(&constraint, name);
        }

        if t.flags.contains(TypeFlags::Union)
            && let TypeData::Union(u) = &t.data
        {
            let found: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .filter_map(|c| self.get_type_of_property_of_contextual_type(c, name))
                .collect();
            return match found.len() {
                0 => None,
                1 => Some(found.into_iter().next().unwrap()),
                _ => {
                    let types = found;
                    if types.iter().all(|x| Arc::ptr_eq(x, &types[0])) {
                        Some(Arc::clone(&types[0]))
                    } else {
                        Some(self.get_union_type(types))
                    }
                }
            };
        }

        if matches!(&t.data, TypeData::Mapped(_))
            && let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
            && m.template_type.is_some()
            && m.name_type.is_none()
        {
            let constraint = m.constraint_type.clone()?;
            let name_literal = self.get_string_literal_type(name);

            let gate_target = self.reduced_keyof_for_contextual_gate(&constraint);
            let gate_ok = match &gate_target {
                Some(g) => self.is_type_assignable_to(&name_literal, g),
                None => {
                    if matches!(&constraint.data, TypeData::Index(idx)
                    if idx.target.as_ref().is_some_and(|tgt| {
                        tgt.flags.contains(TypeFlags::TypeParameter)
                            && self
                                .get_constraint_of_type_parameter(tgt)
                                .is_none()
                    })) {
                        true
                    } else {
                        self.is_type_assignable_to(&name_literal, &constraint)
                    }
                }
            };
            if !gate_ok {
                return None;
            }
            let tp = m.type_parameter.clone().unwrap();
            let template = m.template_type.clone().unwrap();
            let substituted =
                self.substitute_infer_type_parameters(&template, &[tp], &[name_literal]);

            if let TypeData::IndexedAccess(ia) = &substituted.data
                && let (Some(obj), Some(idx)) = (&ia.object_type, &ia.index_type)
            {
                let resolved = self.get_indexed_access_type(obj, idx);
                if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                    return Some(resolved);
                }
            }
            return Some(substituted);
        }

        if let TypeData::Conditional(c) = &t.data {
            let mut branches: Vec<Arc<Type>> = Vec::new();
            if let Some(root) = &c.root
                && let Some(node) = &root.node
                && let crate::ast::NodeData::ConditionalTypeNode(cd) = &node.data
            {
                let true_t = self.get_type_from_type_node(&cd.true_type);
                let false_t = self.get_type_from_type_node(&cd.false_type);
                for branch in [true_t, false_t] {
                    if let Some(found) = self.get_type_of_property_of_contextual_type(&branch, name)
                    {
                        branches.push(found);
                    }
                }
            }
            return match branches.len() {
                0 => None,
                1 => Some(branches.pop().unwrap()),
                _ => {
                    if branches.iter().all(|b| Arc::ptr_eq(b, &branches[0])) {
                        Some(Arc::clone(&branches[0]))
                    } else {
                        Some(self.get_union_type(branches))
                    }
                }
            };
        }

        if t.flags.contains(TypeFlags::Intersection)
            && let TypeData::Intersection(i) = &t.data
        {
            for c in &i.union_or_intersection.types {
                if let Some(found) = self.get_type_of_property_of_contextual_type(c, name) {
                    return Some(found);
                }
            }
            return None;
        }

        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }

        let name_literal = self.get_string_literal_type(name);
        if let Some(info) = self.get_applicable_index_info(t, &name_literal) {
            return info.value_type.clone();
        }
        None
    }

    pub(crate) fn reduced_keyof_for_contextual_gate(&mut self, constraint: &Arc<Type>) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;
        let TypeData::Index(idx) = &constraint.data else {
            return None;
        };
        let target = idx.target.as_ref()?;
        if !target.flags.contains(TypeFlags::TypeParameter) {
            return None;
        }
        let target_constraint = self.get_constraint_of_type_parameter(target)?;
        Some(self.get_index_type(&target_constraint))
    }

    pub fn get_contextual_type(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let parent = match &node.parent {
            Some(p) => Arc::clone(p),
            None => return None,
        };

        match parent.kind {
            SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::BindingElement => {
                self.get_contextual_type_for_initializer_expression(node, _context_flags)
            }
            SyntaxKind::ArrowFunction | SyntaxKind::ReturnStatement => {
                self.get_contextual_type_for_return_expression(node, _context_flags)
            }
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
                self.get_contextual_type_for_argument(&parent, node)
            }

            SyntaxKind::SatisfiesExpression => {
                if let crate::ast::NodeData::SatisfiesExpression(d) = &parent.data {
                    Some(self.get_type_from_type_node(&d.type_node))
                } else {
                    None
                }
            }
            SyntaxKind::BinaryExpression => {
                self.get_contextual_type_for_binary_operand(node, _context_flags)
            }
            SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
                self.get_contextual_type_for_object_literal_element(&parent, _context_flags)
            }
            SyntaxKind::ArrayLiteralExpression => {
                self.get_contextual_type_for_array_literal_element(node, &parent, _context_flags)
            }

            SyntaxKind::ParenthesizedExpression | SyntaxKind::NonNullExpression => {
                self.get_contextual_type(&parent, _context_flags)
            }
            _ => None,
        }
    }

    pub fn get_contextual_signature(
        &mut self,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let t = self.get_contextual_type(node, ContextFlags::Signature)?;
        if let TypeData::Union(u) = &t.data {
            let mut first: Option<Arc<Signature>> = None;
            for current in &u.union_or_intersection.types {
                let Some(signature) = self.get_contextual_call_signature(current, node) else {
                    continue;
                };
                match &first {
                    None => first = Some(signature),
                    Some(f) => {
                        if f.parameters.len() != signature.parameters.len() {
                            return None;
                        }
                    }
                }
            }
            return first;
        }
        self.get_contextual_call_signature(&t, node)
    }

    pub(crate) fn get_contextual_call_signature(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let signatures = self.get_signatures_of_type(t, SignatureKind::Call);
        signatures
            .into_iter()
            .find(|s| !self.is_arity_smaller(s, node))
    }

    pub(crate) fn is_arity_smaller(&self, signature: &Arc<Signature>, target: &Arc<crate::ast::Node>) -> bool {
        let Some(parameters) = function_like_parameters(target) else {
            return false;
        };
        let mut target_parameter_count: i32 = 0;
        for param in parameters.iter() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.initializer.is_some()
                || pd.question_token.is_some()
                || pd.dot_dot_dot_token.is_some()
            {
                break;
            }
            target_parameter_count += 1;
        }
        if let Some(first) = parameters.iter().next() {
            if is_this_parameter_node(first) {
                target_parameter_count -= 1;
            }
        }
        let has_effective_rest = signature.flags.contains(SignatureFlags::HasRestParameter);
        let parameter_count =
            signature.parameters.len() as i32 - if has_effective_rest { 1 } else { 0 };
        !has_effective_rest && parameter_count < target_parameter_count
    }

}
