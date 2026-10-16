#![allow(unused_imports)]

use crate::checker::typenode_import_query::*;

impl Checker {
    pub(crate) fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_type_query_node"); 
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    fn module_specifier_of_external_ref(&self, module_reference: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("module_specifier_of_external_ref"); 
        let tsox_frontend::ast::NodeData::ExternalModuleReference(emr) = &module_reference.data
        else {
            return None;
        };
        let tsox_frontend::ast::NodeData::StringLiteral(s) = &emr.expression.data else {
            return None;
        };
        Some(s.text.trim_matches(['"', '\'', '`']).to_string())
    }

    fn resolve_module_file_symbol_relative(&self, spec: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_file_symbol_relative"); 
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

    /// Go node16/nodenext 下 ESM 文件相对无扩展名说明符的解析硬失败推定：
    /// resolver 相对臂禁目录查找与隐式扩展（resolver.go:1448-1453 的 !esmMode
    /// 门、:1475-1479 loadModuleFromFile），程序解析记录未解析，checker
    /// resolveExternalModule（checker.go:16279）报 TS2834 后返回 nil，导入符号
    /// 落 any。本仓 checker 侧相对说明符经朴素文件表解析（程序解析记录未
    /// 接线），此处按同一 resolver 规则推定未解析；模式取法对齐程序解析入口：
    /// 显式 resolution-mode 覆盖（仅 type-only）优先，否则文件 implied 格式
    pub(crate) fn node_next_esm_relative_import_unresolved(
        &self,
        import_decl: &Arc<Node>,
        decl_file: Option<&tsox_frontend::ast::SourceFile>,
        spec: &str,
    ) -> bool { ::tsox_core::fntrace::enter("node_next_esm_relative_import_unresolved");
        use tsox_core::core::compiler_options::ModuleKind;
        if !(spec.starts_with("./") || spec.starts_with("../"))
            || tsox_core::tspath::has_extension(spec)
        {
            return false;
        }
        if !matches!(
            self.compiler_options.module,
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext
        ) {
            return false;
        }
        let Some(file) = decl_file else {
            return false;
        };
        let read = |p: &str| self.program.read_file(p);
        let mode = match import_resolution_mode_override_of(import_decl) {
            ModuleKind::None => {
                tsox_tsoptions::tsoptions::implied_node_format_of_file(&file.file_name, &read)
            }
            overridden => overridden,
        };
        mode == ModuleKind::ESNext
    }

    pub(crate) fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_import_alias_target_symbol");
        // import X = require("./m") 形式：目标 = 模块的 export= 符号
        if let Some(decl) = alias
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ImportEqualsDeclaration(_)))
        {
            if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                // Go getTargetOfImportEqualsDeclaration（checker.go:15254）内部
                // 实体名引用（import O = Outer）经
                // getSymbolOfPartOfRightHandSideOfImportEquals（checker.go:15272-
                // 15291）resolveEntityName 解目标符号；别名链递归由调用侧
                // resolve_alias_base 承担
                if matches!(
                    data.module_reference.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    // Go 整名（import Y = A）按 Namespace 含义、限定名（import
                    // Y = A.B）按 Value|Type|Namespace 含义 dontResolveAlias
                    // 解析——就地同名局部值（namespace 内 var A 遮蔽全局 A）
                    // 被 meaning 过滤跳过，落全局命名空间符号
                    return self.get_symbol_of_part_of_right_hand_side_of_import_equals(
                        &data.module_reference,
                    );
                }
                let spec = self.module_specifier_of_external_ref(&data.module_reference)?;
                let spec_loc = match &data.module_reference.data {
                    NodeData::ExternalModuleReference(ext) => ext.expression.loc,
                    _ => data.module_reference.loc,
                };
                let decl_file = self
                    .get_source_file_of_node(decl)
                    .or_else(|| self.current_file.clone());
                let module_sym = match if spec.starts_with("./") || spec.starts_with("../") {
                    match decl_file.as_ref() {
                        Some(f) => {
                            let dir = match f.file_name.rfind('/') {
                                Some(i) => f.file_name[..i].to_string(),
                                None => String::new(),
                            };
                            self.resolve_module_file_symbol_in(&dir, &spec)
                        }
                        None => self.resolve_module_file_symbol_relative(&spec),
                    }
                } else {
                    self.resolve_module_file_symbol(&spec)
                } {
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
                                decl_file.clone(),
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
                        let disk_resolved = decl_file.as_ref().and_then(|f| {
                            self.program.resolve_external_module_path(
                                &trimmed,
                                &f.file_name,
                                tsox_core::core::compiler_options::ModuleKind::None,
                            )
                        });
                        if disk_resolved.is_none()
                            && !self.diagnostics.get_all().iter().any(|d| {
                                d.code == message.code && d.loc == spec_loc
                            })
                            && !self.program.diagnostics().iter().any(|d| {
                                d.code == message.code && d.loc == spec_loc
                            })
                        {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                decl_file.clone(),
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
        let (member_name, import_decl, alias_decl): (Option<String>, Arc<Node>, Arc<Node>) = {
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
                NodeData::ImportClause(_) => (Some("default".to_string()), decl.clone(), decl),
                NodeData::ImportSpecifier(d) => (
                    Some(
                        d.property_name
                            .as_ref()
                            .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    ),
                    decl.clone(),
                    decl,
                ),
                // import * as N：目标 = 模块符号整体（无成员名）
                NodeData::NamespaceImport(_) => (None, decl.clone(), decl),
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
        if !tsox_frontend::ast::is_string_literal_like(&module_spec_node) {
            return None;
        }
        let module_spec = module_spec_node.text().to_string();
        let decl_file = self
            .get_source_file_of_node(&import_decl)
            .or_else(|| self.current_file.clone());
        let esm_relative_unresolved = self.node_next_esm_relative_import_unresolved(
            &import_decl,
            decl_file.as_deref(),
            &module_spec,
        );
        // Go resolveExternalModule（checker.go:16089）相对说明符只认程序级解析
        // 记录（GetResolvedModule，compiler/program.go:654）。module
        // augmentation 体内的相对 import 不进 file.Imports（parser
        // collectModuleReferences，references.go:51-73，对外部模块文件不遍历
        // 增广体），程序记录必无条目 → 解析失败，别名落 errorType（.types 渲染
        // any，如 moduleAugmentationDisallowedExtensions 的 ./x0）
        let spec_is_relative = module_spec.starts_with("./") || module_spec.starts_with("../");
        let resolution_recorded = self.program.records_module_resolutions();
        let module_sym = if esm_relative_unresolved {
            None
        } else if spec_is_relative && resolution_recorded {
            decl_file
                .as_ref()
                .and_then(|f| self.program.get_resolved_module(&f.file_name, &module_spec))
                .and_then(|path| self.program.get_source_file(&path))
                // 记录命中但文件未入程序（重定向等）：对齐 Go
                // GetSourceFileForResolvedModule 缺失，按未解析处理
                .and_then(|sf| self.program.symbol_map().symbol_of(&sf.node).cloned())
                .map(|sym| self.get_merged_symbol(&sym))
        } else if spec_is_relative {
            let dir = decl_file.as_ref().map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            });
            match dir {
                Some(dir) => self.resolve_module_file_symbol_in(&dir, &module_spec),
                None => self.resolve_module_file_symbol(&module_spec),
            }
        } else {
            self.resolve_module_file_symbol(&module_spec)
        };
        let module_sym = if spec_is_relative && resolution_recorded {
            // 记录型程序：相对说明符不做盘上重解兜底（Go 无此臂，记录即裁决）
            module_sym
        } else {
            module_sym.or_else(|| {
                if esm_relative_unresolved {
                    return None;
                }
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let cur = decl_file.clone()?;
                let path = self.program.resolve_external_module_path(
                    &trimmed,
                    &cur.file_name,
                    tsox_core::core::compiler_options::ModuleKind::None,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()
            })
        };
        let module_sym = match module_sym {
            Some(sym) => sym,
            // Go getTargetOfNamespaceImport → resolveExternalModuleName：
            // 模块解析失败在说明符处报 2307 系
            None => {
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let (message, args) =
                    tsox_frontend::parser::cannot_resolve_module_error(&self.compiler_options, &trimmed);
                let disk_resolved = decl_file.as_ref().and_then(|f| {
                    self.program.resolve_external_module_path(
                        &trimmed,
                        &f.file_name,
                        tsox_core::core::compiler_options::ModuleKind::None,
                    )
                });
                let pattern_ambient = tsox_frontend::ast::pattern_ambient_module_with_attributes_exists(
                    self.program.source_files(),
                    &trimmed,
                    tsox_frontend::ast::import_attributes_of_declaration(&import_decl).as_ref(),
                );
                if disk_resolved.is_none()
                    && !pattern_ambient
                    && !self
                        .diagnostics
                        .get_all()
                        .iter()
                        .any(|d| d.code == message.code && d.loc == module_spec_node.loc)
                    && !self.program.diagnostics().iter().any(|d| {
                        d.code == message.code && d.loc == module_spec_node.loc
                    })
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        decl_file.clone(),
                        module_spec_node.loc,
                        message.clone(),
                        args,
                    ));
                }
                return None;
            }
        };
        // import * as N：Go getTargetOfNamespaceImport → resolveESModuleSymbol
        //（checker.go:15442/16501）经 resolveExternalModuleSymbol 穿透模块
        // export= 解到赋值目标符号，模块型有调用签名或可合成 default 时再包
        // 合成 default 包装型（augmentExportEquals7：export = <() => void> 的
        // import * as lib 呈 { default: () => void; }，非裸目标型）；包装
        // 不可得时回落 export= 穿透原臂。aliasTarget 按 Go resolveAlias
        //（checker.go:17227-17248 links.aliasTarget）驻留：克隆符号
        //（cloneTypeAsModuleType checker.go:16661-16675 每次 new_symbol 新
        // 对象）不驻留则 .types 渲染端可及链身份判定
        //（isAccessible symbolaccessibility.go:684-708 ctx.symbol ==
        // resolvedAliasSymbol）落空，回落容器限定名 import("./a").a
        if member_name.is_none() {
            if let Some(links) = self.alias_symbol_links.get(alias)
                && let Some(cached) = links.alias_target.clone()
                && !Arc::ptr_eq(&cached, alias)
            {
                return Some(cached);
            }
            if let Some(resolved) =
                self.resolve_es_module_symbol(Some(&module_sym), &alias_decl, &module_spec_node)
            {
                self.alias_symbol_links
                    .get_or_default(alias)
                    .alias_target = Some(Arc::clone(&resolved));
                return Some(resolved);
            }
            return self.resolve_import_alias_target_of_module(&module_sym);
        }
        let member_name = member_name.unwrap_or_default();
        // Go getTargetOfImportClause/getTargetOfImportSpecifier（checker.go:15329
        // 起）default 成员统一走 getTargetOfModuleDefault（checker.go:15351）：
        // exportDefault 缺失而可有合成 default（ambient/declaration 模块、含
        // export= 的 TS 文件，canHaveSyntheticDefault checker.go:15645）时穿透
        // export=（resolveExternalModuleSymbol checker.go:16491）取赋值目标——
        // `declare module "react" { export = React }` 的
        // `import React from "react"` 由此得 namespace React（typeof React）；
        // 普通具名 default 导出经 resolveExportByName 同位解析，行为不变
        if member_name == "default" {
            return self.get_target_of_module_default(&module_sym, &alias_decl, true);
        }
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

                    // Go getTargetOfAliasDeclaration（ExportAssignment 臂）：
                    // 实体名在模块作用域解析（含 file locals），非仅 members/exports
                    match self.resolve_export_assignment_target(&cur) {
                        Some(n) if !Arc::ptr_eq(&n, &cur) => cur = n,
                        _ => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::Interface
                        | tsox_frontend::ast::SymbolFlags::TypeAlias
                        | tsox_frontend::ast::SymbolFlags::Class
                        | tsox_frontend::ast::SymbolFlags::ENUM
                        | tsox_frontend::ast::SymbolFlags::TypeParameter,
                );
                // Go getTargetOfAliasDeclaration（ExportAssignment 臂）：目标实体
                // 直接返回（含值意义符号），别名本身不再回落
                if has_type_meaning || cur.flags.intersects(tsox_frontend::ast::SymbolFlags::VALUE)
                {
                    Some(cur)
                } else {
                    Some(t)
                }
            }
            other => other,
        };
        // Go resolveAlias 全链语义：`export { A }` 再导出的 import 别名是
        // 中间纯 alias，继续递归跟到最终非 alias 目标（环由跳数上限截断）
        let mut resolved = resolved;
        for _ in 0..4 {
            let Some(t) = resolved.clone() else { break };
            // Go resolveAlias + ast.IsNonLocalAlias（utilities.go）：仅纯 alias
            // 目标继续透传；interface/const 等与 import 合并的双意义符号是
            // resolveAlias 的链终点，停在自身以保留其类型含义
            if !t.flags.contains(tsox_frontend::ast::SymbolFlags::Alias)
                || Arc::ptr_eq(&t, alias)
                || t.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::VALUE
                        | tsox_frontend::ast::SymbolFlags::TYPE
                        | tsox_frontend::ast::SymbolFlags::NAMESPACE,
                )
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

    /// Go getTargetOfImportSpecifier（checker.go:15463-15478）：具名导入目标
    /// 以 dontResolveAlias=TRUE 取 getExportOfModule（checker.go:15614-15622）
    /// 的模块自有导出表原始条目（getExportsOfModuleWorker =
    /// maps.Clone(symbol.Exports)，具名 re-export 为表中延迟别名，不深解）。
    /// 仅适用于 ImportSpecifier 形态的别名；其余形态返回 None 由调用方走
    /// 既有深解链
    pub(crate) fn import_specifier_raw_export_entry(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("import_specifier_raw_export_entry");
        let decl = alias
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportSpecifier)
            .cloned()?;
        let member_name = match &decl.data {
            NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };
        let mut import_decl = decl.parent()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent()?;
        }
        let module_spec_node = match &import_decl.data {
            NodeData::ImportDeclaration(d) => Arc::clone(&d.module_specifier),
            _ => return None,
        };
        if !tsox_frontend::ast::is_string_literal_like(&module_spec_node) {
            return None;
        }
        let module_spec = module_spec_node.text().to_string();
        let decl_file = self
            .get_source_file_of_node(&import_decl)
            .or_else(|| self.current_file.clone());
        let module_sym = if module_spec.starts_with("./") || module_spec.starts_with("../") {
            let dir = decl_file.as_ref().map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            });
            match dir {
                Some(dir) => self.resolve_module_file_symbol_in(&dir, &module_spec),
                None => self.resolve_module_file_symbol(&module_spec),
            }
        } else {
            self.resolve_module_file_symbol(&module_spec)
        }
        .or_else(|| {
            let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
            let cur = decl_file.clone()?;
            let path = self.program.resolve_external_module_path(
                &trimmed,
                &cur.file_name,
                tsox_core::core::compiler_options::ModuleKind::None,
            )?;
            let sf = self.program.get_source_file(&path)?;
            self.program.symbol_map().symbol_of(&sf.node).cloned()
        })?;
        module_sym.exports.get(&member_name).cloned()
    }

    /// Go getTargetOfExportSpecifier（checker.go:15783-15797）：ExportSpecifier
    /// 解析 ExportDeclaration 模块说明符指向模块的成员
    ///（dontRecursivelyResolve=TRUE 不再深解成员别名）。仅适用于
    /// ExportSpecifier 形态的别名；其余形态返回 None 由调用方走既有深解链
    pub(crate) fn export_specifier_module_member(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("export_specifier_module_member");
        let decl = alias
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportSpecifier)
            .cloned()?;
        let member_name = match &decl.data {
            NodeData::ExportSpecifier(es) => es
                .property_name
                .as_ref()
                .unwrap_or(&es.name)
                .text()
                .trim_matches(['"', '\'', '`'])
                .to_string(),
            _ => return None,
        };
        let export_decl = decl.parent()?;
        let module_spec = match &export_decl.data {
            NodeData::ExportDeclaration(d) => d.module_specifier.as_ref()?,
            _ => return None,
        };
        let module_spec = module_spec.text().trim_matches(['"', '\'', '`']).to_string();
        let decl_file = self
            .get_source_file_of_node(&export_decl)
            .or_else(|| self.current_file.clone());
        let module_sym = if module_spec.starts_with("./") || module_spec.starts_with("../") {
            let dir = decl_file.as_ref().map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            });
            match dir {
                Some(dir) => self.resolve_module_file_symbol_in(&dir, &module_spec),
                None => self.resolve_module_file_symbol(&module_spec),
            }
        } else {
            self.resolve_module_file_symbol(&module_spec)
        }
        .or_else(|| {
            let cur = decl_file.clone()?;
            let path = self.program.resolve_external_module_path(
                &module_spec,
                &cur.file_name,
                tsox_core::core::compiler_options::ModuleKind::None,
            )?;
            let sf = self.program.get_source_file(&path)?;
            self.program.symbol_map().symbol_of(&sf.node).cloned()
        })?;
        self.resolve_module_member_symbol(&module_sym, &member_name, 8)
    }

    pub(crate) fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("file_module_exported_member"); 
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
                    if stmt.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export)
                        && let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data
                    {
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
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_import_alias_target_of_module"); 
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

/// Go GetModeForUsageLocation 的显式 resolution-mode 覆盖：仅 type-only
/// 导入/导出携带单键 `resolution-mode` 属性时生效（checker.go:16159 起，
/// import_resolution_mode_override 语义同源）
fn import_resolution_mode_override_of(
    import_decl: &Arc<Node>,
) -> tsox_core::core::compiler_options::ModuleKind {
    use tsox_core::core::compiler_options::ModuleKind;
    let tsox_frontend::ast::NodeData::ImportDeclaration(d) = &import_decl.data else {
        return ModuleKind::None;
    };
    let type_only = d.import_clause.as_ref().is_some_and(|c| {
        matches!(&c.data, NodeData::ImportClause(ic)
            if ic.phase_modifier == Some(SyntaxKind::TypeKeyword))
    });
    let Some(attrs) = d.attributes.as_ref() else {
        return ModuleKind::None;
    };
    if !type_only {
        return ModuleKind::None;
    }
    let tsox_frontend::ast::NodeData::ImportAttributes(data) = &attrs.data else {
        return ModuleKind::None;
    };
    if data.attributes.len() != 1 {
        return ModuleKind::None;
    }
    let tsox_frontend::ast::NodeData::ImportAttribute(attr) = &data.attributes.nodes[0].data else {
        return ModuleKind::None;
    };
    if attr.name.text() != "resolution-mode" {
        return ModuleKind::None;
    }
    match attr.value.text() {
        "import" => ModuleKind::ESNext,
        "require" => ModuleKind::CommonJS,
        _ => ModuleKind::None,
    }
}
