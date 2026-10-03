#![allow(unused_imports)]

use crate::checker::typenode_references::*;
use std::sync::Arc;

const ARG_SHELL_RESOLVE_CAP: u64 = 256;

impl Checker {
    pub(crate) fn push_interface_shell(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("push_interface_shell"); 
        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol as usize;
        if self.pending_interface_shells.contains_key(&key) {
            return None;
        }
        let shell = Arc::new(Type {
            flags: crate::checker::types::TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Reference,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: crate::checker::types::TypeData::Object(Default::default()),
        });
        self.pending_interface_shells.insert(key, Arc::clone(&shell));
        Some(shell)
    }

    pub(crate) fn fill_interface_shell(
        &mut self,
        shell: &Arc<Type>,
        result: Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("fill_interface_shell"); 
        let shell_empty = shell
            .as_structured()
            .is_some_and(|s| s.members.entries.is_empty() && s.index_infos.is_empty());
        if !shell_empty {
            return result;
        }
        let sptr = Arc::as_ptr(shell) as *mut crate::checker::types::Type;
        let rptr = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
        unsafe {
            (*sptr).flags = (*rptr).flags;
            (*sptr).object_flags = (*rptr).object_flags;
            (*sptr).symbol = (*rptr).symbol.clone();
            (*sptr).alias = (*rptr).alias.clone();
            std::mem::swap(&mut (*sptr).data, &mut (*rptr).data);
        }
        Arc::clone(shell)
    }

    pub(crate) fn pending_arg_shell(
        &mut self,
        key: &[usize],
        symbol: &Arc<Symbol>,
        args: Vec<Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("pending_arg_shell"); 
        if let Some((existing, _)) = self.pending_arg_shells.get(key) {
            return Arc::clone(existing);
        }
        let shell = Arc::new(Type {
            flags: crate::checker::types::TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Reference,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: crate::checker::types::TypeData::Object(
                crate::checker::types::ObjectTypeData { node: None,
                    structured: Default::default(),
                    target: None,
                    mapper: None,
                    type_arguments: args,
                },
            ),
        });
        self.arg_shell_seq += 1;
        let seq = self.arg_shell_seq;
        let owned: Vec<usize> = key.to_vec();
        self.pending_arg_shells.insert(owned, (Arc::clone(&shell), seq));
        shell
    }

    pub(crate) fn backfill_pending_arg_shells(
        &mut self,
        symbol: &Arc<Symbol>,
        own_key: Option<&Vec<usize>>,
        own_result: &Arc<Type>,
        entry_seq: u64,
    ) { ::tsox_core::fntrace::enter("backfill_pending_arg_shells"); 
        let sym_head = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol as usize;
        if let Some(own) = own_key
            && own.first() == Some(&sym_head)
            && let Some((shell, seq)) = self
                .pending_arg_shells
                .get(own)
                .map(|(s, q)| (Arc::clone(s), *q))
        {
            Self::fill_arg_shell_in_place(&shell, own_result);
            if seq > entry_seq {
                self.pending_arg_shells.remove(own);
            }
        }
        while let Some((key, shell)) = self
            .pending_arg_shells
            .iter()
            .find(|(k, (_, seq))| k.first() == Some(&sym_head) && *seq > entry_seq)
            .map(|(k, (v, _))| (k.clone(), Arc::clone(v)))
        {
            let Some(args) = shell.as_object().map(|o| o.type_arguments.clone()) else {
                self.pending_arg_shells.remove(&key);
                continue;
            };
            if self.arg_shell_resolves < ARG_SHELL_RESOLVE_CAP {
                self.arg_shell_resolves += 1;
                let full = self.resolve_interface_type_ex(symbol, Some(args));
                if let Some((remaining, _)) = self.pending_arg_shells.get(&key) {
                    let remaining = Arc::clone(remaining);
                    Self::fill_arg_shell_in_place(&remaining, &full);
                }
            }
            self.pending_arg_shells.remove(&key);
        }
    }

    fn fill_arg_shell_in_place(shell: &Arc<Type>, full: &Arc<Type>) { ::tsox_core::fntrace::enter("fill_arg_shell_in_place"); 
        let sptr = Arc::as_ptr(shell) as *mut crate::checker::types::Type;
        let rptr = Arc::as_ptr(full) as *mut crate::checker::types::Type;
        unsafe {
            (*sptr).flags = (*rptr).flags;
            (*sptr).object_flags = (*rptr).object_flags;
            if let (
                crate::checker::types::TypeData::Object(dst),
                crate::checker::types::TypeData::Object(src),
            ) = (&mut (*sptr).data, &(*rptr).data)
            {
                dst.structured.members = src.structured.members.clone();
                dst.structured.properties = src.structured.properties.clone();
                dst.structured.signatures = src.structured.signatures.clone();
                dst.structured.call_signature_count = src.structured.call_signature_count;
                dst.structured.index_infos = src.structured.index_infos.clone();
                dst.target = src.target.clone();
                dst.mapper = src.mapper.clone();
            }
        }
    }
}
