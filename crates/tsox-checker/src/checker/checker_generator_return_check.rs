#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::*;
use crate::checker::types::TypeFlags;

impl Checker {
    /// Go checkSignatureDeclaration 的生成器分支：注解返回型非 void 时，
    /// 由注解型提取 yield/return/next 迭代型合成 Generator 实例并检查
    /// 对注解型的可赋值性（TS2322/TS2741 族）
    pub(crate) fn check_generator_return_annotation(
        &mut self,
        node: &Arc<Node>,
        type_node: &Arc<Node>,
    ) {
        let is_generator = match &node.data {
            NodeData::FunctionDeclaration(d) => d.asterisk_token.is_some(),
            NodeData::MethodDeclaration(d) => d.asterisk_token.is_some(),
            _ => false,
        };
        if !is_generator {
            return;
        }
        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
        let return_type = self.get_type_from_type_node(type_node);
        if return_type.flags.contains(TypeFlags::Void) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                type_node.loc,
                tsox_core::diagnostics::messages_generated::
                    A_GENERATOR_CANNOT_HAVE_A_VOID_TYPE_ANNOTATION,
                Vec::new(),
            ));
            return;
        }
        self.check_generator_instantiation_assignability(&return_type, is_async, type_node);
    }

    /// Go checkGeneratorInstantiationAssignabilityToReturnType
    fn check_generator_instantiation_assignability(
        &mut self,
        return_type: &Arc<crate::checker::types::Type>,
        is_async: bool,
        error_node: &Arc<Node>,
    ) {
        let any = self.get_any_type();
        let unknown = self.unknown_type();
        let types =
            self.iteration_types_of_generator_function_return_type(return_type, is_async);
        let yield_type = types.yield_type.clone().unwrap_or_else(|| Arc::clone(&any));
        let ret_type = types
            .return_type
            .clone()
            .unwrap_or_else(|| Arc::clone(&yield_type));
        let next_type = types.next_type.clone().unwrap_or_else(|| unknown);
        let instantiation =
            self.create_generator_type(&yield_type, &ret_type, &next_type, is_async);
        self.check_type_assignable_to_and_optionally_elaborate(
            &instantiation,
            return_type,
            Some(error_node),
            None,
            None,
            None,
        );
    }

    /// Go getIterationTypesOfGeneratorFunctionReturnType：先 Iterable 通道，
    /// 无类型再 Iterator 通道
    pub(crate) fn iteration_types_of_generator_function_return_type(
        &mut self,
        t: &Arc<crate::checker::types::Type>,
        is_async: bool,
    ) -> crate::checker::checker_iteration::IterationTypes {
        use crate::checker::checker_iteration::IterationUse;
        if t.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return crate::checker::checker_iteration::IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let use_ = IterationUse::GeneratorReturnType { is_async };
        let mut pending = Vec::new();
        let result = self.iteration_types_of_iterable(use_, t, None);
        if result.has_types() {
            return result;
        }
        let fast: &[&str] = if is_async {
            &["AsyncGenerator", "AsyncIterator"]
        } else {
            &["Generator", "Iterator", "IterableIterator"]
        };
        self.iteration_types_of_iterator(t, fast, None, is_async, &mut pending)
    }

    /// Go createGeneratorType：全局 Generator/AsyncGenerator 接口按
    /// [T, TReturn, TNext] 实例化
    fn create_generator_type(
        &mut self,
        yield_type: &Arc<crate::checker::types::Type>,
        ret_type: &Arc<crate::checker::types::Type>,
        next_type: &Arc<crate::checker::types::Type>,
        is_async: bool,
    ) -> Arc<crate::checker::types::Type> {
        let name = if is_async { "AsyncGenerator" } else { "Generator" };
        let Some(symbol) = self.globals.get(name).cloned() else {
            return self.get_any_type();
        };
        self.resolve_interface_type_ex(
            &symbol,
            Some(vec![
                Arc::clone(yield_type),
                Arc::clone(ret_type),
                Arc::clone(next_type),
            ]),
        )
    }
}
