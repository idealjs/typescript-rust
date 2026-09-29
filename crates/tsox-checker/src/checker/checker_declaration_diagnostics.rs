#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated::PROPERTY_0_OF_EXPORTED_ANONYMOUS_CLASS_TYPE_MAY_NOT_BE_PRIVATE_OR_PROTECTED;
use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

impl Checker {
    /// 声明发射诊断（Go declaration emit 串行化检查）：
    /// 导出函数的返回型引用局部类（声明发射只能匿名化）时，
    /// 其 private/protected 成员报 TS4094
    pub(crate) fn check_declaration_diagnostics(&mut self, statements: &[Arc<Node>]) {
        if !self.compiler_options.declaration.is_true() {
            return;
        }
        self.check_declaration_diagnostics_in(statements);
    }

    fn check_declaration_diagnostics_in(&mut self, statements: &[Arc<Node>]) {
        for stmt in statements {
            match &stmt.data {
                NodeData::FunctionDeclaration(_) => {
                    if stmt.has_syntactic_modifier(ModifierFlags::Export) {
                        self.check_exported_function_return(stmt);
                    }
                }
                NodeData::ModuleDeclaration(d) => {
                    if let Some(body) = &d.body {
                        let inner: Vec<Arc<Node>> = match &body.data {
                            NodeData::ModuleBlock(mb) => {
                                mb.statements.iter().cloned().collect()
                            }
                            _ => Vec::new(),
                        };
                        self.check_declaration_diagnostics_in(&inner);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_exported_function_return(&mut self, decl: &Arc<Node>) {
        let NodeData::FunctionDeclaration(d) = &decl.data else {
            return;
        };
        if d.type_node.is_some() {
            return;
        }
        let Some(body) = &d.body else {
            return;
        };
        let sig = self.get_type_of_function_like(decl);
        let Some(structured) = sig.as_structured() else {
            return;
        };
        let Some(call) = structured.call_signatures().first() else {
            return;
        };
        let Some(ret) = self.get_return_type_of_signature(call) else {
            return;
        };
        let Some(class_sym) = ret.symbol.as_ref() else {
            return;
        };
        if !class_sym.flags.contains(SymbolFlags::Class) {
            return;
        }
        let Some(class_decl) = class_sym
            .declarations
            .iter()
            .find(|dn| dn.kind == SyntaxKind::ClassDeclaration)
        else {
            return;
        };
        if !class_decl
            .parent()
            .as_ref()
            .is_some_and(|p| Self::node_inside_function_body(p))
            || class_decl.has_syntactic_modifier(ModifierFlags::Export)
        {
            let _ = body;
            return;
        }
        let NodeData::ClassDeclaration(cd) = &class_decl.data else {
            return;
        };
        for member in cd.members.iter() {
            if member.has_syntactic_modifier(ModifierFlags::Private | ModifierFlags::Protected) {
                let name = member
                    .name()
                    .map(|n| n.text().to_string())
                    .unwrap_or_default();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    member.loc,
                    PROPERTY_0_OF_EXPORTED_ANONYMOUS_CLASS_TYPE_MAY_NOT_BE_PRIVATE_OR_PROTECTED,
                    vec![name],
                ));
            }
        }
    }

    fn node_inside_function_body(node: &Arc<Node>) -> bool {
        let mut cur = Some(Arc::clone(node));
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::Block => return true,
                SyntaxKind::SourceFile => return false,
                _ => {}
            }
            cur = n.parent();
        }
        false
    }
}
