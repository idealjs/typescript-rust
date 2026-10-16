use crate::checker::inference::*;

impl Checker {
    /// Go resolveStructuredTypeMembers 的实例化语义在成员读取侧的补位：
    /// 空成员壳（create_array_type 手工实例、构建窗口中途快照）经符号与
    /// 类型实参重解析出完整实例，成员类型已完成实参替换；与
    /// is_object_type_related_to 的壳重解析同构。返回 None 表示无需水化
    pub(crate) fn instantiated_members_shell(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiated_members_shell"); 
        let sym = t.symbol.as_ref()?;
        if !sym.declarations.iter().any(|d| {
            matches!(
                d.data,
                tsox_frontend::ast::NodeData::InterfaceDeclaration(_)
            )
        }) {
            return None;
        }
        if !t.as_structured()?.members.entries.is_empty() {
            return None;
        }
        if self
            .pending_interface_shells
            .contains_key(&(Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol as usize))
        {
            return None;
        }
        let args = t.as_object().map(|o| o.type_arguments.clone());
        let resolved = self.resolve_interface_type_ex(sym, args);
        if !Arc::ptr_eq(&resolved, t)
            && resolved
                .as_structured()
                .is_some_and(|s| !s.members.entries.is_empty())
        {
            return Some(resolved);
        }
        None
    }
}
