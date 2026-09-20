#![allow(unused_imports)]

use crate::checker::checker::*;
use tsox_core::diagnostics::messages_generated as msgs;
use tsox_core::diagnostics::Message;

impl Checker {
    // Go checkGrammarModuleElementContext：模块元素出现在 SourceFile/
    // ModuleBlock/ModuleDeclaration 之外时报错并中止后续检查
    fn grammar_module_element_context_holds(&mut self, node: &Arc<Node>, message: &Message) -> bool {
        let ok = node.parent().is_some_and(|p| {
            matches!(
                p.kind,
                SyntaxKind::SourceFile | SyntaxKind::ModuleBlock | SyntaxKind::ModuleDeclaration
            )
        });
        if !ok {
            self.grammar_error_on_node(node, message);
        }
        ok
    }

    // Go checkModuleDeclaration/checkImportDeclaration/checkImportEqualsDeclaration/
    // checkExportDeclaration/checkExportAssignment 的上下文文法入口，返回 false
    // 表示非法上下文已报错需中止
    pub fn check_module_element_context(&mut self, node: &Arc<Node>) -> bool {
        let message = match &node.data {
            NodeData::ImportDeclaration(_) | NodeData::ImportEqualsDeclaration(_) => {
                &msgs::AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
            }
            NodeData::ExportDeclaration(_) => {
                &msgs::AN_EXPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
            }
            NodeData::ExportAssignment(d) => {
                if d.is_export_equals {
                    &msgs::AN_EXPORT_ASSIGNMENT_MUST_BE_AT_THE_TOP_LEVEL_OF_A_FILE_OR_MODULE_DECLARATION
                } else {
                    &msgs::A_DEFAULT_EXPORT_MUST_BE_AT_THE_TOP_LEVEL_OF_A_FILE_OR_MODULE_DECLARATION
                }
            }
            NodeData::ModuleDeclaration(d) => {
                if d.name.kind == SyntaxKind::StringLiteral {
                    &msgs::AN_AMBIENT_MODULE_DECLARATION_IS_ONLY_ALLOWED_AT_THE_TOP_LEVEL_IN_A_FILE
                } else {
                    &msgs::A_NAMESPACE_DECLARATION_IS_ONLY_ALLOWED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
                }
            }
            _ => return true,
        };
        self.grammar_module_element_context_holds(node, message)
    }

    // Go checkExternalImportOrExportDeclaration：模块名非字符串字面量报
    // TS1141；非文件级且非 ambient 外部模块内的导入/导出报 TS1147/TS1194
    pub fn check_external_import_or_export_declaration(&mut self, node: &Arc<Node>) -> bool {
        let module_name = match &node.data {
            NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
            NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
            NodeData::ImportEqualsDeclaration(d) => match &d.module_reference.data {
                NodeData::ExternalModuleReference(ext) => Some(Arc::clone(&ext.expression)),
                _ => None,
            },
            _ => None,
        };
        let Some(module_name) = module_name else {
            return false;
        };
        if module_name_is_missing(&module_name) {
            return false;
        }
        if module_name.kind != SyntaxKind::StringLiteral {
            self.grammar_error_on_node(&module_name, &msgs::STRING_LITERAL_EXPECTED);
            return false;
        }
        let parent = node.parent();
        let in_ambient_external_module = parent.as_ref().is_some_and(|p| {
            p.kind == SyntaxKind::ModuleBlock
                && p.parent().is_some_and(|gp| {
                    matches!(&gp.data, NodeData::ModuleDeclaration(md) if md.name.kind == SyntaxKind::StringLiteral)
                })
        });
        if parent.is_some_and(|p| p.kind != SyntaxKind::SourceFile) && !in_ambient_external_module {
            let message = if node.kind == SyntaxKind::ExportDeclaration {
                &msgs::EXPORT_DECLARATIONS_ARE_NOT_PERMITTED_IN_A_NAMESPACE
            } else {
                &msgs::IMPORT_DECLARATIONS_IN_A_NAMESPACE_CANNOT_REFERENCE_A_MODULE
            };
            self.grammar_error_on_node(&module_name, message);
            return false;
        }
        true
    }

    // Go checkExportDeclaration：namespace 内具名导出声明报 TS1194
    //（ambient 命名空间内无 from 的导出除外）
    pub fn check_export_declaration_namespace(&mut self, node: &Arc<Node>) {
        let NodeData::ExportDeclaration(d) = &node.data else {
            return;
        };
        let named_exports = d
            .export_clause
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::NamedExports);
        if !named_exports {
            return;
        }
        let Some(parent) = node.parent() else {
            return;
        };
        if parent.kind != SyntaxKind::ModuleBlock {
            return;
        }
        let in_ambient_external_module = parent.parent().is_some_and(|gp| {
            matches!(&gp.data, NodeData::ModuleDeclaration(md) if md.name.kind == SyntaxKind::StringLiteral)
        });
        if in_ambient_external_module {
            return;
        }
        let in_ambient_namespace_declaration =
            d.module_specifier.is_none() && self.declaration_is_ambient(node);
        if !in_ambient_namespace_declaration {
            self.grammar_error_on_node(node, &msgs::EXPORT_DECLARATIONS_ARE_NOT_PERMITTED_IN_A_NAMESPACE);
        }
    }
}

fn module_name_is_missing(module_name: &Arc<Node>) -> bool {
    module_name.loc.pos() >= module_name.loc.end()
}

impl Checker {
    // Go checkExternalModuleNameInGlobalScope：非法上下文中止后仍解析模块名，
    // 未解析报 TS2307（副作用导入与函数内容器跳过）
    pub fn check_external_module_name_in_global_scope(&mut self, node: &Arc<Node>) {
        let mut container = node.parent();
        while let Some(c) = &container
            && c.kind == SyntaxKind::Block
        {
            container = c.parent();
        }
        if container.is_none_or(|c| c.kind != SyntaxKind::SourceFile) {
            return;
        }
        if let NodeData::ImportDeclaration(d) = &node.data
            && d.import_clause.is_none()
        {
            return;
        }
        let module_name = match &node.data {
            NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
            NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
            NodeData::ImportEqualsDeclaration(d) => match &d.module_reference.data {
                NodeData::ExternalModuleReference(ext) => Some(Arc::clone(&ext.expression)),
                _ => None,
            },
            _ => None,
        };
        let Some(module_name) = module_name else {
            return;
        };
        if module_name.kind != SyntaxKind::StringLiteral {
            return;
        }
        let spec = module_name.text().trim_matches(['"', '\'', '`']).to_string();
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let resolved = if spec.starts_with('.') || spec.starts_with("..") {
            self.program
                .resolve_external_module_path(&spec, &file.file_name, ModuleKind::None)
        } else {
            // Go resolveExternalModuleNameWorker：先查 global 表（块内
            // declare module 也会进文件符号表），再走文件/ambient 解析
            let file_sym = self
                .program
                .symbol_map()
                .symbol_of(&file.node)
                .cloned();
            let in_symbol_table = self.globals.get(&spec).is_some()
                || file_sym.as_ref().is_some_and(|s| {
                    s.members.get(&spec).is_some() || s.exports.get(&spec).is_some()
                })
                || file_has_ambient_module_named(&file.node, &spec);
            if in_symbol_table {
                Some(String::new())
            } else {
                self.resolve_module_file_symbol(&spec)
                    .map(|_| String::new())
                    .or_else(|| {
                        self.program
                            .resolve_external_module_path(&spec, &file.file_name, ModuleKind::None)
                    })
            }
        };
        if let Some(path) = &resolved {
            // Go resolveExternalModule：文件解析成功但目标无模块指示（脚本
            // 文件，sourceFile.Symbol 为 nil）时报 TS2306
            if let Some(sf) = self.program.get_source_file(path)
                && sf.external_module_indicator.is_none()
                && sf.common_js_module_indicator.is_none()
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    Some(file),
                    module_name.loc,
                    msgs::FILE_0_IS_NOT_A_MODULE,
                    vec![path.clone()],
                ));
            }
        } else {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                module_name.loc,
                msgs::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                vec![spec],
            ));
        }
    }
}

// Go binder：块内 declare module 仍声明进文件容器符号表；移植侧落入块
// locals，此处按语法兜底检索同名 ambient 模块声明
fn file_has_ambient_module_named(file: &Arc<Node>, spec: &str) -> bool {
    fn walk(stmts: &[Arc<Node>], spec: &str) -> bool {
        for stmt in stmts {
            if let NodeData::ModuleDeclaration(md) = &stmt.data
                && md.name.kind == SyntaxKind::StringLiteral
                && md.name.text().trim_matches(['"', '\'', '`']) == spec
            {
                return true;
            }
            if let NodeData::ModuleBlock(block) = &stmt.data
                && walk(&block.statements.nodes, spec)
            {
                return true;
            }
            if let NodeData::Block(block) = &stmt.data
                && walk(&block.statements.nodes, spec)
            {
                return true;
            }
        }
        false
    }
    match &file.data {
        NodeData::SourceFile(sf) => walk(&sf.statements.nodes, spec),
        _ => false,
    }
}
