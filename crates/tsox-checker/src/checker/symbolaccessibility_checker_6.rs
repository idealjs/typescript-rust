#![allow(unused_imports)]

use crate::checker::symbolaccessibility::*;

impl Checker {
    /// 匿名类表达式的显示容器：匿名声明不入任何符号表（Go binder
    /// bindAnonymousDeclaration 不落容器 locals，binder.go:1304-1310），实例
    /// 型按名显示的限定段取词法宿主——自类表达式沿父链透明穿越无符号层
    /// （箭头简写体、返回语句、函数体块），首个持符号声明（变量声明/函数
    /// 声明）为限定段；直接赋值形态（const C = class{}，父即变量声明）不
    /// 认领——该形态段名本身经 getNameOfSymbolAsWritten 的变量名分支呈
    /// 裸名（constInClassExpression 基线 >C : typeof C）
    pub(crate) fn container_of_anonymous_class_symbol(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if !symbol.flags.intersects(SymbolFlags::Class)
            || symbol.name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_CLASS
        {
            return None;
        }
        let decl = symbol.declarations.first()?;
        if decl.kind != SyntaxKind::ClassExpression {
            return None;
        }
        let mut current = decl.parent();
        if current
            .as_ref()
            .is_some_and(|node| node.kind == SyntaxKind::VariableDeclaration)
        {
            return None;
        }
        while let Some(node) = current {
            match node.kind {
                SyntaxKind::VariableDeclaration | SyntaxKind::FunctionDeclaration => {
                    return self.program.symbol_map().symbol_of(&node).map(Arc::clone);
                }
                _ => current = node.parent(),
            }
        }
        None
    }
}
