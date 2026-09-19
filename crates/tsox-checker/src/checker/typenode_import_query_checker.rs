#![allow(unused_imports)]

use crate::checker::typenode_import_query::*;

impl Checker {
    pub(crate) fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    fn module_specifier_of_external_ref(&self, module_reference: &Arc<Node>) -> Option<String> {
        let tsox_frontend::ast::NodeData::ExternalModuleReference(emr) = &module_reference.data
        else {
            return None;
        };
        let tsox_frontend::ast::NodeData::StringLiteral(s) = &emr.expression.data else {
            return None;
        };
        Some(s.text.trim_matches(['"', '\'', '`']).to_string())
    }

    fn resolve_module_file_symbol_relative(&self, spec: &str) -> Option<Arc<Symbol>> {
        // Go moduleSpecifierIsRelative：目录级文件解析只对相对说明符，
        // 裸说明符仅走 ambient/node_modules（否则同目录文件按基名误命中）
        if !(spec.starts_with("./") || spec.starts_with("../")) {
            return None;
        }
        let file = self.display_enclosing_file.clone().or_else(|| self.current_file.clone())?;
        let dir = match file.file_name.rfind('/') {
            Some(i) => file.file_name[..i].to_string(),
            None => String::new(),
        };
        self.resolve_module_file_symbol_in(&dir, spec)
    }

    pub(crate) fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        // import X = require("./m") 形式：目标 = 模块的 export= 符号
        if std::env::var_os("TSOX_DEBUG_QI").is_some() && alias.name == "C" {
            eprintln!("[req-alias] C decls={:?}", alias.declarations.iter().map(|d| d.kind).collect::<Vec<_>>());
        }
        if let Some(decl) = alias
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ImportEqualsDeclaration(_)))
        {
            if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                let spec = self.module_specifier_of_external_ref(&data.module_reference)?;
                let module_sym = self.resolve_module_file_symbol_relative(&spec)?;
                // export = X：解析 export assignment 指向的符号（export=X 的 X 标识符）
                let export_equals = module_sym
                    .exports
                    .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    .cloned();
                if std::env::var_os("TSOX_DEBUG_QI").is_some() {
                    eprintln!("[req-alias] ee_found={}", export_equals.is_some());
                }
                if let Some(ee) = export_equals {
                    if let Some(d) = ee
                        .declarations
                        .iter()
                        .find(|d| matches!(d.data, NodeData::ExportAssignment(_)))
                        && let NodeData::ExportAssignment(ea) = &d.data
                    {
                        // X 在模块文件顶层容器里（file locals / module members）
                        let expr_name = ea.expression.text().to_string();
                        let sym_map = self.program.symbol_map();
                        let file_locals = d
                            .parent()
                            .as_ref()
                            .and_then(|sf| sym_map.locals.get(&sf.id()));
                        if let Some(cs) = file_locals.and_then(|l| l.get(&expr_name).cloned()) {
                            return Some(cs);
                        }
                        if let Some(cs) = module_sym.members.get(&expr_name).cloned() {
                            return Some(cs);
                        }
                    }
                    return Some(ee);
                }
                return Some(module_sym);
            }
        }
        let (member_name, import_decl): (Option<String>, Arc<Node>) = {
            let decl = alias
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause
                            | SyntaxKind::ImportSpecifier
                            | SyntaxKind::NamespaceImport
                    )
                })?
                .clone();
            match &decl.data {
                NodeData::ImportClause(_) => (Some("default".to_string()), decl),
                NodeData::ImportSpecifier(d) => (
                    Some(
                        d.property_name
                            .as_ref()
                            .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    ),
                    decl,
                ),
                // import * as N：目标 = 模块符号整体（无成员名）
                NodeData::NamespaceImport(_) => (None, decl),
                _ => return None,
            }
        };
        let mut import_decl = import_decl.parent()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent()?;
        }
        let module_spec = match &import_decl.data {
            NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self.resolve_module_file_symbol(&module_spec).or_else(|| {
            let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
            let cur = self.current_file.clone()?;
            let path = self.program.resolve_external_module_path(
                &trimmed,
                &cur.file_name,
                tsox_core::core::compiler_options::ModuleKind::None,
            )?;
            let sf = self.program.get_source_file(&path)?;
            self.program.symbol_map().symbol_of(&sf.node).cloned()
        });
        let module_sym = module_sym?;
        // import * as N：别名目标即模块符号
        if member_name.is_none() {
            return Some(module_sym);
        }
        let member_name = member_name.unwrap_or_default();
        let resolved = self
            .resolve_module_member_symbol(&module_sym, &member_name, 8)
            .or_else(|| self.file_module_exported_member(&module_sym, &member_name));
        let resolved = match resolved {
            Some(t)
                if !t.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::Interface
                        | tsox_frontend::ast::SymbolFlags::TypeAlias
                        | tsox_frontend::ast::SymbolFlags::Class
                        | tsox_frontend::ast::SymbolFlags::ENUM
                        | tsox_frontend::ast::SymbolFlags::TypeParameter,
                ) =>
            {
                let mut cur = Arc::clone(&t);
                for _ in 0..4 {
                    if cur.flags != tsox_frontend::ast::SymbolFlags::Alias {
                        break;
                    }

                    let next = cur
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .and_then(|d| match &d.data {
                            NodeData::ExportAssignment(ea)
                                if matches!(
                                    ea.expression.kind,
                                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                ) =>
                            {
                                Some(ea.expression.text().to_string())
                            }
                            _ => None,
                        })
                        .and_then(|n| {
                            module_sym
                                .members
                                .get(&n)
                                .cloned()
                                .or_else(|| module_sym.exports.get(&n).cloned())
                        });
                    match next {
                        Some(n) => cur = n,
                        None => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::Interface
                        | tsox_frontend::ast::SymbolFlags::TypeAlias
                        | tsox_frontend::ast::SymbolFlags::Class
                        | tsox_frontend::ast::SymbolFlags::ENUM
                        | tsox_frontend::ast::SymbolFlags::TypeParameter,
                );
                if has_type_meaning { Some(cur) } else { Some(t) }
            }
            other => other,
        };
        // Go resolveAlias 全链语义：`export { A }` 再导出的 import 别名是
        // 中间纯 alias，继续递归跟到最终非 alias 目标（环由跳数上限截断）
        let mut resolved = resolved;
        for _ in 0..4 {
            let Some(t) = resolved.clone() else { break };
            // Go resolveAlias：Alias 位仍在即继续（import 别名与 const 合并的
            // 双意义符号也须跟到 import 目标定类型意义）
            if !t.flags.contains(tsox_frontend::ast::SymbolFlags::Alias)
                || Arc::ptr_eq(&t, alias)
                || !t.declarations.iter().any(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause
                            | SyntaxKind::ImportSpecifier
                            | SyntaxKind::NamespaceImport
                    )
                })
            {
                break;
            }
            match self.resolve_import_alias_target_symbol(&t) {
                Some(next) if !Arc::ptr_eq(&next, &t) => resolved = Some(next),
                _ => break,
            }
        }
        resolved
    }

    pub(crate) fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !module_sym
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return None;
        }
        if let Some(s) = module_sym.exports.get(name) {
            return Some(Arc::clone(s));
        }
        let sym_map = self.program.symbol_map();
        let mut found: Option<Arc<Symbol>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(ea) => {
                    if !ea.is_export_equals && name == "default" && found.is_none() {
                        let by_name = match &ea.expression.kind {
                            SyntaxKind::Identifier => module_sym
                                .members
                                .get(ea.expression.text())
                                .cloned()
                                .or_else(|| module_sym.exports.get(ea.expression.text()).cloned()),
                            _ => None,
                        };
                        found = by_name.or_else(|| {
                            sym_map.symbol_of(stmt).cloned().or_else(|| {
                                stmt.expression()
                                    .and_then(|e| sym_map.symbol_of(e).cloned())
                            })
                        });
                    }
                }
                NodeData::VariableStatement(vs) => {
                    if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                        for decl in vdl.declarations.iter() {
                            if decl.name().is_some_and(|n| n.text() == name) {
                                found = sym_map.symbol_of(decl).cloned();
                            }
                        }
                    }
                }
                _ => {
                    if stmt.name().is_some_and(|n| n.text() == name)
                        && (stmt.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export)
                            || stmt
                                .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Default))
                    {
                        found = sym_map.symbol_of(stmt).cloned();
                    }
                }
            }
            false
        });
        found
    }
}
