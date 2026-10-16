#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
use crate::checker::mig::r19k5_flags_ext::TypeFlagsExt;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::Arc;

impl Checker {
    pub fn get_target_of_binary_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_target_of_binary_expression"); 
        let NodeData::BinaryExpression(d) = &node.data else {
            return None;
        };
        if d.operator_token.kind == SyntaxKind::EqualsToken {
            let _left_type = self.check_expression_cached(&d.left);
            return self.resolve_entity_name(
                &d.right,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                true,
                true,
                Some(node),
            );
        }
        if node.kind == SyntaxKind::ElementAccessExpression
            || node.kind == SyntaxKind::PropertyAccessExpression
        {
            return self.get_target_of_access_expression(node);
        }
        None
    }

    pub fn get_export_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
        _location: &Arc<Node>,
        _dont_resolve_alias: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_export_of_module");
        if let Some(exp) = module_symbol.exports.get(name) {
            return Some(Arc::clone(exp));
        }
        // Go getExportOfModule（checker.go:15614-15624）经 getExportsOfSymbol
        //（16867-16876，Module 位走 getExportsOfModule worker 展开 export *）
        // 取解析导出后查名——跨包 export * 再导模块（pkg2 → @raymondfeng/pkg1
        // → ./types）的具名成员由此可达；本仓原始 exports 表只有 export * 占位
        self.resolve_module_member_symbol(module_symbol, name, 8)
    }

    pub fn resolve_es_module_symbol(
        &mut self,
        module_symbol: Option<&Arc<Symbol>>,
        node: &Arc<Node>,
        module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_es_module_symbol"); 
        let module_symbol = module_symbol?;
        let mut symbol = self.resolve_external_module_symbol_go(module_symbol);
        if tsox_frontend::ast::mig::m3g_2::is_non_local_alias(
            Some(symbol.as_ref()),
            SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
        ) {
            if let Some(source) = self.get_symbol_of_declaration(node) {
                symbol = self.resolve_indirection_alias(&source, &symbol);
            }
        }
        let reference_parent = module_specifier.parent()?;
        let namespace_import = if is_import_declaration(&reference_parent) {
            tsox_frontend::ast::mig::m3e_4::get_namespace_declaration_node(&reference_parent)
        } else {
            None
        };
        let is_import_call = tsox_frontend::ast::is_import_call(&reference_parent);
        if namespace_import.is_none() && !is_import_call {
            return Some(symbol);
        }
        let reference: Arc<Node> = if is_import_call {
            match &reference_parent.data {
                NodeData::CallExpression(d) => d.arguments.nodes.first()?.clone(),
                _ => return Some(symbol),
            }
        } else {
            match &reference_parent.data {
                NodeData::ImportDeclaration(d) => Arc::clone(&d.module_specifier),
                _ => return Some(symbol),
            }
        };
        let typ = self.get_type_of_symbol(&symbol);
        let import_attributes_type =
            self.get_import_attributes_type_for_module_specifier(&reference);
        if let Some(default_only_type) = self.get_type_with_synthetic_default_only(
            Some(&typ),
            &symbol,
            module_symbol,
            &reference,
            import_attributes_type.as_ref(),
        ) {
            return Some(self.clone_type_as_module_type(&symbol, &default_only_type, &reference_parent));
        }
        let target_file = module_symbol
            .declarations
            .iter()
            .find(|d| is_source_file(d))
            .and_then(|d| self.get_source_file_of_node(d));
        let usage_mode = self.get_emit_syntax_for_module_specifier_expression(&reference);
        let mut export_module_dot_exports_symbol: Option<Arc<Symbol>> = None;
        if namespace_import.is_some()
            && target_file.is_some()
            && matches!(
                self.module_kind,
                tsox_core::core::compiler_options::ModuleKind::Node20
                    | tsox_core::core::compiler_options::ModuleKind::NodeNext
            )
            && usage_mode == tsox_core::core::compiler_options::ModuleKind::CommonJS
            && target_file.as_ref().is_some_and(|f| {
                self.program.get_emit_module_format_of_file(&f.file_name)
                    == tsox_core::core::compiler_options::ModuleKind::ESNext
            })
        {
            export_module_dot_exports_symbol = self.get_export_of_module(
                &symbol,
                tsox_frontend::ast::INTERNAL_SYMBOL_NAME_MODULE_EXPORTS,
                namespace_import.as_ref().unwrap(),
                true,
            );
        }
        if let Some(export_module_dot_exports_symbol) = export_module_dot_exports_symbol {
            if self.has_signatures(&typ) {
                return Some(self.clone_type_as_module_type(
                    &export_module_dot_exports_symbol,
                    &typ,
                    &reference_parent,
                ));
            }
            return Some(export_module_dot_exports_symbol);
        }
        let is_esm_cjs_ref = target_file.is_some()
            && crate::checker::mig::wc3_2::is_esm_format_import_importing_commonjs_format_file(
                usage_mode,
                target_file
                    .as_ref()
                    .map(|f| self.program.get_emit_module_format_of_file(&f.file_name))
                    .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None),
            );
        if self.has_signatures(&typ)
            || self
                .get_property_of_type_ex(
                    &typ,
                    tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT,
                    true,
                    false,
                )
                .is_some()
            || is_esm_cjs_ref
        {
            let module_type = if typ.flags.intersects(TypeFlags::StructuredType) {
                self.get_type_with_synthetic_default_import_type(
                    &typ,
                    &symbol,
                    module_symbol,
                    Some(&reference),
                )
            } else {
                let parent = symbol.parent().unwrap_or_else(|| Arc::clone(module_symbol));
                self.create_default_property_wrapper_for_module(&symbol, &parent, None)
            };
            return Some(self.clone_type_as_module_type(&symbol, &module_type, &reference_parent));
        }
        Some(symbol)
    }

    pub fn can_have_synthetic_default(
        &mut self,
        file: Option<&Arc<Node>>,
        module_symbol: &Arc<Symbol>,
        dont_resolve_alias: bool,
        usage: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("can_have_synthetic_default"); 
        use crate::checker::mig::m2a::r18k8_flags::{
            MODULE_KIND_COMMON_JS, MODULE_KIND_ES_NEXT, MODULE_KIND_NODE16, MODULE_KIND_NODE_NEXT,
        };
        use crate::checker::utilities_get_assignment_target::is_syntactic_default;
        use crate::checker::utilities_has_only_expression_initialization::has_export_assignment_symbol;
        use tsox_core::core::compiler_options::ModuleKind;
        // Go canHaveSyntheticDefault（checker.go:15645-15703）
        let file_sf = file.and_then(|f| self.get_source_file_of_node(f));
        let mut usage_mode = ModuleKind::None;
        if file.is_some() {
            usage_mode = self.get_emit_syntax_for_module_specifier_expression(usage);
        }
        if file.is_some() && usage_mode != ModuleKind::None {
            let target_mode = file_sf
                .as_ref()
                .map(|sf| self.emit_implied_node_format_for_file(&sf.file_name))
                .unwrap_or(ModuleKind::None);
            if usage_mode == MODULE_KIND_ES_NEXT
                && target_mode == MODULE_KIND_COMMON_JS
                && MODULE_KIND_NODE16 <= self.module_kind
                && self.module_kind <= MODULE_KIND_NODE_NEXT
            {
                // Go checker.go:15653-15655：node16/nodenext 下 ESM 使用处导入
                // CJS 目标恒有合成 default
                return true;
            }
            if usage_mode == MODULE_KIND_ES_NEXT && target_mode == MODULE_KIND_ES_NEXT {
                return false;
            }
            // Go checker.go:15662-15675 project-reference 臂：checker 侧
            // Program trait 无 GetRedirectForResolution /
            // GetProjectReferenceFromOutputDts（语料程序无 project
            // references），按无重定向跳过
        }
        let is_declaration_file = file_sf.as_ref().is_some_and(|sf| sf.is_declaration_file);
        if file.is_none() || is_declaration_file {
            // Go checker.go:15677-15695：声明文件/无文件符号模块按解析后的
            // default 导出符号声明判定语法 default，再查 __esModule
            let default_export_symbol =
                self.resolve_export_by_name(module_symbol, "default", None, true);
            if default_export_symbol
                .as_ref()
                .is_some_and(|s| s.declarations.iter().any(|d| is_syntactic_default(d)))
            {
                return false;
            }
            if self
                .resolve_export_by_name(module_symbol, "__esModule", None, dont_resolve_alias)
                .is_some()
            {
                return false;
            }
            return true;
        }
        let f = file.unwrap();
        // Go checker.go:15696-15698：TS 文件仅在含 export= 时才有合成 default
        if !is_in_js_file(&f) {
            return has_export_assignment_symbol(module_symbol);
        }
        // Go checker.go:15701-15702：JS 文件无 ES2015+ 模块语法（external 指示
        // 为空或为文件自身）且无 __esModule 时才有合成 default
        let sf = file_sf.unwrap();
        (sf.external_module_indicator.is_none()
            || sf.external_module_indicator
                .as_ref()
                .is_some_and(|ind| ind.id() == sf.node.id()))
            && self
                .resolve_export_by_name(module_symbol, "__esModule", None, dont_resolve_alias)
                .is_none()
    }
}
