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
        if let Some(decl) = alias
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ImportEqualsDeclaration(_)))
        {
            if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                let spec = self.module_specifier_of_external_ref(&data.module_reference)?;
                let spec_loc = match &data.module_reference.data {
                    NodeData::ExternalModuleReference(ext) => ext.expression.loc,
                    _ => data.module_reference.loc,
                };
                let module_sym = match self.resolve_module_file_symbol_relative(&spec) {
                    Some(sym) => {
                        // Go resolveExternalModule：目标文件无模块指示（脚本）
                        // 报 TS2306，参数为解析后文件名
                        let not_module_file = self
                            .program
                            .source_files()
                            .iter()
                            .find(|f| {
                                f.external_module_indicator.is_none()
                                    && f.common_js_module_indicator.is_none()
                                    && sym
                                        .declarations
                                        .iter()
                                        .any(|d| Arc::ptr_eq(d, &f.node))
                            })
                            .map(|f| f.file_name.clone());
                        if let Some(file_name) = not_module_file
                            && !self
                                .diagnostics
                                .get_all()
                                .iter()
                                .any(|d| d.code == 2306 && d.loc == spec_loc)
                        {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                spec_loc,
                                tsox_core::diagnostics::messages_generated::FILE_0_IS_NOT_A_MODULE,
                                vec![file_name],
                            ));
                        }
                        sym
                    }
                    None => {
                        // Go resolveExternalModuleName：import= require 别名
                        // 解析失败在说明符处报模块解析错误（2307 系）
                        let trimmed = spec.trim_matches(['"', '\'', '`']).to_string();
                        let (message, args) =
                            tsox_frontend::parser::cannot_resolve_module_error(
                                &self.compiler_options,
                                &trimmed,
                            );
                        if !self.diagnostics.get_all().iter().any(|d| {
                            d.code == message.code && d.loc == spec_loc
                        }) {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                spec_loc,
                                message.clone(),
                                args,
                            ));
                        }
                        return None;
                    }
                };
                return self.resolve_import_alias_target_of_module(&module_sym);
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
        let module_spec_node = match &import_decl.data {
            NodeData::ImportDeclaration(d) => Arc::clone(&d.module_specifier),
            _ => return None,
        };
        let module_spec = module_spec_node.text().to_string();
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
        let module_sym = match module_sym {
            Some(sym) => sym,
            // Go getTargetOfNamespaceImport → resolveExternalModuleName：
            // 模块解析失败在说明符处报 2307 系
            None => {
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let (message, args) =
                    tsox_frontend::parser::cannot_resolve_module_error(&self.compiler_options, &trimmed);
                if !self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == message.code && d.loc == module_spec_node.loc)
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        module_spec_node.loc,
                        message.clone(),
                        args,
                    ));
                }
                return None;
            }
        };
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

impl Checker {
    /// Go resolveExternalModuleSymbol：模块带 export= 时目标为导出实体
    ///（export=X 的 X 符号），否则为模块符号本身
    pub(crate) fn resolve_import_alias_target_of_module(
        &mut self,
        module_sym: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let export_equals = module_sym
            .exports
            .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            .cloned();
        if let Some(ee) = export_equals {
            if let Some(d) = ee
                .declarations
                .iter()
                .find(|d| matches!(d.data, NodeData::ExportAssignment(_)))
                && let NodeData::ExportAssignment(ea) = &d.data
            {
                if matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) && let Some(sf) = d.parent()
                {
                    // export = Foo.Member：限定名/属性访问在所在文件作用域解析
                    self.push_scope(&sf);
                    let target = self.resolve_qualified_symbol(&ea.expression);
                    self.pop_scope();
                    if let Some(target) = target {
                        return Some(target);
                    }
                }
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
        Some(Arc::clone(module_sym))
    }
}
