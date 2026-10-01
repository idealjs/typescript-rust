#![allow(unused_imports)]

use crate::checker::typenode_references::*;

const RECOVERABLE_BASE_RETRY_CAP: u32 = 3;

impl Checker {
    pub fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {
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
        if let Some(key) = &instantiation_key {
            let cached_residue = self
                .interface_instantiation_cache
                .get(key)
                .is_some_and(|(_, t)| self.interface_shell_residue(t));
            if cached_residue {
                self.interface_instantiation_cache.remove(key);
            } else if let Some(cached) = self.interface_instantiation_cache.get(key) {
                return Arc::clone(&cached.1);
            }
        }

        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
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
        let mut recoverable_base_retries = 0u32;
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
            let recoverable_base = pass.base_residue || pass.base_degraded;
            if (live_shell_base || recoverable_base)
                && !degraded_accepted
                && recoverable_base_retries < RECOVERABLE_BASE_RETRY_CAP
            {
                recoverable_base_retries += 1;
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

        let cache_result = !heritage_degraded || (degraded_accepted && !live_shell_base);
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
