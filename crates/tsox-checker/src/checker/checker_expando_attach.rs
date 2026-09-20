#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;

impl Checker {
    /// Go getExpandoSymbol：函数表达式/箭头函数的 expando 成员挂在函数
    /// 自身符号或其 const 变量初始式容器符号上；函数型计算期即并入，
    /// 使 `const f: IFoo = () => {}` 的初始式类型自带赋值声明的成员
    pub(crate) fn attach_expando_to_function_expression_type(
        &mut self,
        node: &Arc<Node>,
        base: Arc<Type>,
    ) -> Arc<Type> {
        let mut result = base;
        if let Some(own) = self.program.symbol_map().symbol_of(node).map(Arc::clone) {
            result = self.attach_function_expando_type(&own, result);
        }
        if let Some(owner) = node
            .parent()
            .as_ref()
            .filter(|p| p.kind == SyntaxKind::VariableDeclaration)
            .and_then(|p| p.name())
            .and_then(|n| self.resolve_identifier(&n))
        {
            result = self.attach_function_expando_type(&owner, result);
        }
        result
    }
}
