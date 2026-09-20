#![allow(unused_imports)]

use crate::checker::checker_classes::*;

// Go getBaseConstructorTypeOfClass 的合法性检查段：extends 表达式的
// 值类型须为构造器类型（含构造签名），否则 TS2507
impl Checker {
    pub(crate) fn check_base_constructor_type(&mut self, expr: &Arc<Node>) {
        if expr.kind == SyntaxKind::NullKeyword {
            return;
        }
        if matches!(expr.kind, SyntaxKind::Identifier | SyntaxKind::PropertyAccessExpression)
            && let Some(symbol) = self.resolve_entity_name_class_symbol(expr)
            && !symbol.flags.intersects(SymbolFlags::VALUE)
        {
            return;
        }
        let value_type = self.get_type_of_node(expr);
        if value_type.flags.contains(TypeFlags::Any) || self.is_error_type(&value_type) {
            return;
        }
        // Go getSignaturesOfType：类型参数经约束解析构造签名
        let signatures_in = if value_type.is_type_parameter() {
            self.get_constraint_of_type_parameter(&value_type)
                .unwrap_or_else(|| Arc::clone(&value_type))
        } else {
            Arc::clone(&value_type)
        };
        let has_construct_signatures = !self
            .get_signatures_of_type(&signatures_in, SignatureKind::Construct)
            .is_empty();
        if has_construct_signatures {
            return;
        }
        let type_str = self.type_to_string(&value_type);
        let file = self.current_file.clone();
        let mut diag = tsox_frontend::ast::Diagnostic::new(
            file,
            expr.loc,
            tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_A_CONSTRUCTOR_FUNCTION_TYPE,
            vec![type_str],
        );
        if value_type.is_type_parameter() {
            let ctor_return = self
                .get_constraint_of_type_parameter(&value_type)
                .and_then(|constraint| {
                    constraint
                        .as_structured()?
                        .construct_signatures()
                        .first()
                        .and_then(|sig| self.get_return_type_of_signature(sig))
                })
                .map(|t| self.type_to_string(&t))
                .unwrap_or_else(|| "unknown".to_string());
            if let Some(tp_symbol) = &value_type.symbol {
                let related_loc = tp_symbol
                    .declarations
                    .first()
                    .map(|d| d.loc)
                    .unwrap_or(expr.loc);
                let related_file = tp_symbol
                    .declarations
                    .first()
                    .and_then(|d| self.get_source_file_of_node(d))
                    .or_else(|| self.current_file.clone());
                let related = tsox_frontend::ast::Diagnostic::new(
                    related_file,
                    related_loc,
                    tsox_core::diagnostics::messages_generated::
                        DID_YOU_MEAN_FOR_0_TO_BE_CONSTRAINED_TO_TYPE_NEW_ARGS_COLON_ANY_1,
                    vec![tp_symbol.name.clone(), ctor_return],
                );
                diag.related_information.push(related);
            }
        }
        self.diagnostics.add(diag);
    }
}

impl Checker {
    // Go checkClassDeclaration 的 mixin 分支：基构造类型是类型变量时，
    // 自身构造器须为单 rest any 参数（TS2545）
    pub(crate) fn check_mixin_constructor_type(&mut self, class_node: &Arc<Node>) {
        let Some(heritage_element) = class_extends_heritage_element(class_node) else {
            return;
        };
        let expr = expression_with_type_arguments_expression(&heritage_element);
        if expr.kind == SyntaxKind::NullKeyword {
            return;
        }
        if matches!(expr.kind, SyntaxKind::Identifier | SyntaxKind::PropertyAccessExpression)
            && let Some(symbol) = self.resolve_entity_name_class_symbol(&expr)
            && !symbol.flags.intersects(SymbolFlags::VALUE)
        {
            return;
        }
        let value_type = self.get_type_of_node(&expr);
        if value_type.flags.contains(TypeFlags::Any)
            || self.is_error_type(&value_type)
            || !value_type.is_type_parameter()
        {
            return;
        }
        let static_type = self.get_type_of_class_declaration(class_node);
        let is_mixin_ctor = static_type
            .as_structured()
            .map(|s| s.construct_signatures())
            .is_some_and(|sigs| {
                sigs.len() == 1
                    && sigs[0].type_parameters.is_empty()
                    && sigs[0].parameters.len() == 1
                    && sigs[0].has_rest_parameter()
                    && {
                        let param_type = self.get_type_of_symbol(&sigs[0].parameters[0]);
                        param_type.flags.contains(TypeFlags::Any)
                            || self
                                .get_array_element_type(&param_type)
                                .flags
                                .contains(TypeFlags::Any)
                    }
            });
        if !is_mixin_ctor {
            let loc = class_node
                .name()
                .map(|n| n.loc)
                .unwrap_or(class_node.loc);
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                tsox_core::diagnostics::messages_generated::
                    A_MIXIN_CLASS_MUST_HAVE_A_CONSTRUCTOR_WITH_A_SINGLE_REST_PARAMETER_OF_TYPE_ANY,
                vec![],
            ));
        }
    }

    // Go getDefaultConstructSignatures：无自有构造器时从基构造类型继承构造
    // 签名（类型变量经约束解析，交集聚合），返回类型改写为本类实例类型
    pub(crate) fn inherit_base_constructor_signatures(
        &mut self,
        class_node: &Arc<Node>,
        instance_type: &Arc<Type>,
        out: &mut Vec<Arc<Signature>>,
    ) {
        use crate::checker::checker_classes_ctor_super_calls::{
            class_extends_heritage_element, expression_with_type_arguments_expression,
        };
        use crate::checker::types::Signature;
        use std::sync::OnceLock;
        let Some(heritage_element) = class_extends_heritage_element(class_node) else {
            return;
        };
        let expr = expression_with_type_arguments_expression(&heritage_element);
        if expr.kind == SyntaxKind::NullKeyword {
            return;
        }
        let value_type = self.get_type_of_node(&expr);
        if value_type.flags.contains(TypeFlags::Any) || self.is_error_type(&value_type) {
            return;
        }
        let base_type = if value_type.is_type_parameter() {
            self.get_constraint_of_type_parameter(&value_type)
                .unwrap_or_else(|| Arc::clone(&value_type))
        } else {
            Arc::clone(&value_type)
        };
        for sig in self.get_signatures_of_type(&base_type, SignatureKind::Construct) {
            let inherited = Signature {
                id: sig.id,
                flags: sig.flags,
                min_argument_count: sig.min_argument_count,
                resolved_min_argument_count: sig.resolved_min_argument_count,
                declaration: sig.declaration.clone(),
                type_parameters: sig.type_parameters.clone(),
                parameters: sig.parameters.clone(),
                this_parameter: sig.this_parameter.clone(),
                resolved_return_type: OnceLock::from(Arc::clone(instance_type)),
                resolved_type_predicate: sig.resolved_type_predicate.clone(),
                target: Some(Arc::clone(&sig)),
                mapper: sig.mapper.clone(),
                isolated_signature_type: OnceLock::new(),
                instantiated_parameter_types: sig.instantiated_parameter_types.clone(),
            };
            out.push(Arc::new(inherited));
        }
    }
}
