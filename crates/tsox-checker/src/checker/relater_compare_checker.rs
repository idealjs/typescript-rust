#![allow(unused_imports)]

use crate::checker::relater_compare::*;

fn parameter_has_type_annotation(p: &Arc<tsox_frontend::ast::Node>) -> bool {
    matches!(
        &p.data,
        tsox_frontend::ast::NodeData::ParameterDeclaration(d) if d.type_node.is_some()
    )
}

impl Checker {
    pub fn compare_types_identical(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_identical_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_simple(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_subtype_of(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn check_type_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_assignable_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_comparable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
    ) -> bool {
        self.is_type_comparable_to(source, target)
    }

    pub fn check_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
    ) -> bool {
        self.is_type_related_to(source, target, relation)
    }

    pub(crate) fn elaborate_error(
        &mut self,
        expr: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        // Go elaborateError：目标是（含）泛型条件型时不 elaboration
        if Self::type_is_or_has_generic_conditional(target) {
            return false;
        }
        for kind in [SignatureKind::Construct, SignatureKind::Call] {
            if self.elaborate_did_you_mean_to_call_or_construct(
                expr,
                source,
                target,
                relation,
                kind,
                out.as_deref_mut(),
            ) {
                return true;
            }
        }
        match expr.kind {
            tsox_frontend::ast::SyntaxKind::ParenthesizedExpression => {
                let inner = match &expr.data {
                    tsox_frontend::ast::NodeData::ParenthesizedExpression(d) => {
                        Arc::clone(&d.expression)
                    }
                    _ => return false,
                };
                self.elaborate_error(&inner, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::ObjectLiteralExpression => {
                self.elaborate_object_literal(expr, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::ArrayLiteralExpression => {
                self.elaborate_array_literal(expr, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::ArrowFunction => {
                self.elaborate_arrow_function(expr, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::BinaryExpression => {
                let (op, right) = match &expr.data {
                    tsox_frontend::ast::NodeData::BinaryExpression(d) => {
                        (d.operator_token.kind, Arc::clone(&d.right))
                    }
                    _ => return false,
                };
                if matches!(
                    op,
                    tsox_frontend::ast::SyntaxKind::EqualsToken
                        | tsox_frontend::ast::SyntaxKind::CommaToken
                ) {
                    return self.elaborate_error(&right, source, target, relation, out);
                }
                false
            }
            _ => false,
        }
    }

    fn type_is_or_has_generic_conditional(t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Conditional) {
            return true;
        }
        if t.flags.contains(TypeFlags::Intersection)
            && let Some(ui) = t.as_union_or_intersection()
        {
            return ui.types.iter().any(Self::type_is_or_has_generic_conditional);
        }
        false
    }

    // Go elaborateDidYouMeanToCallOrConstruct：源的调用/构造签名返回型可
    // 赋给目标时，在表达式节点上重新报错并附「是否想调用」提示
    fn elaborate_did_you_mean_to_call_or_construct(
        &mut self,
        expr: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        kind: SignatureKind,
        out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        let signatures = self.get_signatures_of_type(source, kind);
        let mut matches = false;
        for sig in &signatures {
            let Some(ret) = self.get_return_type_of_signature(sig) else {
                continue;
            };
            if ret.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                continue;
            }
            if self.is_type_related_to(&ret, target, relation) {
                matches = true;
                break;
            }
        }
        if !matches {
            return false;
        }
        let mut diags: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        if !self.check_type_related_to_and_optionally_elaborate(
            source,
            target,
            relation,
            Some(expr),
            None,
            None,
            Some(&mut diags),
        ) && let Some(mut diagnostic) = diags.into_iter().next()
        {
            let message = if kind == SignatureKind::Construct {
                tsox_core::diagnostics::messages_generated::DID_YOU_MEAN_TO_USE_NEW_WITH_THIS_EXPRESSION
            } else {
                tsox_core::diagnostics::messages_generated::DID_YOU_MEAN_TO_CALL_THIS_EXPRESSION
            };
            let file = self.get_source_file_of_node(expr);
            diagnostic.related_information.push(tsox_frontend::ast::Diagnostic::new(
                file,
                expr.loc,
                message,
                Vec::new(),
            ));
            match out {
                Some(o) => o.push(diagnostic),
                None => self.diagnostics.add(diagnostic),
            }
            return true;
        }
        false
    }

    // Go elaborateArrowFunction：表达式体且参数无注解的箭头函数，把返回
    // 表达式对目标签名返回型（多签名为其并集）重新 elaboration
    pub(crate) fn elaborate_arrow_function(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        let data = match &node.data {
            tsox_frontend::ast::NodeData::ArrowFunction(d) => d,
            _ => return false,
        };
        if data.body.kind == tsox_frontend::ast::SyntaxKind::Block {
            return false;
        }
        if data
            .parameters
            .iter()
            .any(|p| parameter_has_type_annotation(p))
        {
            return false;
        }
        let Some(source_sig) = self.get_single_call_signature(source) else {
            return false;
        };
        let target_signatures = self.get_signatures_of_type(target, SignatureKind::Call);
        if target_signatures.is_empty() {
            return false;
        }
        let return_expression = Arc::clone(&data.body);
        let source_return = match self.get_return_type_of_signature(&source_sig) {
            Some(t) => t,
            None => return false,
        };
        let target_returns: Vec<Arc<Type>> = target_signatures
            .iter()
            .filter_map(|s| self.get_return_type_of_signature(s))
            .collect();
        let target_return = self.get_union_type(target_returns);
        if self.is_type_related_to(&source_return, &target_return, relation) {
            return false;
        }
        if self.elaborate_error(
            &return_expression,
            &source_return,
            &target_return,
            relation,
            out.as_deref_mut(),
        ) {
            return true;
        }
        match out.as_deref_mut() {
            Some(o) => {
                self.check_type_related_to_and_optionally_elaborate(
                    &source_return,
                    &target_return,
                    relation,
                    Some(&return_expression),
                    None,
                    None,
                    Some(o),
                );
            }
            None => {
                self.check_type_related_to_and_optionally_elaborate(
                    &source_return,
                    &target_return,
                    relation,
                    Some(&return_expression),
                    None,
                    None,
                    None,
                );
            }
        }
        true
    }

    pub(crate) fn elaborate_object_literal(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        if target.flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never
                | TypeFlags::Enum
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral,
        ) {
            return false;
        }
        let properties = match &node.data {
            tsox_frontend::ast::NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => return false,
        };
        let mut reported = false;
        for prop in properties.iter() {
            if prop.kind == tsox_frontend::ast::SyntaxKind::SpreadAssignment {
                continue;
            }
            let (name_node, initializer): (
                &Arc<tsox_frontend::ast::Node>,
                Option<Arc<tsox_frontend::ast::Node>>,
            ) = match &prop.data {
                tsox_frontend::ast::NodeData::PropertyAssignment(d) => {
                    (&d.name, Some(Arc::clone(&d.initializer)))
                }
                tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => (&d.name, None),
                _ => continue,
            };
            let name = self.get_property_name_from_node(name_node);
            if name.is_empty() {
                continue;
            }
            // Go getBestMatchIndexedAccessTypeOrUndefined：联合目标直接取
            // 属性失败时，按最佳匹配成分取属性
            let (target_prop_type, prop_owner) = match self.get_type_of_property_of_type(target, &name)
            {
                Some(t) => (t, Arc::clone(target)),
                None => {
                    if target.flags.contains(TypeFlags::Union) {
                        match self.get_best_matching_type_for_error(source, target) {
                            Some(best) => match self.get_type_of_property_of_type(&best, &name) {
                                Some(t) => (t, best),
                                None => continue,
                            },
                            None => continue,
                        }
                    } else {
                        continue;
                    }
                }
            };
            // Go getIndexedAccessType：可选属性取声明型（不含 undefined），
            // 此处属性查询合成了 `| undefined`，按声明型剥除
            let target_prop_type = match self.get_property_of_type(&prop_owner, &name) {
                Some(sym)
                    if sym.flags.contains(SymbolFlags::Optional)
                        && target_prop_type.flags.contains(TypeFlags::Union) =>
                {
                    self.remove_undefined_from_union(&target_prop_type)
                }
                _ => target_prop_type,
            };
            let Some(source_prop_type) = self.get_type_of_property_of_type(source, &name) else {
                continue;
            };
            if self.is_type_related_to(&source_prop_type, &target_prop_type, relation) {
                continue;
            }
            if let Some(init) = initializer
                && self.elaborate_error(
                    &init,
                    &source_prop_type,
                    &target_prop_type,
                    relation,
                    out.as_deref_mut(),
                )
            {
                reported = true;
                continue;
            }

            match out.as_deref_mut() {
                Some(o) => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        Some(o),
                    );
                }
                None => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        None,
                    );
                }
            }
            reported = true;
        }
        reported
    }
}
