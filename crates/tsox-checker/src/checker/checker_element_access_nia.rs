use std::sync::Arc;

use tsox_frontend::ast::{Diagnostic, Node};

use crate::checker::checker::*;

impl Checker {
    // Go checkElementAccessExpression isForInVariableForNumericPropertyNames：
    // for-in 变量在数字名对象上作索引按 number 解析
    pub(crate) fn effective_index_arg_type(&mut self, arg_expr: &Arc<Node>) -> Arc<Type> {
        let arg_type = self.get_type_of_node(arg_expr);
        if arg_type.flags.contains(TypeFlags::String)
            && self.for_in_variable_of_numeric_names_object(arg_expr)
        {
            return self.number_type();
        }
        arg_type
    }

    fn for_in_variable_of_numeric_names_object(&mut self, arg_expr: &Arc<Node>) -> bool {
        if arg_expr.kind != SyntaxKind::Identifier {
            return false;
        }
        let Some(sym) = self.resolve_identifier(arg_expr) else {
            return false;
        };
        sym.declarations.iter().any(|d| {
            let Some(list) = d.parent() else { return false };
            if list.kind != SyntaxKind::VariableDeclarationList {
                return false;
            }
            let Some(stmt) = list.parent() else { return false };
            if stmt.kind != SyntaxKind::ForInStatement {
                return false;
            }
            let Some(expr) = stmt.expression() else { return false };
            let obj_type = self.get_type_of_node(expr);
            self.is_array_type(&obj_type)
                || obj_type.as_structured().is_some_and(|s| {
                    s.index_infos.len() == 1
                        && s.index_infos[0]
                            .key_type
                            .as_ref()
                            .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                })
        })
    }

    // Go getPropertyTypeForIndexType 尾段：无成员且无适用索引签名时的 nia 报告
    pub(crate) fn report_element_access_implicit_any(
        &mut self,
        node: &Arc<Node>,
        obj_type: &Arc<Type>,
        arg_expr: &Arc<Node>,
        arg_type: &Arc<Type>,
    ) {
        if !self.no_implicit_any
            || obj_type
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never)
            || crate::checker::utilities::is_type_error(obj_type)
        {
            return;
        }
        if !arg_type.flags.intersects(
            TypeFlags::Any
                | TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::StringMapping
                | TypeFlags::Number
                | TypeFlags::NumberLiteral
                | TypeFlags::ESSymbol
                | TypeFlags::EnumLiteral,
        ) {
            return;
        }
        let literal_name = self.literal_element_access_name(arg_expr);
        let is_object_literal = obj_type
            .object_flags
            .contains(crate::checker::types::ObjectFlags::ObjectLiteral);
        if is_object_literal {
            if let Some(name) = literal_name {
                let args = vec![name, self.type_to_string(obj_type)];
                self.emit_index_diagnostic(
                    node.loc,
                    tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                    args,
                );
                return;
            }
            if arg_type.flags.intersects(TypeFlags::String | TypeFlags::Number) {
                return;
            }
        }
        if self.type_has_number_index(obj_type) {
            self.emit_index_diagnostic(
                arg_expr.loc,
                tsox_core::diagnostics::messages_generated::
                    ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_INDEX_EXPRESSION_IS_NOT_OF_TYPE_NUMBER,
                vec![],
            );
            return;
        }
        let object_display = self.type_to_string(obj_type);
        let chain = self.index_diagnostic_chain(arg_type, &object_display);
        let index_display = self.type_to_string(arg_type);
        let mut diag = Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            tsox_core::diagnostics::messages_generated::
                ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_EXPRESSION_OF_TYPE_0_CAN_T_BE_USED_TO_INDEX_TYPE_1,
            vec![index_display, object_display],
        );
        if let Some(chain) = chain {
            diag.message_chain.push(chain);
        }
        self.diagnostics.add(diag);
    }

    fn emit_index_diagnostic(
        &mut self,
        loc: tsox_core::core::text::TextRange,
        message: tsox_core::diagnostics::Message,
        args: Vec<String>,
    ) {
        self.diagnostics
            .add(Diagnostic::new(self.current_file.clone(), loc, message, args));
    }

    fn index_diagnostic_chain(
        &mut self,
        arg_type: &Arc<Type>,
        object_display: &str,
    ) -> Option<Diagnostic> {
        let literal_value = |t: &Arc<Type>| -> Option<String> {
            if let crate::checker::types::TypeData::Literal(l) = &t.data {
                return Some(match &l.value {
                    crate::checker::types::LiteralValue::String(s) => s.clone(),
                    crate::checker::types::LiteralValue::Number(n) => n.to_string(),
                    _ => return None,
                });
            }
            None
        };
        let chain2339 = |name: String, self_: &Checker| {
            Some(Diagnostic::new(
                self_.current_file.clone(),
                tsox_core::core::text::TextRange::default(),
                tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![name, object_display.to_string()],
            ))
        };
        if arg_type.flags.intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral) {
            literal_value(arg_type).and_then(|v| chain2339(v, self))
        } else if arg_type.flags.contains(TypeFlags::EnumLiteral) {
            let name = arg_type
                .symbol
                .as_ref()
                .map(|s| format!("[{}]", s.name))
                .unwrap_or_else(|| format!("[{}]", self.type_to_string(arg_type)));
            chain2339(name, self)
        } else if arg_type.flags.intersects(TypeFlags::String | TypeFlags::Number) {
            Some(Diagnostic::new(
                self.current_file.clone(),
                tsox_core::core::text::TextRange::default(),
                tsox_core::diagnostics::messages_generated::
                    NO_INDEX_SIGNATURE_WITH_A_PARAMETER_OF_TYPE_0_WAS_FOUND_ON_TYPE_1,
                vec![self.type_to_string(arg_type), object_display.to_string()],
            ))
        } else {
            None
        }
    }

    pub(crate) fn type_has_number_index(&self, obj_type: &Arc<Type>) -> bool {
        self.is_array_type(obj_type)
            || obj_type.as_structured().is_some_and(|s| {
                s.index_infos.iter().any(|info| {
                    info.key_type
                        .as_ref()
                        .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                })
            })
            || obj_type
                .symbol
                .as_ref()
                .is_some_and(|s| s.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM))
    }

    pub(crate) fn literal_element_access_name(&self, arg: &Arc<Node>) -> Option<String> {
        match &arg.data {
            tsox_frontend::ast::NodeData::StringLiteral(data) => Some(data.text.clone()),
            tsox_frontend::ast::NodeData::NumericLiteral(data) => Some(data.text.clone()),
            _ => None,
        }
    }
}
