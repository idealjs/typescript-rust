#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_type_parameter_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        type_name: &Arc<Node>,
    ) -> Arc<Type> {
        if self.in_static_member_type {
            let tp_decl = symbol
                .value_declaration
                .clone()
                .or_else(|| symbol.declarations.first().cloned());
            let owned_by_class = tp_decl.is_some_and(|d| {
                let mut cur = d.parent();
                while let Some(a) = cur {
                    match a.kind {
                        tsox_frontend::ast::SyntaxKind::ClassDeclaration
                        | tsox_frontend::ast::SyntaxKind::ClassExpression => return true,
                        tsox_frontend::ast::SyntaxKind::SourceFile => return false,
                        _ => cur = a.parent(),
                    }
                }
                false
            });
            if owned_by_class {
                use tsox_core::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS;
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                    Vec::new(),
                ));
            }
        }


        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        for map in self.type_argument_stack.iter().rev() {
            if let Some(t) = map.get(&key) {
                return Arc::clone(t);
            }
        }
        // 符号实例漂移回退：多树副本下帧键与解析符号不同实例，按名字+容器名
        // 等价命中（同一声明的语义副本）
        for map in self.type_argument_stack.iter().rev() {
            for (k, v) in map.iter() {
                if self.type_param_symbols_equivalent(unsafe { &**k }, symbol) {
                    return Arc::clone(v);
                }
            }
        }


        for frame in self.type_argument_name_frames.iter().rev() {
            for (frame_sym, t) in frame.iter().rev() {
                if Arc::ptr_eq(frame_sym, symbol)
                    || (frame_sym.name == symbol.name
                        && self.type_param_symbols_equivalent(frame_sym, symbol))
                {
                    return Arc::clone(t);
                }
            }
        }
        return self.get_type_parameter_from_symbol(symbol);
    }

    pub(crate) fn resolve_type_alias_reference(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        // Go getTypeAliasInstantiation：带实参引用的循环按 (符号, 实参) 判定，
        // 嵌套的不同实参（Deep<T> 内解析 Deep<T[K]>）不受外层守卫影响
        let has_type_args = type_arguments.is_some();
        let mut args_frame: Option<()> = None;
        if has_type_args {
            // 进行中判定用实参节点 id（解析实参类型本身可能递归回本别名）
            let arg_node_ids: Vec<u32> = type_arguments
                .as_ref()
                .map(|args| args.iter().map(|a| a.id() as u32).collect())
                .unwrap_or_default();
            let pre_frame = (key as usize, arg_node_ids);
            let in_progress = self
                .alias_args_resolution_stack
                .iter()
                .any(|f| f.0 == pre_frame.0 && f.1 == pre_frame.1);
            if in_progress {
                return self.error_type();
            }
            self.alias_args_resolution_stack.push(pre_frame);
            args_frame = Some(());
        } else if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }
        let resolved = if !has_type_args {
            let cached = self
                .type_alias_links
                .get(&symbol)
                .and_then(|l| l.declared_type.clone());
            cached.unwrap_or_else(|| {
                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let found = self.resolve_alias_body(symbol);
                self.in_static_member_type = saved_static;
                // 环窗口内解析出的 error 不驻留声明型缓存（Go 仅缓存完成的
                // 别名解析；error 驻留会永久掩盖窗口外的正确结果）
                if !crate::checker::utilities::is_type_error(&found) {
                    self.type_alias_links.get_or_default(symbol).declared_type =
                        Some(Arc::clone(&found));
                }
                found
            })
        } else {
            // Go getTypeAliasInstantiation：取声明型（声明上下文一次解析、缓存），
            // 再以 mapper 替换类型参数——不重读 body 节点（嵌套别名会互递归）
            let arg_types: Vec<Arc<Type>> = match &type_arguments {
                Some(args) => args
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect(),
                None => Vec::new(),
            };
            let frame = (key as usize, arg_types.iter().map(|t| t.id).collect::<Vec<_>>());
            if let Some(cached) = self.alias_instantiation_cache.get(&frame).cloned() {
                // 坏上下文（空作用域等）解析出的 error 驻留会永久掩盖正确结果，
                // 命中即废弃重算
                if cached.intrinsic_name() != Some("error") {
                    // 提前返回也必须弹 frame，否则栈泄漏会让后续同引用被
                    // in_progress 误判为循环
                    self.alias_args_resolution_stack.pop();
                    return cached;
                }
                self.alias_instantiation_cache.remove(&frame);
            }
            let declared = {
                let cached = self
                    .type_alias_links
                    .get(&symbol)
                    .and_then(|l| l.declared_type.clone());
                cached.unwrap_or_else(|| {
                    let saved_static = self.in_static_member_type;
                    self.in_static_member_type = false;
                    let found = self.resolve_alias_body(symbol);
                    self.in_static_member_type = saved_static;
                    if !crate::checker::utilities::is_type_error(&found) {
                        self.type_alias_links.get_or_default(symbol).declared_type =
                            Some(Arc::clone(&found));
                    }
                    found
                })
            };
            let (tp_symbols, _type_node) = self.collect_alias_type_params_and_body(symbol);
            let tp_types: Vec<Arc<Type>> = tp_symbols
                .iter()
                .map(|tp| self.get_type_parameter_from_symbol(tp))
                .collect();
            let declared_is_conditional = matches!(&declared.data, TypeData::Conditional(_));
            let found = if tp_types.is_empty() || arg_types.is_empty() {
                declared
            } else {
                self.substitute_infer_type_parameters(&declared, &tp_types, &arg_types)
            };
            // Go getConditionalType（checker.go 24784）：alias 传播到匿名对象/
            // mapped/挂起条件的实例化结果，但**条件的解析分支**不带 alias
            // （result = instantiateType(branch) 独立实例化），hover 显示为
            // 展开形态
            let found_is_deferred_conditional = matches!(
                &found.data,
                TypeData::Conditional(c)
                    if c.resolved_true_type.get().is_none() && c.resolved_false_type.get().is_none()
            );
            if !declared_is_conditional || found_is_deferred_conditional {
                let alias = crate::checker::types::TypeAlias::new(
                    Some(Arc::clone(symbol)),
                    arg_types,
                );
                let ptr = Arc::as_ptr(&found) as *mut crate::checker::types::Type;
                unsafe {
                    if (*ptr).alias.is_none() {
                        (*ptr).alias = Some(Box::new(alias));
                    }
                }
            }
            self.alias_instantiation_cache.insert(frame, Arc::clone(&found));
            found
        };
        if args_frame.is_some() {
            self.alias_args_resolution_stack.pop();
        } else {
            self.pop_type_resolution();
        }
        resolved
    }
}
