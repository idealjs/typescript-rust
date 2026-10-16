#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_parameter_from_symbol");
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                // Go tryGetDeclaredTypeOfSymbol（checker.go:24511-24515）对类型
                // 参数恒产声明型变量（TypeFlags.TypeVariable）；type_alias_links
                // 为别名/类型参数共用缓存，非类型变量驻留不信任，落重算并回写
                if t.flags.contains(TypeFlags::TypeParameter) {
                    return Arc::clone(t);
                }
            }
        }
        // 双缓存收敛（Go 单例对象语义）：declared_type_links 先建的实例
        //（get_declared_type_of_type_parameter 构造，constraint 恒 None）在
        // 符号无约束声明时与本套语义等价，直接收编共享消灭 ptr 分叉（实例
        // 化 mapper 恒等坍缩源）；有约束声明的裸实例不收编——本套构造会
        // 解析约束，收编将使约束查询退化
        if let Some(t) = self
            .declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
            && t.flags.contains(TypeFlags::TypeParameter)
        {
            let has_constraint_decl = symbol.declarations.iter().any(|d| {
                matches!(
                    &d.data,
                    tsox_frontend::ast::NodeData::TypeParameterDeclaration(td) if td.constraint.is_some()
                )
            });
            let resolved_constraint = match &t.data {
                crate::checker::types::TypeData::TypeParameter(tp) => tp.constraint.clone(),
                _ => None,
            };
            if resolved_constraint.is_some() || !has_constraint_decl {
                self.type_alias_links
                    .get_or_default(symbol)
                    .declared_type = Some(Arc::clone(&t));
                return t;
            }
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if !self.type_parameter_resolving.insert(sym_key) {
            return Arc::new(Type {
                flags: TypeFlags::TypeParameter,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::TypeParameter(TypeParameterData {
                    constrained: ConstrainedTypeData::default(),
                    constraint: None,
                    target: None,
                    mapper: None,
                    is_this_type: false,
                    resolved_default_type: std::sync::RwLock::new(None),
                }),
            });
        }
        let mut constraint: Option<Arc<Type>> = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeParameterDeclaration(data) = &decl.data {
                if let Some(constraint_node) = &data.constraint {
                    // 约束按声明形态解析并缓存：清空进行中的实例化映射，
                    // 否则首个实例化语境（如 ReadonlyArray<string> 的成员解析）
                    // 会把代入后的约束烤进符号缓存，污染后续所有实例
                    let saved_stack = std::mem::take(&mut self.type_argument_stack);
                    let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
                    constraint = Some(self.get_type_from_type_node(constraint_node));
                    self.type_argument_stack = saved_stack;
                    self.type_argument_name_frames = saved_frames;
                    if let Some(c) = &constraint
                        && c.flags.contains(TypeFlags::Any)
                        && !self.is_error_type(c)
                    {
                        constraint = Some(self.unknown_type());
                    }
                }
                break;
            }
        }

        if let Some(c) = &constraint
            && self.constraint_chain_is_circular(sym_key, c)
        {
            if self.ts2313_reported.insert(sym_key) {
                let loc = symbol
                    .declarations
                    .iter()
                    .find_map(|d| match &d.data {
                        NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.as_ref().map(|cn| cn.loc)
                        }
                        _ => None,
                    })
                    .or_else(|| symbol.declarations.first().map(|d| d.loc))
                    .unwrap_or_default();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                    vec![symbol.name.clone()],
                ));
            }
            constraint = None;
        }
        let tp = Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: std::sync::RwLock::new(None),
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&tp));
        self.type_parameter_resolving.remove(&sym_key);
        tp
    }

    pub(crate) fn constraint_chain_is_circular(
        &self,
        start_key: usize,
        constraint: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("constraint_chain_is_circular");
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current = constraint;
        for _ in 0..50 {
            let TypeData::TypeParameter(tp) = &current.data else {
                return false;
            };
            let Some(sym) = &current.symbol else {
                return false;
            };
            let key = Arc::as_ptr(sym) as usize;
            if !visited.insert(key) {
                return true;
            }
            match &tp.constraint {
                Some(next) => current = next,
                None => return key == start_key,
            }
        }
        false
    }
}
