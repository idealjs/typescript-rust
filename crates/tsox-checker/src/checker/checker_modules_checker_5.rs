#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn module_can_have_synthetic_default(
        &mut self,
        module_symbol: &Arc<Symbol>,
        usage: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("module_can_have_synthetic_default");
        use crate::checker::utilities_get_assignment_target::is_syntactic_default;
        use crate::checker::utilities_has_only_expression_initialization::has_export_assignment_symbol;
        use tsox_core::core::compiler_options::ModuleKind as M;
        let file = module_symbol.declarations.iter().find(|d| {
            matches!(&d.data, tsox_frontend::ast::NodeData::SourceFile(_))
        });
        let file = match file {
            Some(d) => self.get_source_file_of_node(d),
            None => None,
        };
        // Go canHaveSyntheticDefault（checker.go:15645-15703）usageMode 臂按
        // usage 节点（模块说明符）取 emit 语法；无 usage 节点可用的
        // member-lookup 近似调用方退回当前检查文件 import 语句的 emit 语法
        //（usage_emit_mode_for_import，对齐 getEmitSyntaxForUsageLocationWorker
        // 的 fileEmitMode 尾段）
        let mut usage_mode = M::None;
        if file.is_some() {
            usage_mode = match usage {
                Some(u) => self.get_emit_syntax_for_module_specifier_expression(u),
                None => self
                    .current_file
                    .as_ref()
                    .map(|u| self.usage_emit_mode_for_import(&u.file_name))
                    .unwrap_or(M::None),
            };
        }
        if let Some(f) = &file {
            if usage_mode != M::None {
                let target_mode = self.emit_implied_node_format_for_file(&f.file_name);
                if usage_mode == M::ESNext
                    && target_mode == M::CommonJS
                    && M::Node16 <= self.module_kind
                    && self.module_kind <= M::NodeNext
                {
                    // Go checker.go:15653-15655：node16/nodenext 下 ESM 使用处导入
                    // CJS 目标恒有合成 default（先于语法 default/__esModule 判定）
                    return true;
                }
                if usage_mode == M::ESNext && target_mode == M::ESNext {
                    // Go checker.go:15657-15661：两侧均确定为 ESM 时无合成 default
                    return false;
                }
                // Go checker.go:15662-15675 project-reference 臂：checker 侧
                // Program trait 无 GetRedirectForResolution /
                // GetProjectReferenceFromOutputDts（语料程序无 project
                // references），按无重定向跳过
            }
        }
        // Go checker.go:15677-15695：声明文件/无文件符号模块按解析后的 default
        // 导出符号声明判定语法 default，再查 __esModule
        let is_declaration = file.as_ref().is_some_and(|f| f.is_declaration_file);
        if file.is_none() || is_declaration {
            let default_export_symbol =
                self.resolve_export_by_name(module_symbol, "default", None, true);
            if default_export_symbol
                .as_ref()
                .is_some_and(|s| s.declarations.iter().any(|d| is_syntactic_default(d)))
            {
                return false;
            }
            if module_symbol.exports.get("__esModule").is_some() {
                return false;
            }
            return true;
        }
        let f = file.as_ref().unwrap();
        // Go checker.go:15696-15698：TS 文件仅在含 export= 时才有合成 default
        //（语法 default/__esModule 不参与判定）
        let is_js = matches!(
            f.script_kind,
            tsox_frontend::ast::ScriptKind::Js | tsox_frontend::ast::ScriptKind::Jsx
        );
        if !is_js {
            return has_export_assignment_symbol(module_symbol);
        }
        // Go checker.go:15701-15702：JS 文件无 ES2015+ 模块语法（external 指示
        // 为空或为文件自身）且无 __esModule 时才有合成 default
        (f.external_module_indicator.is_none()
            || f.external_module_indicator
                .as_ref()
                .is_some_and(|ind| ind.id() == f.node.id()))
            && module_symbol.exports.get("__esModule").is_none()
    }

    pub(crate) fn declaring_dir_of(&self, node: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("declaring_dir_of"); 
        self.get_source_file_of_node(node)
            .or_else(|| self.current_file.clone())
            .map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            })
    }
}
