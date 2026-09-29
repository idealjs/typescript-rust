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
        let prop_name = self.property_name_from_index(arg_type);
        let has_prop_name = prop_name.is_some();
        let is_object_literal = obj_type
            .object_flags
            .contains(crate::checker::types::ObjectFlags::ObjectLiteral);
        if is_object_literal {
            if let Some(name) = prop_name.clone() {
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
        let apparent = self.primitive_apparent_object_type(obj_type);
        let object_display = self.type_to_string(&apparent);
        let is_global_this = apparent.symbol.as_ref().is_some_and(|s| {
            self.global_this_symbol
                .as_ref()
                .is_some_and(|g| Arc::ptr_eq(g, s))
        });
        if is_global_this
            && has_prop_name
            && self
                .globals
                .get(prop_name.as_deref().unwrap())
                .is_some_and(|sym| {
                    sym.flags.intersects(
                        tsox_frontend::ast::SymbolFlags::BlockScopedVariable
                            | tsox_frontend::ast::SymbolFlags::Class
                            | tsox_frontend::ast::SymbolFlags::ENUM,
                    )
                })
        {
            self.emit_index_diagnostic(
                node.loc,
                tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![prop_name.unwrap(), object_display],
            );
            return;
        }
        if has_prop_name
            && self.type_has_static_property(prop_name.as_deref().unwrap(), &apparent)
        {
            let arg_text = self.node_source_text(arg_expr).unwrap_or_default();
            self.emit_index_diagnostic(
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_TO_ACCESS_THE_STATIC_MEMBER_2_INSTEAD,
                vec![
                    prop_name.clone().unwrap(),
                    object_display.clone(),
                    format!("{object_display}[{arg_text}]"),
                ],
            );
            return;
        }
        if self.type_has_number_index(&apparent) {
            self.emit_index_diagnostic(
                arg_expr.loc,
                tsox_core::diagnostics::messages_generated::
                    ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_INDEX_EXPRESSION_IS_NOT_OF_TYPE_NUMBER,
                vec![],
            );
            return;
        }
        if has_prop_name {
            let name = prop_name.as_deref().unwrap();
            if let Some(sugg) = self.suggest_property_spelling(name, &apparent) {
                self.emit_index_diagnostic(
                    arg_expr.loc,
                    tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2,
                    vec![name.to_string(), object_display.clone(), sugg],
                );
                return;
            }
        }
        if let Some(sugg) = self.suggest_index_signature_call(&apparent, node, arg_type) {
            self.emit_index_diagnostic(
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_TYPE_0_HAS_NO_INDEX_SIGNATURE_DID_YOU_MEAN_TO_CALL_1,
                vec![object_display, sugg],
            );
            return;
        }
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

    // Go getReducedApparentType 的原始类型近似：primitive 取对应全局接口声明型
    pub(crate) fn primitive_apparent_object_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let Some(interface_name) = self.primitive_interface_name(t) else {
            return t.clone();
        };
        let Some(sym) = self.globals.get(interface_name).cloned() else {
            return t.clone();
        };
        self.type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .unwrap_or_else(|| self.resolve_interface_type(&sym, None))
    }

    pub(crate) fn primitive_interface_index_value(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let apparent = self.primitive_apparent_object_type(t);
        let structured = apparent.as_structured()?;
        structured
            .index_infos
            .iter()
            .find_map(|info| info.value_type.clone())
    }

    // Go getSuggestionForNonexistentProperty
    fn suggest_property_spelling(&self, name: &str, t: &Arc<Type>) -> Option<String> {
        let st = t.as_structured()?;
        let rune_len = name.chars().count();
        let maximum_length_difference = 2.max((rune_len as f64 * 0.34) as usize);
        let mut best_distance = (rune_len as f64 * 0.4).floor() + 0.9;
        let mut best: Option<String> = None;
        let mut members: Vec<&String> = st.members.entries.keys().collect();
        members.sort();
        for cand in members {
            let cand = cand.as_str();
            if cand.is_empty()
                || cand.starts_with('"')
                || cand.starts_with('\'')
                || cand.starts_with('`')
                || cand.starts_with('\u{FE}')
            {
                continue;
            }
            let cand_len = cand.chars().count();
            if cand_len < 3 && !cand.eq_ignore_ascii_case(name) {
                continue;
            }
            if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                continue;
            }
            if cand == name {
                continue;
            }
            let Some(d) =
                crate::checker::checker_attach_explicit_type_arguments::levenshtein_with_max(
                    name,
                    cand,
                    best_distance,
                )
            else {
                continue;
            };
            if d < best_distance {
                best_distance = d;
                best = Some(cand.to_string());
            }
        }
        best
    }

    // Go getSuggestionForNonexistentIndexSignature
    fn suggest_index_signature_call(
        &mut self,
        apparent: &Arc<Type>,
        node: &Arc<Node>,
        keyed: &Arc<Type>,
    ) -> Option<String> {
        if !keyed.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::StringMapping
                | TypeFlags::Number
                | TypeFlags::NumberLiteral,
        ) {
            return None;
        }
        let suggested = if crate::checker::checker_object_literal_is_destructuring_target::is_assignment_target(node)
        {
            "set"
        } else {
            "get"
        };
        let prop = self.get_property_of_type(apparent, suggested)?;
        let t = self.get_type_of_symbol(&prop);
        let sig = self.get_single_call_signature(&t)?;
        if self.get_min_argument_count(&sig) < 1 {
            return None;
        }
        let param0 = self.get_type_at_position(&sig, 0);
        if !self.is_type_assignable_to(keyed, &param0) {
            return None;
        }
        let base = match &node.data {
            tsox_frontend::ast::NodeData::ElementAccessExpression(d) => {
                self.try_reference_to_string(&d.expression)
            }
            _ => String::new(),
        };
        Some(if base.is_empty() {
            suggested.to_string()
        } else {
            format!("{base}.{suggested}")
        })
    }

    // Go tryGetPropertyAccessOrIdentifierToString
    fn try_reference_to_string(&self, e: &Arc<Node>) -> String {
        match e.kind {
            SyntaxKind::Identifier => e.text().to_string(),
            SyntaxKind::PropertyAccessExpression => {
                if let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) = &e.data {
                    let base = self.try_reference_to_string(&d.expression);
                    if !base.is_empty() {
                        return format!("{base}.{}", d.name.text());
                    }
                }
                String::new()
            }
            SyntaxKind::ElementAccessExpression => {
                if let tsox_frontend::ast::NodeData::ElementAccessExpression(d) = &e.data {
                    let base = self.try_reference_to_string(&d.expression);
                    let arg = &d.argument_expression;
                    if !base.is_empty()
                        && matches!(
                            arg.kind,
                            SyntaxKind::Identifier
                                | SyntaxKind::StringLiteral
                                | SyntaxKind::NumericLiteral
                                | SyntaxKind::PrivateIdentifier
                        )
                    {
                        return format!("{base}.{}", arg.text());
                    }
                }
                String::new()
            }
            _ => String::new(),
        }
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
