use crate::checker::relater_relate_impl_chunk::*;
use crate::checker::utilities_has_only_expression_initialization::is_object_or_array_literal_type;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecursionIdentity {
    Node(u64),
    Symbol(*const crate::checker::types::Symbol),
    Type(crate::checker::types::TypeId),
}

impl Checker {
    fn get_recursion_identity_target(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_recursion_identity_target"); 
        let mut cur = Arc::clone(t);
        loop {
            if let TypeData::IndexedAccess(ia) = &cur.data {
                match ia.object_type.as_ref() {
                    Some(obj) => {
                        cur = Arc::clone(obj);
                        continue;
                    }
                    None => return cur,
                }
            }
            if cur
                .object_flags
                .contains(ObjectFlags::Mapped | ObjectFlags::Instantiated)
            {
                let modifiers = self.get_modifiers_type_from_mapped_type(&cur);
                let has_symbol_anchor = modifiers.symbol.is_some()
                    || (modifiers.flags.contains(TypeFlags::Intersection)
                        && modifiers
                            .types()
                            .is_some_and(|ts| ts.iter().any(|c| c.symbol.is_some())));
                if has_symbol_anchor {
                    cur = modifiers;
                    continue;
                }
            }
            return cur;
        }
    }

    fn recursion_identity_from_target(&self, t: &Arc<Type>) -> RecursionIdentity { ::tsox_core::fntrace::enter("recursion_identity_from_target"); 
        if t.flags.contains(TypeFlags::Object) && !is_object_or_array_literal_type(t) {
            if let Some(sym) = t.symbol.as_ref()
                && !(t.object_flags.contains(ObjectFlags::Anonymous)
                    && sym.flags.contains(SymbolFlags::Class))
                && !t.object_flags.contains(ObjectFlags::FromTypeNode)
            {
                return RecursionIdentity::Symbol(Arc::as_ptr(sym));
            }
            if t.object_flags.contains(ObjectFlags::Tuple)
                && !t.object_flags.contains(ObjectFlags::FromTypeNode)
            {
                if let Some(target) = t.as_object().and_then(|o| o.target.as_ref()) {
                    return RecursionIdentity::Type(target.id);
                }
            }
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(sym) = t.symbol.as_ref() {
                return RecursionIdentity::Symbol(Arc::as_ptr(sym));
            }
        }
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(node) = ct.root.as_ref().and_then(|r| r.node.as_ref()) {
                return RecursionIdentity::Node(node.id());
            }
        }
        RecursionIdentity::Type(t.id)
    }

    fn stack_entry_matches_identity(
        &mut self,
        t: &Arc<Type>,
        identity: RecursionIdentity,
    ) -> bool { ::tsox_core::fntrace::enter("stack_entry_matches_identity"); 
        if t.flags.contains(TypeFlags::Intersection) {
            if let Some(constituents) = t.types() {
                return constituents
                    .iter()
                    .any(|c| self.stack_entry_matches_identity(c, identity));
            }
            return false;
        }
        let target = self.get_recursion_identity_target(t);
        self.recursion_identity_from_target(&target) == identity
    }

    pub(crate) fn inference_is_deeply_nested_type(
        &mut self,
        t: &Arc<Type>,
        stack: &[Arc<Type>],
        max_depth: usize,
    ) -> bool { ::tsox_core::fntrace::enter("inference_is_deeply_nested_type"); 
        if stack.len() < max_depth {
            return false;
        }
        let target = self.get_recursion_identity_target(t);
        if target.flags.contains(TypeFlags::Intersection) {
            if let Some(constituents) = target.types() {
                for c in constituents {
                    if self.is_deeply_nested_type(c, stack, max_depth) {
                        return true;
                    }
                }
            }
            return false;
        }
        let identity = self.recursion_identity_from_target(&target);
        let mut count = 0usize;
        let mut last_id: crate::checker::types::TypeId = 0;
        for s in stack {
            if self.stack_entry_matches_identity(s, identity) {
                if s.id >= last_id {
                    count += 1;
                    if count >= max_depth {
                        return true;
                    }
                }
                last_id = s.id;
            }
        }
        false
    }

    pub(crate) fn relater_is_deeply_nested_type(
        &mut self,
        t: &Arc<Type>,
        stack: &[Arc<Type>],
        max_depth: usize,
    ) -> bool { ::tsox_core::fntrace::enter("relater_is_deeply_nested_type"); 
        if stack.len() < max_depth {
            return false;
        }
        let target = self.get_recursion_identity_target(t);
        if target.flags.contains(TypeFlags::Intersection) {
            if let Some(constituents) = target.types() {
                for c in constituents {
                    if self.relater_is_deeply_nested_type(c, stack, max_depth) {
                        return true;
                    }
                }
            }
            return false;
        }
        let identity = self.recursion_identity_from_target(&target);
        let mut count = 0usize;
        let mut last_id: crate::checker::types::TypeId = 0;
        for s in stack {
            if self.stack_entry_matches_identity(s, identity) {
                // Go isDeeplyNestedType 按 RecursionFlags 只把对应侧入栈，计数
                // 用非递减 id 判定「递归产生的新实例」；Rust 侧双栈无条件同时
                // 入栈，同型重复帧（等 id）会污染计数，这里收紧为严格递增 id，
                // 只统计递归真正新建的同根实例
                if s.id > last_id {
                    count += 1;
                    if count >= max_depth {
                        return true;
                    }
                }
                last_id = s.id;
            }
        }
        false
    }
}
