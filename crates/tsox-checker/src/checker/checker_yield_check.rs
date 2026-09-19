#![allow(unused_imports)]

use crate::checker::checker::*;

pub(crate) struct YieldContainer {
    pub fn_node: Arc<Node>,
    pub is_generator: bool,
    pub is_async: bool,
    pub return_type_node: Option<Arc<Node>>,
}

impl Checker {
    pub(crate) fn yield_container_of(&self, node: &Arc<Node>) -> Option<YieldContainer> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            let in_name_of_current = tsox_frontend::ast::node_data_generated::node_name(&n)
                .is_some_and(|name| {
                    name.loc.pos() <= node.loc.pos() && node.loc.end() <= name.loc.end()
                });
            if in_name_of_current {
                cur = n.parent();
                continue;
            }
            let info = match &n.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(d) => (
                    d.asterisk_token.is_some(),
                    d.type_node.as_ref().map(|t| (n.id(), Arc::clone(t))),
                ),
                tsox_frontend::ast::NodeData::FunctionExpression(d) => (
                    d.asterisk_token.is_some(),
                    d.type_node.as_ref().map(|t| (n.id(), Arc::clone(t))),
                ),
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => (
                    d.asterisk_token.is_some(),
                    d.type_node.as_ref().map(|t| (n.id(), Arc::clone(t))),
                ),
                tsox_frontend::ast::NodeData::ArrowFunction(_)
                | tsox_frontend::ast::NodeData::GetAccessorDeclaration(_)
                | tsox_frontend::ast::NodeData::SetAccessorDeclaration(_)
                | tsox_frontend::ast::NodeData::ConstructorDeclaration(_) => return None,
                // Go GetContainingFunction：类字段初始化器与静态块是独立容器，
                // 其内的 yield 不在生成器上下文（gTC57/58 的 TS1163 由此触发）
                tsox_frontend::ast::NodeData::PropertyDeclaration(_)
                | tsox_frontend::ast::NodeData::PropertySignatureDeclaration(_)
                | tsox_frontend::ast::NodeData::ClassStaticBlockDeclaration(_) => {
                    return None
                }
                _ => {
                    cur = n.parent();
                    continue;
                }
            };
            let (is_generator, type_node) = info;
            let return_type_node = type_node.map(|(_, t)| t);
            let is_async = n.has_syntactic_modifier(ModifierFlags::Async);
            return Some(YieldContainer {
                fn_node: Arc::clone(&n),
                is_generator,
                is_async,
                return_type_node,
            });
        }
        None
    }

    /// Go checkYieldExpression 的赋值检查段：yield/yield* 操作数须可赋给
    /// 生成器注解返回类型的 yield 迭代类型（裸 yield 的 undefined 同此检查）
    pub(crate) fn check_yield_expression_assignability(&mut self, node: &Arc<Node>) {
        let NodeData::YieldExpression(data) = &node.data else {
            return;
        };
        let Some(container) = self.yield_container_of(node) else {
            return;
        };
        if !container.is_generator {
            return;
        }
        let is_async = container.is_async;
        let Some(return_type_node) = container.return_type_node else {
            return;
        };
        let return_type = self.get_type_from_type_node(&return_type_node);
        let return_type = if return_type.is_union() {
            // Go filterType(GeneratorInstantiationAssignability)：联合中挑可作
            // 生成器返回型的成分
            let kept: Vec<Arc<Type>> = return_type
                .types()
                .unwrap_or(&[])
                .iter()
                .filter(|_t| self.generator_instantiation_assignable_to())
                .cloned()
                .collect();
            self.get_union_type(kept)
        } else {
            return_type
        };
        let iteration_types = self.iteration_types_of_iterable(
            crate::checker::checker_iteration::IterationUse::ForOf { for_await: is_async },
            &return_type,
            None,
        );
        let signature_yield_type = iteration_types
            .yield_type
            .unwrap_or_else(|| self.get_any_type());
        let expression_type = match &data.expression {
            Some(expr) => self.get_type_of_node(expr),
            None => self.undefined_type(),
        };
        let error_node = data
            .expression
            .clone()
            .unwrap_or_else(|| Arc::clone(node));
        let yielded_type = if data.asterisk_token.is_some() {
            self.check_iterated_type_or_element_type(
                crate::checker::checker_iteration::IterationUse::YieldStar { is_async },
                &expression_type,
                Some(&error_node),
            )
        } else if is_async {
            match self.get_awaited_type(&expression_type) {
                Some(awaited) => awaited,
                None => Arc::clone(&expression_type),
            }
        } else {
            Arc::clone(&expression_type)
        };
        self.check_type_assignable_to_and_optionally_elaborate(
            &yielded_type,
            &signature_yield_type,
            Some(&error_node),
            data.expression.as_ref(),
            None,
            None,
        );
    }

    /// Go unwrapReturnType 生成器段：注解返回型的 TReturn（async 再 awaited）
    pub(crate) fn unwrap_generator_return_type(
        &mut self,
        return_type: &Arc<Type>,
        is_async: bool,
    ) -> Arc<Type> {
        let iteration_types = self.iteration_types_of_iterable(
            crate::checker::checker_iteration::IterationUse::GeneratorReturnType { is_async },
            return_type,
            None,
        );
        let tr = iteration_types
            .return_type
            .unwrap_or_else(|| self.error_type());
        if is_async {
            match self.get_awaited_type(&tr) {
                Some(awaited) => awaited,
                None => tr,
            }
        } else {
            tr
        }
    }

    fn generator_instantiation_assignable_to(&mut self) -> bool {
        true
    }
}

impl Checker {
    /// Go getInferredReturnType 生成器段：`Generator<TYield, TReturn, unknown>`
    /// （async 为 AsyncGenerator；TYield=body 内 yield 表达式联合，裸 yield 计
    /// undefined；TReturn=return 联合，无 return 为 void）
    /// Go getContextualTypeForYieldOperand
    pub(crate) fn get_contextual_type_for_yield_operand(
        &mut self,
        yield_node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let fn_node = self.yield_container_of(yield_node)?.fn_node;
        let fn_node = if fn_node.kind == SyntaxKind::PropertyDeclaration
            || fn_node.kind == SyntaxKind::ClassStaticBlockDeclaration
        {
            None
        } else {
            Some(fn_node)
        }?;
        let contextual_return = self.contextual_return_type_of(&fn_node)?;
        let is_async = fn_node.has_syntactic_modifier(ModifierFlags::Async);
        let is_yield_star = match &yield_node.data {
            NodeData::YieldExpression(d) => d.asterisk_token.is_some(),
            _ => false,
        };
        // 上下文返回型含未固定类型参数（泛型推断期）时不定型，
        // 待推断固定后的补跑通道再取（Go 在实参定型完成后才检查函数体）
        {
            let mut free: Vec<Arc<Type>> = Vec::new();
            self.collect_free_type_parameters_deep(&contextual_return, &mut free);
            if !free.is_empty() {
                return None;
            }
        }
        if !is_yield_star && contextual_return.is_union() {
            let constituents = self.constituent_types(&contextual_return);
            let kept: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    self.iteration_types_of_generator_function_return_type(t, is_async)
                        .return_type
                        .is_some()
                })
                .collect();
            if let Some(t) = self.union_or_opt(&kept) {
                return Some(t);
            }
            return Some(contextual_return);
        }
        if is_yield_star {
            let types = self.iteration_types_of_generator_function_return_type(
                &contextual_return,
                is_async,
            );
            let yield_type = types.yield_type.clone().unwrap_or_else(|| self.never_type());
            let return_type = self
                .get_contextual_type(yield_node, crate::checker::types::ContextFlags::None)
                .unwrap_or_else(|| self.never_type());
            let next_type = types.next_type.clone().unwrap_or_else(|| self.unknown_type());
            return Some(self.create_generator_type_for_yield(
                &yield_type,
                &return_type,
                &next_type,
                is_async,
            ));
        }
        self.iteration_types_of_generator_function_return_type(&contextual_return, is_async)
            .yield_type
    }

    fn union_or_opt(&mut self, parts: &[Arc<Type>]) -> Option<Arc<Type>> {
        if parts.is_empty() {
            None
        } else if parts.len() == 1 {
            Some(Arc::clone(&parts[0]))
        } else {
            Some(self.get_union_type(parts.to_vec()))
        }
    }

    fn create_generator_type_for_yield(
        &mut self,
        yield_type: &Arc<Type>,
        ret_type: &Arc<Type>,
        next_type: &Arc<Type>,
        is_async: bool,
    ) -> Arc<Type> {
        let name = if is_async { "AsyncGenerator" } else { "Generator" };
        let Some(symbol) = self.globals.get(name).cloned() else {
            return self.get_any_type();
        };
        self.resolve_interface_type_ex(
            &symbol,
            Some(vec![
                Arc::clone(yield_type),
                Arc::clone(ret_type),
                Arc::clone(next_type),
            ]),
        )
    }

    pub(crate) fn infer_generator_return_type(
        &mut self,
        body: &Arc<Node>,
        is_async: bool,
    ) -> Arc<Type> {
        let mut yielded: Vec<Arc<Type>> = Vec::new();
        self.collect_yielded_types(body, is_async, &mut yielded);
        let mut returned: Vec<Arc<Type>> = Vec::new();
        self.collect_generator_return_types(body, is_async, &mut returned);

        // Go getReturnTypeFromBody 生成器分支：空 yield=never；yield 并集加宽；
        // next 取上下文迭代型（注解型第三实参），无则 unknown
        let yield_type = if yielded.is_empty() {
            self.never_type()
        } else {
            let union = self.get_union_type(yielded);
            self.get_widened_type(&union)
        };
        let return_type = if returned.is_empty() {
            self.void_type()
        } else {
            let union = self.get_union_type(returned);
            self.get_widened_type(&union)
        };
        // Go getContextualIterationType(Next)：取上下文返回型引用的类型实参
        // —— ≥3 实参取第三；2 实参且目标接口第三参有默认（IterableIterator/
        // Generator 族 TNext=any）取 any；其余 unknown（如 Iterable<T> 单参）
        let contextual_next = body.parent().and_then(|fn_node| {
            // 仅调用/构造实参位的函数表达式取上下文 next（Go 该路径给出
            // IterableIterator<T,U> 的 TNext 默认 any）；变量赋值位的自然
            // 推断无上下文 next（unknown）
            let in_call_arg = fn_node.parent().is_some_and(|p| {
                p.kind == SyntaxKind::CallExpression || p.kind == SyntaxKind::NewExpression
            });
            if !in_call_arg {
                return None;
            }
            let ctx = self.contextual_return_type_of(&fn_node)?;
            let args = ctx.as_object().map(|o| o.type_arguments.clone()).unwrap_or_default();
            if args.len() >= 3 {
                return Some(Arc::clone(&args[2]));
            }
            if args.len() == 2 {
                let target_name = ctx
                    .target()
                    .and_then(|t| t.symbol.as_ref().map(|s| s.name.clone()))
                    .or_else(|| ctx.symbol.as_ref().map(|s| s.name.clone()));
                if target_name.as_deref().is_some_and(|n| {
                    matches!(n, "IterableIterator" | "Generator" | "AsyncGenerator" | "Iterator" | "AsyncIterator")
                }) {
                    return Some(self.get_any_type());
                }
            }
            None
        });
        let next_type = contextual_next.unwrap_or_else(|| self.unknown_type());

        let global_name = if is_async {
            "AsyncGenerator"
        } else {
            "Generator"
        };
        let Some(sym) = self.globals.get(global_name).cloned() else {
            return return_type;
        };
        self.resolve_interface_type_ex(&sym, Some(vec![yield_type, return_type, next_type]))
    }

    fn collect_yielded_types(
        &mut self,
        node: &Arc<Node>,
        is_async: bool,
        out: &mut Vec<Arc<Type>>,
    ) {
        use tsox_frontend::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::YieldExpression => {
                if let NodeData::YieldExpression(data) = &node.data {
                    let t = match &data.expression {
                        Some(expr) => {
                            if data.asterisk_token.is_some() {
                                let operand = self.get_type_of_node(expr);
                                self.check_iterated_type_or_element_type(
                                    crate::checker::checker_iteration::IterationUse::YieldStar {
                                        is_async,
                                    },
                                    &operand,
                                    None,
                                )
                            } else if is_async {
                                let operand = self.get_type_of_node(expr);
                                match self.get_awaited_type(&operand) {
                                    Some(a) => a,
                                    None => operand,
                                }
                            } else {
                                self.get_type_of_node(expr)
                            }
                        }
                        None => self.undefined_type(),
                    };
                    out.push(t);
                }
                return;
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => return,
            _ => {}
        }
        for_each_child(node, |child| {
            self.collect_yielded_types(child, is_async, out);
            false
        });
    }

    fn collect_generator_return_types(
        &mut self,
        node: &Arc<Node>,
        is_async: bool,
        out: &mut Vec<Arc<Type>>,
    ) {
        use tsox_frontend::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        let mut t = self.get_type_of_node(expr);
                        if is_async {
                            if let Some(a) = self.get_awaited_type(&t) {
                                t = a;
                            }
                        }
                        out.push(t);
                    }
                }
                return;
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => return,
            _ => {}
        }
        for_each_child(node, |child| {
            self.collect_generator_return_types(child, is_async, out);
            false
        });
    }
}
