#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_const_property_assignment(&mut self, node: &Arc<Node>) {
        let (obj_expr, name, name_loc) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name, data.name.loc)
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                let arg = &data.argument_expression;
                if arg.kind != SyntaxKind::StringLiteral {
                    return;
                }
                (&data.expression, arg, arg.loc)
            }
            _ => return,
        };
        if obj_expr.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(sym) = self.resolve_identifier(obj_expr) else {
            return;
        };
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return;
        }
        let name_text = name.text();
        let member = base
            .exports
            .get(name_text)
            .or_else(|| base.members.get(name_text))
            .cloned()
            .or_else(|| {
                base.declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .find_map(|d| {
                        self.program
                            .symbol_map()
                            .locals
                            .get(&d.id())
                            .and_then(|l| l.get(name_text).cloned())
                    })
            });
        if member.is_some_and(|m| self.symbol_is_const_variable(&m)) {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                vec![name_text.to_string()],
            ));
        }
    }
}
