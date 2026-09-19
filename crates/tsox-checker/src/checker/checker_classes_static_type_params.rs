#![allow(unused_imports)]

use crate::checker::checker_classes::*;
use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

impl Checker {
    /// Go NameResolver ClassDeclaration case（nameresolver.go）：名称在所属类
    /// members（类型参数）命中且解析链从 static 成员直接进入类时报 TS2302。
    /// 此处在静态成员检查期对其子树内全部类型引用统一判定，等价于解析期
    /// 逐引用命中（值位类型参数引用不存在，类型位全覆盖）
    pub(crate) fn check_static_member_class_type_params(&mut self, member: &Arc<Node>) {
        if !matches!(
            member.kind,
            SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::ClassStaticBlockDeclaration
        ) {
            return;
        }
        let is_static = member.kind == SyntaxKind::ClassStaticBlockDeclaration
            || member.has_syntactic_modifier(ModifierFlags::Static);
        if !is_static {
            return;
        }

        let mut refs: Vec<Arc<Node>> = Vec::new();
        collect_type_reference_names(member, &mut refs);
        for id in refs {
            if self.class_type_param_reached_from_static(&id) {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    id.loc,
                    tsox_core::diagnostics::messages_generated::
                        STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                    Vec::new(),
                ));
            }
        }
    }

    /// Go resolveName 上溯链的静态成员违规判定：从类型名向上，中途任何
    /// 容器 locals 命中同名（如方法自有类型参数）即无违规；类/接口 members
    /// 命中时按进入类的那一跳是否为 static 成员定夺；未命中继续上溯
    fn class_type_param_reached_from_static(&self, id: &Arc<Node>) -> bool {
        let name = id.text();
        let symbol_map = self.program.symbol_map();
        let mut last: Option<Arc<Node>> = None;
        let mut cur = id.parent();
        while let Some(a) = cur {
            if let Some(locals) = symbol_map.locals.get(&a.id())
                && locals.get(name).is_some()
            {
                return false;
            }
            if matches!(
                a.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
            ) && let Some(sym) = symbol_map.symbols.get(&a.id())
                && sym.members.get(name).is_some()
            {
                return last.is_some_and(|l| {
                    l.kind == SyntaxKind::ClassStaticBlockDeclaration
                        || l.has_syntactic_modifier(ModifierFlags::Static)
                });
            }
            last = Some(Arc::clone(&a));
            cur = a.parent();
        }
        false
    }
}

fn collect_type_reference_names(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
    if node.kind == SyntaxKind::TypeReference
        && let NodeData::TypeReferenceNode(data) = &node.data
        && data.type_name.kind == SyntaxKind::Identifier
    {
        out.push(Arc::clone(&data.type_name));
    }
    tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
        collect_type_reference_names(child, out);
        false
    });
}
