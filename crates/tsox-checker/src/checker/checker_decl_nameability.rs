#![allow(unused_imports)]

use crate::checker::checker_modules::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};

const MAX_WALK_DEPTH: usize = 4;
const MAX_CANDIDATES: usize = 8;

impl Checker {
    /// 声明发射「cannot be named」检查：导出变量推断类型引用的符号，
    /// 其所在模块只能以深入 node_modules 的 specifier 引用时报 TS2883。
    pub(crate) fn check_type_nameability(
        &mut self,
        file: &Arc<SourceFile>,
        name_node: &Arc<Node>,
        decl_name: &str,
        decl_type: &Arc<Type>,
        imported_files: &[String],
        imported_specs: &[String],
        imported_symbol_ids: &[u64],
    ) {
        let mut candidates = Vec::new();
        self.collect_nameability_candidates(decl_type, 0, &mut candidates);
        for sym in candidates {
            if self.report_unnameable_symbol(
                file,
                name_node,
                decl_name,
                &sym,
                imported_files,
                imported_specs,
                imported_symbol_ids,
            ) {
                return;
            }
        }
    }

    fn collect_nameability_candidates(
        &mut self,
        t: &Arc<Type>,
        depth: usize,
        out: &mut Vec<Arc<Symbol>>,
    ) {
        if depth > MAX_WALK_DEPTH || out.len() >= MAX_CANDIDATES {
            return;
        }
        if let Some(sym) = &t.symbol {
            out.push(Arc::clone(sym));
            if sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::TypeAliasDeclaration)
            {
                let alias_ty = self.get_type_of_symbol(sym);
                self.collect_nameability_candidates(&alias_ty, depth + 1, out);
            }
        }
        if let Some(alias) = &t.alias {
            if let Some(sym) = &alias.symbol {
                out.push(Arc::clone(sym));
            }
            for arg in &alias.type_arguments {
                self.collect_nameability_candidates(arg, depth + 1, out);
            }
        }
        if let Some(structured) = t.as_structured() {
            for sig in structured.call_signatures() {
                if let Some(ret) = self.get_return_type_of_signature(sig) {
                    self.collect_nameability_candidates(&ret, depth + 1, out);
                }
                for param in &sig.parameters {
                    let param_ty = self.get_type_of_symbol(param);
                    self.collect_nameability_candidates(&param_ty, depth + 1, out);
                }
            }
        }
        if let Some(obj) = t.as_object() {
            for arg in &obj.type_arguments {
                self.collect_nameability_candidates(arg, depth + 1, out);
            }
        }
        if let Some(ui) = t.as_union_or_intersection() {
            for part in &ui.types {
                self.collect_nameability_candidates(part, depth + 1, out);
            }
        }
        if let TypeData::Tuple(td) = &t.data {
            for info in &td.element_infos {
                if let Some(el) = &info.type_ {
                    self.collect_nameability_candidates(el, depth + 1, out);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn report_unnameable_symbol(
        &mut self,
        file: &Arc<SourceFile>,
        name_node: &Arc<Node>,
        decl_name: &str,
        sym: &Arc<Symbol>,
        imported_files: &[String],
        imported_specs: &[String],
        imported_symbol_ids: &[u64],
    ) -> bool {
        let Some(decl) = sym.declarations.first() else {
            return false;
        };
        let Some(decl_file) = self.get_source_file_of_node(decl) else {
            return false;
        };
        if decl_file.file_name == file.file_name
            || !decl_file.file_name.contains("/node_modules/")
            || imported_files.iter().any(|f| f == &decl_file.file_name)
            || imported_symbol_ids.contains(&sym.id())
            || self.symbol_in_ambient_module_named(sym, imported_specs)
        {
            return false;
        }
        let Some(spec) = self.decl_emit_specifier_for(&file.file_name, &decl_file.file_name) else {
            return false;
        };
        if !spec.contains("/node_modules/") {
            return false;
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            Some(Arc::clone(file)),
            name_node.loc,
            tsox_core::diagnostics::messages_generated::
                THE_INFERRED_TYPE_OF_0_CANNOT_BE_NAMED_WITHOUT_A_REFERENCE_TO_2_FROM_1_THIS_IS_LIKELY_NOT_PORTABLE_A_TYPE_ANNOTATION_IS_NECESSARY,
            vec![decl_name.to_string(), spec, sym.name.clone()],
        ));
        true
    }

    /// 模块 specifier：优先 node_modules 外层锚定的非相对形式（需可解析到同一模块），
    /// 否则退化为相对路径。指向包内被 index 再导出的文件时收敛到包根。
    fn decl_emit_specifier_for(&self, from_file: &str, target_file: &str) -> Option<String> {
        let target = self.decl_specifier_target_file(target_file)?;
        if let Some(bare) = self.bare_specifier_for_module(&target) {
            if self.bare_specifier_resolves_to(&bare, from_file, &target) {
                return Some(bare);
            }
        }
        Some(relative_module_specifier(from_file, &target))
    }

    fn decl_specifier_target_file(&self, target_file: &str) -> Option<String> {
        let (dir, base) = target_file.rsplit_once('/')?;
        if base == "index.d.ts" || base == "index.ts" {
            return Some(target_file.to_string());
        }
        let stem = base
            .strip_suffix(".d.ts")
            .or_else(|| base.strip_suffix(".ts"))?;
        let index = self
            .program
            .get_source_file(&format!("{dir}/index.d.ts"))
            .or_else(|| self.program.get_source_file(&format!("{dir}/index.ts")))?;
        let reexport = format!("./{stem}");
        if index.text.contains("export *") && index.text.contains(&reexport) {
            return Some(index.file_name.clone());
        }
        Some(target_file.to_string())
    }

    fn bare_specifier_for_module(&self, target_file: &str) -> Option<String> {
        let idx = target_file.find("/node_modules/")?;
        let rest = &target_file[idx + "/node_modules/".len()..];
        let without_ext = drop_trailing_index(&strip_module_extension(rest));
        if without_ext.is_empty() {
            return None;
        }
        Some(without_ext)
    }

    fn bare_specifier_resolves_to(&self, bare: &str, from_file: &str, target_file: &str) -> bool {
        let resolved = self.program.resolve_external_module_path(
            bare,
            from_file,
            tsox_core::core::compiler_options::ModuleKind::None,
        );
        let Some(resolved) = resolved else {
            return false;
        };
        module_identity(&resolved) == module_identity(target_file)
    }
}

fn strip_module_extension(path: &str) -> String {
    for ext in [".d.ts", ".mts", ".cts", ".tsx", ".ts"] {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    path.to_string()
}

fn drop_trailing_index(path: &str) -> String {
    path.strip_suffix("/index").unwrap_or(path).to_string()
}

fn module_identity(path: &str) -> String {
    drop_trailing_index(&strip_module_extension(path))
}

fn relative_module_specifier(from_file: &str, target_file: &str) -> String {
    let from_dir = from_file.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let to_identity = module_identity(target_file);
    let from_segs: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to_segs: Vec<&str> = to_identity.split('/').filter(|s| !s.is_empty()).collect();
    let mut common = 0;
    while common < from_segs.len()
        && common < to_segs.len()
        && from_segs[common] == to_segs[common]
    {
        common += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_segs.len() {
        parts.push("..".to_string());
    }
    for seg in &to_segs[common..] {
        parts.push((*seg).to_string());
    }
    let spec = parts.join("/");
    if spec.starts_with("..") {
        spec
    } else {
        format!("./{spec}")
    }
}
