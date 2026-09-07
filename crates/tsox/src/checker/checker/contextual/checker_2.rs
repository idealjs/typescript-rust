#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_valid_const_assertion_argument(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::TemplateExpression => true,
            SyntaxKind::ParenthesizedExpression => match &node.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    self.is_valid_const_assertion_argument(&p.expression)
                }
                _ => false,
            },
            SyntaxKind::PrefixUnaryExpression => match &node.data {
                crate::ast::NodeData::PrefixUnaryExpression(p) => {
                    let arg_kind = p.operand.kind;
                    (p.operator == SyntaxKind::MinusToken
                        && matches!(
                            arg_kind,
                            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                        ))
                        || (p.operator == SyntaxKind::PlusToken
                            && arg_kind == SyntaxKind::NumericLiteral)
                }
                _ => false,
            },
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                let (obj, _name) = match &node.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (Some(d.expression.clone()), d.name.text().to_string())
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (Some(d.expression.clone()), arg.text().to_string())
                        } else {
                            (None, String::new())
                        }
                    }
                    _ => (None, String::new()),
                };
                match obj {
                    Some(obj) if obj.kind == SyntaxKind::Identifier => self
                        .resolve_qualified_symbol(node)
                        .or_else(|| self.resolve_identifier(&obj))
                        .map(|sym| {
                            sym.flags
                                .intersects(SymbolFlags::ENUM | SymbolFlags::EnumMember)
                        })
                        .unwrap_or(false),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(crate) fn is_const_type_node(type_node: &Arc<Node>) -> bool {
        type_node.kind == SyntaxKind::ConstKeyword
    }

    pub(crate) fn check_delete_operand(&mut self, operand: &Arc<Node>) {
        let mut target = operand;
        while target.kind == SyntaxKind::ParenthesizedExpression {
            let inner = match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                _ => break,
            };
            target = inner;
        }
        match target.kind {
            SyntaxKind::Identifier => {
                let strict = self
                    .program
                    .options()
                    .get_strict_option_value(self.program.options().always_strict)
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                if strict {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            X_DELETE_CANNOT_BE_CALLED_ON_AN_IDENTIFIER_IN_STRICT_MODE,
                        vec![],
                    ));
                }
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    target.loc,
                    crate::diagnostics::messages_generated::
                        THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_A_PROPERTY_REFERENCE,
                    vec![],
                ));
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                let (obj_expr, name, _name_loc) = match &target.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (&d.expression, d.name.text().to_string(), d.name.loc)
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (&d.expression, arg.text().to_string(), arg.loc)
                        } else {
                            return;
                        }
                    }
                    _ => return,
                };

                if matches!(&target.data, crate::ast::NodeData::PropertyAccessExpression(d) if d.name.kind == SyntaxKind::PrivateIdentifier)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_PRIVATE_IDENTIFIER,
                        vec![],
                    ));
                    return;
                }

                let obj_type = self.get_type_of_node(obj_expr);
                if obj_type.flags.contains(TypeFlags::Any) {
                    return;
                }
                if self.is_property_readonly(&obj_type, &name) {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_READ_ONLY_PROPERTY,
                        vec![name],
                    ));
                    return;
                }

                if let Some(structured) = obj_type.as_structured() {
                    let readonly_index = structured.index_infos.iter().any(|info| {
                        info.is_readonly
                            && info
                                .key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                    });
                    if readonly_index {
                        let type_name = self.type_to_string(&obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                            vec![type_name],
                        ));
                        return;
                    }
                }

                if self.strict_null_checks && self.has_property_of_type(&obj_type, &name) {
                    let prop = obj_type.as_structured().and_then(|s| {
                        s.properties
                            .iter()
                            .find(|p| p.name == name)
                            .map(|p| Arc::clone(p))
                    });
                    let deletable = prop.as_ref().is_some_and(|p| {
                        if p.flags.contains(SymbolFlags::Optional) {
                            return true;
                        }
                        let t = self.get_type_of_symbol(p);
                        t.flags.intersects(
                            TypeFlags::Undefined
                                | TypeFlags::Any
                                | TypeFlags::Unknown
                                | TypeFlags::Never,
                        ) || match &t.data {
                            crate::checker::TypeData::Union(u) => {
                                u.union_or_intersection.types.iter().any(|m| {
                                    m.flags.intersects(
                                        TypeFlags::Undefined
                                            | TypeFlags::Any
                                            | TypeFlags::Unknown
                                            | TypeFlags::Never,
                                    )
                                })
                            }
                            _ => false,
                        }
                    }) || obj_type.as_structured().is_some_and(|s| {
                        s.index_infos.iter().any(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                        })
                    });
                    if !deletable {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_OPTIONAL,
                            vec![],
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_const_assignment_target(&mut self, operand: &Arc<Node>) {
        let mut target = operand;
        loop {
            target = match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                crate::ast::NodeData::NonNullExpression(n) => &n.expression,
                _ => break,
            };
        }
        let operand = target;
        if operand.kind == SyntaxKind::PropertyAccessExpression
            || operand.kind == SyntaxKind::ElementAccessExpression
        {
            self.check_const_property_assignment(operand);
            return;
        }
        if operand.kind != SyntaxKind::Identifier {
            return;
        }
        if let Some(symbol) = self.resolve_identifier(operand)
            && self.symbol_is_const_variable(&symbol)
        {
            let name_text = operand.text();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                vec![name_text.to_string()],
            ));
        }
    }
}
