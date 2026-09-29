#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::is_identifier;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::scanner::TokenFlags;

#[path = "r34k4_helpers.rs"]
pub mod r34k4_helpers;
pub use r34k4_helpers::*;

#[path = "r36k29_defs.rs"]
pub mod r36k29_defs;

pub use crate::mig::m4h_8::r36k26_defs;

use r36k26_defs::EmitContextCommentRangeR36k26;

use crate::mig::m4g::r33k7_defs::PrivateIdentifierKind;
use crate::printer::NodeFactory;
use tsox_frontend::format::mig::m4o::EmitFlags;

impl<'a> NodeFactory<'a> {
    pub fn new_add_disposable_resource_helper(
        &self,
        env_binding: &Arc<Node>,
        value: &Arc<Node>,
        async_: bool,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, add_disposable_resource_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__addDisposableResource"),
            None,
            None,
            self.new_node_list(vec![
                Arc::clone(env_binding),
                Arc::clone(value),
                self.new_keyword_expression(if async_ {
                    SyntaxKind::TrueKeyword
                } else {
                    SyntaxKind::FalseKeyword
                }),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_async_delegator_helper(&self, expression: &Arc<Node>) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, await_helper());
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, async_delegator_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__asyncDelegator"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(expression)]),
            NodeFlags::empty(),
        )
    }

    pub fn new_async_generator_helper(
        &self,
        generator_func: &Arc<Node>,
        has_lexical_this: bool,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, await_helper());
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, async_generator_helper());
        self.emit_context_mut().add_emit_flags(
            generator_func,
            EmitFlags::ASYNC_FUNCTION_BODY | EmitFlags::REUSE_TEMP_VARIABLE_SCOPE,
        );
        let this_arg = if has_lexical_this {
            self.new_keyword_expression(SyntaxKind::ThisKeyword)
        } else {
            self.new_void_zero_expression()
        };
        self.new_call_expression(
            &self.new_unscoped_helper_name("__asyncGenerator"),
            None,
            None,
            self.new_node_list(vec![
                this_arg,
                self.new_identifier("arguments"),
                Arc::clone(generator_func),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_async_values_helper(&self, expression: &Arc<Node>) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, async_values_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__asyncValues"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(expression)]),
            NodeFlags::empty(),
        )
    }

    pub fn new_await_helper(&self, expression: &Arc<Node>) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, await_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__await"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(expression)]),
            NodeFlags::empty(),
        )
    }

    pub fn new_awaiter_helper(
        &self,
        has_lexical_this: bool,
        arguments_expression: Option<&Arc<Node>>,
        parameters: Option<Arc<NodeList>>,
        body: &Arc<Node>,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, awaiter_helper());

        let params = match parameters {
            Some(parameters) => parameters,
            None => self.new_node_list(Vec::new()),
        };

        let generator_func = self.new_function_expression(
            None,
            Some(&self.new_token(SyntaxKind::AsteriskToken)),
            None,
            None,
            &params,
            None,
            None,
            body,
        );

        self.emit_context_mut().add_emit_flags(
            &generator_func,
            EmitFlags::ASYNC_FUNCTION_BODY | EmitFlags::REUSE_TEMP_VARIABLE_SCOPE,
        );

        let this_arg = if has_lexical_this {
            self.new_keyword_expression(SyntaxKind::ThisKeyword)
        } else {
            self.new_void_zero_expression()
        };

        let args_arg = match arguments_expression {
            Some(arguments_expression) => Arc::clone(arguments_expression),
            None => self.new_void_zero_expression(),
        };

        self.new_call_expression(
            &self.new_unscoped_helper_name("__awaiter"),
            None,
            None,
            self.new_node_list(vec![
                this_arg,
                args_arg,
                self.new_void_zero_expression(),
                generator_func,
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_class_private_field_get_helper(
        &self,
        receiver: &Arc<Node>,
        state: &Arc<Node>,
        kind: PrivateIdentifierKind,
        fn_: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, class_private_field_get_helper());
        let kind_literal = self.new_string_literal(&kind.to_string(), 0);
        let mut args = vec![Arc::clone(receiver), Arc::clone(state), kind_literal];
        if let Some(fn_) = fn_ {
            args.push(Arc::clone(fn_));
        }
        self.new_call_expression(
            &self.new_unscoped_helper_name("__classPrivateFieldGet"),
            None,
            None,
            self.new_node_list(args),
            NodeFlags::empty(),
        )
    }

    pub fn new_class_private_field_in_helper(
        &self,
        state: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, class_private_field_in_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__classPrivateFieldIn"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(state), Arc::clone(receiver)]),
            NodeFlags::empty(),
        )
    }

    pub fn new_class_private_field_set_helper(
        &self,
        receiver: &Arc<Node>,
        state: &Arc<Node>,
        value: &Arc<Node>,
        kind: PrivateIdentifierKind,
        fn_: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, class_private_field_set_helper());
        let kind_literal = self.new_string_literal(&kind.to_string(), 0);
        let mut args = vec![
            Arc::clone(receiver),
            Arc::clone(state),
            Arc::clone(value),
            kind_literal,
        ];
        if let Some(fn_) = fn_ {
            args.push(Arc::clone(fn_));
        }
        self.new_call_expression(
            &self.new_unscoped_helper_name("__classPrivateFieldSet"),
            None,
            None,
            self.new_node_list(args),
            NodeFlags::empty(),
        )
    }

    pub fn new_decorate_helper(
        &self,
        decorator_expressions: Vec<Arc<Node>>,
        target: &Arc<Node>,
        member_name: Option<&Arc<Node>>,
        descriptor: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, decorate_helper());

        let mut arguments_array =
            vec![self.new_array_literal_expression(
                &self.new_node_list(decorator_expressions),
                true,
            )];
        arguments_array.push(Arc::clone(target));
        if let Some(member_name) = member_name {
            arguments_array.push(Arc::clone(member_name));
            if let Some(descriptor) = descriptor {
                arguments_array.push(Arc::clone(descriptor));
            }
        }

        self.new_call_expression(
            &self.new_unscoped_helper_name("__decorate"),
            None,
            None,
            self.new_node_list(arguments_array),
            NodeFlags::empty(),
        )
    }

    pub fn new_dispose_resources_helper(&self, env_binding: &Arc<Node>) -> Arc<Node> {
        r36k26_defs::EmitContextCommentRangeR36k26::request_emit_helper(self.emit_context, dispose_resources_helper());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__disposeResources"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(env_binding)]),
            NodeFlags::empty(),
        )
    }
}
