#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_interface_type_ex"); 
        let merged_symbol = self.get_merged_symbol(symbol);
        let symbol: &Arc<Symbol> = &merged_symbol;
        let has_type_args = type_args.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                if !crate::checker::utilities::is_type_error(&cached) {
                    return cached;
                }
            }
        }

        let instantiation_key: Option<Vec<usize>> = type_args.as_ref().map(|args| {
            let mut key = Vec::with_capacity(args.len() + 1);
            key.push(Arc::as_ptr(symbol) as *const Symbol as usize);
            key.extend(
                args.iter()
                    .map(|t| Arc::as_ptr(t) as *const crate::checker::types::Type as usize),
            );
            key
        });

        let pinned_args: Option<Vec<Arc<Type>>> = type_args.clone();
        if let Some(key) = &instantiation_key
            && let Some(cached) = self.interface_instantiation_cache.get(key)
        {
            return Arc::clone(&cached.1);
        }

        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        // Go getDeclaredTypeOfClassOrInterface（checker.go:18327-18351）建型即驻留、
        // 无 pushTypeResolution：解析中的接口重入引用是声明型缓存命中，不构成环、
        // 不标记外层帧（unionTypeWithIndexedLiteralType：type U = Idx | ... 的
        // 别名体经 Idx 索引签名值型重入接口解析窗，push 失败的环标记把 U 自己的
        // 帧标假，getDeclaredTypeOfTypeAlias pop 读假丢弃已求得的联合型返回
        // error，索引值型随之染 error 致初值字面量误拓宽）
        if self.is_resolving(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) && let Some(shell) = self.pending_interface_shells.get(&(key as usize))
        {
            let shell = Arc::clone(shell);
            let args = type_args.unwrap_or_default();
            if args.is_empty() {
                return shell;
            }
            if let Some(ikey) = &instantiation_key {
                return self.pending_arg_shell(ikey, symbol, args);
            }
            return shell;
        }
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            if let Some(shell) = self.pending_interface_shells.get(&(key as usize)) {
                let shell = Arc::clone(shell);
                let args = type_args.unwrap_or_default();
                if args.is_empty() {
                    return shell;
                }
                if let Some(ikey) = &instantiation_key {
                    return self.pending_arg_shell(ikey, symbol, args);
                }
                return shell;
            }
            self.heritage_degraded_events += 1;
            return self.error_type();
        }
        let shell_key = key as usize;
        let arg_shell_entry_seq = self.arg_shell_seq;
        let shell = self.push_interface_shell(symbol);
        let shell_cleanup = |checker: &mut Checker| {
            if shell.is_some() {
                checker.pending_interface_shells.remove(&shell_key);
            }
        };

        let interface_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
            .cloned()
            .collect();

        let epoch_at_entry = self.heritage_degraded_events;
        let mut heritage_degraded = false;
        let mut lineage_degraded = false;
        let mut degraded_accepted = false;
        let mut live_shell_base = false;
        let mut result;
        loop {
            let pass_epoch = self.heritage_degraded_events;
            let pass = self.resolve_interface_pass(
                symbol,
                &interface_decls,
                has_type_args,
                pinned_args.clone().unwrap_or_default(),
            );
            result = pass.result;
            lineage_degraded = pass.lineage_degraded;
            live_shell_base = pass.live_shell_base;
            if pass.base_degraded
                || pass.base_shell
                || self.heritage_degraded_events != pass_epoch
                || self.heritage_degraded_events != epoch_at_entry
            {
                heritage_degraded = true;
            }
            if heritage_degraded {
                let sym_key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol as usize;
                let retries = self.heritage_retry_counts.entry(sym_key).or_insert(0);
                *retries += 1;
                degraded_accepted = *retries > crate::checker::checker::HERITAGE_RETRY_LIMIT;
            }
            if live_shell_base && !degraded_accepted {
                continue;
            }
            break;
        }
        self.pop_type_resolution();
        shell_cleanup(self);

        let result = match (
            shell.as_ref(),
            has_type_args,
            crate::checker::utilities::is_type_error(&result),
        ) {
            (Some(shell), false, false) => self.fill_interface_shell(shell, result),
            _ => result,
        };

        self.backfill_pending_arg_shells(
            symbol,
            instantiation_key.as_ref(),
            &result,
            arg_shell_entry_seq,
        );

        if has_type_args {
            self.instantiate_interface_member_this_to_instance(&result, symbol);
        }

        let cache_result = !heritage_degraded || (degraded_accepted && !live_shell_base);
        // 环窗口内急切求值的索引签名值型可能为断路 error（别名重入经
        // resolve_type_alias_reference_with_node / getDeclaredTypeOfTypeAlias 的
        // push 环判定）：该产物不驻留，待无窗重解取真型（Go 索引信息惰性解析
        // 不存在此窗口；unionTypeWithIndexedLiteralType 的初值字面量误拓宽即
        // 此残留）
        let transient_index_error = !self.type_resolution_stack.is_empty()
            && result.as_structured().is_some_and(|s| {
                s.index_infos.iter().any(|info| {
                    info.value_type
                        .as_ref()
                        .is_some_and(|v| crate::checker::utilities::is_type_error(v))
                })
            });
        let cache_result = cache_result && !transient_index_error;
        if degraded_accepted && self.heritage_degraded_events != epoch_at_entry {
            self.heritage_degraded_events = epoch_at_entry;
        }
        if heritage_degraded || lineage_degraded {
            self.degraded_type_ptrs.insert(result.id);
        }
        if !has_type_args && cache_result {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        }
        if let Some(key) = instantiation_key {
            if cache_result {
                let pin = pinned_args.clone().unwrap_or_default();
                self.interface_instantiation_cache
                    .insert(key, (pin, Arc::clone(&result)));
            }
        }
        result
    }
}
