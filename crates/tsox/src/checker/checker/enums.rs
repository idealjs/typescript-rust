use std::sync::Arc;

use crate::ast::{ModifierFlags, Node, NodeData, Symbol, SyntaxKind};
use crate::evaluator::{EvalResult, EvalValue};
use crate::jsnum;

use super::*;

impl Checker {
    pub(crate) fn check_enum_member(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::EnumMember(data) = &node.data {
            if let Some(init) = &data.initializer {
                self.check_expression(init);

                let ambient = node
                    .parent
                    .as_ref()
                    .is_some_and(|p| p.has_syntactic_modifier(ModifierFlags::Ambient))
                    || self.ambient_context_depth > 0
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if ambient && !Self::is_constant_enum_initializer(init) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        init.loc,
                        crate::diagnostics::messages_generated::
                            IN_AMBIENT_ENUM_DECLARATIONS_MEMBER_INITIALIZER_MUST_BE_CONSTANT_EXPRESSION,
                        vec![],
                    ));
                }
            }
        }
    }

    fn is_constant_enum_initializer(init: &Arc<Node>) -> bool {
        match &init.data {
            crate::ast::NodeData::NumericLiteral(_)
            | crate::ast::NodeData::StringLiteral(_)
            | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_) => true,
            crate::ast::NodeData::Identifier(_) => true,
            crate::ast::NodeData::PrefixUnaryExpression(u) => {
                matches!(
                    u.operator,
                    SyntaxKind::PlusToken | SyntaxKind::MinusToken | SyntaxKind::TildeToken
                ) && Self::is_constant_enum_initializer(&u.operand)
            }
            crate::ast::NodeData::BinaryExpression(b) => {
                matches!(
                    b.operator_token.kind,
                    SyntaxKind::PlusToken
                        | SyntaxKind::MinusToken
                        | SyntaxKind::AsteriskToken
                        | SyntaxKind::SlashToken
                        | SyntaxKind::PercentToken
                        | SyntaxKind::LessThanLessThanToken
                        | SyntaxKind::GreaterThanGreaterThanToken
                        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
                        | SyntaxKind::AmpersandToken
                        | SyntaxKind::BarToken
                        | SyntaxKind::CaretToken
                ) && Self::is_constant_enum_initializer(&b.left)
                    && Self::is_constant_enum_initializer(&b.right)
            }
            crate::ast::NodeData::ParenthesizedExpression(p) => {
                Self::is_constant_enum_initializer(&p.expression)
            }
            _ => false,
        }
    }

    pub fn get_declaration_of_kind(
        &self,
        symbol: &Arc<Symbol>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        symbol.declarations.iter().find(|d| d.kind == kind).cloned()
    }

    pub fn get_enum_member_value(&mut self, node: &Arc<Node>) -> EvalResult {
        if let Some(parent) = node.parent.as_ref() {
            self.compute_enum_member_values(parent);
        }
        self.enum_member_links
            .get(node)
            .map(|l| l.value.clone())
            .unwrap_or_else(EvalResult::none)
    }

    fn compute_enum_member_values(&mut self, node: &Arc<Node>) {
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
        _previous: Option<&Arc<Node>>,
    ) -> EvalResult {
        let has_initializer =
            matches!(&member.data, NodeData::EnumMember(d) if d.initializer.is_some());
        if has_initializer {
            return self.compute_constant_enum_member_value(member);
        }
        match auto_value {
            Some(v) => EvalResult::new(
                Some(EvalValue::Number(jsnum::Number(v))),
                false,
                false,
                false,
            ),
            None => EvalResult::none(),
        }
    }

    fn compute_constant_enum_member_value(&mut self, member: &Arc<Node>) -> EvalResult {
        let initializer = match &member.data {
            NodeData::EnumMember(d) => match &d.initializer {
                Some(init) => Arc::clone(init),
                None => return EvalResult::none(),
            },
            _ => return EvalResult::none(),
        };
        crate::evaluator::evaluate_expression(&initializer, Some(member), noop_entity_fn)
    }
}
