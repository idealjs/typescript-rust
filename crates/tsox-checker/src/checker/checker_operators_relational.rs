use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;

use crate::checker::checker::*;
use crate::checker::checker_object_literal_is_destructuring_target::is_entity_name_expression;
use crate::checker::types::TypeData;
use crate::checker::types::TypeFlags;

impl Checker {
    pub(crate) fn check_binary_relational_operator_error(
        &mut self,
        node: &Arc<Node>,
        data: &tsox_frontend::ast::node_data_generated::BinaryExpressionData,
    ) {
        use SyntaxKind::*;
        let op = data.operator_token.kind;
        if !matches!(
            op,
            LessThanToken | GreaterThanToken | LessThanEqualsToken | GreaterThanEqualsToken
        ) {
            return;
        }
        let lt = self.get_type_of_node(&data.left);
        let rt = self.get_type_of_node(&data.right);
        if !self.check_for_disallowed_es_symbol_operand(&data.left, &data.right, &lt, &rt, op) {
            return;
        }
        let lt = self.check_non_null_type(&lt, &data.left);
        let rt = self.check_non_null_type(&rt, &data.right);
        let lt = self.get_base_type_of_literal_type_for_comparison(&lt);
        let rt = self.get_base_type_of_literal_type_for_comparison(&rt);
        if lt.flags.contains(TypeFlags::Any) || rt.flags.contains(TypeFlags::Any) {
            return;
        }
        let l_num = self.assignable_to_number_or_bigint(&lt);
        let r_num = self.assignable_to_number_or_bigint(&rt);
        let compatible = l_num && r_num
            || !l_num && !r_num && self.are_types_comparable(&lt, &rt);
        if !compatible {
            let lt_str = self.type_to_string(&lt);
            let rt_str = self.type_to_string(&rt);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPES_1_AND_2,
                vec![Self::op_display(op).to_string(), lt_str, rt_str],
            ));
        }
    }

    fn assignable_to_number_or_bigint(&mut self, t: &Arc<Type>) -> bool {
        let n = self.number_type();
        if self.is_type_assignable_to(t, &n) {
            return true;
        }
        let b = self.bigint_type();
        self.is_type_assignable_to(t, &b)
    }

    pub(crate) fn check_for_disallowed_es_symbol_operand(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
        lt: &Arc<Type>,
        rt: &Arc<Type>,
        op: SyntaxKind,
    ) -> bool {
        let offending = if self.maybe_essymbol_considering_constraint(lt) {
            Some(left)
        } else if self.maybe_essymbol_considering_constraint(rt) {
            Some(right)
        } else {
            None
        };
        match offending {
            Some(operand) => {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    operand.loc,
                    tsox_core::diagnostics::messages_generated::
                        THE_0_OPERATOR_CANNOT_BE_APPLIED_TO_TYPE_SYMBOL,
                    vec![Self::op_display(op).to_string()],
                ));
                false
            }
            None => true,
        }
    }

    pub(crate) fn maybe_essymbol_considering_constraint(&self, t: &Arc<Type>) -> bool {
        self.maybe_essymbol_considering_constraint_at(t, 10)
    }

    fn maybe_essymbol_considering_constraint_at(&self, t: &Arc<Type>, depth: u32) -> bool {
        if depth == 0 {
            return false;
        }
        let symbol_like = TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol;
        if t.flags.intersects(symbol_like) {
            return true;
        }
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.maybe_essymbol_considering_constraint_at(c, depth - 1));
        }
        if t.flags.contains(TypeFlags::TypeParameter)
            && let Some(constraint) = self.get_constraint_of_type_parameter(t)
        {
            return self.maybe_essymbol_considering_constraint_at(&constraint, depth - 1);
        }
        false
    }

    pub(crate) fn get_base_type_of_literal_type_for_comparison(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(
            TypeFlags::StringLiteral | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
        ) {
            return self.string_type();
        }
        if t.flags.intersects(TypeFlags::NumberLiteral | TypeFlags::EnumLiteral | TypeFlags::Enum)
        {
            return self.number_type();
        }
        if t.flags.intersects(TypeFlags::BigIntLiteral) {
            return self.bigint_type();
        }
        if t.flags.intersects(TypeFlags::BooleanLiteral) {
            return self.boolean_type();
        }
        if let TypeData::Union(u) = &t.data {
            let mapped = u
                .union_or_intersection
                .types
                .iter()
                .map(|m| self.get_base_type_of_literal_type_for_comparison(m))
                .collect();
            return self.build_union_from_types(mapped);
        }
        t.clone()
    }

    pub(crate) fn check_non_null_type(&mut self, t: &Arc<Type>, node: &Arc<Node>) -> Arc<Type> {
        let nullable = TypeFlags::Null | TypeFlags::Undefined;
        if self.strict_null_checks && t.flags.contains(TypeFlags::Unknown) {
            let text = if is_entity_name_expression(node) {
                self.node_source_text(node).unwrap_or_default()
            } else {
                String::new()
            };
            if !text.is_empty() && text.len() < 100 {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::X_0_IS_OF_TYPE_UNKNOWN,
                    vec![text],
                ));
            } else {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::OBJECT_IS_OF_TYPE_UNKNOWN,
                    Vec::new(),
                ));
            }
            return self.error_type();
        }
        if node.kind == SyntaxKind::NullKeyword && t.flags.intersects(nullable) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                tsox_core::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                vec!["null".to_string()],
            ));
            return self.error_type();
        }
        if matches!(node.kind, SyntaxKind::UndefinedKeyword | SyntaxKind::Identifier)
            && (node.kind != SyntaxKind::Identifier || node.text() == "undefined")
            && t.flags.contains(TypeFlags::Undefined)
        {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                tsox_core::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                vec!["undefined".to_string()],
            ));
            return self.error_type();
        }
        if self.report_possibly_null_or_undefined(node, t, false) {
            let non_nullable = self.remove_nullable_from_union(t);
            if non_nullable
                .flags
                .intersects(nullable | TypeFlags::Never)
            {
                return self.error_type();
            }
            return non_nullable;
        }
        t.clone()
    }
}

impl Checker {
    // 一元 ++/--（Go checkPrefixUnaryExpression/checkPostfixUnaryExpression 的算术分支）
    pub(crate) fn check_unary_arithmetic_operand(&mut self, operand: &Arc<Node>) {
        let unwrapped = Self::skip_reference_wrappers(operand);
        if self.in_strict_context()
            && unwrapped.kind == SyntaxKind::Identifier
            && matches!(unwrapped.text(), "eval" | "arguments")
        {
            let mut in_class = false;
            let mut anc = unwrapped.parent();
            while let Some(p) = anc {
                if matches!(p.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
                    in_class = true;
                    break;
                }
                anc = p.parent();
            }
            let is_module = self
                .current_file
                .as_ref()
                .is_some_and(|f| f.external_module_indicator.is_some());
            let msg = if in_class {
                tsox_core::diagnostics::messages_generated::CODE_CONTAINED_IN_A_CLASS_IS_EVALUATED_IN_JAVASCRIPT_S_STRICT_MODE_WHICH_DOES_NOT_ALLOW_THIS_USE_OF_0_FOR_MORE_INFORMATION_SEE_HTTPS_COLON_SLASH_SLASHDEVELOPER_MOZILLA_ORG_SLASHEN_US_SLASHDOCS_SLASHWEB_SLASHJAVASCRIPT_SLASHREFERENCE_SLASHSTRICT_MODE
            } else if is_module {
                tsox_core::diagnostics::messages_generated::
                    INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else {
                tsox_core::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE
            };
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                msg,
                vec![unwrapped.text().to_string()],
            ));
        }
        if unwrapped.kind == SyntaxKind::UndefinedKeyword {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_NOT_A_VARIABLE,
                vec!["undefined".to_string()],
            ));
            return;
        }
        if matches!(unwrapped.kind, SyntaxKind::Identifier | SyntaxKind::UndefinedKeyword)
            && let Some(sym) = self.resolve_identifier(&unwrapped)
        {
            let base = self.resolve_alias_base(sym);
            let variable = base
                .flags
                .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable);
            if !variable {
                let msg = if base.flags.intersects(SymbolFlags::ENUM) {
                    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM
                } else if base.flags.contains(SymbolFlags::Class) {
                    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS
                } else if base.flags.intersects(SymbolFlags::ValueModule) {
                    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_NAMESPACE
                } else if base.flags.contains(SymbolFlags::Function) {
                    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION
                } else {
                    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_NOT_A_VARIABLE
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    operand.loc,
                    msg,
                    vec![unwrapped.text().to_string()],
                ));
                return;
            }
        }
        if let Some((loc, name, enum_name)) = self.enum_readonly_access_loc(&unwrapped) {
            if name.is_empty() {
                // 非字面量索引经逆映射签名读取（string）：算术不合法 + 写只读
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    operand.loc,
                    tsox_core::diagnostics::messages_generated::
                        AN_ARITHMETIC_OPERAND_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                    Vec::new(),
                ));
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                    vec![format!("typeof {enum_name}")],
                ));
                return;
            }
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                tsox_core::diagnostics::messages_generated::
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                vec![name],
            ));
            return;
        }
        let t = self.get_type_of_node(operand);
        let t = self.check_non_null_type(&t, operand);
        if !self.assignable_to_number_or_bigint(&t) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                tsox_core::diagnostics::messages_generated::
                    AN_ARITHMETIC_OPERAND_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                Vec::new(),
            ));
            return;
        }
        self.check_increment_reference_expression(operand);
    }

    fn skip_reference_wrappers(node: &Arc<Node>) -> Arc<Node> {
        let mut n = Arc::clone(node);
        loop {
            let next = match &n.data {
                tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => Some(Arc::clone(&p.expression)),
                tsox_frontend::ast::NodeData::NonNullExpression(e) => Some(Arc::clone(&e.expression)),
                tsox_frontend::ast::NodeData::AsExpression(a) => Some(Arc::clone(&a.expression)),
                tsox_frontend::ast::NodeData::TypeAssertion(t) => Some(Arc::clone(&t.expression)),
                _ => None,
            };
            match next {
                Some(x) => n = x,
                None => return n,
            }
        }
    }

    // 枚举成员访问（E.B / E["B"]）在赋值位是只读属性：返回报错位与属性名
    fn enum_readonly_access_loc(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<(tsox_core::core::text::TextRange, String, String)> {
        match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                let is_enum = pa.expression.kind == SyntaxKind::Identifier
                    && self
                        .resolve_identifier(&pa.expression)
                        .map(|s| self.resolve_alias_base(s).flags.intersects(SymbolFlags::ENUM))
                        .unwrap_or(false);
                if is_enum {
                    Some((
                        pa.name.loc,
                        pa.name.text().to_string(),
                        pa.expression.text().to_string(),
                    ))
                } else {
                    None
                }
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(ea) => {
                let is_enum = ea.expression.kind == SyntaxKind::Identifier
                    && self
                        .resolve_identifier(&ea.expression)
                        .map(|s| self.resolve_alias_base(s).flags.intersects(SymbolFlags::ENUM))
                        .unwrap_or(false);
                if !is_enum {
                    return None;
                }
                let enum_name = ea.expression.text().to_string();
                match ea.argument_expression.kind {
                    SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => Some((
                        ea.argument_expression.loc,
                        ea.argument_expression.text().to_string(),
                        enum_name,
                    )),
                    _ => Some((node.loc, String::new(), enum_name)),
                }
            }
            _ => None,
        }
    }

    fn check_increment_reference_expression(&mut self, operand: &Arc<Node>) {
        let node = Self::skip_reference_wrappers(operand);
        let is_access = matches!(
            node.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
        );
        if node.kind != SyntaxKind::Identifier && !is_access {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                Vec::new(),
            ));
            return;
        }
        let optional_chain = match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => pa.question_dot_token.is_some(),
            tsox_frontend::ast::NodeData::ElementAccessExpression(ea) => ea.question_dot_token.is_some(),
            _ => false,
        };
        if optional_chain {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_OPERAND_OF_AN_INCREMENT_OR_DECREMENT_OPERATOR_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
                Vec::new(),
            ));
        }
    }
}
