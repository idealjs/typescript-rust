#![allow(unused_imports)]

use crate::checker::flow_impl_chunk::*;
use tsox_frontend::ast::{ModifierFlags, NodeData, SyntaxKind};

impl Checker {
    pub fn check_no_implicit_returns(&mut self, node: &Arc<Node>, type_node: Option<&Arc<Node>>) {
        if !self.compiler_options.no_implicit_returns.is_true() {
            return;
        }
        let Some(body) = function_like_body(node) else {
            return;
        };
        if body.kind != SyntaxKind::Block {
            return;
        }
        if !self.function_has_implicit_return(node) {
            return;
        }
        let has_explicit_return = Self::function_body_has_explicit_return(&body);
        if type_node.is_some() {
            let unwrapped = declared_unwrapped_return_type(self, node, type_node);
            self.report_all_code_paths_error(node, type_node, &unwrapped, has_explicit_return);
            return;
        }
        if !has_explicit_return {
            return;
        }
        let inferred = self.infer_function_return_type(Some(node), Some(&body), None);
        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
        let unwrapped = self.unwrap_async_return_type(inferred, is_async);
        if self.maybe_type_of_kind(&unwrapped, TypeFlags::Void)
            || unwrapped
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Undefined)
        {
            return;
        }
        let error_loc = function_like_name_loc(node).unwrap_or(node.loc);

        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            error_loc,
            tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
            vec![],
        ));
    }

    pub fn check_bare_return_statement(
        &mut self,
        node: &Arc<Node>,
        container: Option<&Arc<Node>>,
        annotated: Option<Arc<Type>>,
    ) {
        let never = annotated
            .as_ref()
            .is_some_and(|t| t.flags.contains(TypeFlags::Never));
        if self.strict_null_checks || never {
            let Some(expected) = annotated else {
                return;
            };
            if !expected.flags.contains(TypeFlags::Any)
                && !self.is_type_assignable_to(&self.undefined_type(), &expected)
            {
                let expected_str = self.type_to_string(&expected);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec!["undefined".to_string(), expected_str],
                ));
            }
            return;
        }
        if !self.compiler_options.no_implicit_returns.is_true() {
            return;
        }
        let Some(container) = container else {
            return;
        };
        if container.kind == SyntaxKind::Constructor {
            return;
        }
        let t = match annotated {
            Some(t) => t,
            None => {
                let Some(body) = function_like_body(container) else {
                    return;
                };
                let inferred =
                    self.infer_function_return_type(Some(container), Some(&body), None);
                let is_async = container.has_syntactic_modifier(ModifierFlags::Async);
                self.unwrap_async_return_type(inferred, is_async)
            }
        };
        if self.maybe_type_of_kind(&t, TypeFlags::Void)
            || t
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Undefined)
        {
            return;
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
            vec![],
        ));
    }

    pub fn check_all_code_paths_annotated(
        &mut self,
        node: &Arc<Node>,
        type_node: &Arc<Node>,
    ) {
        if function_like_body(node).map(|b| b.kind) != Some(SyntaxKind::Block) {
            return;
        }
        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
        let raw = self.get_type_from_type_node(type_node);
        let unwrapped = self.unwrap_async_return_type(raw, is_async);
        if self.maybe_type_of_kind(&unwrapped, TypeFlags::Void)
            || unwrapped
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Undefined)
        {
            return;
        }
        if !self.function_has_implicit_return(node) {
            return;
        }
        let has_explicit_return = function_like_body(node)
            .map(|body| Self::function_body_has_explicit_return(&body))
            .unwrap_or(false);
        self.report_all_code_paths_error(node, Some(type_node), &unwrapped, has_explicit_return);
    }

    pub(crate) fn report_all_code_paths_error(
        &mut self,
        node: &Arc<Node>,
        type_node: Option<&Arc<Node>>,
        unwrapped: &Arc<Type>,
        has_explicit_return: bool,
    ) {
        let error_loc = type_node.map_or(node.loc, |tn| tn.loc);
        if !has_explicit_return {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                error_loc,
                tsox_core::diagnostics::messages_generated::
                    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                vec![],
            ));
            return;
        }
        if self.strict_null_checks {
            let undefined_type = self.undefined_type();
            if !self.is_type_assignable_to(&undefined_type, unwrapped) {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    error_loc,
                    tsox_core::diagnostics::messages_generated::
                        FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                    vec![],
                ));
                return;
            }
        }
        if !self.compiler_options.no_implicit_returns.is_true() {
            return;
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            error_loc,
            tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
            vec![],
        ));
    }
}

pub(crate) fn declared_unwrapped_return_type(
    checker: &mut Checker,
    node: &Arc<Node>,
    type_node: Option<&Arc<Node>>,
) -> Arc<Type> {
    let Some(tn) = type_node else {
        return checker.get_any_type();
    };
    let raw = checker.get_type_from_type_node(tn);
    let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
    checker.unwrap_async_return_type(raw, is_async)
}

pub fn function_like_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(d.body.clone()),
        NodeData::ArrowFunction(d) => Some(d.body.clone()),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        _ => None,
    }
}

pub(crate) fn function_like_name_loc(node: &Arc<Node>) -> Option<tsox_core::core::text::TextRange> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.name.as_ref().map(|n| n.loc),
        NodeData::MethodDeclaration(d) => Some(d.name.loc),
        NodeData::ArrowFunction(_) | NodeData::FunctionExpression(_) => {
            let has_async = node.has_syntactic_modifier(ModifierFlags::Async);
            has_async.then(|| {
                tsox_core::core::text::TextRange::new(node.loc.pos() - "async ".len(), node.loc.end())
            })
        }
        _ => None,
    }
}
