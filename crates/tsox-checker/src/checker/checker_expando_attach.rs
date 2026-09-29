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

    pub(crate) fn is_empty_literal_type(&self, t: &Arc<Type>) -> bool {
        match t.intrinsic_name().as_deref() {
            Some("never") => self.strict_null_checks,
            Some("undefined") => !self.strict_null_checks,
            _ => false,
        }
    }

    pub(crate) fn expando_parent_has_type_annotation(
        &self,
        host: &Arc<tsox_frontend::ast::Symbol>,
    ) -> bool {
        let Some(vd) = host.value_declaration.as_ref() else {
            return false;
        };
        match &vd.data {
            tsox_frontend::ast::NodeData::FunctionExpression(_)
            | tsox_frontend::ast::NodeData::ArrowFunction(_) => {
                self.declaration_container_has_type_annotation(vd)
            }
            tsox_frontend::ast::NodeData::VariableDeclaration(d) => {
                d.type_node.is_some()
                    && d.initializer.as_ref().is_some_and(|init| {
                        matches!(
                            init.kind,
                            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
                        )
                    })
            }
            _ => false,
        }
    }

    fn declaration_container_has_type_annotation(&self, node: &Arc<Node>) -> bool {
        node.parent()
            .and_then(|p| self.program.symbol_map().symbol_of(&p))
            .and_then(|s| s.value_declaration.clone())
            .and_then(|vd| match &vd.data {
                tsox_frontend::ast::NodeData::VariableDeclaration(d) => d.type_node.clone(),
                _ => None,
            })
            .is_some()
    }

    pub(crate) fn report_expando_implicit_any_array(&mut self, node: &Arc<Node>, member: &str) {
        if !self.no_implicit_any {
            return;
        }
        let dup = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 7008 && d.loc.pos() == node.loc.pos());
        if dup {
            return;
        }
        let file = self
            .current_file
            .clone()
            .or_else(|| self.get_source_file_of_node(node));
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            node.loc,
            tsox_core::diagnostics::messages_generated::MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
            vec![member.to_string(), "any[]".to_string()],
        ));
    }
}
