use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, SymbolFlags, SyntaxKind,
};
use crate::diagnostics::messages_generated::*;







use super::*;


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
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => {

            }
            SyntaxKind::MetaProperty => {

                let _ = self.get_type_of_node(node);
            }
            SyntaxKind::BinaryExpression => {
                if let crate::ast::NodeData::BinaryExpression(data) = &node.data {

                    self.check_binary_arith_pre(node, data);

                    if data.operator_token.kind == SyntaxKind::CommaToken
                        && !self.is_indirect_call_comma(node)
                        && !self.expression_has_side_effects(&data.left)
                        && !self.diagnostics.get_all().iter().any(|d| {
                            d.code == 2695
                                && d.file
                                    .as_ref()
                                    .map(|f| Arc::ptr_eq(f, self.current_file.as_ref().unwrap_or(&f)))
                                    .unwrap_or(false)
                                && d.loc == data.left.loc
                        })
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.left.loc,
                            crate::diagnostics::messages_generated::
                                LEFT_SIDE_OF_COMMA_OPERATOR_IS_UNUSED_AND_HAS_NO_SIDE_EFFECTS,
                            Vec::new(),
                        ));
                    }
                    self.check_expression(&data.left);

                    if matches!(
                        data.operator_token.kind,
                        crate::ast::SyntaxKind::AmpersandAmpersandToken
                            | crate::ast::SyntaxKind::BarBarToken
                    ) {
                        self.check_truthiness_of_type(&data.left);
                    }

                    let rhs_frame = {
                        let mut lhs: &Arc<Node> = &data.left;
                        loop {
                            match &lhs.data {
                                crate::ast::NodeData::ParenthesizedExpression(p) => {
                                    lhs = &p.expression;
                                }
                                crate::ast::NodeData::NonNullExpression(n) => {
                                    lhs = &n.expression;
                                }
                                _ => break,
                            }
                        }
                        if matches!(
                            data.operator_token.kind,
                            crate::ast::SyntaxKind::QuestionQuestionEqualsToken
                                | crate::ast::SyntaxKind::BarBarEqualsToken
                                | crate::ast::SyntaxKind::AmpersandAmpersandEqualsToken
                        ) {
                            self.logical_rhs_frame(data.operator_token.kind, lhs)
                        } else {
                            None
                        }
                    };
                    match rhs_frame {
                        Some((sym, t)) => {
                            self.logical_rhs_narrowing_frames.push((sym, t));
                            self.check_expression(&data.right);
                            self.logical_rhs_narrowing_frames.pop();
                        }
                        None => self.check_expression(&data.right),
                    }
                    self.check_binary_plus_operator_error(node, data);
                    use crate::ast::SyntaxKind::*;

                    if data.operator_token.kind == EqualsToken
                        && data.left.kind == SyntaxKind::PropertyAccessExpression
                    {
                        if let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                        {
                            let obj_type = self.get_type_of_node(&pa.expression);
                            let name_text = pa.name.text();
                            if self.is_property_readonly(&obj_type, name_text) {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    pa.name.loc,
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                                    vec![name_text.to_string()],
                                ));
                            }
                        }
                    }
                    let mut assigned_target_blocks_type_check = false;

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::PropertyAccessExpression
                        && let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                        && pa.expression.kind == SyntaxKind::Identifier
                        && let Some(enum_sym) = self.resolve_identifier(&pa.expression)
                        && self
                            .resolve_alias_base(enum_sym)
                            .flags
                            .intersects(SymbolFlags::ENUM)
                    {
                        let name_text = pa.name.text();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            pa.name.loc,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                            vec![name_text.to_string()],
                        ));

                        assigned_target_blocks_type_check = true;
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        let name_text = data.left.text().to_string();
                        if let Some(sym) = self.resolve_identifier(&data.left)
                            && let base = self.resolve_alias_base(sym)
                        {
                            let msg = if base.flags.contains(SymbolFlags::Class) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS)
                            } else if base.flags.intersects(SymbolFlags::ENUM) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM)
                            } else if base.flags.contains(SymbolFlags::Function) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION)
                            } else {
                                None
                            };
                            if let Some(msg) = msg {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    data.left.loc,
                                    msg,
                                    vec![name_text],
                                ));

                                assigned_target_blocks_type_check = true;
                            }
                        }
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        if let Some(symbol) = self.resolve_identifier(&data.left) {
                            if self.symbol_is_const_variable(&symbol) {
                                let name_text = data.left.text();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    data.left.loc,
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                                    vec![name_text.to_string()],
                                ));
                            }
                        }
                    }

                    if data.operator_token.kind == EqualsToken
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        if let Some(target) = self.declared_annotation_type_of(&data.left) {

                            if matches!(
                                data.right.kind,
                                SyntaxKind::ObjectLiteralExpression
                                    | SyntaxKind::ArrayLiteralExpression
                                    | SyntaxKind::TypeAssertionExpression
                                    | SyntaxKind::AsExpression
                            ) {
                                self.check_contextual_elements(
                                    &data.right,
                                    &target,
                                    data.right.loc,
                                );
                            }
                        }
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && matches!(
                            data.left.kind,
                            SyntaxKind::PropertyAccessExpression
                                | SyntaxKind::ElementAccessExpression
                        )
                    {
                        self.check_const_property_assignment(&data.left);
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && !assigned_target_blocks_type_check
                    {
                        self.check_assignment_compat(node, data);
                    }

                    let is_equality_op = matches!(
                        data.operator_token.kind,
                        EqualsEqualsToken
                            | ExclamationEqualsToken
                            | EqualsEqualsEqualsToken
                            | ExclamationEqualsEqualsToken
                    );
                    if is_equality_op {
                        let left_type = self.get_type_of_node(&data.left);
                        let right_type = self.get_type_of_node(&data.right);

                        let skip_flags = TypeFlags::Any
                            .union(TypeFlags::Unknown)
                            .union(TypeFlags::Never)
                            .union(TypeFlags::Null)
                            .union(TypeFlags::Undefined);
                        if !left_type.flags.intersects(skip_flags)
                            && !right_type.flags.intersects(skip_flags)
                            && !self.are_types_comparable(&left_type, &right_type)
                        {
                            let left_str = self.type_to_string(&left_type);
                            let right_str = self.type_to_string(&right_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                node.loc,
                                THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
                                vec![left_str, right_str],
                            ));
                        }
                    }
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);

                    if data.operator == SyntaxKind::ExclamationToken {
                        self.check_truthiness_of_type(&data.operand);
                    }

                    if matches!(data.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let crate::ast::NodeData::PostfixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);
                    if matches!(data.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ClassExpression => {

                if let crate::ast::NodeData::ClassExpression(data) = &node.data {
                    self.enclosing_class_stack.push(Arc::clone(node));

                    self.push_scope(node);

                    let this_type = self.build_class_instance_type_with_base(node);
                    self.this_type_stack.push(this_type);
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                    self.this_type_stack.pop();
                    self.pop_scope();
                    self.enclosing_class_stack.pop();
                }
            }
            SyntaxKind::CallExpression => {
                if let crate::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for (i, arg) in data.arguments.iter().enumerate() {
                        self.check_call_arg_with_context(&data.expression, i, arg);
                    }
                }
                self.check_call_arguments(node,  false);
            }
            SyntaxKind::NewExpression => {
                if let crate::ast::NodeData::NewExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    if let Some(args) = &data.arguments {
                        for (i, arg) in args.iter().enumerate() {
                            self.check_call_arg_with_context(&data.expression, i, arg);
                        }
                    }

                    let mut reported_abstract = false;
                    if data.expression.kind == SyntaxKind::Identifier {
                        if let Some(symbol) = self.resolve_identifier(&data.expression) {
                            if self.symbol_is_abstract_class(&symbol) {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                                    vec![],
                                ));
                                reported_abstract = true;
                            }
                        }
                    }

                    if !reported_abstract {
                        let callee_type = self.get_type_of_node(&data.expression);
                        if self.type_includes_abstract_constructor(&callee_type) {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                node.loc,
                                CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                                vec![],
                            ));
                        }
                    }
                }
                self.check_call_arguments(node,  true);
            }
            SyntaxKind::PropertyAccessExpression => {

                if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
                self.check_property_access(node);
            }
            SyntaxKind::ElementAccessExpression => {
                if let crate::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_expression(&data.argument_expression);

                    if data.question_dot_token.is_none() {
                        let obj_type = self.get_type_of_node(&data.expression);
                        self.report_possibly_null_or_undefined(
                            &data.expression,
                            &obj_type,
                            false,
                        );
                    }
                }
            }
            SyntaxKind::ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    self.check_expression(&data.condition);
                    self.check_truthiness_of_type(&data.condition);
                    self.check_expression(&data.when_true);
                    self.check_expression(&data.when_false);
                }
            }
            SyntaxKind::ArrayLiteralExpression => {
                if let crate::ast::NodeData::ArrayLiteralExpression(data) = &node.data {
                    for elem in data.elements.iter() {
                        self.check_expression(elem);
                    }
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                if let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data {

                    let is_destructuring_assignment_target = node.parent.as_ref().is_some_and(
                        |p| match &p.data {
                            crate::ast::NodeData::BinaryExpression(b) => {
                                b.operator_token.kind == SyntaxKind::EqualsToken
                                    && Arc::ptr_eq(&b.left, node)
                            }
                            _ => false,
                        },
                    );
                    if is_destructuring_assignment_target
                        && self.in_ctor_body_stack.last() == Some(&true)
                        && let Some(rhs) = node.parent.as_ref().and_then(|p| {
                            match &p.data {
                                crate::ast::NodeData::BinaryExpression(b) => {
                                    Some(Arc::clone(&b.right))
                                }
                                _ => None,
                            }
                        })
                        && rhs.kind == SyntaxKind::ThisKeyword
                    {
                        let this_type = self.get_type_of_node(&rhs);
                        for prop in data.properties.iter() {
                            let Some(name_node) = prop.name() else { continue };
                            if name_node.kind == SyntaxKind::ComputedPropertyName {
                                continue;
                            }
                            let prop_text = name_node.text().to_string();
                            self.report_abstract_property_access_in_ctor(
                                &name_node,
                                &prop_text,
                                &this_type,
                            );
                        }
                    }

                    if !is_destructuring_assignment_target {
                        {
                            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                                std::collections::HashMap::new();
                        for prop in data.properties.iter() {
                            let Some(name_node) = prop.name() else {
                                continue;
                            };
                            let name = if name_node.kind == SyntaxKind::ComputedPropertyName {

                                let expr = match &name_node.data {
                                    crate::ast::NodeData::ComputedPropertyName(c) => {
                                        Arc::clone(&c.expression)
                                    }
                                    _ => Arc::clone(name_node),
                                };
                                match expr.kind {
                                    SyntaxKind::NumericLiteral
                                    | SyntaxKind::StringLiteral
                                    | SyntaxKind::Identifier => expr.text().to_string(),
                                    SyntaxKind::PrefixUnaryExpression => {
                                        let crate::ast::NodeData::PrefixUnaryExpression(u) =
                                            &expr.data
                                        else {
                                            continue;
                                        };
                                        let sign = if u.operator == SyntaxKind::MinusToken {
                                            "-"
                                        } else {
                                            ""
                                        };
                                        match &u.operand.data {
                                            crate::ast::NodeData::NumericLiteral(n) => {
                                                format!("{sign}{}", n.text)
                                            }
                                            _ => continue,
                                        }
                                    }
                                    SyntaxKind::PropertyAccessExpression => {

                                        let sym = self.resolve_qualified_symbol(&expr);
                                        match sym.as_ref().and_then(|s| s.value_declaration.clone())
                                        {
                                            Some(decl) => match self.get_constant_value(&decl) {
                                                Some(v) => v,
                                                None => continue,
                                            },
                                            None => continue,
                                        }
                                    }
                                    _ => continue,
                                }
                            } else {
                                match name_node.kind {
                                    SyntaxKind::StringLiteral
                                    | SyntaxKind::NumericLiteral
                                    | SyntaxKind::Identifier => name_node.text().to_string(),
                                    _ => continue,
                                }
                            };
                            seen.entry(name).or_default().push(prop);
                        }
                        for (_, group) in seen.iter() {

                            let accessor_pair = group.iter().all(|p| {
                                matches!(p.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
                            }) && group.len() == 2;
                            if group.len() > 1 && !accessor_pair {
                                for (i, prop) in group.iter().enumerate() {
                                    if i == 0 {
                                        continue;
                                    }
                                    if let Some(name_node) = prop.name() {
                                        let name = name_node.text().to_string();
                                        let file = self.current_file.clone();
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            file,
                                            name_node.loc,
                                            crate::diagnostics::messages_generated::
                                                AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_PROPERTIES_WITH_THE_SAME_NAME,
                                            vec![name],
                                        ));
                                    }
                                }
                            }
                        }
                        }
                    }

                    for prop in data.properties.iter() {

                        let has_setter = data.properties.iter().any(|p| {
                            p.kind == SyntaxKind::SetAccessor
                                && p.name().is_some_and(|n| {
                                    n.text()
                                        == prop.name().map(|n| n.text()).unwrap_or_default()
                                })
                        });
                        if prop.kind == SyntaxKind::GetAccessor
                            && !has_setter
                            && self.no_implicit_any
                            && let crate::ast::NodeData::GetAccessorDeclaration(gd) = &prop.data
                            && gd.type_node.is_none()
                            && self.getter_return_reaches_this(prop)
                        {
                            let name_loc = Self::member_name_node(prop)
                                .map(|n| n.loc)
                                .unwrap_or(prop.loc);
                            let name = Self::member_name_node(prop)
                                .map(|n| n.text().to_string())
                                .unwrap_or_default();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    X_0_IMPLICITLY_HAS_RETURN_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_RETURN_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ONE_OF_ITS_RETURN_EXPRESSIONS,
                                vec![name],
                            ));
                        }
                    }

                    let this_typed = self.no_implicit_this
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| {
                                f.file_name.ends_with(".js") || f.file_name.ends_with(".jsx")
                            });

                    let mut contextual_this: Option<Arc<Type>> = None;
                    {
                        let mut literal = Arc::clone(node);
                        loop {
                            let ctx = self.get_contextual_type(&literal, ContextFlags::None);
                            if let Some(t) = ctx
                                .as_ref()
                                .and_then(|t| self.this_type_marker_argument(t, 0))
                            {
                                contextual_this = Some(t);
                                break;
                            }
                            match &literal.parent.as_ref().map(|p| (p.kind, p.parent.clone())) {
                                Some((SyntaxKind::PropertyAssignment, Some(pp))) => {
                                    literal = Arc::clone(pp);
                                }
                                _ => break,
                            }
                        }
                    }
                    let literal_this = match contextual_this {
                        Some(t) => t,
                        None => self.build_object_literal_this_type(node),
                    };
                    for prop in data.properties.iter() {

                        let method_like = matches!(
                            prop.kind,
                            SyntaxKind::MethodDeclaration
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                        );

                        if let Some(name) = Self::member_name_node(prop) {
                            self.check_computed_property_name(&name);
                        }
                        if method_like && this_typed {
                            self.this_type_stack.push(Arc::clone(&literal_this));
                        }
                        self.check_object_literal_element(prop);
                        if method_like && this_typed {
                            self.this_type_stack.pop();
                        }
                    }
                }
            }
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {

                let mut contextual_param_count = self
                    .call_arg_arrow_context
                    .last_mut()
                    .map(|v| std::mem::replace(v, 0))
                    .unwrap_or(0);
                if contextual_param_count == 0 {

                    contextual_param_count = self
                        .contextual_signature_of_arrow(node)
                        .map_or(0, |sig| sig.parameters.len());
                }
                match &node.data {
                    crate::ast::NodeData::ArrowFunction(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);

                        for param in d.parameters.iter() {
                            self.check_parameter_default_initializer(param);
                        }
                    }
                    crate::ast::NodeData::FunctionExpression(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                        for param in d.parameters.iter() {
                            self.check_parameter_default_initializer(param);
                        }
                    }
                    _ => {}
                }

                if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
                    self.this_container_stack
                        .push(ThisContainerKind::PlainFunction);
                }
                self.check_function_like_body(node);
                if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::TemplateExpression => {
                if let crate::ast::NodeData::TemplateExpression(data) = &node.data {
                    for span in data.template_spans.iter() {
                        if let crate::ast::NodeData::TemplateSpan(span_data) = &span.data {
                            self.check_expression(&span_data.expression);
                        }
                    }
                }
            }
            SyntaxKind::AwaitExpression => {
                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::YieldExpression => {
                if let crate::ast::NodeData::YieldExpression(data) = &node.data {

                    if !self.enclosing_function_is_generator(node) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            node.loc,
                            crate::diagnostics::messages_generated::
                                A_YIELD_EXPRESSION_IS_ONLY_ALLOWED_IN_A_GENERATOR_BODY,
                            vec![],
                        ));
                    }
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            SyntaxKind::SpreadElement => {
                if let crate::ast::NodeData::SpreadElement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::AsExpression => {

                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(
                        node,
                        &data.expression,
                        &data.type_node,
                    );

                    if Self::is_const_type_node(&data.type_node)
                        && !self.is_valid_const_assertion_argument(&data.expression)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.expression.loc,
                            crate::diagnostics::messages_generated::
                                A_CONST_ASSERTION_CAN_ONLY_BE_APPLIED_TO_REFERENCES_TO_ENUM_MEMBERS_OR_STRING_NUMBER_BOOLEAN_ARRAY_OR_OBJECT_LITERALS,
                            vec![],
                        ));
                    }
                }
            }
            SyntaxKind::TypeAssertionExpression => {

                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(
                        node,
                        &data.expression,
                        &data.type_node,
                    );
                }
            }
            SyntaxKind::NonNullExpression => {
                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SatisfiesExpression => {
                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TypeOfExpression => {
                if let crate::ast::NodeData::TypeOfExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::DeleteExpression => {
                if let crate::ast::NodeData::DeleteExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_delete_operand(&data.expression);
                }
            }
            SyntaxKind::VoidExpression => {
                if let crate::ast::NodeData::VoidExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TaggedTemplateExpression => {
                if let crate::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    self.check_expression(&data.tag);
                    self.check_expression(&data.template);
                }
            }
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {

                let opening = match node.kind {
                    SyntaxKind::JsxElement => match &node.data {
                        crate::ast::NodeData::JsxElement(d) => Some(Arc::clone(&d.opening_element)),
                        _ => None,
                    },
                    SyntaxKind::JsxSelfClosingElement => Some(Arc::clone(node)),
                    SyntaxKind::JsxFragment => match &node.data {
                        crate::ast::NodeData::JsxFragment(d) => {
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
                    if let crate::ast::NodeData::JsxElement(d) = &node.data
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
                if let crate::ast::NodeData::JsxExpression(data) = &node.data {

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

    fn collect_return_expressions(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
        crate::ast::node_data_generated::for_each_child(node, |child| {
            match child.kind {
                SyntaxKind::ReturnStatement => {
                    if let crate::ast::NodeData::ReturnStatement(r) = &child.data
                        && let Some(expr) = &r.expression
                    {
                        out.push(Arc::clone(expr));
                    }
                    false
                }
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => false,
                _ => {
                    Self::collect_return_expressions(child, out);
                    false
                }
            }
        });
    }

    fn subtree_contains_this(node: &Arc<Node>) -> bool {
        let mut found = false;
        fn walk(root: &Arc<Node>, n: &Arc<Node>, found: &mut bool) {
            if *found {
                return;
            }
            if n.kind == SyntaxKind::ThisKeyword {
                *found = true;
                return;
            }

            if !Arc::ptr_eq(n, root)
                && matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                )
            {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |c| {
                walk(root, c, found);
                *found
            });
        }

        if matches!(
            node.kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        walk(node, node, &mut found);
        found
    }

    fn getter_return_reaches_this(&mut self, accessor: &Arc<Node>) -> bool {
        let crate::ast::NodeData::GetAccessorDeclaration(gd) = &accessor.data else {
            return false;
        };
        let Some(body) = &gd.body else {
            return false;
        };
        let mut returns = Vec::new();
        Self::collect_return_expressions(&body, &mut returns);
        if returns.is_empty() {
            return false;
        }

        let mut this_aliases: Vec<String> = Vec::new();
        crate::ast::node_data_generated::for_each_child(&body, |stmt| {
            if stmt.kind == SyntaxKind::VariableStatement
                && let crate::ast::NodeData::VariableStatement(vs) = &stmt.data
                && let crate::ast::NodeData::VariableDeclarationList(vdl) =
                    &vs.declaration_list.data
            {
                for decl in vdl.declarations.iter() {
                    if let (Some(name), Some(init)) = (decl.name(), {
                            match &decl.data {
                                crate::ast::NodeData::VariableDeclaration(vd) => vd.initializer.clone(),
                                _ => None,
                            }
                        }) {
                        if name.kind == SyntaxKind::Identifier
                            && Self::subtree_contains_this(&init)
                        {
                            this_aliases.push(name.text().to_string());
                        }
                    }
                }
            }
            false
        });
        returns.iter().any(|r| {
            Self::subtree_contains_this(r)
                || {
                    let mut hit = false;
                    fn walk(n: &Arc<Node>, aliases: &[String], hit: &mut bool) {
                        if *hit {
                            return;
                        }
                        if n.kind == SyntaxKind::Identifier
                            && aliases.iter().any(|a| a == n.text())
                        {
                            *hit = true;
                            return;
                        }
                        crate::ast::node_data_generated::for_each_child(n, |c| {
                            walk(c, aliases, hit);
                            *hit
                        });
                    }
                    walk(r, &this_aliases, &mut hit);
                    hit
                }
        })
    }

    fn this_type_marker_argument(&self, t: &Arc<Type>, depth: usize) -> Option<Arc<Type>> {
        if depth > 4 {
            return None;
        }
        let constituent_types: Option<Vec<Arc<Type>>> = match &t.data {
            TypeData::Union(u) => Some(u.union_or_intersection.types.to_vec()),
            TypeData::Intersection(i) => Some(i.union_or_intersection.types.to_vec()),
            _ => None,
        };
        if let Some(types) = constituent_types {
            return types
                .iter()
                .find_map(|c| self.this_type_marker_argument(c, depth + 1));
        }
        let obj = t.as_object()?;
        if obj.type_arguments.len() == 1
            && t.symbol.as_ref().is_some_and(|s| s.name == "ThisType")
        {
            return Some(Arc::clone(&obj.type_arguments[0]));
        }
        None
    }

    fn build_object_literal_this_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data else {
            return self.get_any_type();
        };
        let mut symbol_table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for prop in data.properties.iter() {
            let Some(name_node) = Self::member_name_node(prop) else {
                continue;
            };
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
            ) {
                continue;
            }
            let name = name_node.text().to_string();
            let (member_type, readonly) = match &prop.data {
                crate::ast::NodeData::PropertyAssignment(pa) => {

                    let t = self.get_type_of_node(&pa.initializer);
                    (self.get_widened_type_of_literal(&t), false)
                }
                crate::ast::NodeData::ShorthandPropertyAssignment(sa) => {
                    let t = self.get_type_of_node(&sa.name);
                    (t, false)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    let t = match &gd.type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    (t, true)
                }

                crate::ast::NodeData::MethodDeclaration(_) => {
                    let mut method_sym = crate::ast::Symbol::new(
                        crate::ast::SymbolFlags::Method,
                        name.clone(),
                    );
                    method_sym.declarations = vec![Arc::clone(prop)];
                    let method_sym = Arc::new(method_sym);
                    symbol_table.insert(name, Arc::clone(&method_sym));
                    props.push(method_sym);
                    continue;
                }
                _ => continue,
            };
            let prop_sym = Arc::new(crate::ast::Symbol::new(
                crate::ast::SymbolFlags::Property,
                name.clone(),
            ));
            if readonly {

                let sym_mut = Arc::as_ptr(&prop_sym) as *mut crate::ast::Symbol;
                unsafe {
                    (*sym_mut).check_flags |= crate::ast::CheckFlags::Readonly;
                }
            }
            self.value_symbol_links.insert(
                &prop_sym,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name, Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous
                | crate::checker::types::ObjectFlags::ObjectLiteral,
            id: 0,
            symbol: None,
            alias: None,
            data: crate::checker::types::TypeData::Object(
                crate::checker::types::ObjectTypeData {
                    structured: crate::checker::types::StructuredTypeData {
                        members: symbol_table,
                        properties: props,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        })
    }

    fn check_object_literal_element(&mut self, node: &Arc<Node>) {

        if let Some(name) = node.name()
            && name.kind == SyntaxKind::PrivateIdentifier
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                vec![],
            ));
            return;
        }

        match node.kind {
            SyntaxKind::PropertyAssignment => {
                if let crate::ast::NodeData::PropertyAssignment(data) = &node.data {

                    self.check_expression(&data.initializer);
                }
            }
            SyntaxKind::ShorthandPropertyAssignment => {

                if let crate::ast::NodeData::ShorthandPropertyAssignment(data) = &node.data {
                    self.check_identifier_reference(&data.name);
                }
            }
            SyntaxKind::SpreadAssignment => {
                if let crate::ast::NodeData::SpreadAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {

                self.check_class_member(node);
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
    }

    fn check_function_like_body(&mut self, node: &Arc<Node>) {

        self.get_type_of_node(node);

        self.in_ctor_body_stack.push(false);
        let (body, type_node): (Option<Arc<Node>>, Option<Arc<Node>>) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            crate::ast::NodeData::ArrowFunction(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            _ => (None, None),
        };
        if let Some(body) = body {

            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }

            let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
            let declared_return = type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn))
                .map(|t| self.unwrap_async_return_type(t, is_async));
            self.return_type_stack.push(declared_return);
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => {

                    self.check_expression(&body);
                    if let Some(expected) =
                        self.return_type_stack.last().and_then(|opt| opt.clone())
                    {
                        let actual = self.get_type_of_node(&body);
                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let actual_str = self.type_to_string(&actual);
                            let expected_str = self.type_to_string(&expected);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                body.loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![actual_str, expected_str],
                            ));
                        }
                    }
                }
            }
            self.return_type_stack.pop();
            self.in_ctor_body_stack.pop();
            if is_arrow {
                self.pop_arrow_function_scope();
            } else {
                self.pop_function_scope();
            }
        }
    }

    pub(crate) fn walk_children_for_expressions(&mut self, node: &Arc<Node>) {

        let children: Vec<Arc<Node>> = {
            let mut collected = Vec::new();
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collected.push(Arc::clone(child));
                false
            });
            collected
        };
        for child in &children {

            if is_expression_position_kind(child.kind) {
                self.check_expression(child);
            } else if is_statement_kind(child.kind) {
                self.check_statement(child);
            }

        }
    }

    fn check_jsx_element(&mut self, node: &Arc<Node>) {

        let opening_element: Option<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => Some(Arc::clone(&data.opening_element)),
            crate::ast::NodeData::JsxSelfClosingElement(_) => Some(Arc::clone(node)),
            _ => None,
        };
        let children: Vec<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => data.children.iter().cloned().collect(),
            crate::ast::NodeData::JsxFragment(data) => data.children.iter().cloned().collect(),
            _ => Vec::new(),
        };

        if let Some(opening) = opening_element {
            let attributes: Option<Arc<Node>> = match &opening.data {
                crate::ast::NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.attributes)),
                crate::ast::NodeData::JsxSelfClosingElement(data) => {
                    Some(Arc::clone(&data.attributes))
                }
                _ => None,
            };
            if let Some(attrs) = attributes {
                if let crate::ast::NodeData::JsxAttributes(data) = &attrs.data {
                    for attr in data.properties.iter() {
                        self.check_jsx_attribute(attr);
                    }
                }
            }

        }

        for child in &children {
            self.check_jsx_child(child);
        }
    }

    fn check_jsx_attribute(&mut self, node: &Arc<Node>) {
        match &node.data {
            crate::ast::NodeData::JsxAttribute(data) => {
                if let Some(init) = &data.initializer {
                    self.check_expression(init);
                }
            }
            crate::ast::NodeData::JsxSpreadAttribute(data) => {
                self.check_expression(&data.expression);
            }
            _ => {}
        }
    }

    fn check_jsx_child(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                self.check_expression(node);
            }
            SyntaxKind::JsxExpression => {
                self.check_expression(node);
            }

            _ => {}
        }
    }

    pub(crate) fn cannot_find_name_message_for(name: &str) -> Option<&'static crate::diagnostics::Message> {
        use crate::diagnostics::messages_generated as mg;
        match name {
            "document" | "console" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM,
            ),
            "process" | "require" | "Buffer" | "module" | "NodeJS" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            ),
            _ => None,
        }
    }

    fn check_parameter_default_initializer(&mut self, param: &Arc<Node>) {
        if let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(init) = &pd.initializer
        {
            self.check_expression(init);
        }
    }

    fn check_identifier_reference(&mut self, node: &Arc<Node>) {

        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return,
        };

        if name.is_empty() {
            return;
        }

        if !is_valid_identifier_text(name) {
            return;
        }

        if is_declaration_name(node) {
            return;
        }

        if is_property_access_name(node) {
            return;
        }

        if self.check_invalid_initializer_reference(node, name) {
            return;
        }

        if !self.ts2304_reporting_allowed_for(node) {
            return;
        }

        if let Some(symbol) = self.resolve_identifier(node) {

            if name == "arguments"
                && self.arguments_symbol.is_some()
                && Arc::ptr_eq(&symbol, self.arguments_symbol.as_ref().unwrap())
            {
                let mut cur = node.parent.as_ref();
                let mut in_initializer_or_static_block = false;
                while let Some(a) = cur {
                    match a.kind {
                        SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor => break,
                        SyntaxKind::ArrowFunction => {
                            cur = a.parent.as_ref();
                            continue;
                        }
                        SyntaxKind::PropertyDeclaration
                        | SyntaxKind::ClassStaticBlockDeclaration => {
                            in_initializer_or_static_block = true;
                            break;
                        }
                        _ => {}
                    }
                    cur = a.parent.as_ref();
                }
                if in_initializer_or_static_block {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_ARGUMENTS_CANNOT_BE_REFERENCED_IN_PROPERTY_INITIALIZERS_OR_CLASS_STATIC_INITIALIZATION_BLOCKS,
                        Vec::new(),
                    ));
                    return;
                }
            }

            let is_export_assignment_name = node
                .parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment);
            let base = self.resolve_alias_base(Arc::clone(&symbol));

            let is_true_namespace = base
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ModuleDeclaration
                    && d.name().is_some_and(|n| {
                        !matches!(n.kind, SyntaxKind::StringLiteral)
                    }));
            if !is_export_assignment_name
                && base.flags.contains(SymbolFlags::ValueModule)
                && is_true_namespace
                && !self.namespace_usable_as_value(&base)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                    vec![name.to_string()],
                ));
                return;
            }

            self.check_block_scoped_variable_used_before_declaration(node, &symbol, name);

            self.check_variable_used_before_assigned(node, &symbol, name);
            return;
        }

        let file = self.current_file.clone();

        {
            let is_primitive_type_name = matches!(
                name,
                "any" | "string" | "number" | "boolean" | "never" | "unknown"
            );
            let reported = if is_primitive_type_name {

                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file.clone(),
                    node.loc,
                    crate::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
                true
            } else {

                let type_hit = self
                    .resolve_identifier_with_meaning(node, SymbolFlags::TYPE)
                    .map(|s| self.resolve_alias_base(s));
                if let Some(sym) = type_hit
                    && !sym.flags.intersects(SymbolFlags::VALUE)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                        vec![name.to_string()],
                    ));
                    true
                } else {
                    false
                }
            };
            if reported {
                return;
            }
        }

        let diagnostic = if let Some(class) = self.enclosing_class_stack.last().cloned() {
            let class_name = Self::class_name_text(&class);
            if let Some(is_member_static) = self.class_member_static_by_name(&class, name) {
                if is_member_static {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0,
                        vec![name.to_string(), class_name],
                    )
                } else if self.this_container_stack.last() == Some(&ThisContainerKind::InstanceMember)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0,
                        vec![name.to_string()],
                    )
                } else if let Some(suggestion) =
                    self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(
                    name,
                    SymbolFlags::VALUE,
                ) {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        CANNOT_FIND_NAME_0,
                        vec![name.to_string()],
                    )
                }
            } else if let Some(suggestion) = self.find_name_suggestion(
                name,
                SymbolFlags::VALUE,
            ) {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                    vec![name.to_string(), suggestion],
                )
            } else {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    CANNOT_FIND_NAME_0,
                    vec![name.to_string()],
                )
            }
        } else if let Some(msg) = Self::cannot_find_name_message_for(name) {
            crate::ast::Diagnostic::new(file, node.loc, *msg, vec![name.to_string()])
        } else if let Some(suggestion) =
            self.find_name_suggestion(name, SymbolFlags::VALUE)
        {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                vec![name.to_string(), suggestion],
            )
        } else {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                *Self::cannot_find_name_message_for(name).unwrap_or(&CANNOT_FIND_NAME_0),
                vec![name.to_string()],
            )
        };
        self.diagnostics.add(diagnostic);
    }

    pub(crate) fn check_super_before_this(&mut self, body: &Arc<Node>) {
        fn visit(
            c: &mut Checker,
            n: &Arc<Node>,
            super_seen: &mut bool,
        ) {
            if n.kind == SyntaxKind::ThisKeyword {
                if !*super_seen {
                    let file = c.current_file.clone();
                    c.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        n.loc,
                        crate::diagnostics::messages_generated::
                            X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_THIS_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS,
                        vec![],
                    ));
                }
                return;
            }

            if n.kind == SyntaxKind::CallExpression
                && let crate::ast::NodeData::CallExpression(call) = &n.data
                && call.expression.kind == SyntaxKind::SuperKeyword
            {
                for arg in call.arguments.iter() {
                    visit(c, arg, super_seen);
                }
                *super_seen = true;
                return;
            }

            if matches!(
                n.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return;
            }

            if matches!(n.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                visit(c, child, super_seen);
                false
            });
        }
        let mut super_seen = false;
        visit(self, body, &mut super_seen);
    }
}
