#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_contextual_type_for_argument(
        &mut self,
        call_node: &Arc<crate::ast::Node>,
        arg_node: &Arc<crate::ast::Node>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let args = match &call_node.data {
            NodeData::CallExpression(data) => Some(&data.arguments),
            NodeData::NewExpression(data) => data.arguments.as_ref(),
            _ => None,
        }?;

        let arg_index = args.iter().position(|a| Arc::ptr_eq(a, arg_node))?;

        let is_new = matches!(&call_node.data, NodeData::NewExpression(_));
        let expression_type = match &call_node.data {
            NodeData::CallExpression(data) => Some(self.get_type_of_node(&data.expression)),
            NodeData::NewExpression(data) => Some(self.get_type_of_node(&data.expression)),
            _ => None,
        }?;
        let kind = if is_new {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        let signatures = self.get_signatures_of_type(&expression_type, kind);

        let sig = signatures
            .iter()
            .find(|s| s.parameters.len() > arg_index)
            .or_else(|| signatures.first())?
            .clone();

        if arg_index >= sig.parameters.len() {
            return None;
        }

        let base_param_type = self
            .signature_instantiated_param_type(&sig, arg_index)
            .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[arg_index]));

        if !sig.type_parameters.is_empty() {
            let key = call_node.id();
            if self.resolving_contextual_calls.insert(key) {
                let sibling_args: Vec<Arc<crate::ast::Node>> = args
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| {
                        !matches!(
                            a.kind,
                            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
                        )
                    })
                    .map(|(_, a)| Arc::clone(a))
                    .collect();
                let inferred = self.infer_call_type_arguments(call_node, &sig, &sibling_args);
                self.resolving_contextual_calls.remove(&key);
                if !inferred.is_empty() {
                    return Some(self.substitute_infer_type_parameters(
                        &base_param_type,
                        &sig.type_parameters,
                        &inferred,
                    ));
                }
            }
        }
        Some(base_param_type)
    }

    pub(crate) fn get_contextual_type_for_binary_operand(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let parent = node.parent.as_ref()?;
        let binary = match &parent.data {
            NodeData::BinaryExpression(data) => data,
            _ => return None,
        };

        if !Arc::ptr_eq(node, &binary.right) {
            return None;
        }

        match binary.operator_token.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => self
                .assignment_target_type(&binary.left)
                .or_else(|| Some(self.get_type_of_node(&binary.left))),
            SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => {
                let binary_ctx = self.get_contextual_type(&parent, _context_flags);
                if Arc::ptr_eq(node, &binary.right) && binary_ctx.is_none() {
                    return Some(self.get_type_of_node(&binary.left));
                }
                binary_ctx
            }
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::CommaToken => {
                self.get_contextual_type(&parent, _context_flags)
            }
            _ => None,
        }
    }

    pub(crate) fn get_contextual_type_for_object_literal_element(
        &mut self,
        node: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;

        let object_literal = node.parent.as_ref()?;

        let contextual_type = self.get_contextual_type(object_literal, _context_flags)?;

        let name = match &node.data {
            NodeData::PropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                NodeData::StringLiteral(s) => Some(s.text.clone()),
                _ => None,
            },
            NodeData::ShorthandPropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                _ => None,
            },
            _ => None,
        }?;

        self.get_type_of_property_of_contextual_type(&contextual_type, &name)
    }

    pub(crate) fn get_contextual_type_for_array_literal_element(
        &mut self,
        _node: &crate::ast::Node,
        parent: &Arc<crate::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let contextual_type = self.get_contextual_type(parent, _context_flags)?;

        let type_args = self.get_type_arguments(&contextual_type);
        if !type_args.is_empty() {
            return Some(Arc::clone(&type_args[0]));
        }

        if let Some(structured) = contextual_type.as_structured() {
            for index_info in &structured.index_infos {
                if let Some(ref value_type) = index_info.value_type {
                    return Some(Arc::clone(value_type));
                }
            }
        }

        None
    }

    pub(crate) fn get_covariant_inference(
        &mut self,
        inference: &InferenceInfo,
        _signature: &Arc<Signature>,
    ) -> Option<Arc<Type>> {
        if inference.candidates.is_empty() {
            return None;
        }
        let candidates = self.union_object_and_array_literal_candidates(&inference.candidates);
        let primitive_constraint = self.has_primitive_constraint(&inference.type_parameter)
            || self.is_const_type_variable(&inference.type_parameter, 0);
        let widen_literal_types = !primitive_constraint
            && inference.top_level
            && (inference.is_fixed
                || !self.is_type_parameter_at_top_level_in_return_type(
                    _signature,
                    &inference.type_parameter,
                ));
        let base_candidates: Vec<Arc<Type>> = if primitive_constraint {
            candidates
                .iter()
                .map(|t| self.get_regular_type_of_literal_type(t))
                .collect()
        } else if widen_literal_types {
            candidates
                .iter()
                .map(|t| self.get_widened_literal_type(t))
                .collect()
        } else {
            candidates
        };
        let unwidened_type = if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            self.get_union_type(base_candidates)
        } else {
            self.get_common_supertype(&base_candidates)
        };
        Some(self.get_widened_type(&unwidened_type))
    }

    pub(crate) fn get_contravariant_inference(&mut self, inference: &InferenceInfo) -> Option<Arc<Type>> {
        if inference.contra_candidates.is_empty() {
            return None;
        }
        if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            Some(self.get_intersection_type(inference.contra_candidates.clone()))
        } else {
            Some(self.get_common_subtype(&inference.contra_candidates))
        }
    }

    pub(crate) fn union_object_and_array_literal_candidates(
        &self,
        candidates: &[Arc<Type>],
    ) -> Vec<Arc<Type>> {
        if candidates.len() > 1 {
            let object_literals: Vec<Arc<Type>> = candidates
                .iter()
                .filter(|t| self.is_object_or_array_literal_type(t))
                .cloned()
                .collect();
            if !object_literals.is_empty() {
                let literals_type = self.create_union_type(object_literals);
                let non_literal_types: Vec<Arc<Type>> = candidates
                    .iter()
                    .filter(|t| !self.is_object_or_array_literal_type(t))
                    .cloned()
                    .collect();
                let mut result = non_literal_types;
                result.push(literals_type);
                return result;
            }
        }
        candidates.to_vec()
    }

    pub(crate) fn has_primitive_constraint(&self, t: &Arc<Type>) -> bool {
        let constraint = self.get_constraint_of_type_parameter(t);
        if let Some(constraint) = constraint {
            let c = if constraint.flags.contains(TypeFlags::Conditional) {
                self.get_default_constraint_of_conditional_type(&constraint)
            } else {
                Some(constraint)
            };
            if let Some(c) = c {
                return self.maybe_type_of_kind(
                    &c,
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::BigInt
                        | TypeFlags::Boolean
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::Index
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                );
            }
        }
        false
    }

    pub(crate) fn is_type_parameter_at_top_level(&self, t: &Type, tp: &Type, depth: i32) -> bool {
        if crate::checker::utilities::type_parameters_match(t, tp) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union | TypeFlags::Intersection) {
            if let Some(types) = t.types() {
                return types
                    .iter()
                    .any(|tt| self.is_type_parameter_at_top_level(tt, tp, depth));
            }
        }
        false
    }

}
