//! hover quickinfo 移植补充：对照 Go ls/hover.go + checker 对应分支，逐步承接
//! nodebuilder_checker_12 中补丁式实现之外的保真逻辑。

use std::sync::Arc;

use crate::checker::nodebuilder::*;
use tsox_frontend::ast::{Node, SyntaxKind};

impl Checker {
    /// hover 类型显示统一走这里的 flags（Go typeFormatFlags + MultilineObjectLiterals）
    pub fn hover_type_to_string(&mut self, t: &Arc<crate::checker::types::Type>) -> String {
        use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
        self.type_to_string_ex(
            t,
            TypeFormatFlags::MULTILINE_OBJECT_LITERALS
                .union(TypeFormatFlags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE),
        )
    }

    /// 签名返回段渲染：type predicate（`this is T` / `x is T` / `asserts …`）优先，
    /// 否则返回类型（对齐 Go signatureToString 的谓词输出）
    pub(crate) fn append_signature_return_parts(
        &mut self,
        parts: &mut Vec<SymbolDisplayPart>,
        sig: &Arc<crate::checker::types::Signature>,
    ) {
        if let Some(pred) = self.compute_type_predicate_of_signature(sig) {
            match pred.kind {
                crate::checker::types::TypePredicateKind::AssertsThis => {
                    push_space(parts, ": ");
                    push_keyword(parts, "asserts this");
                }
                crate::checker::types::TypePredicateKind::AssertsIdentifier => {
                    push_space(parts, ": ");
                    push_keyword(parts, "asserts ");
                    push_part(parts, &pred.parameter_name, DisplayPartKind::ParameterName);
                }
                crate::checker::types::TypePredicateKind::This => {
                    push_space(parts, ": ");
                    push_keyword(parts, "this");
                }
                crate::checker::types::TypePredicateKind::Identifier => {
                    push_space(parts, ": ");
                    push_part(parts, &pred.parameter_name, DisplayPartKind::ParameterName);
                }
            }
            if let Some(t) = pred.t {
                push_keyword(parts, " is ");
                parts.extend(self.type_to_display_parts(&t));
            }
            return;
        }
        let ret = self
            .get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type());
        push_space(parts, ": ");
        parts.extend(self.type_to_display_parts(&ret));
    }

    /// `x as const` 中的 const：Go 侧解析为 Transient|TypeAlias 名为 const 的符号，
    /// 显示 `type const = <断言表达式类型>`（checker.go getTypeFromTypeReference 特判）
    pub fn quick_info_parts(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> {
        if let Some(parts) = self.const_assertion_parts(node) {
            return parts;
        }
        Vec::new()
    }

    pub(crate) fn const_assertion_parts(&mut self, node: &Arc<Node>) -> Option<Vec<SymbolDisplayPart>> {
        if node.kind != SyntaxKind::ConstKeyword {
            return None;
        }
        let assertion = node.parent()?;
        if assertion.kind != SyntaxKind::AsExpression {
            return None;
        }
        let expr = match &assertion.data {
            NodeData::AsExpression(d) if d.type_node.kind == SyntaxKind::ConstKeyword => {
                Arc::clone(&d.expression)
            }
            _ => return None,
        };
        let t = self.get_type_of_node(&expr);
        let mut parts = Vec::new();
        push_keyword(&mut parts, "type");
        push_space(&mut parts, " ");
        push_part(&mut parts, "const", DisplayPartKind::InterfaceName);
        push_space(&mut parts, " = ");
        parts.extend(self.type_to_display_parts(&t));
        Some(parts)
    }
}
