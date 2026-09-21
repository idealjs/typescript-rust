#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_type_parameter_reference(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        let mut hit_depth = usize::MAX;
        let mut hit: Option<Arc<Type>> = None;
        for (depth, map) in self.type_argument_stack.iter().rev().enumerate() {
            if let Some(t) = map.get(&key) {
                hit_depth = depth;
                hit = Some(Arc::clone(t));
                break;
            }
        }
        if hit.is_none() {
            // 符号实例漂移回退：多树副本下帧键与解析符号不同实例，按名字+容器名
            // 等价命中（同一声明的语义副本）
            'outer: for (depth, map) in self.type_argument_stack.iter().rev().enumerate() {
                for (k, v) in map.iter() {
                    if self.type_param_symbols_equivalent(unsafe { &**k }, symbol) {
                        hit_depth = depth;
                        hit = Some(Arc::clone(v));
                        break 'outer;
                    }
                }
            }
        }

        // Go 组合 mapper 语义：帧命中值为类型参数且更底层帧仍有绑定时逐层
        // 追踪到底（别名帧 TOuter→TInner 与当前替换帧 TInner→Arg 的组合）
        if let Some(mut t) = hit {
            let mut depth = hit_depth;
            for _ in 0..self.type_argument_stack.len() {
                if !t.flags.contains(crate::checker::types::TypeFlags::TypeParameter) {
                    break;
                }
                let Some(next_sym) = t.symbol.as_ref() else {
                    break;
                };
                let next_key = Arc::as_ptr(next_sym) as *const tsox_frontend::ast::Symbol;
                let mut next: Option<Arc<Type>> = None;
                let mut scanned = 0;
                for map in self.type_argument_stack.iter().rev().skip(depth + 1) {
                    scanned += 1;
                    if let Some(u) = map.get(&next_key) {
                        next = Some(Arc::clone(u));
                        break;
                    }
                }
                match next {
                    Some(u) => {
                        depth += scanned;
                        t = u;
                    }
                    None => break,
                }
            }
            return t;
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
        // Go getNoInferType（checker.go 27744）：NoInfer 实参含类型参数时包装为
        // unknown 约束的 Substitution（推断期候选被 is_no_infer_type 拦截，
        // 关系/显示按 base 展开）；实参具体时走常规别名展开。base 取当前语境
        // 解析值（实例化时已代入），包装是否保留按清栈后是否仍含类型参数判定
        if symbol.name == "NoInfer"
            && let Some(args) = &type_arguments
            && args.len() == 1
        {
            let arg_node = args.iter().next().expect("checked len").clone();
            let stacked = self.get_type_from_type_node(&arg_node);
            let saved_stack = std::mem::take(&mut self.type_argument_stack);
            let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
            let unmapped = self.get_type_from_type_node(&arg_node);
            self.type_argument_stack = saved_stack;
            self.type_argument_name_frames = saved_frames;
            if crate::checker::type_contains_type_parameter(&unmapped) {
                let base = if crate::checker::type_contains_type_parameter(&stacked) {
                    unmapped
                } else {
                    stacked
                };
                return Arc::new(Type {
                    flags: TypeFlags::Substitution,
                    object_flags: ObjectFlags::None,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Substitution(SubstitutionTypeData {
                        constrained: ConstrainedTypeData::default(),
                        base_type: Some(base),
                        constraint: Some(self.unknown_type()),
                    }),
                });
            }
        }
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
            let mut arg_types = arg_types;
            arg_types.extend(self.alias_missing_default_type_arguments(
                symbol,
                &tp_symbols,
                &arg_types,
            ));
            if let Some(mapped) = self.intrinsic_alias_instantiation(symbol, &arg_types) {
                self.alias_args_resolution_stack.pop();
                return mapped;
            }
            let tp_types: Vec<Arc<Type>> = tp_symbols
                .iter()
                .map(|tp| self.get_type_parameter_from_symbol(tp))
                .collect();
            let declared_is_conditional = matches!(&declared.data, TypeData::Conditional(_));
            let found = if tp_types.is_empty() || arg_types.is_empty() {
                Arc::clone(&declared)
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
                // Go instantiateTypeWithAlias：泛型别名实例化仅当声明体本身
                // 携带 alias（对象字面量/union/intersection/mapped/挂起条件/
                // deferred 引用体）时传播实例化后的 alias；indexed access 等
                // 无 alias 声明体（Go getAliasForTypeNode 不附着）不传播
                if !tp_types.is_empty()
                    && declared
                        .alias
                        .as_ref()
                        .is_some_and(|a| a.symbol.is_some())
                {
                    let alias = crate::checker::types::TypeAlias::new(
                        declared.alias.as_ref().and_then(|a| a.symbol.clone()),
                        arg_types,
                    );
                    let ptr = Arc::as_ptr(&found) as *mut crate::checker::types::Type;
                    unsafe {
                        if (*ptr).alias.is_none() {
                            (*ptr).alias = Some(Box::new(alias));
                        }
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
