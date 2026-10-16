#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

impl Checker {
    /// 容器内成员声明名的合并体回落：宿主容器符号归并后从其成员表取
    /// 持有该声明且声明集大于节点侧符号的成员符号。Go binder 的
    /// node.Symbol 与容器成员表同指针，getSymbolOfDeclaration
    /// （checker.go:15181-15188）经 getMergedSymbol（checker.go:
    /// 15141-15148）一步即得合并体；本侧节点绑符号可与成员表条目分属
    /// 并行谱系，checker 池后续实例的 populate 面对已写回的合并文件表
    /// 不再过 merge_symbol（ptr_eq 早退）、无归并记录产生，解析层
    /// if_absent 记录（add_method_signature_member 对成员节点绑符号
    /// 落单声明实例）得以先占链位——声明名查询落单声明符号，.types 呈
    /// 单签名（promisesWithConstraints 的 >then 应为 lib.es5 与本文件
    /// 声明合并后的双签名对象形态）。容器解析阶梯：节点绑符号 →
    /// get_merged_symbol → 顶层脚本容器回落文件符号表（globals →
    /// locals/members 按名，populate 写回产物，跨 checker 实例稳定）
    pub(crate) fn container_member_table_symbol_of(
        &self,
        sym: &Arc<Symbol>,
        name_node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("container_member_table_symbol_of");
        let decl = name_node.parent()?;
        let mut container_decl = decl.parent()?;
        if container_decl.kind == SyntaxKind::ModuleBlock {
            container_decl = container_decl.parent()?;
        }
        if container_decl.kind == SyntaxKind::SourceFile {
            return None;
        }
        let container_sym = self
            .program
            .symbol_map()
            .symbol_of(&container_decl)
            .cloned()?;
        let merged_container = self.get_merged_symbol(&container_sym);
        let merged_container = if Arc::ptr_eq(&merged_container, &container_sym) {
            self.script_top_level_table_symbol_of(&container_sym, container_decl.name()?)?
        } else {
            merged_container
        };
        let table_sym = merged_container
            .members
            .get(sym.name.as_str())
            .or_else(|| merged_container.exports.get(sym.name.as_str()))?;
        if table_sym.name != sym.name {
            return None;
        }
        if !table_sym.declarations.iter().any(|d| Arc::ptr_eq(d, &decl)) {
            return None;
        }
        if table_sym.declarations.len() <= sym.declarations.len() {
            return None;
        }
        Some(Arc::clone(table_sym))
    }
}
