#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn module_can_have_synthetic_default(
        &mut self,
        module_symbol: &Arc<Symbol>,
    ) -> bool {
        if self.module_has_syntactic_default(module_symbol) {
            return false;
        }
        if module_symbol.exports.get("__esModule").is_some() {
            return false;
        }
        let file = module_symbol.declarations.iter().find(|d| {
            matches!(&d.data, tsox_frontend::ast::NodeData::SourceFile(_))
        });
        let file = match file {
            Some(d) => self.get_source_file_of_node(d),
            None => None,
        };
        // Go canHaveSyntheticDefault：ESM 使用处导入 CJS 目标在 node16/nodenext
        // 下恒有合成 default（usageMode/targetMode 分支）
        if let Some(f) = &file {
            let usage_esm = self
                .current_file
                .as_ref()
                .is_some_and(|u| u.file_name.ends_with(".mjs"));
            let target_cjs = f.common_js_module_indicator.is_some()
                || f.file_name.ends_with(".cjs");
            let node_format_module = matches!(
                self.compiler_options.module,
                tsox_core::core::compiler_options::ModuleKind::Node16
                    | tsox_core::core::compiler_options::ModuleKind::Node18
                    | tsox_core::core::compiler_options::ModuleKind::Node20
                    | tsox_core::core::compiler_options::ModuleKind::NodeNext
            );
            if usage_esm && target_cjs && node_format_module {
                return true;
            }
        }
        let is_ambient_or_declaration = module_symbol.declarations.iter().any(|d| match &d.data {
            tsox_frontend::ast::NodeData::ModuleDeclaration(_) => true,
            tsox_frontend::ast::NodeData::SourceFile(_) => self
                .get_source_file_of_node(d)
                .is_some_and(|f| f.is_declaration_file),
            _ => false,
        });
        if is_ambient_or_declaration {
            return true;
        }
        let Some(file) = file else {
            return true;
        };
        // Go canHaveSyntheticDefault 尾段：TS 文件仅在含 export= 时才有合成
        // default；JS 文件则要求无 ES2015+ 模块语法（external 指示为空或为
        // 文件自身），CJS 脚本（module.exports= 已置指示）无合成 default
        let is_js = matches!(
            file.script_kind,
            tsox_frontend::ast::ScriptKind::Js | tsox_frontend::ast::ScriptKind::Jsx
        );
        if !is_js {
            return module_symbol.exports.get("export=").is_some();
        }
        file.external_module_indicator.is_none()
            || file
                .external_module_indicator
                .as_ref()
                .is_some_and(|ind| ind.id() == file.node.id())
    }

    pub(crate) fn declaring_dir_of(&self, node: &Arc<Node>) -> Option<String> {
        self.get_source_file_of_node(node)
            .or_else(|| self.current_file.clone())
            .map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            })
    }
}
