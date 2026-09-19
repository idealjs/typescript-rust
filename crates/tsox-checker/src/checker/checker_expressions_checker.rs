#![allow(unused_imports)]

use crate::checker::checker_expressions::*;

impl Checker {
    pub fn check_expression(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;
        match node.kind {
            SyntaxKind::Identifier => {
                self.check_identifier_reference(node);
            }
            SyntaxKind::NumericLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => {}
            SyntaxKind::ThisKeyword => {
                self.check_this_expression_reference(node);
            }
            SyntaxKind::MetaProperty => {
                let _ = self.get_type_of_node(node);
            }
            SyntaxKind::SuperKeyword => {
                self.check_super_expression(node);
            }
            SyntaxKind::BinaryExpression => {
                self.check_binary_expression(node);
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let tsox_frontend::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);

                    if data.operator == SyntaxKind::ExclamationToken {
                        self.check_truthiness_of_type(&data.operand);
                    }

                    if matches!(
                        data.operator,
                        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                    ) {
                        self.check_unary_arithmetic_operand(&data.operand);
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let tsox_frontend::ast::NodeData::PostfixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);
                    if matches!(
                        data.operator,
                        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                    ) {
                        self.check_unary_arithmetic_operand(&data.operand);
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::ParenthesizedExpression => {
                if let tsox_frontend::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ClassExpression => {
                if let tsox_frontend::ast::NodeData::ClassExpression(data) = &node.data {
                    self.enclosing_class_stack.push(Arc::clone(node));

                    self.push_scope(node);
                    self.check_node_decorators(node);

                    let this_type = self.build_class_instance_type_with_base(node);
                    self.this_type_stack.push(this_type);
                    // 类表达式 extends 基类是值位置：走完整表达式检查（tsc checkClassExpression）
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data
                                && hc.token == SyntaxKind::ExtendsKeyword
                            {
                                for type_ref in hc.types.iter() {
                                    if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data {
                                        self.check_expression(&ewa.expression);
                                        self.check_base_constructor_type(&ewa.expression);
                                    }
                                }
                            }
                        }
                    }
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                    self.check_mixin_constructor_type(node);
                    self.check_members_for_override_modifier(node);
                    self.this_type_stack.pop();
                    self.pop_scope();
                    self.enclosing_class_stack.pop();
                }
            }
            SyntaxKind::CallExpression => {
                if let tsox_frontend::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for (i, arg) in data.arguments.iter().enumerate() {
                        self.check_call_arg_with_context(&data.expression, i, arg);
                    }
                }
                self.check_call_arguments(node, false);
                self.check_dynamic_import_extension_rules(node);
            }
            SyntaxKind::NewExpression => {
                self.check_new_expression(node);
            }
            SyntaxKind::QualifiedName => {
                self.check_qualified_name_expression(node);
            }
            SyntaxKind::PropertyAccessExpression => {
                if let tsox_frontend::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
                self.check_property_access(node);
            }
            SyntaxKind::ElementAccessExpression => {
                if let tsox_frontend::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_expression(&data.argument_expression);

                    if data.question_dot_token.is_none() {
                        let obj_type = self.get_type_of_node(&data.expression);
                        self.report_possibly_null_or_undefined(&data.expression, &obj_type, false);
                    }
                }
                // Go checkElementAccessExpression：求取表达式类型以触发
                // 索引键类型检查（TS2538）
                self.get_type_of_node(node);
            }
            SyntaxKind::ConditionalExpression => {
                if let tsox_frontend::ast::NodeData::ConditionalExpression(data) = &node.data {
                    self.check_expression(&data.condition);
                    self.check_truthiness_of_type(&data.condition);
                    self.check_expression(&data.when_true);
                    self.check_expression(&data.when_false);
                }
            }
            SyntaxKind::ArrayLiteralExpression => {
                if let tsox_frontend::ast::NodeData::ArrayLiteralExpression(data) = &node.data {
                    for elem in data.elements.iter() {
                        self.check_expression(elem);
                    }
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.check_object_literal_expression(node);
            }
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {
                self.check_function_like_expression(node);
            }
            SyntaxKind::TemplateExpression => {
                if let tsox_frontend::ast::NodeData::TemplateExpression(data) = &node.data {
                    for span in data.template_spans.iter() {
                        if let tsox_frontend::ast::NodeData::TemplateSpan(span_data) = &span.data {
                            self.check_expression(&span_data.expression);
                        }
                    }
                }
            }
            SyntaxKind::AwaitExpression => {
                if let tsox_frontend::ast::NodeData::AwaitExpression(data) = &node.data {
                    self.check_await_expression_grammar(node);
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::YieldExpression => {
                if let tsox_frontend::ast::NodeData::YieldExpression(data) = &node.data {
                    if !self.enclosing_function_is_generator(node) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            node.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_YIELD_EXPRESSION_IS_ONLY_ALLOWED_IN_A_GENERATOR_BODY,
                            vec![],
                        ));
                    }
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                    self.check_yield_expression_assignability(node);
                    // Go getNextTypeOfYieldExpression：无返回注解的生成器里
                    // yield 结果为 any 且表达式未被使用时报 TS7057（noImplicitAny）
                    if self.compiler_options.no_implicit_any.is_true()
                        && self.yield_container_of(node).is_some_and(|c| {
                            c.is_generator && c.return_type_node.is_none()
                        })
                    {
                        // Go expressionResultIsUnused：表达式语句/void/for 头位
                        // 之外即「被使用」，被使用且无上下文型才报
                        let mut p = node.parent();
                        let mut unused = false;
                        while let Some(parent) = p {
                            match parent.kind {
                                SyntaxKind::ExpressionStatement
                                | SyntaxKind::VoidExpression => {
                                    unused = true;
                                    break;
                                }
                                SyntaxKind::ParenthesizedExpression => p = parent.parent(),
                                _ => break,
                            }
                        }
                        // Go：有上下文型（含计算属性名位的 string|number|symbol
                        // 约束）时不报
                        let in_computed_name = {
                            let mut cur = node.parent();
                            let mut hit = false;
                            while let Some(a) = cur {
                                if matches!(a.kind, SyntaxKind::ComputedPropertyName | SyntaxKind::Decorator) {
                                    hit = true;
                                    break;
                                }
                                if matches!(
                                    a.kind,
                                    SyntaxKind::ObjectLiteralExpression | SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                                ) {
                                    break;
                                }
                                cur = a.parent();
                            }
                            hit
                        };
                        let has_contextual = in_computed_name
                            || self
                                .get_contextual_type(node, crate::checker::types::ContextFlags::None)
                                .is_some_and(|t| !t.flags.contains(TypeFlags::Any));
                        if !unused && !has_contextual {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                node.loc,
                                tsox_core::diagnostics::messages_generated::
                                    X_YIELD_EXPRESSION_IMPLICITLY_RESULTS_IN_AN_ANY_TYPE_BECAUSE_ITS_CONTAINING_GENERATOR_LACKS_A_RETURN_TYPE_ANNOTATION,
                                vec![],
                            ));
                        }
                    }
                }
            }
            SyntaxKind::SpreadElement => {
                if let tsox_frontend::ast::NodeData::SpreadElement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::AsExpression => {
                if let tsox_frontend::ast::NodeData::AsExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(node, &data.expression, &data.type_node);

                    if Self::is_const_type_node(&data.type_node)
                        && !self.is_valid_const_assertion_argument(&data.expression)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            data.expression.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_CONST_ASSERTION_CAN_ONLY_BE_APPLIED_TO_REFERENCES_TO_ENUM_MEMBERS_OR_STRING_NUMBER_BOOLEAN_ARRAY_OR_OBJECT_LITERALS,
                            vec![],
                        ));
                    }
                }
            }
            SyntaxKind::TypeAssertionExpression => {
                if let tsox_frontend::ast::NodeData::TypeAssertion(data) = &node.data {
                    self.check_expression(&data.expression);
                    // isConstTypeReference：不做 overlap 检查（Go 同）
                    if !crate::checker::utilities_has_only_expression_initialization::is_const_type_reference(&data.type_node) {
                        self.check_assertion_overlap(node, &data.expression, &data.type_node);
                    }
                }
            }
            SyntaxKind::NonNullExpression => {
                if let tsox_frontend::ast::NodeData::NonNullExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SatisfiesExpression => {
                if let tsox_frontend::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TypeOfExpression => {
                if let tsox_frontend::ast::NodeData::TypeOfExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::DeleteExpression => {
                if let tsox_frontend::ast::NodeData::DeleteExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_delete_operand(&data.expression);
                }
            }
            SyntaxKind::VoidExpression => {
                if let tsox_frontend::ast::NodeData::VoidExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TaggedTemplateExpression => {
                if let tsox_frontend::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    self.check_expression(&data.tag);
                    self.check_expression(&data.template);
                }
            }
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                let opening = match node.kind {
                    SyntaxKind::JsxElement => match &node.data {
                        tsox_frontend::ast::NodeData::JsxElement(d) => {
                            Some(Arc::clone(&d.opening_element))
                        }
                        _ => None,
                    },
                    SyntaxKind::JsxSelfClosingElement => Some(Arc::clone(node)),
                    SyntaxKind::JsxFragment => match &node.data {
                        tsox_frontend::ast::NodeData::JsxFragment(d) => {
                            Some(Arc::clone(&d.opening_fragment))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(opening) = opening {
                    self.check_jsx_opening_like_element(&opening);
                }

                if node.kind == SyntaxKind::JsxElement {
                    if let tsox_frontend::ast::NodeData::JsxElement(d) = &node.data
                        && crate::checker::jsx::is_jsx_intrinsic_tag_name(
                            &crate::checker::jsx::jsx_tag_name(&d.closing_element)
                                .unwrap_or_else(|| d.closing_element.clone()),
                        )
                    {
                        self.check_jsx_intrinsic_element(&d.closing_element);
                    }
                }
                self.check_jsx_element(node);
            }
            SyntaxKind::JsxExpression => {
                if let tsox_frontend::ast::NodeData::JsxExpression(data) = &node.data {
                    self.check_grammar_jsx_expression(node);
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }
}

impl Checker {
    /// Go checkQualifiedName（值位限定名）：限定链解析失败时在右段报
    /// TS2339（`new multiM.c()` 的 c 不存在于 multiM）
    pub(crate) fn check_qualified_name_expression(&mut self, node: &Arc<Node>) {
        if let Err((right, ns_path, text)) = self.resolve_qualified_symbol_traced(node) {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                right.loc,
                tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![text, format!("typeof {ns_path}")],
            ));
        }
    }

    /// Go checkGrammarAwaitExpression（IsInTopLevelContext 分支）：顶层 await
    /// 在非模块文件报「文件无 import/export，考虑加空 export」（TS1375）；
    /// 模块文件按 module/target 组合报 TS1378（es2022+ 模块且 es2017+ 目标
    /// 才允许）
    /// Go checkGrammarAwaitOrAwaitUsing（await 表达式形态）：class static block
    /// 无条件禁用（先于 AwaitContext 检查）；非 await 上下文再走顶层/模块门槛
    pub(crate) fn check_await_expression_grammar(&mut self, node: &Arc<Node>) {
        let container = crate::checker::utilities_get_assignment_target::
            get_containing_function_or_class_static_block(node);
        if container.as_ref().is_some_and(|c| {
            c.kind == SyntaxKind::ClassStaticBlockDeclaration
        }) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    X_AWAIT_EXPRESSION_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
                vec![],
            ));
            return;
        }
        if node.flags.contains(NodeFlags::AwaitContext) {
            return;
        }
        if self.is_within_function_like(node) {
            return;
        }
        let Some(file) = self.get_source_file_of_node(node) else {
            return;
        };
        let is_module = tsox_frontend::ast::is_external_module(&file);
        if !is_module {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    X_AWAIT_EXPRESSIONS_ARE_ONLY_ALLOWED_AT_THE_TOP_LEVEL_OF_A_FILE_WHEN_THAT_FILE_IS_A_MODULE_BUT_THIS_FILE_HAS_NO_IMPORTS_OR_EXPORTS_CONSIDER_ADDING_AN_EMPTY_EXPORT_TO_MAKE_THIS_FILE_A_MODULE,
                vec![],
            ));
            return;
        }
        let module_ok = matches!(
            self.compiler_options.module,
            tsox_core::core::compiler_options::ModuleKind::ES2022
                | tsox_core::core::compiler_options::ModuleKind::ESNext
                | tsox_core::core::compiler_options::ModuleKind::System
                | tsox_core::core::compiler_options::ModuleKind::Node16
                | tsox_core::core::compiler_options::ModuleKind::Node18
                | tsox_core::core::compiler_options::ModuleKind::Node20
                | tsox_core::core::compiler_options::ModuleKind::NodeNext
                | tsox_core::core::compiler_options::ModuleKind::Preserve
        );
        let target_ok = self.compiler_options.target
            >= tsox_core::core::compiler_options::ScriptTarget::ES2017;
        if !(module_ok && target_ok) {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    TOP_LEVEL_AWAIT_EXPRESSIONS_ARE_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_NODENEXT_OR_PRESERVE_AND_THE_TARGET_OPTION_IS_SET_TO_ES2017_OR_HIGHER,
                vec![],
            ));
        }
    }

    fn is_within_function_like(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return true,
                SyntaxKind::SourceFile => return false,
                _ => {}
            }
            cur = n.parent();
        }
        false
    }
}
