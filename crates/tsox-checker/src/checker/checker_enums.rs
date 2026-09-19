use std::sync::Arc;

use tsox_frontend::ast::ModifierFlags;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::evaluator::EvalResult;
use tsox_frontend::evaluator::EvalValue;

use crate::checker::checker::*;

impl Checker {
    pub(crate) fn check_enum_member(&mut self, node: &Arc<Node>) {
        if let tsox_frontend::ast::NodeData::EnumMember(data) = &node.data {
            // Go checkEnumDeclaration → checkSourceElements：枚举成员计算名
            // 急切检查（表达式 TS2304/TS2464 + 非字面量 TS1164）
            let name = &data.name;
            if name.kind == SyntaxKind::ComputedPropertyName {
                if let NodeData::ComputedPropertyName(cd) = &name.data
                    && !tsox_frontend::ast::is_string_or_numeric_literal_like(&cd.expression)
                {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            COMPUTED_PROPERTY_NAMES_ARE_NOT_ALLOWED_IN_ENUMS,
                        vec![],
                    ));
                }
            }
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }
            // 值计算路径按 Go computeEnumMemberValue 附带全部成员级诊断
            // （TS1061/TS18056/TS2474/TS2476/TS18033/TS18055/const-enum NaN/Inf）
            self.get_enum_member_value(node);
        }
    }

    fn enum_member_diagnostics_context(&self, member: &Arc<Node>) -> (bool, bool) {
        let parent = member.parent();
        let is_const_enum = parent
            .as_ref()
            .is_some_and(|p| p.has_syntactic_modifier(ModifierFlags::Const));
        let ambient = parent
            .as_ref()
            .is_some_and(|p| p.has_syntactic_modifier(ModifierFlags::Ambient))
            || self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        (is_const_enum, ambient)
    }

    pub fn get_declaration_of_kind(
        &self,
        symbol: &Arc<Symbol>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        symbol.declarations.iter().find(|d| d.kind == kind).cloned()
    }

    pub fn get_enum_member_value(&mut self, node: &Arc<Node>) -> EvalResult {
        if let Some(parent) = node.parent().as_ref() {
            self.compute_enum_member_values(parent);
        }
        self.enum_member_links
            .get(node)
            .map(|l| l.value.clone())
            .unwrap_or_else(EvalResult::none)
    }

    pub(crate) fn compute_enum_member_values(&mut self, node: &Arc<Node>) {
        let already = self
            .node_links
            .get(node)
            .map(|l| l.flags.contains(NodeCheckFlags::EnumValuesComputed))
            .unwrap_or(false);
        if already {
            return;
        }
        self.node_links.get_or_default(node).flags |= NodeCheckFlags::EnumValuesComputed;

        let members: Vec<Arc<Node>> = match &node.data {
            NodeData::EnumDeclaration(data) => data.members.iter().cloned().collect(),
            _ => return,
        };

        let mut auto_value: Option<f64> = Some(0.0);
        let mut previous: Option<Arc<Node>> = None;
        for member in &members {
            let result = self.compute_enum_member_value(member, auto_value, previous.as_ref());
            self.enum_member_links.get_or_default(member).value = result.clone();
            if let Some(EvalValue::Number(n)) = &result.value {
                auto_value = Some(n.0 + 1.0);
            } else {
                auto_value = None;
            }
            previous = Some(Arc::clone(member));
        }
    }

    fn compute_enum_member_value(
        &mut self,
        member: &Arc<Node>,
        auto_value: Option<f64>,
        previous: Option<&Arc<Node>>,
    ) -> EvalResult {
        // Go computeEnumMemberValue：非字面量计算名报 TS1164
        if let Some(name) = member.name()
            && name.kind == SyntaxKind::ComputedPropertyName
            && let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &name.data
            && !tsox_frontend::ast::is_string_or_numeric_literal_like(&cd.expression)
        {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                tsox_core::diagnostics::messages_generated::
                    COMPUTED_PROPERTY_NAMES_ARE_NOT_ALLOWED_IN_ENUMS,
                vec![],
            ));
        }
        if let Some(name) = member.name() {
            let numeric_name_text = |n: &Arc<Node>| -> Option<String> {
                let normalized_numeric = |raw: String| {
                    tsox_core::jsnum::Number::from_string(&raw).to_string()
                };
                match n.kind {
                    SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => self
                        .node_source_text(n)
                        .map(|s| s.trim_matches(|c| c == '"' || c == '\'').to_string()),
                    SyntaxKind::NumericLiteral => {
                        self.node_source_text(n).map(normalized_numeric)
                    }
                    SyntaxKind::ComputedPropertyName => {
                        let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &n.data
                        else {
                            return None;
                        };
                        match cd.expression.kind {
                            SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => self
                                .node_source_text(&cd.expression)
                                .map(|s| s.trim_matches(|c| c == '"' || c == '\'').to_string()),
                            SyntaxKind::NumericLiteral => {
                                self.node_source_text(&cd.expression).map(normalized_numeric)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            };
            let is_numeric_name = name.kind == SyntaxKind::BigIntLiteral
                || numeric_name_text(name).is_some_and(|t| {
                    !t.is_empty()
                        && t != "Infinity"
                        && t != "-Infinity"
                        && t != "NaN"
                        && tsox_core::jsnum::Number::from_string(&t).to_string() == t
                });
            if is_numeric_name {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        AN_ENUM_MEMBER_CANNOT_HAVE_A_NUMERIC_NAME,
                    vec![],
                ));
            }
        }
        let has_initializer =
            matches!(&member.data, NodeData::EnumMember(d) if d.initializer.is_some());
        if has_initializer {
            return self.compute_constant_enum_member_value(member);
        }
        let name_loc = member.name().map(|n| n.loc).unwrap_or(member.loc);
        let Some(v) = auto_value else {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                name_loc,
                tsox_core::diagnostics::messages_generated::ENUM_MEMBER_MUST_HAVE_INITIALIZER,
                vec![],
            ));
            return EvalResult::none();
        };
        if self.compiler_options.isolated_modules.is_true()
            && let Some(prev) = previous
            && matches!(&prev.data, NodeData::EnumMember(d) if d.initializer.is_some())
        {
            let prev_result = self.enum_member_links.get(prev).map(|l| l.value.clone());
            let prev_ok = prev_result.as_ref().is_some_and(|r| {
                matches!(&r.value, Some(EvalValue::Number(_))) && !r.resolved_other_files
            });
            if prev_result.is_some() && !prev_ok {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_loc,
                    tsox_core::diagnostics::messages_generated::
                        ENUM_MEMBER_FOLLOWING_A_NON_LITERAL_NUMERIC_MEMBER_MUST_HAVE_AN_INITIALIZER_WHEN_ISOLATEDMODULES_IS_ENABLED,
                    vec![],
                ));
            }
        }
        EvalResult::new(
            Some(EvalValue::Number(tsox_core::jsnum::Number(v))),
            false,
            false,
            false,
        )
    }

    fn compute_constant_enum_member_value(&mut self, member: &Arc<Node>) -> EvalResult {
        let initializer = match &member.data {
            NodeData::EnumMember(d) => match &d.initializer {
                Some(init) => Arc::clone(init),
                None => return EvalResult::none(),
            },
            _ => return EvalResult::none(),
        };
        let result = tsox_frontend::evaluator::evaluate_expression(
            &initializer,
            Some(member),
            noop_entity_fn,
        );
        let (is_const_enum, ambient) = self.enum_member_diagnostics_context(member);
        match &result.value {
            Some(EvalValue::Number(n)) => {
                if is_const_enum && (n.0.is_nan() || n.0.is_infinite()) {
                    let message = if n.0.is_nan() {
                        tsox_core::diagnostics::messages_generated::
                            X_CONST_ENUM_MEMBER_INITIALIZER_WAS_EVALUATED_TO_DISALLOWED_VALUE_NAN
                    } else {
                        tsox_core::diagnostics::messages_generated::
                            X_CONST_ENUM_MEMBER_INITIALIZER_WAS_EVALUATED_TO_A_NON_FINITE_VALUE
                    };
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        initializer.loc,
                        message,
                        vec![],
                    ));
                }
            }
            Some(EvalValue::String(_)) => {
                if self.compiler_options.isolated_modules.is_true() && !result.is_syntactically_string
                {
                    let enum_name = member
                        .parent()
                        .and_then(|p| p.name().map(|n| n.text().to_string()));
                    let member_name = format!(
                        "{}.{}",
                        enum_name.unwrap_or_default(),
                        member.name().map(|n| n.text().to_string()).unwrap_or_default()
                    );
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        initializer.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_0_HAS_A_STRING_TYPE_BUT_MUST_HAVE_SYNTACTICALLY_RECOGNIZABLE_STRING_SYNTAX_WHEN_ISOLATEDMODULES_IS_ENABLED,
                        vec![member_name],
                    ));
                }
            }
            _ => {
                if is_const_enum {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        initializer.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_CONST_ENUM_MEMBER_INITIALIZERS_MUST_BE_CONSTANT_EXPRESSIONS,
                        vec![],
                    ));
                } else if ambient {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        initializer.loc,
                        tsox_core::diagnostics::messages_generated::
                            IN_AMBIENT_ENUM_DECLARATIONS_MEMBER_INITIALIZER_MUST_BE_CONSTANT_EXPRESSION,
                        vec![],
                    ));
                } else {
                    let init_type = self.get_type_of_node(&initializer);
                    let number = self.number_type();
                    self.check_type_related_to_and_optionally_elaborate(
                        &init_type,
                        &number,
                        crate::checker::relater::RelationKind::Assignable,
                        Some(&initializer),
                        None,
                        Some(
                            &tsox_core::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_AS_REQUIRED_FOR_COMPUTED_ENUM_MEMBER_VALUES,
                        ),
                        None,
                    );
                }
            }
        }
        result
    }
}
