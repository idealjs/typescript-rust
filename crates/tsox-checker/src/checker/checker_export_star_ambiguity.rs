#![allow(unused_imports)]

use crate::checker::checker::*;

impl Checker {
    /// Go collectExternalModuleInfo→exports 合并的星号歧义检查：
    /// 文件序遍历导出，星号导出的名字与先前来源（具名/本地/更早星号）冲突时
    /// 在该星号语句报 TS2308，责任方为首个提供者模块
    pub(crate) fn check_export_star_ambiguity(&mut self, file: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::SourceFile(sf) = &file.data else {
            return;
        };
        let mut file_sym = None;
        {
            let sym = self.program.symbol_map().symbol_of(file).cloned();
            if sym.is_none() {
                return;
            }
            file_sym = sym;
        }
        let Some(file_sym) = file_sym else {
            return;
        };

        // 名字 → 首个提供者（具名导出的提供者记 "local"）
        let mut provided: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut star_queue: Vec<(Arc<Node>, String)> = Vec::new();

        for stmt in sf.statements.iter() {
            let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data else {
                // 本地具名导出（export class B 等）
                if stmt
                    .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export)
                    && let Some(name) = stmt.name()
                {
                    provided
                        .entry(name.text().to_string())
                        .or_insert_with(|| "local".to_string());
                }
                continue;
            };
            if let Some(clause) = &d.export_clause
                && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data {
                        provided
                            .entry(spec.name.text().to_string())
                            .or_insert_with(|| "local".to_string());
                    }
                }
                continue;
            }
            // 星号（含 export * as ns：命名空间名单独占位不参与歧义）
            if let Some(spec) = &d.module_specifier {
                let text = spec.text().trim_matches(['"', '\'', '`']).to_string();
                star_queue.push((Arc::clone(stmt), text));
            }
        }

        for (stmt, spec) in star_queue {
            let Some(target) = self
                .resolve_module_spec_from(&file_sym, &spec)
                .or_else(|| self.resolve_module_file_symbol(&spec))
            else {
                continue;
            };
            let names = self.module_exported_names(&target, 4);
            for name in names {
                if let Some(first_provider) = provided.get(&name) {
                    if first_provider == "local" || *first_provider == spec {
                        // 本地导出遮蔽；同模块重复星号（export type * + export *）
                        // 不构成歧义
                        continue;
                    }
                    let already = self.diagnostics.get_all().iter().any(|dg| {
                        dg.code == 2308 && dg.loc.pos() == stmt.loc.pos()
                    });
                    if already {
                        break;
                    }
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        stmt.loc,
                        tsox_core::diagnostics::messages_generated::
                            MODULE_0_HAS_ALREADY_EXPORTED_A_MEMBER_NAMED_1_CONSIDER_EXPLICITLY_RE_EXPORTING_TO_RESOLVE_THE_AMBIGUITY,
                        vec![format!("\"{}\"", first_provider.clone()), name],
                    ));
                    break;
                }
                provided.insert(name, spec.clone());
            }
        }
    }

    /// 模块导出名字集合（具名导出 + 本地导出声明 + 惰性星号递归）
    pub(crate) fn module_exported_names(
        &mut self,
        module_sym: &Arc<Symbol>,
        depth: usize,
    ) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        if depth == 0 {
            return names;
        }
        for (name, _) in module_sym.exports.iter() {
            if !name.starts_with(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX) {
                names.push(name.clone());
            }
        }
        let mut stars: Vec<String> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data {
                if let Some(clause) = &d.export_clause {
                    if let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data {
                        for el in ne.elements.iter() {
                            if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) =
                                &el.data
                            {
                                let n = spec.name.text().to_string();
                                if !names.contains(&n) {
                                    names.push(n);
                                }
                            }
                        }
                    }
                    // export * as ns：ns 本身是导出名
                    if clause.kind == SyntaxKind::NamespaceExport
                        && let Some(n) =
                            tsox_frontend::ast::node_data_generated::node_name(clause)
                    {
                        let n = n.text().to_string();
                        if !names.contains(&n) {
                            names.push(n);
                        }
                    }
                } else if let Some(spec) = &d.module_specifier {
                    stars.push(spec.text().trim_matches(['"', '\'', '`']).to_string());
                }
            } else if stmt
                .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export)
                && let Some(n) = stmt.name()
            {
                let n = n.text().to_string();
                if !names.contains(&n) {
                    names.push(n);
                }
            }
            false
        });
        for spec in stars {
            if let Some(target) = self
                .resolve_module_spec_from(module_sym, &spec)
                .or_else(|| self.resolve_module_file_symbol(&spec))
            {
                for n in self.module_exported_names(&target, depth - 1) {
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
            }
        }
        names
    }
}
