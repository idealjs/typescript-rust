#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_parameter_implicit_any(
        &mut self,
        node: &Arc<Node>,
        params: &NodeList,
        contextual_param_count: usize,
    ) {
        if !self.no_implicit_any {
            return;
        }
        for (i, param) in params.iter().enumerate() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.type_node.is_some() || pd.initializer.is_some() {
                continue;
            }
            let name = &pd.name;
            if name.kind == SyntaxKind::ObjectBindingPattern
                || name.kind == SyntaxKind::ArrayBindingPattern
            {
                if !self.declaration_belongs_to_private_ambient_member(param)
                    && self.contextual_type_of_parameter(param).is_none()
                {
                    self.report_implicit_any_binding_elements(name);
                }
                continue;
            }
            if name.kind != SyntaxKind::Identifier || name.text() == "this" {
                continue;
            }

            if i < contextual_param_count {
                continue;
            }

            if self.param_has_typed_jsdoc_tag(node, name.text()) {
                continue;
            }
            if self.declaration_belongs_to_private_ambient_member(param) {
                continue;
            }
            if self.contextual_type_of_parameter(param).is_some() {
                continue;
            }
            if matches!(
                node.kind,
                SyntaxKind::FunctionType | SyntaxKind::MethodSignature | SyntaxKind::CallSignature
            ) && self.parameter_name_resolves_as_type(name)
            {
                let name_text = name.text().to_string();
                let arg_name = format!("arg{i}");
                let type_name = format!(
                    "{name_text}{}",
                    if pd.dot_dot_dot_token.is_some() { "[]" } else { "" }
                );
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    param.loc,
                    tsox_core::diagnostics::messages_generated::
                        PARAMETER_HAS_A_NAME_BUT_NO_TYPE_DID_YOU_MEAN_0_COLON_1,
                    vec![arg_name, type_name],
                ));
                continue;
            }
            let file = self.current_file.clone();
            let name_text = if name.text().is_empty() {
                "(Missing)".to_string()
            } else {
                name.text().to_string()
            };
            let diagnostic = if pd.dot_dot_dot_token.is_some() {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    param.loc,
                    tsox_core::diagnostics::messages_generated::
                        REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE,
                    vec![name_text],
                )
            } else {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    param.loc,
                    tsox_core::diagnostics::messages_generated::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE,
                    vec![name_text, "any".to_string()],
                )
            };
            self.diagnostics.add(diagnostic);
        }
    }

    // Go getTypeForBindingElement → widenTypeForVariableLikeDeclaration(nil, elem, true)：
    // 根声明无类型来源（无初始化式/无注解/无上下文）时，模式内无初始化式的
    // 绑定元素逐个报 TS7031（嵌套模式递归到最内层标识符）
    pub(crate) fn report_implicit_any_binding_elements(&mut self, pattern: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::BindingPattern(bp) = &pattern.data else {
            return;
        };
        for elem in bp.elements.iter() {
            let tsox_frontend::ast::NodeData::BindingElement(be) = &elem.data else {
                continue;
            };
            let Some(elem_name) = &be.name else { continue };
            if be.initializer.is_some() {
                continue;
            }
            if elem_name.kind == SyntaxKind::ObjectBindingPattern
                || elem_name.kind == SyntaxKind::ArrayBindingPattern
            {
                self.report_implicit_any_binding_elements(elem_name);
                continue;
            }
            if elem_name.kind != SyntaxKind::Identifier {
                continue;
            }
            let file = self.current_file.clone();
            let name_text = elem_name.text().to_string();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                elem_name.loc,
                tsox_core::diagnostics::messages_generated::BINDING_ELEMENT_0_IMPLICITLY_HAS_AN_1_TYPE,
                vec![name_text, "any".to_string()],
            ));
        }
    }

    pub(crate) fn declaration_belongs_to_private_ambient_member(&self, decl: &Arc<Node>) -> bool {
        let mut member = Arc::clone(decl);
        loop {
            match member.kind {
                SyntaxKind::BindingElement | SyntaxKind::VariableDeclaration => {
                    let Some(parent) = member.parent() else { break };
                    member = parent;
                }
                _ => break,
            }
        }
        if member.kind == SyntaxKind::Parameter
            && let Some(parent) = member.parent()
        {
            member = parent;
        }
        let is_private = member.has_syntactic_modifier(ModifierFlags::Private)
            || member
                .name()
                .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier);
        let is_ambient = self.ambient_context_depth > 0
            || member.has_syntactic_modifier(ModifierFlags::Ambient)
            || {
                let mut anc = member.parent();
                let mut found = false;
                while let Some(a) = anc {
                    if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                        found = true;
                        break;
                    }
                    anc = a.parent();
                }
                found
            }
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        is_private && is_ambient
    }

    fn parameter_name_resolves_as_type(&self, name: &Arc<Node>) -> bool {
        const TYPE_KEYWORD_NAMES: &[&str] = &[
            "any", "unknown", "never", "void", "undefined", "string", "number", "boolean",
            "bigint", "object", "symbol",
        ];
        let text = name.text();
        if TYPE_KEYWORD_NAMES.contains(&text) {
            return true;
        }
        self.resolve_identifier_with_meaning(name, SymbolFlags::TYPE)
            .is_some()
    }

    pub(crate) fn param_has_typed_jsdoc_tag(&self, node: &Arc<Node>, param_name: &str) -> bool {
        let Some(file) = &self.current_file else {
            return false;
        };
        for jsdoc in file.resolve_jsdoc(node) {
            let tsox_frontend::ast::NodeData::JSDoc(d) = &jsdoc.data else {
                continue;
            };
            let Some(tags) = &d.tags else { continue };
            for tag in tags.iter() {
                if let tsox_frontend::ast::NodeData::JSDocParameterOrPropertyTag(td) = &tag.data
                    && td.name.kind == SyntaxKind::Identifier
                    && td.name.text() == param_name
                    && td.type_expression.is_some()
                {
                    return true;
                }
            }
        }
        false
    }
}
