#![allow(unused_imports)]

use crate::checker::checker_get_excluded_symbol_flags::is_non_local_alias;
use crate::checker::checker_suggestions_resolve::*;

impl Checker {
    // Go globalThisSymbol.Exports 即 globals 表（引用共享）；单表架构下以
    // 符号同一性判定后直接查 globals。同一性按 Go getSymbolIfSameReference
    // 口径经 getMergedSymbol 对照（declare global 增强后 globals 里是克隆体）
    pub(crate) fn global_this_export(
        &self,
        symbol: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("global_this_export"); 
        let merged_view = |s: &Arc<Symbol>| self.get_merged_symbol(s);
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|gt| Arc::ptr_eq(&merged_view(gt), &merged_view(symbol)))
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("global_this_export_of_type"); 
        let symbol = t.symbol.as_ref()?;
        self.global_this_export(symbol, name)
    }

    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("ambient_namespace_locals_visible"); 
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("ambient_namespace_local"); 
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

    pub(crate) fn resolve_alias_target(&mut self, symbol: Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_alias_target"); 
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(symbol);
        }

        // Go resolveAlias（checker.go:17224）经 getDeclarationOfAliasSymbol
        //（checker.go:17361-17364，FindLast 别名类声明）取目标，不读
        // ExportSymbol——该链接只服务 getExportSymbolOfValueSymbolIfExported
        // 的 ExportValue 位映射；default 导出 local+export 对的本地符号与
        // import 合并出 Alias 位后按 export_symbol 直连会把别名解析劫持到
        // 导出符号（getDeclarationSpaces 的别名穿透与值含义链随之错位）

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
                    // Go getTargetOfImportSpecifier（checker.go:15463）→
                    // getExternalModuleMember（15484）→ getExportOfModule
                    //（15614-15624）→ getExportsOfSymbol（16867-16876，Module 位
                    // → getExportsOfModule worker 展开 export *）：模块自有
                    // exports/members 未命中时按同一解析深解具名成员——跨包
                    // `export *` 再导链上的 import 别名目标解到声明符号本身，
                    // 而非回落模块符号（可及链 isAccessible 身份比较随之成立，
                    // 显示链就地呈别名裸名）
                    if let Some(resolved) =
                        self.resolve_module_member_symbol(&module_sym, &import_name, 8)
                    {
                        return Some(resolved);
                    }
                }
                // Go resolveAlias（checker.go:17224-17250）的 aliasTarget =
                // getTargetOfAliasDeclaration NamespaceImport 臂 →
                // getTargetOfNamespaceImport（checker.go:15442-15452）→
                // resolveESModuleSymbol（checker.go:16501-16560）全量返回：
                // export= 模块带签名/default 时是 cloneTypeAsModuleType
                //（checker.go:16661-16675）的克隆符号。可及链命名依赖这一
                // 同一体——trySymbolTable 别名迭代（symbolaccessibility.go:600
                // resolveAlias）→ getCandidateListForSymbol（645-675）→
                // isAccessible（684-708）ctx.symbol==resolvedAliasSymbol 身份
                // 命中呈裸名；解到裸赋值目标（类符号）则与克隆身份落空，
                // getSymbolChain（nodebuilderimpl.go:1152-1186）容器上爬经
                // getAliasForSymbolInContainer（symbolaccessibility.go:356-361
                // container==parent fast-path）回落 import("./a").a 限定名。
                // resolve_es_module_symbol 已实现该全量语义（求值端
                // get_type_of_symbol_alias_inner → resolve_import_alias_target_symbol
                // 同径），本臂委托之；节点链不齐再走深解后备
                if symbol
                    .declarations
                    .iter()
                    .any(|d| d.kind == SyntaxKind::NamespaceImport)
                {
                    if let Some(ns_decl) = symbol
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::NamespaceImport)
                        .cloned()
                        && let Some(mut import_decl) = ns_decl.parent()
                    {
                        while !matches!(import_decl.data, tsox_frontend::ast::NodeData::ImportDeclaration(_))
                            && let Some(p) = import_decl.parent()
                        {
                            import_decl = p;
                        }
                        if let tsox_frontend::ast::NodeData::ImportDeclaration(import_data) =
                            &import_decl.data
                            && tsox_frontend::ast::is_string_literal_like(&import_data.module_specifier)
                        {
                            // Go resolveAlias 的 aliasTarget 驻留
                            //（checker.go:17227-17248 links.aliasTarget）：克隆符号
                            //（cloneTypeAsModuleType checker.go:16661-16675 每次
                            // new_symbol 新对象）须与求值端共享同一对象——
                            // isAccessible（symbolaccessibility.go:684-708）按
                            // ctx.symbol == resolvedAliasSymbol 命中裸名，各造
                            // 克隆则身份落空回落 getSymbolChain 容器限定名
                            // import("./a").a
                            if let Some(links) = self.alias_symbol_links.get(&symbol)
                                && let Some(target) = links.alias_target.clone()
                                && !Arc::ptr_eq(&target, &symbol)
                            {
                                return Some(target);
                            }
                            if let Some(resolved) = self.resolve_es_module_symbol(
                                Some(&module_sym),
                                &ns_decl,
                                &import_data.module_specifier,
                            ) {
                                self.alias_symbol_links
                                    .get_or_default(&symbol)
                                    .alias_target = Some(Arc::clone(&resolved));
                                return Some(resolved);
                            }
                        }
                    }
                    let excludes =
                        tsox_frontend::ast::SymbolFlags::VALUE
                            | tsox_frontend::ast::SymbolFlags::TYPE
                            | tsox_frontend::ast::SymbolFlags::NAMESPACE;
                    let ee = module_sym
                        .exports
                        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                        .cloned();
                    if let Some(ee) = ee {
                        let mut target = Arc::clone(&ee);
                        if is_non_local_alias(&ee, excludes) {
                            let mut seen: Vec<u64> = vec![ee.id()];
                            let mut cur = Arc::clone(&ee);
                            let mut cyclic = false;
                            self.alias_resolution_stack.push(Arc::clone(&ee));
                            for _ in 0..8 {
                                let next = self
                                    .resolve_alias_target(Arc::clone(&cur))
                                    .unwrap_or_else(|| Arc::clone(&cur));
                                if Arc::ptr_eq(&next, &cur) {
                                    break;
                                }
                                if seen.contains(&next.id()) {
                                    cyclic = true;
                                    break;
                                }
                                seen.push(next.id());
                                cur = next;
                                if !is_non_local_alias(&cur, excludes) {
                                    break;
                                }
                            }
                            self.alias_resolution_stack.pop();
                            if !cyclic && !Arc::ptr_eq(&cur, &ee) {
                                target = cur;
                            }
                        }
                        return Some(target);
                    }
                }
                return Some(module_sym);
            }
        }
        // `export { foo }`（无 from）：目标 = 所在文件符号的局部绑定
        //（index.ts 的 import * as foo 的 NamespaceImport alias）
        // 默认导入绑定（Go getTargetOfImportClause）：目标 = 模块 default 导出
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportClause)
        {
            if let NodeData::ImportClause(ic) = &decl.data
                && ic.name.as_ref().is_some_and(|n| n.text() == symbol.name)
                && let Some((module, _)) = self.import_declaration_context(decl)
            {
                if let Some(target) = self.resolve_default_export_target_for(&module) {
                    return Some(target);
                }
                return Some(
                    self.unknown_symbol
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&symbol)),
                );
            }
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportSpecifier)
        {
            let no_from = decl.parent().is_some_and(|clause| {
                clause.parent().is_some_and(|export_decl| {
                    !matches!(
                        &export_decl.data,
                        NodeData::ExportDeclaration(d) if d.module_specifier.is_some()
                    )
                })
            });
            let target_name = if no_from {
                match &decl.data {
                    NodeData::ExportSpecifier(es) => es
                        .property_name
                        .as_ref()
                        .unwrap_or(&es.name)
                        .text()
                        .trim_matches(['"', '\'', '`'])
                        .to_string(),
                    _ => symbol.name.clone(),
                }
            } else {
                symbol.name.clone()
            };
            let mut cur = Arc::clone(decl);
            for _ in 0..6 {
                let Some(parent) = cur.parent() else { break };
                if parent.kind == SyntaxKind::SourceFile {
                    if let Some(locals) = self.program.symbol_map().locals.get(&parent.id())
                        && let Some(target) = locals.get(&target_name).cloned()
                    {
                        return Some(target);
                    }
                    if let Some(sf_sym) = self.program.symbol_map().symbol_of(&parent)
                        && let Some(target) = sf_sym.exports.get(&target_name).cloned()
                        && !(no_from && Arc::ptr_eq(&target, &symbol))
                    {
                        return Some(target);
                    }
                    if no_from
                        && let Some(sf_sym) = self.program.symbol_map().symbol_of(&parent)
                        && let Some(target) = sf_sym.members.get(&target_name).cloned()
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
        //（含 Foo.Member 属性访问形态，Go getTargetOfAccessExpression）
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportAssignment)
            && let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
            && ea.expression.kind == SyntaxKind::ClassExpression
            && let Some(target) = self.program.symbol_map().symbol_of(&ea.expression)
        {
            // Go getTargetOfAliasLikeExpression（checker.go:15833-15835）：
            // IsClassExpression → checkExpressionCached(expression).symbol，
            // export = class B {} 的 export= 别名目标为类表达式符号，别名
            // 解析链（resolveAlias 透传）与可及链身份判定同取该符号
            return Some(Arc::clone(&target));
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportAssignment)
            && let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
            && matches!(
                ea.expression.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::QualifiedName
                    | SyntaxKind::PropertyAccessExpression
            )            && let Some(scope) = decl
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
                {
                    // Go resolveExternalModuleName（checker.go:15942）：相对说明符
                    // 相对声明所在文件解析；非相对走 ambient/node_modules 查找
                    let spec = ext
                        .expression
                        .text()
                        .trim_matches(['"', '\'', '`'])
                        .to_string();
                    let module_sym = if spec.starts_with("./") || spec.starts_with("../") {
                        let decl_file = self.get_source_file_of_node(decl);
                        match decl_file.as_ref() {
                            Some(f) => {
                                let dir = match f.file_name.rfind('/') {
                                    Some(i) => f.file_name[..i].to_string(),
                                    None => String::new(),
                                };
                                self.resolve_module_file_symbol_in(&dir, &spec)
                            }
                            None => None,
                        }
                    } else {
                        self.resolve_module_file_symbol(&spec)
                    };
                    let Some(module_sym) = module_sym else {
                        return None;
                    };
                    // Go getTargetOfImportEqualsDeclaration（外部模块引用）：
                    // 目标 = 模块 export= 别名符号（dontResolveAlias），别名侧
                    // 递归由 resolve_alias_base 承担（环检测对齐 Go resolveAlias）
                    if let Some(export_eq) = module_sym
                        .exports
                        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    {
                        // Go resolveExternalModuleSymbol（checker.go:16488-16496）：
                        // 模块带 export= 时目标恒为该符号（getMergedSymbol），
                        // 非 export= 符号不回落模块符号本身；export= 为纯别名
                        //（export = X）时 resolve_alias_base 透传，对应
                        // Go resolveAlias 的 IsNonLocalAlias →
                        // resolveIndirectionAlias（checker.go:17248-17251）
                        let export_eq = Arc::clone(export_eq);
                        let resolved = self.resolve_alias_base(Arc::clone(&export_eq));
                        if self.alias_circular_reported.contains(&export_eq.id()) {
                            return None;
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
                                    // Go resolveAlias → getTargetOfAliasDeclaration →
                                    // getTargetOfImportEqualsDeclaration：
                                    // getSymbolOfPartOfRightHandSideOfImportEquals
                                    // 按 Namespace/Value|Type|Namespace 含义
                                    // dontResolveAlias 解实体名——就地同名局部值
                                    // 不参与解析（namespace 内 var A 遮蔽全局 A 时
                                    // 仍解到全局命名空间符号）
                                    Some(
                                        self.get_symbol_of_part_of_right_hand_side_of_import_equals(
                                            &ied.module_reference,
                                        ),
                                    )
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
        // Go getTargetOfNamespaceExportDeclaration → resolveExternalModuleSymbol(
        // parent, dontResolveAlias=true)：export= 别名符号原样返回（getMergedSymbol，
        // 不就地解析），递归解析与环检测由 resolve_alias_base 的 is_pure_alias
        // 分支承担（Go resolveIndirectionAlias → resolveAlias）
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::NamespaceExportDeclaration)
            && let Some(parent_sym) = decl
                .parent()
                .and_then(|p| self.program.symbol_map().symbol_of(&p).cloned())
        {
            if let Some(ee) = parent_sym
                .exports
                .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            {
                return Some(self.get_merged_symbol(ee));
            }
            return Some(parent_sym);
        }
        Some(symbol)
    }

    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_file_symbol"); 
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
                            let sym = self.program.symbol_map().symbol_of(stmt).cloned()?;
                            return Some(self.get_merged_symbol(&sym));
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_file_symbol_in"); 
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
        // .js 说明符回退到同名 .ts/.tsx/.d.ts、目录 index。tsc 解析候选
        // （normalizePathForCJSResolution，resolver.go）先经 normalizePath
        // 折叠 ../ ./ 段再试文件—— checker 侧按名匹配前同样归一化，否则
        // ../pkg 形态说明符带字面 .. 段永不命中已加载源文件（跨目录相对导入
        // 误报 TS2307，如 declarationEmitCrossFileImportTypeOfAmbientModule
        // 的 import { item } from "../somepackage"）
        let candidates = [
            format!("{dir}/{raw}"),
            format!("{dir}/{stripped}.ts"),
            format!("{dir}/{stripped}.tsx"),
            format!("{dir}/{stripped}.d.ts"),
            format!("{dir}/{stripped}.js"),
            format!("{dir}/{stripped}.jsx"),
            format!("{dir}/{stripped}/index.ts"),
            format!("{dir}/{stripped}/index.d.ts"),
        ]
        .map(|cand| tsox_core::tspath::normalize_path(&cand));
        let symbol_map = self.program.symbol_map();
        for cand in candidates {
            let sf = self
                .program
                .source_files()
                .iter()
                .find(|f| f.file_name == cand)
                .cloned()
                .or_else(|| {
                    let real = self.program.canonicalize_path(&cand);
                    if real != cand {
                        self.program
                            .source_files()
                            .iter()
                            .find(|f| f.file_name == real)
                            .cloned()
                    } else {
                        None
                    }
                });
            if let Some(sym) = sf.and_then(|sf| symbol_map.symbol_of(&sf.node).cloned()) {
                return Some(self.get_merged_symbol(&sym));
            }
        }
        None
    }

    pub(crate) fn for_each_module_statement(
        &self,
        module_symbol: &Arc<Symbol>,
        mut f: impl FnMut(&Arc<Node>) -> bool,
    ) { ::tsox_core::fntrace::enter("for_each_module_statement"); 
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

    pub(crate) fn class_name_text(class: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("class_name_text"); 
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
    ) -> String { ::tsox_core::fntrace::enter("generic_display_name"); 
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
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("extends_heritage_expr_of"); 
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
    ) -> Option<bool> { ::tsox_core::fntrace::enter("class_member_static_by_name"); 
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
