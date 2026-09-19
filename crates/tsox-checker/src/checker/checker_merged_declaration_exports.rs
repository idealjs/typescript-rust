#![allow(unused_imports)]

use crate::checker::checker::*;
use tsox_frontend::ast::utilities::get_module_instance_state;
use tsox_frontend::ast::utilities::is_ambient_module;
use tsox_frontend::ast::utilities::ModuleInstanceState;

const SPACE_TYPE: u8 = 1 << 0;
const SPACE_VALUE: u8 = 1 << 1;
const SPACE_NAMESPACE: u8 = 1 << 2;

impl Checker {
    pub(crate) fn check_exports_on_merged_declarations(&mut self, node: &Arc<Node>) {
        let Some(symbol) = self.program.symbol_map().symbol_of(node).cloned() else {
            return;
        };
        let declarations = self.merged_declaration_view(node, &symbol);
        let first_of_kind = declarations
            .iter()
            .find(|d| d.kind == node.kind)
            .is_some_and(|d| Arc::ptr_eq(d, node));
        if !first_of_kind {
            return;
        }
        let mut exported = 0u8;
        let mut non_exported = 0u8;
        let mut default_exported = 0u8;
        for d in declarations.iter() {
            let spaces = self.declaration_spaces(d);
            let (is_export, is_default) = self.effective_export_default_flags(d);
            if is_export {
                if is_default {
                    default_exported |= spaces;
                } else {
                    exported |= spaces;
                }
            } else {
                non_exported |= spaces;
            }
        }
        let common_exports_locals = exported & non_exported;
        let common_default_non_default = default_exported & (exported | non_exported);
        if common_exports_locals == 0 && common_default_non_default == 0 {
            return;
        }
        for d in declarations.iter() {
            let spaces = self.declaration_spaces(d);
            let Some(name) = tsox_frontend::ast::utilities::get_name_of_declaration(d) else {
                continue;
            };
            let display = self
                .node_source_text(&name)
                .unwrap_or_else(|| name.text().to_string());
            if common_default_non_default != 0 && spaces & common_default_non_default != 0 {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        MERGED_DECLARATION_0_CANNOT_INCLUDE_A_DEFAULT_EXPORT_DECLARATION_CONSIDER_ADDING_A_SEPARATE_EXPORT_DEFAULT_0_DECLARATION_INSTEAD,
                    vec![display.clone(), display],
                ));
            } else if spaces & common_exports_locals != 0 {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        INDIVIDUAL_DECLARATIONS_IN_MERGED_DECLARATION_0_MUST_BE_ALL_EXPORTED_OR_ALL_LOCAL,
                    vec![display],
                ));
            }
        }
    }

    // Go declareModuleMember 分表：非导出成员进容器节点 locals、导出成员
    // 额外在容器符号 exports 建导出符号；locals/exports 按容器节点隔离，
    // 检查仅在节点直属容器的同名符号声明并集上进行（跨 namespace 块不合并）
    fn merged_declaration_view(
        &self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> {
        let mut decls: Vec<Arc<Node>> = Vec::new();
        let mut push_symbol = |s: &Arc<Symbol>, decls: &mut Vec<Arc<Node>>| {
            for d in s.declarations.iter() {
                if !decls.iter().any(|x| Arc::ptr_eq(x, d)) {
                    decls.push(Arc::clone(d));
                }
            }
        };
        push_symbol(symbol, &mut decls);
        let name = tsox_frontend::ast::utilities::get_name_of_declaration(node)
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| symbol.name.clone());
        if name.is_empty() {
            return decls;
        }
        let enclosing = |n: &Arc<Node>| -> Option<Arc<Node>> {
            let mut cur = n.parent();
            while let Some(p) = cur {
                if matches!(
                    p.kind,
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                ) {
                    return Some(p);
                }
                if matches!(
                    p.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::EnumDeclaration
                        | SyntaxKind::InterfaceDeclaration
                ) {
                    return None;
                }
                cur = p.parent();
            }
            None
        };
        let Some(container) = enclosing(node) else {
            return decls;
        };
        let same_container = |d: &Arc<Node>| enclosing(d).is_some_and(|c| c.id() == container.id());
        decls.retain(|d| same_container(d));
        if let Some(locals) = self.program.symbol_map().locals.get(&container.id())
            && let Some(s) = locals.get(&name)
        {
            push_symbol(s, &mut decls);
        }
        if let Some(cs) = self.program.symbol_map().symbol_of(&container) {
            for table in [&cs.exports, &cs.members] {
                if let Some(s) = table.get(&name) {
                    push_symbol(s, &mut decls);
                }
            }
        }
        decls.retain(|d| same_container(d));
        decls
    }

    fn declaration_spaces(&mut self, node: &Arc<Node>) -> u8 {
        match node.kind {
            SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSDocTypedefTag
            | SyntaxKind::JSDocCallbackTag => SPACE_TYPE,
            SyntaxKind::ModuleDeclaration => {
                if is_ambient_module(node)
                    || get_module_instance_state(node) != ModuleInstanceState::NonInstantiated
                {
                    SPACE_NAMESPACE | SPACE_VALUE
                } else {
                    SPACE_NAMESPACE
                }
            }
            SyntaxKind::ClassDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember => SPACE_TYPE | SPACE_VALUE,
            SyntaxKind::VariableDeclaration
            | SyntaxKind::BindingElement
            | SyntaxKind::FunctionDeclaration => SPACE_VALUE,
            SyntaxKind::MethodSignature | SyntaxKind::PropertySignature => SPACE_TYPE,
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportClause
            | SyntaxKind::ImportSpecifier => self.alias_declaration_spaces(node, 0),
            SyntaxKind::ExportAssignment | SyntaxKind::BinaryExpression => {
                let is_alias = match &node.data {
                    tsox_frontend::ast::NodeData::ExportAssignment(e) => matches!(
                        &e.expression.data,
                        tsox_frontend::ast::NodeData::Identifier(_)
                            | tsox_frontend::ast::NodeData::QualifiedName(_)
                    ),
                    tsox_frontend::ast::NodeData::BinaryExpression(_) => true,
                    _ => false,
                };
                if is_alias {
                    self.alias_declaration_spaces(node, 0)
                } else {
                    SPACE_VALUE
                }
            }
            _ => 0,
        }
    }

    // Go getDeclarationSpaces 别名穿透：resolveAlias 目标声明的空间并集；
    // 自解析（非别名/回到自身）或超长链回退 VALUE 位，防环
    fn alias_declaration_spaces(&mut self, node: &Arc<Node>, depth: u8) -> u8 {
        if depth > 8 {
            return SPACE_VALUE;
        }
        let Some(s) = self.program.symbol_map().symbol_of(node).cloned() else {
            return SPACE_VALUE;
        };
        if !s.flags.contains(SymbolFlags::Alias) {
            return SPACE_VALUE;
        }
        let target = self.resolve_alias_base(Arc::clone(&s));
        if Arc::ptr_eq(&target, &s) {
            return SPACE_VALUE;
        }
        target
            .declarations
            .iter()
            .fold(0u8, |acc, d| {
                acc | match d.kind {
                    SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::NamespaceImport
                    | SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportAssignment
                    | SyntaxKind::BinaryExpression => self.alias_declaration_spaces(d, depth + 1),
                    _ => self.declaration_spaces(d),
                }
            })
    }

    pub(crate) fn effective_export_default_flags(&mut self, node: &Arc<Node>) -> (bool, bool) {
        let mut flags = self.get_combined_modifier_flags(node);
        if !node
            .parent()
            .is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                )
            })
            && node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
            && !flags.contains(ModifierFlags::Ambient)
            && Self::enclosing_export_context_container(node)
                .is_some_and(|c| c.flags.contains(tsox_frontend::ast::NodeFlags::ExportContext))
            && !Self::in_global_scope_augmentation(node)
        {
            flags |= ModifierFlags::Export;
        }
        (
            flags.contains(ModifierFlags::Export),
            flags.contains(ModifierFlags::Default),
        )
    }

    fn enclosing_export_context_container(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut cur = node.parent();
        while let Some(p) = cur {
            if matches!(
                p.kind,
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
            ) {
                return Some(p);
            }
            cur = p.parent();
        }
        None
    }

    fn in_global_scope_augmentation(node: &Arc<Node>) -> bool {
        node.parent()
            .is_some_and(|p| p.kind == SyntaxKind::ModuleBlock)
            && node
                .parent()
                .and_then(|p| p.parent())
                .is_some_and(|gp| tsox_frontend::ast::is_global_scope_augmentation(&gp))
    }
}
