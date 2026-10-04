#![allow(unused_imports)]

use crate::checker::typenode_references::*;
use crate::checker::mig::wc3_3::is_local_type_alias;
use crate::checker::mig::m2a::r20k6_defs::R20K6CheckerExt;
use tsox_frontend::ast::mig::m3g_2::is_type_reference_type;
use tsox_frontend::ast::SymbolFlags;

impl Checker {
    pub(crate) fn resolve_type_parameter_reference(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_type_parameter_reference"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_type_alias_reference"); 
        self.resolve_type_alias_reference_with_node(symbol, type_arguments, None)
    }

    pub(crate) fn resolve_type_alias_reference_with_node(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
        reference_node: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_type_alias_reference_with_node"); 
        // Go getNoInferType（checker.go 27744）：NoInfer 实参为 isNoInferTargetType
        // 时包装为 unknown 约束的 Substitution（推断期候选被 is_no_infer_type 拦截，
        // 关系/显示按 base 展开）；实参具体且非目标形态时走常规别名展开。base 取
        // 当前语境解析值（实例化时已代入），包装是否保留按清栈后的目标形态判定
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
            if self.is_no_infer_target_type(&unmapped) {
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
            let declared = cached.unwrap_or_else(|| {
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
            });
            let (tp_symbols, _) = self.collect_alias_type_params_and_body(symbol);
            if tp_symbols.is_empty() {
                declared
            } else {
                self.instantiate_alias_from_types_with_node(symbol, Vec::new(), reference_node)
            }
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
            self.instantiate_alias_from_types_with_node(symbol, arg_types, reference_node)
        };
        if args_frame.is_some() {
            self.alias_args_resolution_stack.pop();
        } else {
            self.pop_type_resolution();
        }
        resolved
    }

    pub(crate) fn instantiate_alias_from_types(
        &mut self,
        symbol: &Arc<Symbol>,
        arg_types: Vec<Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_alias_from_types"); 
        self.instantiate_alias_from_types_with_node(symbol, arg_types, None)
    }

    pub(crate) fn instantiate_alias_from_types_with_node(
        &mut self,
        symbol: &Arc<Symbol>,
        arg_types: Vec<Arc<Type>>,
        reference_node: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_alias_from_types_with_node"); 
        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        let frame = (key as usize, arg_types.iter().map(|t| t.id).collect::<Vec<_>>());
        if let Some(cached) = self.alias_instantiation_cache.get(&frame).cloned() {
            // 坏上下文（空作用域等）解析出的 error 驻留会永久掩盖正确结果，
            // 命中即废弃重算
            if cached.intrinsic_name() != Some("error") {
                return cached;
            }
            self.alias_instantiation_cache.remove(&frame);
        }
        // 声明体携带自身 alias 元数据时，体内替换会以同参重入本入口：
        // 按 (符号, 实参型) 判定进行中，同帧重入返回 error（Go 循环别名同策略）
        if self.alias_type_instantiation_stack.contains(&frame) {
            return self.error_type();
        }
        self.alias_type_instantiation_stack.push(frame.clone());
        let result =
            self.instantiate_alias_from_types_inner(symbol, arg_types, frame, reference_node);
        self.alias_type_instantiation_stack.pop();
        result
    }

    fn instantiate_alias_from_types_inner(
        &mut self,
        symbol: &Arc<Symbol>,
        arg_types: Vec<Arc<Type>>,
        frame: (usize, Vec<u32>),
        reference_node: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_alias_from_types_inner"); 
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
            if !tp_types.is_empty() {
                // Go getTypeFromTypeAliasReference（checker.go 24997-25021）：
                // 泛型别名引用实例化结果的 alias 取引用级符号——宿主别名声明
                // （getAliasSymbolForTypeNode）或引用名 resolveAlias 后的
                // TypeAlias 符号；instantiateAnonymousType（checker.go 23788）
                // 对结果无条件赋 alias，覆写体内既有标注
                if let Some(alias) =
                    self.alias_attach_for_type_reference(reference_node, symbol, &arg_types)
                {
                    // 替换无效果时 found 即缓存声明类型本体：Go
                    // instantiateTypeWithAlias（checker.go 23411）对不含类型
                    // 变量的目标原样返回不落 alias，共享 Arc 不得覆写
                    if !Arc::ptr_eq(&found, &declared) {
                        let ptr = Arc::as_ptr(&found) as *mut crate::checker::types::Type;
                        unsafe {
                            (*ptr).alias = Some(Box::new(alias));
                        }
                    }
                } else if declared
                    .alias
                    .as_ref()
                    .is_some_and(|a| a.symbol.is_some())
                {
                    // Go instantiateTypeWithAlias：声明体本身携带 alias
                    // （对象字面量/union/intersection/mapped/挂起条件/
                    // deferred 引用体）时传播实例化后的 alias；indexed
                    // access 等无 alias 声明体不传播
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
        }
        self.alias_instantiation_cache.insert(frame, Arc::clone(&found));
        found
    }

    fn alias_attach_for_type_reference(
        &mut self,
        reference_node: Option<&Arc<Node>>,
        symbol: &Arc<Symbol>,
        arg_types: &[Arc<Type>],
    ) -> Option<crate::checker::types::TypeAlias> { ::tsox_core::fntrace::enter("alias_attach_for_type_reference"); 
        let node = reference_node?;
        let host_alias_symbol = self.get_alias_symbol_for_type_node(node);
        let mut new_alias_symbol: Option<Arc<Symbol>> = None;
        if let Some(host) = &host_alias_symbol {
            if is_local_type_alias(symbol) || !is_local_type_alias(host) {
                new_alias_symbol = Some(Arc::clone(host));
            }
        }
        let mut alias_type_arguments: Vec<Arc<Type>> = Vec::new();
        if let Some(sym) = &new_alias_symbol {
            alias_type_arguments = self.get_type_arguments_for_alias_symbol(Some(sym));
        } else if is_type_reference_type(node) {
            let resolved_alias_symbol =
                self.resolve_type_reference_name(node, SymbolFlags::Alias, true);
            if let Some(alias_sym) = resolved_alias_symbol {
                if !Arc::ptr_eq(&alias_sym, &self.unknown_symbol()) {
                    let resolved = self.resolve_alias(&alias_sym);
                    if resolved.flags.contains(SymbolFlags::TypeAlias) {
                        new_alias_symbol = Some(resolved);
                        alias_type_arguments = arg_types.to_vec();
                    }
                }
            }
        }
        new_alias_symbol.map(|sym| {
            crate::checker::types::TypeAlias::new(Some(sym), alias_type_arguments)
        })
    }

}
