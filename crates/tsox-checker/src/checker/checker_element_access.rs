use std::sync::Arc;

use tsox_frontend::ast::Node;

use crate::checker::checker::*;

impl Checker {
    pub(crate) fn get_type_of_element_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, arg_expr, question_dot) = match &node.data {
            tsox_frontend::ast::NodeData::ElementAccessExpression(data) => (
                &data.expression,
                &data.argument_expression,
                data.question_dot_token.is_some(),
            ),
            _ => return self.get_any_type(),
        };

        let obj_precheck = self.get_type_of_node(obj_expr);
        let obj_checked = if question_dot {
            obj_precheck
        } else {
            self.check_non_null_type(&obj_precheck, obj_expr)
        };
        if crate::checker::utilities::is_type_error(&obj_checked) {
            return obj_checked;
        }
        {
            let skip_index_check = obj_checked.flags.intersects(TypeFlags::Any);
            let arg_type = self.get_type_of_node(arg_expr);

            let is_type_param_or_union_of = skip_index_check
                || arg_type.is_type_parameter()
                || (arg_type.is_union()
                    && arg_type
                        .types()
                        .is_some_and(|ts| ts.iter().all(|t| t.is_type_parameter())));
            if !arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                && !is_type_param_or_union_of
            {
                let parts: Vec<Arc<Type>> = if arg_type.is_union() {
                    arg_type.types().map(|ts| ts.to_vec()).unwrap_or_default()
                } else {
                    vec![Arc::clone(&arg_type)]
                };
                for p in parts {
                    if p.flags.intersects(
                        TypeFlags::Any
                            | TypeFlags::Never
                            | TypeFlags::String
                            | TypeFlags::StringLiteral
                            | TypeFlags::Number
                            | TypeFlags::NumberLiteral
                            | TypeFlags::ESSymbol
                            | TypeFlags::UniqueESSymbol
                            | TypeFlags::EnumLiteral
                            | TypeFlags::StringMapping,
                    ) {
                        continue;
                    }
                    let type_str = self.type_to_string(&p);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        arg_expr.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
            }
        }
        let obj_type = obj_checked;
        let effective_arg = self.effective_index_arg_type(arg_expr);

        if obj_type.flags.contains(TypeFlags::Union)
            && let Some(members) = obj_type.types().map(|ts| ts.to_vec())
        {
            let dynamic = effective_arg
                .flags
                .intersects(TypeFlags::String | TypeFlags::Number)
                && !effective_arg.flags.intersects(
                    TypeFlags::Any | TypeFlags::StringLiteral | TypeFlags::NumberLiteral,
                );
            let non_any: Vec<&Arc<Type>> = members
                .iter()
                .filter(|m| {
                    !m.flags.contains(TypeFlags::Any)
                        && !(question_dot
                            && m.flags.intersects(TypeFlags::Null | TypeFlags::Undefined))
                })
                .collect();
            if dynamic
                && !non_any.is_empty()
                && !non_any.iter().all(|m| {
                    self.member_allows_dynamic_index(
                        m,
                        effective_arg.flags.contains(TypeFlags::String),
                    )
                })
            {
                self.report_element_access_implicit_any(node, &obj_type, arg_expr, &effective_arg);
                return self.get_any_type();
            }
            let literal_name = self.literal_element_access_name(arg_expr);
            let mut elem_types: Vec<Arc<Type>> = Vec::new();
            for m in &members {
                if m.flags.contains(TypeFlags::Any) {
                    continue;
                }
                if question_dot && m.flags.intersects(TypeFlags::Null | TypeFlags::Undefined) {
                    continue;
                }
                // Go createUnionOrIntersectionProperty：联合成分缺该名属性且为
                // 对象字面量（无 spread）时贡献 undefined，不报 nia
                if let Some(name) = &literal_name
                    && m.object_flags.contains(crate::checker::types::ObjectFlags::ObjectLiteral)
                    && !m
                        .object_flags
                        .contains(crate::checker::types::ObjectFlags::ContainsSpread)
                    && self.get_property_of_type(m, name).is_none()
                {
                    elem_types.push(self.undefined_type());

                    continue;
                }
                let t = self.element_access_result_type(node, m, arg_expr, &effective_arg);
                if !t.flags.contains(TypeFlags::Any) {
                    elem_types.push(t);
                }
            }
            if !elem_types.is_empty() {
                return self.get_union_type(elem_types);
            }
            return self.get_any_type();
        }
        self.element_access_result_type(node, &obj_type, arg_expr, &effective_arg)
    }

    fn member_allows_dynamic_index(&self, m: &Arc<Type>, want_string: bool) -> bool {
        if m.flags.intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never) {
            return true;
        }
        let has = |string: bool| {
            m.as_structured().is_some_and(|s| {
                s.index_infos.iter().any(|info| {
                    info.key_type
                        .as_ref()
                        .is_some_and(|k| k.flags.contains(if string {
                            TypeFlags::String
                        } else {
                            TypeFlags::Number
                        }))
                })
            })
        };
        if want_string {
            has(true)
        } else {
            has(false) || self.is_array_type(m) || self.is_tuple_type(m) || has(true)
        }
    }

    fn element_access_result_type(
        &mut self,
        node: &Arc<Node>,
        obj_type: &Arc<Type>,
        arg_expr: &Arc<Node>,
        effective_arg: &Arc<Type>,
    ) -> Arc<Type> {
        if self.is_tuple_type(obj_type) {
            if let Some(index) = self.get_constant_numeric_value(arg_expr) {
                if let Some(t) = self.get_tuple_element_type(obj_type, index as usize) {
                    return t;
                }
            }

            return self.get_any_type();
        }

        if self.is_array_type(obj_type) {
            if !effective_arg.flags.intersects(
                TypeFlags::Number
                    | TypeFlags::NumberLiteral
                    | TypeFlags::Any
                    | TypeFlags::EnumLiteral,
            ) {
                self.report_element_access_implicit_any(node, obj_type, arg_expr, effective_arg);
            }
            return self.get_array_element_type(obj_type);
        }

        let prop_name = self
            .property_name_from_index(&effective_arg)
            .or_else(|| self.literal_element_access_name(arg_expr));
        if let Some(member_name) = prop_name {
            if let Some(sym) = self.get_property_of_type(obj_type, &member_name) {
                if let tsox_frontend::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    let self_access = self.is_self_type_access(&data.expression, obj_type);
                    self.mark_property_as_referenced_ex(&sym, Some(node), Some(self_access));
                }
                if let Some(substituted) = self.instantiate_array_member_type(obj_type, &sym) {
                    return self.flow_type_of_access_expression(node, Some(&sym), substituted);
                }
                let prop_type = self.get_type_of_symbol(&sym);
                return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
            }
        }

        if matches!(&obj_type.data, crate::checker::types::TypeData::Mapped(_)) {
            let mapped_result = self.get_indexed_access_type(obj_type, &effective_arg);
            if !mapped_result.flags.contains(TypeFlags::Any) {
                return self.flow_type_of_access_expression(node, None, mapped_result);
            }
        }

        if let Some(sym) = obj_type
            .symbol
            .as_ref()
            .filter(|s| s.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM))
            .map(Arc::clone)
        {
            let all_string_members = !sym.members.is_empty()
                && sym.members.entries.values().all(|m| {
                    m.declarations.iter().all(|d| {
                        matches!(
                            &d.data,
                            tsox_frontend::ast::NodeData::EnumMember(em)
                                if em.initializer.as_ref().is_some_and(|init| {
                                    init.kind == SyntaxKind::StringLiteral
                                })
                        )
                    })
                });
            let arg_is_number = matches!(arg_expr.kind, SyntaxKind::NumericLiteral)
                || effective_arg.flags.intersects(
                    TypeFlags::Number | TypeFlags::NumberLiteral | TypeFlags::EnumLiteral,
                );
            if arg_is_number && !all_string_members {
                let s = self.string_type();
                return self.flow_type_of_access_expression(node, None, s);
            }
        }

        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                if let Some(key_type) = &info.key_type {
                    if key_type.flags.contains(crate::checker::TypeFlags::String)
                        || key_type.flags.contains(crate::checker::TypeFlags::Number)
                    {
                        if let Some(val_type) = &info.value_type {
                            let val_type = Arc::clone(val_type);
                            return self.flow_type_of_access_expression(node, None, val_type);
                        }
                    }
                }
            }
        }
        if let Some(val_type) = self.primitive_interface_index_value(obj_type) {
            return self.flow_type_of_access_expression(node, None, val_type);
        }

        self.report_element_access_implicit_any(node, obj_type, arg_expr, effective_arg);
        self.get_any_type()
    }
}
