#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_property_assigned_in_constructor(
        &self,
        name_node: &Arc<Node>,
        ctor: &Arc<Node>,
    ) -> bool {
        let name_text = match &name_node.data {
            crate::ast::NodeData::Identifier(d) => d.text.as_str(),
            _ => return false,
        };

        let body = match &ctor.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => &d.body,
            _ => return false,
        };
        let Some(body) = body else {
            return false;
        };
        Self::node_contains_this_assignment(body, name_text)
    }

    pub(crate) fn node_contains_this_assignment(node: &Arc<Node>, name: &str) -> bool {
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            if data.operator_token.kind == SyntaxKind::EqualsToken {
                if Self::is_this_property_access(&data.left, name) {
                    return true;
                }
            }
        }

        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if Self::node_contains_this_assignment(child, name) {
                found = true;
                return true;
            }
            false
        });
        found
    }

    pub(crate) fn is_this_property_access(node: &Arc<Node>, name: &str) -> bool {
        match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::Identifier(id) = &data.name.data {
                        return id.text == name;
                    }
                }
                false
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::StringLiteral(sl) = &data.argument_expression.data
                    {
                        return sl.text == name;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn class_member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    pub(crate) fn class_member_name_text(node: &Arc<Node>) -> Option<String> {
        if matches!(node.kind, SyntaxKind::Constructor) {
            return Some("constructor".to_string());
        }
        let name = Self::class_member_name_node(node)?;
        match name.kind {
            SyntaxKind::Identifier | SyntaxKind::NumericLiteral => {
                let text = name.text().to_string();
                if text.is_empty() { None } else { Some(text) }
            }
            SyntaxKind::StringLiteral => Some(format!("\"{}\"", name.text())),
            _ => None,
        }
    }

    pub(crate) fn class_member_has_body(node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::MethodDeclaration(d) if d.body.is_some()
        ) || matches!(
            &node.data,
            crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some()
        )
    }

    pub(crate) fn function_like_params_and_return(
        node: &Arc<Node>,
    ) -> Option<(&Arc<NodeList>, Option<&Arc<Node>>)> {
        match &node.data {
            crate::ast::NodeData::FunctionDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::MethodDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::ConstructorDeclaration(d) => Some((&d.parameters, None)),
            _ => None,
        }
    }

    pub(crate) fn overload_signature_compatible_with_implementation(
        &mut self,
        overload: &Arc<Node>,
        implementation: &Arc<Node>,
    ) -> bool {
        let Some((ov_params, ov_return)) = Self::function_like_params_and_return(overload)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };
        let Some((im_params, im_return)) = Self::function_like_params_and_return(implementation)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };

        let return_ok = match (ov_return, im_return) {
            (Some(ovn), Some(imn)) => {
                let ov_t = self.get_type_from_type_node(&ovn);
                let im_t = self.get_type_from_type_node(&imn);
                ov_t.flags.contains(TypeFlags::Void)
                    || self.is_type_assignable_to(&ov_t, &im_t)
                    || self.is_type_assignable_to(&im_t, &ov_t)
            }
            _ => true,
        };
        if !return_ok {
            return false;
        }

        let n = ov_params.len().min(im_params.len());
        for i in 0..n {
            let ov_tn = match &ov_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let im_tn = match &im_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let (Some(o), Some(m)) = (ov_tn, im_tn) else {
                continue;
            };
            let ov_t = self.get_type_from_type_node(&o);
            let im_t = self.get_type_from_type_node(&m);
            if !self.is_type_assignable_to(&ov_t, &im_t)
                && !self.is_type_assignable_to(&im_t, &ov_t)
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn check_class_member_overloads(&mut self, members: &NodeList) {
        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, m) in members.iter().enumerate() {
            if !matches!(
                m.kind,
                SyntaxKind::Constructor | SyntaxKind::MethodDeclaration
            ) {
                continue;
            }
            if let Some(name) = Self::class_member_name_text(m) {
                groups.entry(name).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {
            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let node = &members.nodes[idx];
                if !Self::class_member_has_body(node) {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_implementation_expected_error(members, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            let last = idxs[idxs.len() - 1];
            if !has_body {
                let node = &members.nodes[last];
                let exempt = node.has_syntactic_modifier(ModifierFlags::Abstract)
                    || matches!(
                        &node.data,
                        crate::ast::NodeData::MethodDeclaration(d) if d.postfix_token.is_some()
                    );
                if !exempt {
                    self.report_implementation_expected_error(members, last);
                }
            } else {
                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| Self::class_member_has_body(&members.nodes[i]))
                    .unwrap_or(last);
                let impl_node = Arc::clone(&members.nodes[impl_idx]);
                for &i in &idxs {
                    if i == impl_idx {
                        continue;
                    }
                    let overload = Arc::clone(&members.nodes[i]);
                    if !self
                        .overload_signature_compatible_with_implementation(&overload, &impl_node)
                        && let Some(name_node) =
                            crate::ast::utilities::get_name_of_declaration(&overload)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }

    pub(crate) fn report_implementation_expected_error(&mut self, members: &NodeList, idx: usize) {
        let node = Arc::clone(&members.nodes[idx]);
        let name_text = Self::class_member_name_text(&node);
        if let Some(sib) = members.nodes.get(idx + 1) {
            if sib.kind == node.kind {
                let sib_name = Self::class_member_name_text(sib);
                let same_name = match (&name_text, &sib_name) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                };

                if same_name {
                    return;
                }
                if Self::class_member_has_body(sib) {
                    let file = self.current_file.clone();
                    let loc = Self::class_member_name_node(sib)
                        .map(|n| n.loc)
                        .unwrap_or(sib.loc);
                    let display_name = name_text.unwrap_or_default();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![display_name],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }

        let file = self.current_file.clone();
        let (loc, message): (crate::core::text::TextRange, crate::diagnostics::Message) = if matches!(
            node.kind,
            SyntaxKind::Constructor
        ) {
            (
                node.loc,
                crate::diagnostics::messages_generated::CONSTRUCTOR_IMPLEMENTATION_IS_MISSING,
            )
        } else {
            (
                    Self::class_member_name_node(&node)
                        .map(|n| n.loc)
                        .unwrap_or(node.loc),
                    crate::diagnostics::messages_generated::
                        FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                )
        };
        let diagnostic = crate::ast::Diagnostic::new(file, loc, message, Vec::new());
        self.diagnostics.add(diagnostic);
    }
}
