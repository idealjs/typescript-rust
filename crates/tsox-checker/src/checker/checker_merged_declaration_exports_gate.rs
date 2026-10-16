#![allow(unused_imports)]

use crate::checker::checker::*;
use tsox_frontend::ast::utilities::is_ambient_module;
use tsox_frontend::ast::utilities::is_external_module;

impl Checker {
    /// Go binder 绑定期的成员容器：向上找最近的 SourceFile/ModuleDeclaration，
    /// 跨入函数/类/枚举/接口则不是模块成员
    pub(crate) fn binder_container_of_member(&self, n: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binder_container_of_member"); 
        let mut cur = n.parent();
        while let Some(p) = cur {
            if matches!(
                p.kind,
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
            ) {
                return Some(p);
            }
            if matches!(
                p.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::Constructor
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::InterfaceDeclaration
            ) {
                return None;
            }
            cur = p.parent();
        }
        None
    }

    /// Go declareModuleMember（binder.go:378）仅导出分支写 node.LocalSymbol 与
    /// local.ExportSymbol 链，checkExportsOnMergedDeclarations（checker.go:7088）
    /// 以「节点或其合并声明集中存在该分支可达声明」为前置闸门。Rust 侧
    /// node.LocalSymbol 未建模（set_local_symbol_of_exportable 为空实现），按
    /// 该分支的可达条件在声明集上等价判定：容器为模块声明或外部模块文件，
    /// 声明非别名、非 ambient 模块声明，且带 export 修饰或容器处于 ExportContext
    pub(crate) fn declaration_reached_binder_exported_branch(
        &mut self,
        d: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("declaration_reached_binder_exported_branch"); 
        if matches!(
            d.kind,
            SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::NamespaceImport
                | SyntaxKind::ImportClause
                | SyntaxKind::ImportSpecifier
                | SyntaxKind::ExportSpecifier
        ) {
            return false;
        }
        if d.kind == SyntaxKind::ModuleDeclaration && is_ambient_module(d) {
            return false;
        }
        let Some(container) = self.binder_container_of_member(d) else {
            return false;
        };
        let export_modifier = self
            .get_combined_modifier_flags(d)
            .contains(ModifierFlags::Export);
        match container.kind {
            SyntaxKind::ModuleDeclaration => {
                export_modifier || container.flags.contains(NodeFlags::ExportContext)
            }
            _ => {
                export_modifier
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| is_external_module(f))
            }
        }
    }

    /// 合并声明集内任一声明曾走 binder 导出分支才继续检查
    pub(crate) fn any_declaration_reached_binder_exported_branch(
        &mut self,
        declarations: &[Arc<Node>],
    ) -> bool { ::tsox_core::fntrace::enter("any_declaration_reached_binder_exported_branch"); 
        for d in declarations {
            if self.declaration_reached_binder_exported_branch(d) {
                return true;
            }
        }
        false
    }
}
