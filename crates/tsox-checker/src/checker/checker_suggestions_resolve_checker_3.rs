#![allow(unused_imports)]

use crate::checker::checker_suggestions_resolve::*;

impl Checker {
    // Go globalThisSymbol.Exports 即 globals 表（引用共享）；单表架构下以
    // 符号同一性判定后直接查 globals
    pub(crate) fn global_this_export(
        &self,
        symbol: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|gt| Arc::ptr_eq(gt, symbol))
        {
            self.globals.get(name).cloned()
        } else {
            None
        }
    }

    pub(crate) fn global_this_export_of_type(
        &self,
        t: &Arc<crate::checker::types::Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let symbol = t.symbol.as_ref()?;
        self.global_this_export(symbol, name)
    }

    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool {
        if std::env::var_os("TSOX_NO_AMBIENT").is_some() {
            return false;
        }
        ns.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ModuleDeclaration
                && (d.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self.ambient_ancestor(d)
                    || self
                        .get_source_file_of_node(d)
                        .is_some_and(|f| f.is_declaration_file))
                && !crate::binder::Binder::has_export_declarations(d)
        })
    }

    pub(crate) fn ambient_namespace_local(
        &self,
        ns: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !self.ambient_namespace_locals_visible(ns) {
            return None;
        }
        ns.declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .find_map(|d| {
                self.program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
                    .cloned()
            })
    }

    pub(crate) fn resolve_alias_target(&mut self, symbol: Arc<Symbol>) -> Option<Arc<Symbol>> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(symbol);
        }

        // binder 对 export/import specifier 已设 export_symbol 直连目标
        //（`export { foo }` → 文件 locals 的绑定）
        if let Some(target) = symbol.export_symbol.as_ref() {
            if !std::ptr::eq(
                std::sync::Arc::as_ptr(target) as *const u8,
                std::sync::Arc::as_ptr(&symbol) as *const u8,
            ) {
                return Some(Arc::clone(target));
            }
        }

        if symbol.declarations.iter().any(|d| {
            matches!(
                d.kind,
                SyntaxKind::NamespaceImport
                    | SyntaxKind::NamespaceExport
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier
            )
        }) {
            if let Some(module_sym) = self.resolve_import_alias_module(&symbol) {
                // Go resolveEntityName：named import 的 alias 目标 = 模块内同名
                // 导出（namespace import 才是模块符号本身）
                if symbol.declarations.iter().any(|d| {
                    matches!(d.kind, SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier)
                }) {
                    // import { P as Q } / export { x as y } from "m"：模块侧名是
                    // property_name，不是本地绑定/导出名
                    let import_name = symbol
                        .declarations
                        .iter()
                        .find_map(|d| match &d.data {
                            NodeData::ImportSpecifier(is) => Some(
                                is.property_name
                                    .as_ref()
                                    .unwrap_or(&is.name)
                                    .text()
                                    .to_string(),
                            ),
                            NodeData::ExportSpecifier(es) => Some(
                                es.property_name
                                    .as_ref()
                                    .unwrap_or(&es.name)
                                    .text()
                                    .to_string(),
                            ),
                            _ => None,
                        })
                        .unwrap_or_else(|| symbol.name.clone());
                    if let Some(named) = module_sym
                        .exports
                        .get(&import_name)
                        .cloned()
                        .or_else(|| module_sym.members.get(&import_name).cloned())
                    {
                        return Some(named);
                    }
                }
                return Some(module_sym);
            }
        }
        // `export { foo }`（无 from）：目标 = 所在文件符号的局部绑定
        //（index.ts 的 import * as foo 的 NamespaceImport alias）
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportSpecifier)
        {
            let mut cur = Arc::clone(decl);
            for _ in 0..6 {
                let Some(parent) = cur.parent() else { break };
                if parent.kind == SyntaxKind::SourceFile {
                    if let Some(locals) = self.program.symbol_map().locals.get(&parent.id())
                        && let Some(target) = locals.get(&symbol.name).cloned()
                    {
                        return Some(target);
                    }
                    if let Some(sf_sym) = self.program.symbol_map().symbol_of(&parent)
                        && let Some(target) = sf_sym.exports.get(&symbol.name).cloned()
                    {
                        return Some(target);
                    }
                    break;
                }
                cur = parent;
            }
        }
        // Go getTargetOfExportAssignment：export default X / export = X 的
        // 别名目标 = 表达式在所在模块作用域解析出的符号
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportAssignment)
            && let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
            && matches!(
                ea.expression.kind,
                SyntaxKind::Identifier | SyntaxKind::QualifiedName
            )
            && let Some(scope) = decl
                .parent()
                .filter(|p| p.kind == SyntaxKind::SourceFile)
                .or_else(|| {
                    decl.parent().and_then(|p| {
                        if p.kind == SyntaxKind::ModuleBlock {
                            p.parent()
                        } else {
                            None
                        }
                    })
                })
        {
            self.push_scope(&scope);
            let target = self.resolve_qualified_symbol(&ea.expression);
            self.pop_scope();
            if let Some(target) = target {
                return Some(target);
            }
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                if let tsox_frontend::ast::NodeData::ExternalModuleReference(ext) =
                    &data.module_reference.data
                    && ext.expression.kind == SyntaxKind::StringLiteral
                    && let Some(module_sym) =
                        self.resolve_module_file_symbol(&ext.expression.text())
                {
                    // Go getTargetOfImportEqualsDeclaration（外部模块引用）：
                    // 目标 = 模块 export= 别名符号（dontResolveAlias），别名侧
                    // 递归由 resolve_alias_base 承担（环检测对齐 Go resolveAlias）
                    if let Some(export_eq) = module_sym
                        .exports
                        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    {
                        let export_eq = Arc::clone(export_eq);
                        let resolved = self.resolve_alias_base(Arc::clone(&export_eq));
                        if self.alias_circular_reported.contains(&export_eq.id()) {
                            return None;
                        }
                        if Arc::ptr_eq(&resolved, &export_eq) {
                            return Some(module_sym);
                        }
                        return Some(resolved);
                    }
                    return Some(module_sym);
                }

                if matches!(
                    data.module_reference.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let mut current = Arc::clone(&symbol);
                    for _ in 0..4 {
                        let next = current
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                            .and_then(|d| {
                                if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(ied) =
                                    &d.data
                                    && matches!(
                                        ied.module_reference.kind,
                                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                    )
                                {
                                    Some(self.resolve_qualified_symbol(&ied.module_reference))
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        match next {
                            Some(n) => current = n,
                            None => break,
                        }
                        if !current.flags.intersects(SymbolFlags::Alias) {
                            return Some(current);
                        }
                    }
                    return Some(current);
                }
            }
        }
        Some(symbol)
    }

    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> {
        // 同 isExternalModuleNameRelative 语义：`.prisma/client` 是包名，不是相对路径
        if !specifier.starts_with("./") && !specifier.starts_with("../") {
            for file in self.program.source_files() {
                if file.external_module_indicator.is_some() {
                    continue;
                }
                if let tsox_frontend::ast::NodeData::SourceFile(sf) = &file.node.data {
                    for stmt in sf.statements.iter() {
                        if let tsox_frontend::ast::NodeData::ModuleDeclaration(md) = &stmt.data
                            && md.name.kind == SyntaxKind::StringLiteral
                            && md.name.text().trim_matches(['"', '\'']) == specifier
                        {
                            return self.program.symbol_map().symbol_of(stmt).cloned();
                        }
                    }
                }
            }
            // node_modules 向上查找（Go loadModuleFromFile 的 node 解析）：
            // <dir>/node_modules/<pkg>/index.{d.ts,ts,...}
            let file = self.display_enclosing_file.clone().or_else(|| self.current_file.clone())?;
            let mut dir = match file.file_name.rfind('/') {
                Some(i) => file.file_name[..i].to_string(),
                None => return None,
            };
            loop {
                let pkg_dir = format!("{dir}/node_modules/{specifier}");
                for index in ["./index.d.ts", "./index.ts", "./index.tsx"] {
                    if let Some(sym) = self.resolve_module_file_symbol_in(&pkg_dir, index) {
                        return Some(sym);
                    }
                }
                // Go loadModuleFromImmediateNodeModulesDirectory：实现包未命中
                // 时回退 @types 包（declaration-only；@scope/name → scope__name）
                let mangled = if let Some(rest) = specifier.strip_prefix('@') {
                    rest.split_once('/')
                        .map(|(scope, pkg)| format!("{scope}__{pkg}"))
                        .unwrap_or_else(|| rest.to_string())
                } else {
                    specifier.to_string()
                };
                let types_dir = format!("{dir}/node_modules/@types/{mangled}");
                for index in ["./index.d.ts", "./index.ts", "./index.tsx"] {
                    if let Some(sym) = self.resolve_module_file_symbol_in(&types_dir, index) {
                        return Some(sym);
                    }
                }
                let parent = match dir.rfind('/') {
                    Some(0) => "/".to_string(),
                    Some(i) => dir[..i].to_string(),
                    None => break,
                };
                if parent == dir {
                    break;
                }
                dir = parent;
            }
            return None;
        }
        let current = self.current_file.as_ref()?;
        let dir = match current.file_name.rfind('/') {
            Some(i) => &current.file_name[..i],
            None => "",
        };
        self.resolve_module_file_symbol_in(dir, specifier)
    }

    pub(crate) fn resolve_module_file_symbol_in(
        &self,
        dir: &str,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        let raw = specifier.strip_prefix("./").unwrap_or(specifier);

        let stripped = raw
            .strip_suffix(".js")
            .or_else(|| raw.strip_suffix(".jsx"))
            .unwrap_or(raw);
        let stripped = stripped
            .strip_suffix(".mjs")
            .or_else(|| stripped.strip_suffix(".cjs"))
            .unwrap_or(stripped);

        // 候选顺序对齐 tsc loadModuleFromFile：精确命中（含 .d.ts/.js 原样）、
        // .js 说明符回退到同名 .ts/.tsx/.d.ts、目录 index
        let candidates = [
            format!("{dir}/{raw}"),
            format!("{dir}/{stripped}.ts"),
            format!("{dir}/{stripped}.tsx"),
            format!("{dir}/{stripped}.d.ts"),
            format!("{dir}/{stripped}.js"),
            format!("{dir}/{stripped}.jsx"),
            format!("{dir}/{stripped}/index.ts"),
            format!("{dir}/{stripped}/index.d.ts"),
        ];
        let symbol_map = self.program.symbol_map();
        for cand in candidates {
            if let Some(sf) = self
                .program
                .source_files()
                .iter()
                .find(|f| f.file_name == cand)
            {
                if let Some(sym) = symbol_map.symbol_of(&sf.node) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        None
    }

    pub(crate) fn for_each_module_statement(
        &self,
        module_symbol: &Arc<Symbol>,
        mut f: impl FnMut(&Arc<Node>) -> bool,
    ) {
        use tsox_frontend::ast::NodeData;
        for decl in &module_symbol.declarations {
            let statements: Option<&Arc<tsox_frontend::ast::NodeList>> = match &decl.data {
                NodeData::SourceFile(sf) => Some(&sf.statements),
                NodeData::ModuleDeclaration(md) => match &md.body {
                    Some(body) => match &body.data {
                        NodeData::ModuleBlock(b) => Some(&b.statements),
                        _ => None,
                    },
                    None => None,
                },
                _ => None,
            };
            if let Some(list) = statements {
                for s in list.iter() {
                    if f(s) {
                        return;
                    }
                }
            }
        }
    }

    pub(crate) fn class_name_text(class: &Arc<Node>) -> String {
        match &class.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_default(),
            tsox_frontend::ast::NodeData::ClassExpression(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    pub(crate) fn generic_display_name(
        &self,
        name: &str,
        type_parameters: Option<&Arc<tsox_frontend::ast::NodeList>>,
    ) -> String {
        let Some(tps) = type_parameters else {
            return name.to_string();
        };
        let names: Vec<&str> = tps
            .nodes
            .iter()
            .filter_map(|tp| match &tp.data {
                tsox_frontend::ast::NodeData::TypeParameterDeclaration(d) => {
                    Some(d.name.text())
                }
                _ => None,
            })
            .collect();
        if names.is_empty() {
            name.to_string()
        } else {
            format!("{name}<{}>", names.join(", "))
        }
    }

    pub(crate) fn extends_heritage_expr_of(
        &self,
        class_node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let heritage = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            tsox_frontend::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        heritage?.iter().find_map(|clause| {
            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data
                && hc.token == SyntaxKind::ExtendsKeyword
            {
                return hc.types.iter().next().cloned();
            }
            None
        })
    }

    pub(crate) fn class_member_static_by_name(
        &self,
        class: &Arc<Node>,
        name: &str,
    ) -> Option<bool> {
        let members = match &class.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.members,
            tsox_frontend::ast::NodeData::ClassExpression(d) => &d.members,
            _ => return None,
        };
        for member in members.iter() {
            let member_name = match &member.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => &d.name,
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => &d.name,
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                _ => continue,
            };
            if member_name.kind == SyntaxKind::Identifier && member_name.text() == name {
                return Some(member.has_syntactic_modifier(ModifierFlags::Static));
            }
        }
        None
    }
}
