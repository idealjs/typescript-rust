#![allow(unused_imports)]

use crate::checker::checker_suggestions_resolve::*;

impl Checker {
    pub(crate) fn collect_unimplemented_abstract_members(
        class: &Arc<Node>,
        base: &Arc<Node>,
        out: &mut Vec<String>,
    ) {
        for member in Self::class_members_of(base).iter() {
            let (name_node, is_abstract_member) = match &member.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind != SyntaxKind::Identifier {
                continue;
            }
            let name = name_node.text();
            if is_abstract_member {
                if !Self::chain_implements(class, name) {
                    out.push(name.to_string());
                }
            } else if out.iter().any(|m| m == name) {
                out.retain(|m| m != name);
            }
        }
    }

    pub(crate) fn first_return_expression(body: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        fn walk(n: &Arc<Node>) -> Option<Arc<Node>> {
            if let tsox_frontend::ast::NodeData::ReturnStatement(d) = &n.data
                && let Some(e) = &d.expression
            {
                return Some(Arc::clone(e));
            }
            let mut found: Option<Arc<Node>> = None;
            tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
                if found.is_none() {
                    found = walk(child);
                }
                found.is_some()
            });
            found
        }
        body.and_then(walk)
    }

    pub(crate) fn chain_implements(class: &Arc<Node>, name: &str) -> bool {
        for member in Self::class_members_of(class).iter() {
            let (name_node, is_abstract) = match &member.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind == SyntaxKind::Identifier && name_node.text() == name && !is_abstract
            {
                return true;
            }
        }

        false
    }

    pub(crate) fn assignments_to_name(
        body: &Arc<Node>,
        name: &str,
    ) -> Vec<(tsox_core::core::text::TextRange, Arc<Node>)> {
        let mut found = Vec::new();
        fn walk(
            n: &Arc<Node>,
            name: &str,
            found: &mut Vec<(tsox_core::core::text::TextRange, Arc<Node>)>,
        ) {
            if let tsox_frontend::ast::NodeData::BinaryExpression(data) = &n.data
                && data.operator_token.kind == SyntaxKind::EqualsToken
                && data.left.kind == SyntaxKind::Identifier
                && data.left.text() == name
            {
                found.push((data.left.loc, Arc::clone(&data.right)));
            }
            tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
                walk(child, name, found);
                false
            });
        }
        walk(body, name, &mut found);
        found
    }

    pub fn resolve_qualified_symbol(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        match self.resolve_qualified_symbol_traced(name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn resolve_qualified_symbol_traced(
        &mut self,
        name: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        match &name.data {
            tsox_frontend::ast::NodeData::Identifier(_) => match self.resolve_identifier(name) {
                Some(s) => Ok(s),
                None => Err((Arc::clone(name), String::new(), String::new())),
            },
            tsox_frontend::ast::NodeData::QualifiedName(data) => {
                self.resolve_qualified_tail(&data.left, &data.right, true)
            }

            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                let mut base = &pa.expression;
                while let tsox_frontend::ast::NodeData::ParenthesizedExpression(p) = &base.data {
                    base = &p.expression;
                }
                if matches!(
                    base.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) {
                    self.resolve_qualified_tail(base, &pa.name, false)
                } else {
                    Err((Arc::clone(name), String::new(), String::new()))
                }
            }
            _ => Err((Arc::clone(name), String::new(), String::new())),
        }
    }

    pub(crate) fn resolve_qualified_tail(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
        entity_name_ctx: bool,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        {
            let mut symbol = self.resolve_qualified_symbol_traced(left)?;
            let path_so_far = qualified_name_text(left);
            // Go resolveQualifiedName：限定名左侧一律按 Namespace 含义解析
            //（Go SymbolFlagsNamespace 含 Enum）。别名链断（对应
            // unknownSymbol 全含义）整体按 unknown 传播不报错；类型含义命中的
            // 左侧走 2694 type-as-namespace，其余 2503
            if entity_name_ctx {
                let (chain_flags, chain_complete) =
                    self.symbol_flags_with_alias_chain_ex(&symbol);
                if !chain_flags.intersects(
                    tsox_frontend::ast::SymbolFlags::NAMESPACE
                        | tsox_frontend::ast::SymbolFlags::ENUM,
                ) {
                    if !chain_complete {
                        return Ok(symbol);
                    }
                    let leftmost = crate::checker::checker::base_identifier_of(left);
                    if chain_flags.intersects(tsox_frontend::ast::SymbolFlags::TYPE) {
                        return Err((leftmost, qualified_name_text(left), String::new()));
                    }
                    return Err((leftmost, String::new(), String::new()));
                }
            }
            symbol = self.resolve_alias_base(symbol);
            // re-export 链（import { foo } → export { foo } → import * as foo）
            // 需循环 follow 到终点（namespace import 符号）才能查成员
            let mut alias_guard = 0;
            while symbol.flags == SymbolFlags::Alias && alias_guard < 10 {
                let next = self.resolve_alias_base(Arc::clone(&symbol));
                if !Arc::ptr_eq(&next, &symbol) {
                    symbol = next;
                    alias_guard += 1;
                    continue;
                }
                // binder 未挂 export_symbol 的 import 别名：检查期解析成员
                //（import {P as Q} from "a" → a 的导出 P → 其命名空间导入模块）
                match self.resolve_import_alias_target_symbol(&symbol) {
                    Some(resolved) if !Arc::ptr_eq(&resolved, &symbol) => {
                        symbol = resolved;
                        alias_guard += 1;
                    }
                    _ => break,
                }
            }

            if symbol.flags == SymbolFlags::Alias
                && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
            {
                symbol = module_sym;
            }

            let text = right.text();
            let mut next = symbol
                .exports
                .get(text)
                .or_else(|| symbol.members.get(text))
                .cloned()
                .or_else(|| self.global_this_export(&symbol, text))
                .or_else(|| self.ambient_namespace_local(&symbol, text))
                .or_else(|| self.object_literal_export_member(&symbol, text))
                .or_else(|| {
                    // 文件模块的星号导出链兜底（export * / export type * 的
                    // 成员在 exports 表外）
                    if symbol.flags.intersects(
                        SymbolFlags::ValueModule | SymbolFlags::NamespaceModule,
                    ) || symbol.declarations.iter().any(|d| {
                        d.kind == SyntaxKind::SourceFile
                    }) {
                        self.resolve_module_member_symbol(&symbol, text, 8)
                    } else {
                        None
                    }
                });

            if next.is_none()
                && let Some(ea_sym) = symbol.exports.get("export=")
                && let Some(decl) = ea_sym
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ExportAssignment)
                && let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
                && ea.is_export_equals
                && matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
            {
                let scope = symbol
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .cloned();
                if let Some(scope) = scope {
                    self.push_scope(&scope);
                    let target = self.resolve_identifier(&ea.expression);
                    self.pop_scope();
                    if let Some(target) = target
                        // Go getExportsOfSymbol：export= 目标不论实例化状态
                        // （纯类型命名空间 NamespaceModule 同样可被穿透查找）
                        && target.flags.intersects(
                            SymbolFlags::ValueModule | SymbolFlags::NamespaceModule,
                        )
                    {
                        next = target
                            .exports
                            .get(text)
                            .or_else(|| target.members.get(text))
                            .cloned()
                            .or_else(|| self.ambient_namespace_local(&target, text));
                    }
                }
            }

            let base_is_unresolved_require_alias = symbol.flags == SymbolFlags::Alias
                && symbol.declarations.iter().any(|d| {
                    if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                        && let tsox_frontend::ast::NodeData::ExternalModuleReference(ext) =
                            &ied.module_reference.data
                        && ext.expression.kind == SyntaxKind::StringLiteral
                    {
                        self.resolve_module_file_symbol(&ext.expression.text())
                            .is_none()
                    } else {
                        false
                    }
                });
            if base_is_unresolved_require_alias {
                return Ok(symbol);
            }
            match next {
                Some(next) => {
                    let resolved = if next.flags.intersects(SymbolFlags::Alias) {
                        let scope = symbol
                            .declarations
                            .iter()
                            .find(|d| {
                                d.kind == SyntaxKind::ModuleDeclaration
                                    || d.kind == SyntaxKind::SourceFile
                            })
                            .cloned();
                        if let Some(ref scope) = scope {
                            self.push_scope(scope);
                        }
                        let base = self.resolve_alias_base(Arc::clone(&next));
                        if scope.is_some() {
                            self.pop_scope();
                        }
                        base
                    } else {
                        match self.follow_alias(&next) {
                            Some(f) => f,
                            None => next,
                        }
                    };
                    Ok(resolved)
                }
                None => {
                    let _ = path_so_far;
                    Err((
                        Arc::clone(right),
                        self.namespace_full_path(&symbol),
                        self.node_source_text(right)
                            .unwrap_or_else(|| text.to_string()),
                    ))
                }
            }
        }
    }

    pub(crate) fn ambient_ancestor(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(a) = cur {
            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            cur = a.parent();
        }
        false
    }
}
