#![allow(unused_imports)]

use crate::checker::checker::*;

// Go computeEnumMemberValue 数值名检查：BigIntLiteral 名与
// isNumericLiteralName（jsnum.FromString(text).String() == text）名报 TS2452。
// 数值字面量文本按 Go 扫描器归一化语义处理（"1.0" 判为数值名）
impl Checker {
    pub(crate) fn check_enum_member_numeric_name(&mut self, member: &Arc<Node>) {
        let Some(name) = member.name() else {
            return;
        };
        let literal_text = |c: &Checker, n: &Arc<Node>| -> Option<String> {
            let raw = c.node_source_text(n)?;
            match n.kind {
                SyntaxKind::NumericLiteral => {
                    Some(tsox_core::jsnum::Number::from_string(&raw).to_string())
                }
                SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => {
                    Some(raw.trim_matches(|ch| ch == '"' || ch == '\'').to_string())
                }
                _ => None,
            }
        };
        let literal_name_node = |n: &Arc<Node>| -> Option<Arc<Node>> {
            match n.kind {
                SyntaxKind::ComputedPropertyName => match &n.data {
                    tsox_frontend::ast::NodeData::ComputedPropertyName(cd) => {
                        Some(Arc::clone(&cd.expression))
                    }
                    _ => None,
                },
                _ => Some(Arc::clone(n)),
            }
            .filter(|e| {
                matches!(
                    e.kind,
                    SyntaxKind::StringLiteral
                        | SyntaxKind::NumericLiteral
                        | SyntaxKind::NoSubstitutionTemplateLiteral
                )
            })
        };
        let is_numeric_name = name.kind == SyntaxKind::BigIntLiteral
            || literal_name_node(name)
                .and_then(|n| literal_text(self, &n))
                .is_some_and(|t| {
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
}
