#![allow(unused_imports)]

use crate::checker::checker_classes::*;
use crate::checker::{RelationKind, SignatureCheckMode, SignatureKind};

impl Checker {
    pub(crate) fn is_property_assigned_in_constructor(
        &self,
        name_node: &Arc<Node>,
        ctor: &Arc<Node>,
    ) -> bool {
        let name_text = match &name_node.data {
            tsox_frontend::ast::NodeData::Identifier(d) => d.text.as_str(),
            tsox_frontend::ast::NodeData::PrivateIdentifier(d) => d.text.as_str(),
            _ => return false,
        };

        let body = match &ctor.data {
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => &d.body,
            _ => return false,
        };
        let Some(body) = body else {
            return false;
        };
        Self::node_contains_this_assignment(body, name_text)
    }

    pub(crate) fn node_contains_this_assignment(node: &Arc<Node>, name: &str) -> bool {
        if let tsox_frontend::ast::NodeData::BinaryExpression(data) = &node.data {
            if data.operator_token.kind == SyntaxKind::EqualsToken {
                if Self::is_this_property_access(&data.left, name) {
                    return true;
                }
                // 解构赋值目标（Go isPropertyInitializedInConstructor 走流分析，
                // 模式内任意 this.<name> 出现位即赋值目标）
                if matches!(
                    data.left.kind,
                    SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression
                ) && Self::contains_this_property_reference(&data.left, name)
                {
                    return true;
                }
            }
        }

        let mut found = false;
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            if Self::node_contains_this_assignment(child, name) {
                found = true;
                return true;
            }
            false
        });
        found
    }

    fn contains_this_property_reference(node: &Arc<Node>, name: &str) -> bool {
        if Self::is_this_property_access(node, name) {
            return true;
        }
        let mut found = false;
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            if Self::contains_this_property_reference(child, name) {
                found = true;
                return true;
            }
            false
        });
        found
    }

    pub(crate) fn is_this_property_access(node: &Arc<Node>, name: &str) -> bool {
        match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let tsox_frontend::ast::NodeData::Identifier(id) = &data.name.data {
                        return id.text == name;
                    }
                    if let tsox_frontend::ast::NodeData::PrivateIdentifier(id) = &data.name.data {
                        return id.text == name;
                    }
                }
                false
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let tsox_frontend::ast::NodeData::StringLiteral(sl) =
                        &data.argument_expression.data
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
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
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
            tsox_frontend::ast::NodeData::MethodDeclaration(d) if d.body.is_some()
        ) || matches!(
            &node.data,
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some()
        )
    }

    pub(crate) fn function_like_params_and_return(
        node: &Arc<Node>,
    ) -> Option<(&Arc<NodeList>, Option<&Arc<Node>>)> {
        match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => Some((&d.parameters, None)),
            _ => None,
        }
    }

    pub(crate) fn overload_signature_compatible_with_implementation(
        &mut self,
        overload: &Arc<Node>,
        implementation: &Arc<Node>,
    ) -> bool {
        let (Some(impl_sig), Some(ov_sig)) = (
            self.signature_of_declaration_node(implementation),
            self.signature_of_declaration_node(overload),
        ) else {
            return true;
        };
        let erased_impl = self.get_erased_signature(&impl_sig);
        let erased_ov = self.get_erased_signature(&ov_sig);
        let source_ret = self
            .get_return_type_of_signature(&erased_impl)
            .unwrap_or_else(|| self.get_any_type());
        let target_ret = self
            .get_return_type_of_signature(&erased_ov)
            .unwrap_or_else(|| self.get_any_type());
        if target_ret.flags.contains(TypeFlags::Void)
            || self.is_type_assignable_to(&target_ret, &source_ret)
            || self.is_type_assignable_to(&source_ret, &target_ret)
        {
            return self
                .compare_signatures_related(
                    &erased_impl,
                    &erased_ov,
                    SignatureCheckMode::IgnoreReturnTypes,
                    RelationKind::Assignable,
                )
                .is_true();
        }
        false
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
                        tsox_frontend::ast::NodeData::MethodDeclaration(d) if d.postfix_token.is_some()
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
                    {
                        // Go checkFunctionOrAccessorPropertyDeclarationImplementation：
                        // 首个不兼容重载报 2394（构造子落在整个声明）并中止
                        let file = self.current_file.clone();
                        let mut diag = tsox_frontend::ast::Diagnostic::new(
                            file.clone(),
                            overload.loc,
                            tsox_core::diagnostics::messages_generated::
                                THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                            Vec::new(),
                        );
                        diag.related_information.push(tsox_frontend::ast::Diagnostic::new(
                            file,
                            impl_node.loc,
                            tsox_core::diagnostics::messages_generated::
                                THE_IMPLEMENTATION_SIGNATURE_IS_DECLARED_HERE,
                            Vec::new(),
                        ));
                        self.diagnostics.add(diag);
                        break;
                    }
                }
            }
        }
    }

    // Go getSignatureFromDeclaration：直接从声明构建签名，不经符号签名表
    // （实现签名在 getSignaturesOfSymbol 语义下被剔除，表内查不到）
    fn signature_of_declaration_node(&mut self, node: &Arc<Node>) -> Option<Arc<Signature>> {
        if node.kind == SyntaxKind::Constructor {
            let class_node = node.parent()?;
            let owner_symbol = self.program.symbol_map().symbol_of(&class_node).cloned()?;
            let instance_type = self.get_declared_type_of_symbol(&owner_symbol);
            let params = match &node.data {
                tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => &d.parameters,
                _ => return None,
            };
            return Some(self.build_signature_from_function_like_type_node(
                params,
                instance_type,
                true,
                None,
                Some(Arc::clone(node)),
            ));
        }
        let (parameters, type_node) = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => {
                (&d.parameters, d.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => {
                (&d.parameters, d.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::MethodSignatureDeclaration(d) => {
                (&d.parameters, d.type_node.as_ref())
            }
            _ => return None,
        };
        let return_type = match type_node {
            Some(tn) => self.get_type_from_type_node(tn),
            None => self.get_any_type(),
        };
        Some(self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
            false,
            None,
            Some(Arc::clone(node)),
        ))
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
                    let diagnostic = tsox_frontend::ast::Diagnostic::new(
                        file,
                        loc,
                        tsox_core::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![display_name],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }

        let file = self.current_file.clone();
        let (loc, message): (
            tsox_core::core::text::TextRange,
            tsox_core::diagnostics::Message,
        ) = if matches!(node.kind, SyntaxKind::Constructor) {
            (
                node.loc,
                tsox_core::diagnostics::messages_generated::CONSTRUCTOR_IMPLEMENTATION_IS_MISSING,
            )
        } else {
            (
                    Self::class_member_name_node(&node)
                        .map(|n| n.loc)
                        .unwrap_or(node.loc),
                    tsox_core::diagnostics::messages_generated::
                        FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                )
        };
        let diagnostic = tsox_frontend::ast::Diagnostic::new(file, loc, message, Vec::new());
        self.diagnostics.add(diagnostic);
    }
}
