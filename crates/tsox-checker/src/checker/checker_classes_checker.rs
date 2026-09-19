#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_class_member(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);
        self.check_node_decorators(node);

        if node.kind == SyntaxKind::Constructor {
            self.check_multiple_constructor_implementations(node);
        }

        self.check_private_name_conflicts(node);
        self.check_member_dynamic_name_grammar(node);

        self.check_static_member_class_type_params(node);

        match node.kind {
            SyntaxKind::PropertyDeclaration => {
                if let tsox_frontend::ast::NodeData::PropertyDeclaration(data) = &node.data {
                    self.check_computed_property_name(&data.name);

                    let ambient = self.ambient_context_depth > 0
                        || node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || {
                            let mut anc = node.parent();
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
                    if ambient
                        && self.no_implicit_any
                        && data.type_node.is_none()
                        && data.initializer.is_none()
                        && !self.declaration_belongs_to_private_ambient_member(node)
                        && data.name.kind == SyntaxKind::Identifier
                    {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.name.loc,
                            tsox_core::diagnostics::messages_generated::
                                MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
                            vec![data.name.text().to_string(), "any".to_string()],
                        ));
                    }

                    if node.has_syntactic_modifier(ModifierFlags::Abstract)
                        && data.initializer.is_some()
                    {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.name.loc,
                            tsox_core::diagnostics::messages_generated::
                                PROPERTY_0_CANNOT_HAVE_AN_INITIALIZER_BECAUSE_IT_IS_MARKED_ABSTRACT,
                            vec![data.name.text().to_string()],
                        ));
                    }

                    if let Some(tn) = &data.type_node {
                        self.check_type_annotation(tn);
                    }
                    if let Some(init) = &data.initializer {
                        let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                        self.this_container_stack.push(if is_static {
                            ThisContainerKind::StaticMember
                        } else {
                            ThisContainerKind::InstanceMember
                        });
                        self.check_expression(init);
                        self.this_container_stack.pop();

                        if let Some(tn) = &data.type_node {
                            let target = self.get_type_from_type_node(tn);
                            let anchor = data.name.loc;
                            self.check_contextual_elements(init, &target, anchor);
                            // Go checkVariableLikeDeclaration：声明型属性初始化器
                            // 对注解型做完整可赋值检查（TS2322 报在属性名位）
                            let init_type = self.get_type_of_node(init);
                            let mut init_node: &Arc<Node> = init;
                            while init_node.kind == SyntaxKind::ParenthesizedExpression {
                                let inner = match &init_node.data {
                                    tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                                        Some(&p.expression)
                                    }
                                    _ => None,
                                };
                                match inner {
                                    Some(i) => init_node = i,
                                    None => break,
                                }
                            }
                            if init_node.kind != SyntaxKind::ObjectLiteralExpression {
                                self.check_type_assignable_to_and_optionally_elaborate(
                                    &init_type,
                                    &target,
                                    Some(&data.name),
                                    Some(init),
                                    None,
                                    None,
                                );
                            }
                        }
                    }
                }
            }
            SyntaxKind::PropertySignature => {
                if let tsox_frontend::ast::NodeData::PropertySignatureDeclaration(data) = &node.data
                {
                    self.check_computed_property_name(&data.name);
                }
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let tsox_frontend::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data
                {
                    self.check_grammar_modifiers(node);
                    self.this_container_stack
                        .push(ThisContainerKind::StaticMember);
                    self.check_statement(&data.body);
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                self.check_class_accessor_member(node);
            }
            _ => {}
        }
    }

    // Go checkDecorators 的表达式遍历部分（签名检查/emit helpers 不在此层）；
    // NodeCanBeDecorated 失败的节点跳过（checkGrammarModifiers 已报 TS1206）
    pub(crate) fn check_node_decorators(&mut self, node: &Arc<Node>) {
        if !self.node_can_be_decorated(node) {
            return;
        }
        let Some(modifiers) = node.modifiers().cloned() else {
            return;
        };
        for modifier in modifiers.iter() {
            if modifier.kind == SyntaxKind::Decorator {
                if let tsox_frontend::ast::NodeData::Decorator(d) = &modifier.data {
                    self.check_expression(&d.expression);
                }
            }
        }
    }
}
