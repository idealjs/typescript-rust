#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_assertion_overlap(
        &mut self,
        node: &Arc<Node>,
        expr: &Arc<Node>,
        type_node: &Arc<Node>,
    ) {
        if type_node.kind == SyntaxKind::TypeReference && type_node.text() == "const" {
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        let target_type = self.get_type_from_type_node(type_node);
        let error_type = self.error_type();
        let exempt = |t: &Arc<Type>| {
            Arc::ptr_eq(t, &error_type)
                || t.flags.contains(TypeFlags::Any)
                || t.flags.contains(TypeFlags::Unknown)
                || t.flags.contains(TypeFlags::Never)
        };
        if exempt(&expr_type) || exempt(&target_type) {
            return;
        }
        let expr_base = if crate::checker::is_literal_type(&expr_type) {
            self.get_base_type_of_literal_type(&expr_type)
        } else {
            expr_type
        };

        let comparable = self.is_type_comparable_to(&expr_base, &target_type)
            || self.is_type_comparable_to(&target_type, &expr_base);
        if !comparable {
            let source_str = self.type_to_string(&expr_base);
            let target_str = self.type_to_string(&target_type);
            let file = self.current_file.clone();
            let mut diag = crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST,
                vec![source_str, target_str],
            );

            if let Some((prop_loc, prop_name, elem_target_str)) =
                self.assertion_excess_detail(&expr, &expr_base, &target_type)
            {
                diag.loc = prop_loc;
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    prop_loc,
                    crate::diagnostics::messages_generated::
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![prop_name, elem_target_str],
                ));
            }
            self.diagnostics.add(diag);
        }
    }

    pub(crate) fn assertion_excess_detail(
        &mut self,
        expr: &Arc<Node>,
        expr_type: &Arc<Type>,
        target_type: &Arc<Type>,
    ) -> Option<(TextRange, String, String)> {
        let (elem_source, elem_target, literal_node) = match &expr.data {
            NodeData::ObjectLiteralExpression(_) => (
                Arc::clone(expr_type),
                Arc::clone(target_type),
                Arc::clone(expr),
            ),
            NodeData::ArrayLiteralExpression(d) => {
                let first_obj = d
                    .elements
                    .iter()
                    .find(|e| matches!(&e.data, NodeData::ObjectLiteralExpression(_)))?;
                let st = self.element_type_of(expr_type)?;
                let tt = self.element_type_of(target_type)?;
                (st, tt, Arc::clone(first_obj))
            }
            _ => return None,
        };
        let prop_name = self.get_excess_property_name(&elem_source, &elem_target)?;
        let prop_loc = self.find_object_literal_property_name_node(&literal_node, &prop_name)?;
        let elem_target_str = self.type_to_string(&elem_target);
        Some((prop_loc, prop_name, elem_target_str))
    }

    pub(crate) fn element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Object) {
            if let TypeData::Object(obj) = &t.data
                && !obj.type_arguments.is_empty()
            {
                return Some(Arc::clone(&obj.type_arguments[0]));
            }
        }
        None
    }

    pub(crate) fn check_accessor_in_type_context(&mut self, member: &Arc<Node>) {
        let body = match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
            crate::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
            _ => return,
        };
        if let Some(body) = body {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                body.loc,
                crate::diagnostics::messages_generated::
                    AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                vec![],
            ));
        }
    }

    pub(crate) fn check_interface_members(&mut self, members: &NodeList) {
        {
            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                std::collections::HashMap::new();
            for member in members.iter() {
                if let Some(name_node) = member.name() {
                    let name = match name_node.kind {
                        SyntaxKind::StringLiteral
                        | SyntaxKind::NumericLiteral
                        | SyntaxKind::Identifier
                        | SyntaxKind::PrivateIdentifier => name_node.text().to_string(),
                        _ => continue,
                    };
                    seen.entry(name).or_default().push(member);
                }
            }
            for (_, group) in seen.iter() {
                let all_methods = group.iter().all(|m| m.kind == SyntaxKind::MethodSignature);
                let accessor_pair = group
                    .iter()
                    .all(|m| matches!(m.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor))
                    && group.iter().any(|m| m.kind == SyntaxKind::GetAccessor)
                    && group.iter().any(|m| m.kind == SyntaxKind::SetAccessor);
                if group.len() > 1 && !all_methods && !accessor_pair {
                    for m in group {
                        if let Some(name_node) = m.name() {
                            let name = name_node.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_node.loc,
                                crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                vec![name],
                            ));
                        }
                    }
                }
            }
        }
        for member in members.iter() {
            match member.kind {
                SyntaxKind::MethodSignature => {
                    let crate::ast::NodeData::MethodSignatureDeclaration(d) = &member.data else {
                        continue;
                    };
                    self.check_parameter_property_modifiers(&d.parameters, false);
                    self.check_parameter_implicit_any(member, &d.parameters, 0);
                    for p in d.parameters.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &d.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && d.type_node.is_none()
                        && d.name.kind == SyntaxKind::Identifier
                    {
                        let file = self.current_file.clone();
                        let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            d.name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![d.name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::ConstructSignature | SyntaxKind::CallSignature => {
                    let (params, type_node) = match &member.data {
                        crate::ast::NodeData::ConstructSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        crate::ast::NodeData::CallSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        _ => continue,
                    };
                    self.check_parameter_property_modifiers(params, false);
                    self.check_parameter_implicit_any(member, params, 0);
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any && type_node.is_none() {
                        let message = if member.kind == SyntaxKind::ConstructSignature {
                            crate::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        } else {
                            crate::diagnostics::messages_generated::
                                CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        };
                        let file = self.current_file.clone();
                        let diagnostic =
                            crate::ast::Diagnostic::new(file, member.loc, message, vec![]);
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::PropertySignature => {
                    if let crate::ast::NodeData::PropertySignatureDeclaration(d) = &member.data {
                        self.check_type_annotation(&d.type_node);
                    }
                }

                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    self.check_accessor_in_type_context(member);
                }
                _ => {}
            }
        }
    }
}
