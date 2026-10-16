#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    /// Go getTargetOfModuleDefault：default 导出符号解析；export default <entity>
    /// 的 Alias 跟随其实体目标，export= 模块回落合成 default（resolveExternalModuleSymbol）
    pub(crate) fn resolve_default_export_target_for(
        &mut self,
        module_sym: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_default_export_target_for"); 
        let mut target = self.resolve_module_member_symbol(module_sym, "default", 8);
        if target.is_none() {
            // Go getTargetOfModuleDefault：无合成 default 的模块不走 export= 回落
            //（module_can_have_synthetic_default 对齐 Go canHaveSyntheticDefault）
            if !self.module_can_have_synthetic_default(module_sym, None) {
                return None;
            }
            let export_eq = module_sym
                .exports
                .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                .cloned();
            if let Some(ee) = export_eq {
                let resolved = self.resolve_export_equals_target(&ee);
                if resolved.flags.intersects(SymbolFlags::VALUE) {
                    target = Some(resolved);
                }
            }
            // Go getTargetOfModuleDefault（checker.go:15390-15400）hasSyntheticDefault/
            // hasDefaultOnly 臂：无真实 default 导出且可合成 default 时 alias 目标 =
            // resolveExternalModuleSymbol（模块符号本体，有 export= 为 export= 符号）；
            // 空手而回使 resolve_alias 落 unknown，可及链经该别名进模块 exports 表
            // 寻段（getCandidateListForSymbol）断链，显示链坍缩 import("./m").X
            // 而非别名限定（allowSyntheticDefaultImports1 anchor L8）
            if target.is_none() {
                target = Some(self.resolve_external_module_symbol(module_sym, true));
            }
        }
        for _ in 0..4 {
            let cur = target.clone()?;
            let is_default_alias = cur.flags.contains(SymbolFlags::Alias)
                && cur.declarations.iter().any(|d| {
                    matches!(&d.data, NodeData::ExportAssignment(ea) if !ea.is_export_equals)
                });
            if !is_default_alias {
                break;
            }
            match self.resolve_export_assignment_target(&cur) {
                Some(next) if !Arc::ptr_eq(&next, &cur) => target = Some(next),
                _ => break,
            }
        }
        target
    }

    pub(crate) fn resolve_module_member_symbol(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_member_symbol"); 
        if depth == 0 {
            return None;
        }
        let sym = self.namespace_member_recursive(module_sym, name);
        if let Some(sym) = &sym {
            if let Some(target) = &sym.export_symbol
                && !Arc::ptr_eq(target, &sym)
            {
                return Some(Arc::clone(target));
            }
        }
        // export= 模块（module.exports = foo）的具名导入：目标在 export= 值的
        // 属性上（Go tryGetMemberInModuleExportsAndProperties）
        if sym.is_none()
            && let Some(target) = self.try_get_member_in_module_exports_and_properties(name, module_sym)
        {
            return Some(target);
        }

        let mut clause_hits: Vec<(String, Option<String>)> = Vec::new();
        let mut ns_export_target: Option<Arc<Symbol>> = None;
        let mut star_specs: Vec<String> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && d.export_clause.is_none()
                && let Some(spec) = d.module_specifier.as_ref()
            {
                star_specs.push(spec.text().trim_matches(['"', '\'', '`']).to_string());
            }
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && clause.kind == SyntaxKind::NamespaceExport
                && tsox_frontend::ast::node_data_generated::node_name(clause)
                    .is_some_and(|n| n.text() == name)
                && let Some(spec) = d.module_specifier.as_ref()
            {
                // export * as a from '...'：成员名映射目标模块符号（命名空间对象）
                if let Some(text) = self.resolve_module_spec_from(
                    module_sym,
                    spec.text().trim_matches(['"', '\'', '`']),
                ) {
                    ns_export_target = Some(text);
                }
                return true;
            }
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        let imported = spec
                            .property_name
                            .as_ref()
                            .unwrap_or(&spec.name)
                            .text()
                            .trim_matches(['"', '\'', '`'])
                            .to_string();
                        let module_text = d.module_specifier.as_ref().map(|module_spec| {
                            module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string()
                        });
                        clause_hits.push((imported, module_text));
                        return true;
                    }
                }
            }
            false
        });
        if let Some(target) = ns_export_target {
            return Some(target);
        }
        // 普通 `export * from '...'`：递归星号目标查名（具名/本地优先已保证）
        for spec in star_specs {
            let Some(target) = self
                .resolve_module_spec_from(module_sym, &spec)
                .or_else(|| self.resolve_module_file_symbol(&spec))
            else {
                continue;
            };
            if let Some(found) =
                self.resolve_module_member_symbol(&target, name, depth - 1)
            {
                return Some(found);
            }
        }
        for (imported, module_text) in clause_hits {
            // Go getTargetOfExportSpecifier（checker.go:15783-15797）：无 from
            // 的 `export { local as name }` 经 resolveEntityName 在所在文件
            // 作用域解析本地名（locals 优先，exports/members 次之）——重入
            // 模块 exports 表查同名条目会漏纯本地绑定（default 导出 local+
            // export 对与 import 合并出的别名符号等），值含义链随之断
            if module_text.is_none()
                && let Some(file_node) = module_sym
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::SourceFile)
                && let Some(sym) = self.export_specifier_file_scope_target(&file_node, &imported)
            {
                return Some(sym);
            }
            let target_module = match module_text {
                None => Arc::clone(module_sym),
                Some(text) => match self.resolve_module_spec_from(module_sym, &text) {
                    Some(m) => m,
                    None => continue,
                },
            };
            if let Some(target) =
                self.resolve_module_member_symbol(&target_module, &imported, depth - 1)
            {
                return Some(target);
            }
        }
        sym
    }

    // Go getTargetOfExportSpecifier 的文件作用域解析（resolveEntityName）：
    // 直名 locals→members→exports。命中符号不作含义过滤——值/类型链由调用方
    // （resolve_alias_base 的 is_pure_alias 递归、getSymbolFlagsEx 式链积）继续
    pub(crate) fn export_specifier_file_scope_target(
        &self,
        file_node: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        symbol_map
            .locals
            .get(&file_node.id())
            .and_then(|l| l.get(name))
            .or_else(|| {
                symbol_map
                    .symbol_of(file_node)
                    .and_then(|sym| sym.members.get(name))
            })
            .or_else(|| {
                symbol_map
                    .symbol_of(file_node)
                    .and_then(|sym| sym.exports.get(name))
            })
            .cloned()
    }

    pub(crate) fn resolve_module_spec_from(
        &self,
        base_module: &Arc<Symbol>,
        specifier: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_spec_from"); 
        // tspath isExternalModuleNameRelative 语义：仅 ./ ../ 开头是相对路径，
        // `.prisma/client` 这类点前缀包名走 node_modules 解析
        let base_file = base_module
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d));
        if !specifier.starts_with("./") && !specifier.starts_with("../") {
            // Go resolveModuleName 以导入文件为 containingFile：re-export 链中途
            // 解析包名说明符必须相对 base 模块所在目录，不能沿用当前检查文件
            if let Some(f) = &base_file
                && let Some(sym) = self.resolve_package_symbol_from(&f.file_name, specifier)
            {
                return Some(sym);
            }
            return self.resolve_module_file_symbol(specifier);
        }
        let dir = match &base_file {
            Some(f) => f
                .file_name
                .rfind('/')
                .map(|i| f.file_name[..i].to_string())
                .unwrap_or_default(),
            None => return None,
        };
        if let Some(sym) = self.resolve_module_file_symbol_in(&dir, specifier) {
            return Some(sym);
        }
        let file = base_file?;
        let path = self.program.resolve_external_module_path(
            specifier,
            &file.file_name,
            tsox_core::core::compiler_options::ModuleKind::None,
        )?;
        let sf = self.program.get_source_file(&path)?;
        Some(self.get_merged_symbol(&self.program.symbol_map().symbol_of(&sf.node).cloned()?))
    }

    fn resolve_package_symbol_from(
        &self,
        containing_file: &str,
        spec: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_package_symbol_from"); 
        if let Some(path) = self.program.resolve_external_module_path(
            spec,
            containing_file,
            tsox_core::core::compiler_options::ModuleKind::None,
        ) && let Some(sf) = self.program.get_source_file(&path)
            && let Some(sym) = self.program.symbol_map().symbol_of(&sf.node).cloned()
        {
            return Some(self.get_merged_symbol(&sym));
        }
        let mut dir = match containing_file.rfind('/') {
            Some(i) => containing_file[..i].to_string(),
            None => return None,
        };
        loop {
            let pkg_dir = format!("{dir}/node_modules/{spec}");
            for index in ["./index.d.ts", "./index.ts", "./index.tsx"] {
                if let Some(sym) = self.resolve_module_file_symbol_in(&pkg_dir, index) {
                    return Some(sym);
                }
            }
            let mangled = if let Some(rest) = spec.strip_prefix('@') {
                rest.split_once('/')
                    .map(|(scope, pkg)| format!("{scope}__{pkg}"))
                    .unwrap_or_else(|| rest.to_string())
            } else {
                spec.to_string()
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
        None
    }

    pub(crate) fn spec_of_dynamic_import_call(&self, node: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("spec_of_dynamic_import_call");
        if node.kind != SyntaxKind::CallExpression {
            return None;
        }
        let (callee, args) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, &d.arguments),
            _ => return None,
        };
        if callee.kind != SyntaxKind::ImportKeyword {
            return None;
        }
        let spec_node = args.iter().next()?;
        if spec_node.kind != SyntaxKind::StringLiteral {
            return None;
        }
        Some(spec_node.text().trim_matches(['"', '\'', '`']).to_string())
    }
}
