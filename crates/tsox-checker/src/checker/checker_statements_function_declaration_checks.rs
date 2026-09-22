#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_function_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);

        if let tsox_frontend::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                self.check_cjs_reserved_top_level_name(node, name);
            }
        }

        self.check_duplicate_function_implementations(node);

        self.check_overload_implementation_follows(node);

        // Go checkFunctionDeclaration：合并符号上检查重复实现/过载一致性
        //（每符号一次；本节点轮到其声明序列末位时由 once-guard 收敛）
        if !node.flags.contains(NodeFlags::JavaScriptFile)
            && let Some(symbol) = self.get_symbol_of_declaration(node)
        {
            self.check_function_or_constructor_symbol(&symbol);
        }
        if let tsox_frontend::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(tps) = &data.type_parameters {
                let _ = tps;
            }
            self.check_grammar_parameter_list(&data.parameters);

            self.check_parameter_property_modifiers(&data.parameters, false);

            self.check_parameter_implicit_any(node, &data.parameters, 0);
            for p in data.parameters.iter() {
                // Go checkParameterInitializer：参数默认初始化式按表达式检查
                //（内嵌箭头的隐式 any 参数由此覆盖）
                self.check_parameter_default_initializer(p);
                if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data
                    && let Some(pt) = &pd.type_node
                {
                    self.check_type_annotation(pt);
                }
            }
            if let Some(tn) = &data.type_node {
                self.check_type_annotation(tn);
                self.check_generator_return_annotation(node, tn);
            }

            if self.no_implicit_any
                && data.type_node.is_none()
                && data.body.is_none()
                && let Some(name) = &data.name
                && name.kind == SyntaxKind::Identifier
                && !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.has_parse_diagnostics)
            {
                let file = self.current_file.clone();
                let diagnostic = tsox_frontend::ast::Diagnostic::new(
                        file,
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                        vec![name.text().to_string(), "any".to_string()],
                    );
                self.diagnostics.add(diagnostic);
            }
        }

        self.check_unmatched_jsdoc_parameters(node);

        let fn_type = self.get_type_of_function_like(node);

        let fn_symbol = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(data) => {
                data.name.as_ref().and_then(|n| self.resolve_identifier(n))
            }
            _ => None,
        };
        let fn_type = match &fn_symbol {
            Some(sym) => self.attach_function_expando_type(sym, fn_type),
            None => fn_type,
        };
        self.type_node_links.get_or_default(node).resolved_type = Some(fn_type.clone());
        if let tsox_frontend::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                if let Some(symbol) = self.resolve_identifier(name) {
                    let symbol_type = match self.merged_class_function_symbol_type(&symbol) {
                        Some(t) => t,
                        None => match self.build_overload_function_type(&symbol) {
                            Some(overload_type) => {
                                self.attach_function_expando_type(&symbol, overload_type)
                            }
                            None => fn_type.clone(),
                        },
                    };
                    self.value_symbol_links
                        .get_or_default(&symbol)
                        .resolved_type = Some(symbol_type.clone());
                    self.type_node_links.get_or_default(name).resolved_type = Some(symbol_type);
                }
            }
        }

        self.push_function_scope(node);
        self.break_continue_context_stack
            .push(BreakContinueContext {
                kind: BreakContinueContextKind::Function,
                label: None,
                is_iteration: false,
            });

        let declared_return = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(data) => {
                let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                // Go checkSignatureDeclaration：生成器注解返回 void 报 TS2505
                if data.asterisk_token.is_some()
                    && let Some(tn) = data.type_node.as_ref()
                {
                    let raw = self.get_type_from_type_node(tn);
                    if raw.flags.contains(TypeFlags::Void) {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            tn.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_GENERATOR_CANNOT_HAVE_A_VOID_TYPE_ANNOTATION,
                            vec![],
                        ));
                    }
                }
                // Go checkAsyncFunctionReturnType：async 非生成器注解非全局
                // Promise 引用报 TS1064，Promise 内 thenable 成员报 TS1058
                if is_async && data.asterisk_token.is_none()
                    && let Some(tn) = data.type_node.as_ref()
                {
                    self.check_async_function_return_type(node, tn);
                }
                data.type_node
                    .as_ref()
                    .map(|tn| self.get_type_from_type_node(tn))
                    .map(|t| self.unwrap_async_return_type(t, is_async))
            }
            _ => None,
        };
        self.return_type_stack.push(declared_return.clone());
        self.in_ctor_body_stack.push(false);

        self.this_container_stack
            .push(ThisContainerKind::PlainFunction);
        if let tsox_frontend::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(body) = &data.body {
                self.check_statement(body);
            }
        }
        self.this_container_stack.pop();

        // tsgo 对生成器不发 TS2355/TS6039（GetFunctionFlags 对函数声明返回
        // Invalid 的可观察行为：声明返回类型按生成器语义解包后不再走到
        // must-return 检查），oracle 实证见 corpus-fix-session-2
        let is_generator = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.asterisk_token.is_some(),
            _ => false,
        };
        if is_generator {
            self.return_type_stack.pop();
            self.in_ctor_body_stack.pop();
            self.break_continue_context_stack.pop();
            self.pop_function_scope();
            return;
        }
        if let tsox_frontend::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(tn) = data.type_node.as_ref() {
                self.check_all_code_paths_annotated(node, tn);
            } else {
                self.check_no_implicit_returns(node, None);
            }
        }
        self.return_type_stack.pop();
        self.in_ctor_body_stack.pop();
        self.break_continue_context_stack.pop();
        self.pop_function_scope();
    }
}
