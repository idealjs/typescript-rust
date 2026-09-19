#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    /// follow_alias 的解析兜底：export_symbol 链断开时按声明形态
    /// （ImportSpecifier/ImportClause/NamespaceImport/ExportSpecifier）解析目标
    pub fn follow_alias_resolving(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let chained = self.follow_alias(symbol)?;
        if !Arc::ptr_eq(&chained, symbol) {
            return Some(chained);
        }
        let r = self.resolve_alias_by_declaration(symbol);
        r
    }

    pub(crate) fn resolve_alias_by_declaration(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        // 合并符号（如 JS 文件里 import 与 @typedef 同名合并）的声明列表混有
        // 非别名声明；alias 解析只看别名形态声明（对齐 Go resolveAlias）
        let decl = symbol
            .declarations
            .iter()
            .find(|d| is_alias_declaration(d))
            .cloned()
            .or_else(|| {
                symbol
                    .value_declaration
                    .clone()
                    .or_else(|| symbol.declarations.first().cloned())
            })?;
        match &decl.data {
            NodeData::ImportSpecifier(d) => {
                let name = d
                    .property_name
                    .as_ref()
                    .unwrap_or(&d.name)
                    .text()
                    .trim_matches(['"', '\'', '`'])
                    .to_string();
                let (module, _) = self.import_declaration_context(&decl)?;
                self.resolve_module_member_symbol(&module, &name, 8)
            }
            NodeData::NamespaceImport(_) => {
                let (module, _) = self.import_declaration_context(&decl)?;
                Some(module)
            }
            NodeData::ImportClause(d) => {
                let (module, spec) = self.import_declaration_context(&decl)?;
                if let Some(default_name) = &d.name
                    && default_name.text() == symbol.name
                {
                    return self.resolve_default_export_target(&module);
                }
                let _ = spec;
                Some(module)
            }
            NodeData::ExportSpecifier(d) => {
                let name = d
                    .property_name
                    .as_ref()
                    .unwrap_or(&d.name)
                    .text()
                    .trim_matches(['"', '\'', '`'])
                    .to_string();
                let export_decl = self
                    .ancestor_of_kind(&decl, SyntaxKind::ExportDeclaration)?;
                let NodeData::ExportDeclaration(ed) = &export_decl.data else {
                    return None;
                };
                let file_module = self.module_symbol_of_containing_file(&export_decl)?;
                let module = match &ed.module_specifier {
                    Some(spec) => {
                        let text = spec.text().trim_matches(['"', '\'', '`']).to_string();
                        self.resolve_module_spec_from(&file_module, &text)?
                    }
                    // 无 from 的 export {X}：目标是本模块的本地声明（members），
                    // 不得回到 exports 表（会命中别名自身）
                    None => {
                        return file_module.members.get(&name).cloned();
                    }
                };
                self.resolve_module_member_symbol(&module, &name, 8)
            }
            NodeData::ImportEqualsDeclaration(d) => {
                // Go getTargetOfImportEqualsDeclaration：外部模块引用经
                // resolveExternalModuleSymbol 穿透 export=（目标为导出实体
                // 而非模块符号本身）
                if let NodeData::ExternalModuleReference(ext) = &d.module_reference.data {
                    let spec = ext
                        .expression
                        .text()
                        .trim_matches(['"', '\'', '`'])
                        .to_string();
                    return self
                        .resolve_module_file_symbol(&spec)
                        .and_then(|module_sym| {
                            self.resolve_import_alias_target_of_module(&module_sym)
                        });
                }
                self.resolve_qualified_symbol_traced(&d.module_reference).ok()
            }
            // JS 赋值别名（module.exports.x = expr，绑定期 expression_is_alias）：
            // 目标是右侧表达式符号（Go getTargetOfAliasDeclaration 的 Binary 分支）
            NodeData::BinaryExpression(d) => self.resolve_identifier(&d.right),
            // JS `var x = require("./m")`（binder 绑为 Alias）：目标是模块符号
            //（Go getTargetOfAliasDeclaration 的 VariableDeclaration →
            // getTargetOfImportEqualsDeclaration）
            NodeData::VariableDeclaration(d) => {
                let NodeData::CallExpression(call) = &d.initializer.as_ref()?.data else {
                    return None;
                };
                if !matches!(&call.expression.data, NodeData::Identifier(i) if i.text == "require")
                {
                    return None;
                }
                let spec_node = call.arguments.nodes.first()?;
                if !matches!(
                    spec_node.kind,
                    SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
                ) {
                    return None;
                }
                let spec = spec_node
                    .text()
                    .trim_matches(['"', '\'', '`'])
                    .to_string();
                let file_module = self.module_symbol_of_containing_file(&decl);
                let file_module = file_module?;
                let r = self.resolve_module_spec_from(&file_module, &spec);
                r
            }
            NodeData::ExportAssignment(d) if d.is_export_equals => {
                // export = <expr>：别名目标是表达式符号（含 Foo.Member 形态）
                match d.expression.kind {
                    SyntaxKind::Identifier
                    | SyntaxKind::QualifiedName
                    | SyntaxKind::PropertyAccessExpression => {
                        self.resolve_qualified_symbol(&d.expression)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// import 声明上下文：(目标模块符号, 说明符文本)
    pub(crate) fn import_declaration_context(
        &mut self,
        decl: &Arc<Node>,
    ) -> Option<(Arc<Symbol>, String)> {
        let import_decl = self
            .ancestor_of_kind(decl, SyntaxKind::ImportDeclaration)?;
        let NodeData::ImportDeclaration(d) = &import_decl.data else {
            return None;
        };
        let spec = d
            .module_specifier
            .text()
            .trim_matches(['"', '\'', '`'])
            .to_string();
        let file_module = self.module_symbol_of_containing_file(&import_decl)?;
        let module = self.resolve_module_spec_from(&file_module, &spec)?;
        Some((module, spec))
    }

    fn module_symbol_of_containing_file(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let file = self.get_source_file_of_node(node)?;
        let program = self.program.symbol_map();
        program.symbol_of(&file.node).map(Arc::clone)
    }

    fn ancestor_of_kind<'a>(
        &self,
        node: &'a Arc<Node>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            if n.kind == kind {
                return Some(n);
            }
            cur = n.parent();
        }
        None
    }
}

fn is_alias_declaration(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportClause
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceExport
            | SyntaxKind::ExportAssignment
    )
}

impl Checker {
    /// 模块说明符字符串（import/export 的 specifier、require(...)、import()）
    /// 的 quickinfo 符号：名字按说明符文本（Go getSymbolAtLocation 32033-32043）
    pub(crate) fn module_symbol_of_specifier(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let spec = node.text().trim_matches(['"', '\'', '`']).to_string();
        let is_specifier_slot = node.parent().as_ref().is_some_and(|p| {
            let slot = match &p.data {
                NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
                NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
                _ => None,
            };
            slot.is_some_and(|s| Arc::ptr_eq(&s, node))
        }) || matches!(
            (&node.parent().as_ref().map(|p| p.kind), &node.parent().as_ref().map(|p| &p.data)),
            (Some(SyntaxKind::CallExpression), Some(NodeData::CallExpression(d)))
                if d.expression.kind == SyntaxKind::ImportKeyword
                    || matches!(&d.expression.data, NodeData::Identifier(i) if i.text == "require")
        ) || node
            .parent()
            .as_ref()
            .is_some_and(|p| {
                matches!(&p.data,
                    NodeData::ExternalModuleReference(d) if Arc::ptr_eq(&d.expression, node))
            });
        if !is_specifier_slot {
            return None;
        }
        let Some(file) = self.get_source_file_of_node(node) else {
            return None;
        };
        let file_module = self
            .program
            .symbol_map()
            .symbol_of(&file.node)
            .map(Arc::clone)?;
        let resolved = self.resolve_module_spec_from(&file_module, &spec)?;
        let mut sym = Symbol::new(SymbolFlags::ValueModule, format!("\"{spec}\""));
        sym.value_declaration = resolved.value_declaration.clone();
        Some(Arc::new(sym))
    }
}
