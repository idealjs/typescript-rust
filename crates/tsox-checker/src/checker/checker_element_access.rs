use std::sync::Arc;

use tsox_frontend::ast::Node;

use crate::checker::checker::*;
use crate::checker::mig::wc3_2::is_const_enum_object_type;
use crate::checker::utilities_has_only_expression_initialization::get_assignment_target_kind;
use crate::checker::utilities_token_is_identifier_or_keyword::AssignmentKind;

impl Checker {
    pub(crate) fn get_type_of_element_access(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_element_access"); 
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

            // Go getIndexedAccessTypeOrUndefined（checker.go:28549-28551）：
            // string-index-only 对象的 string/number 型键先换 stringType，
            // shouldDefer 对 stringType 判 false 保持急解析
            let string_index_shortcut = self.is_string_index_signature_only_type_worker(&obj_checked)
                && !arg_type.flags.intersects(TypeFlags::Undefined | TypeFlags::Null)
                && self.is_type_assignable_to_kind(&arg_type, TypeFlags::String | TypeFlags::Number);
            // Go shouldDeferIndexedAccessType（checker.go:28994-28996）：索引型为
            // 泛型（isGenericIndexType，getGenericObjectFlags checker.go:26378
            // InstantiableNonPrimitive|Index 位）时高阶索引访问恒延迟
            if !string_index_shortcut
                && (arg_type
                    .flags
                    .intersects(
                        TypeFlags::Index
                            | TypeFlags::IndexedAccess
                            | TypeFlags::TemplateLiteral
                            | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE,
                    )
                    || matches!(&arg_type.data, TypeData::IndexedAccess(_)))
            {
                if obj_checked.flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
                    return obj_checked;
                }
                // Go checkElementAccessExpression（checker.go:8652-8653）读位
                // ExpressionPosition 经 getIndexedAccessTypeOrUndefined（同一
                // indexedAccessTypes 驻留表，保证与 compute/substitute 侧同实例）
                let indexed = self
                    .get_indexed_access_type_or_undefined(
                        &obj_checked,
                        &arg_type,
                        AccessFlags::ExpressionPosition,
                        Some(node.as_ref()),
                        None,
                    )
                    .unwrap_or_else(|| self.error_type());
                return self.flow_type_of_access_expression(node, None, indexed);
            }

            let is_type_param_or_union_of = skip_index_check
                || arg_type.is_type_parameter()
                || (arg_type.is_union()
                    && arg_type
                        .types()
                        .is_some_and(|ts| ts.iter().all(|t| t.is_type_parameter())));
            let mut invalid_index_type = false;
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
                    invalid_index_type = true;
                    let type_str = if arg_expr.kind == SyntaxKind::BigIntLiteral {
                        "bigint".to_string()
                    } else {
                        self.type_to_string(&p)
                    };
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        arg_expr.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
            }
            // Go checkElementAccessExpression（checker.go:8669）：
            // getIndexedAccessTypeOrUndefined 任一索引成分无解整体取 errorType
            if invalid_index_type {
                return self.error_type();
            }
        }
        let obj_type = obj_checked;
        let effective_arg = self.effective_index_arg_type(arg_expr);

        // Go checkElementAccessExpression（checker.go:8654-8657）：写目标的泛型对象类型
        //（非 this 型参数）置 NoIndexSignatures；getPropertyTypeForIndexType
        //（checker.go:28649-28715）：命名属性命中优先，仅当无命中且适用索引签名
        // 键型非 number 时只可读（TS2862），按 Go 返回错误类型收口阻断后续报错
        if get_assignment_target_kind(node) != AssignmentKind::None
            && self.is_generic_object_type(&obj_type)
            && !self.is_this_type_parameter(&obj_type)
        {
            let early_name = self
                .property_name_from_index(&effective_arg)
                .or_else(|| self.literal_element_access_name(arg_expr));
            let named_hit = early_name
                .as_ref()
                .is_some_and(|name| self.get_property_of_type(&obj_type, name).is_some());
            if !named_hit {
                let string_key = self.string_type();
                let index_info = self
                    .get_applicable_index_info(&obj_type, &effective_arg)
                    .or_else(|| self.get_index_info_of_type(&obj_type, &string_key));
                if let Some(info) = &index_info
                    && !info
                        .key_type
                        .as_ref()
                        .is_some_and(|k| Arc::ptr_eq(k, &self.number_type()))
                {
                    let arg = self.type_to_string(&obj_type);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_IS_GENERIC_AND_CAN_ONLY_BE_INDEXED_FOR_READING,
                        vec![arg],
                    ));
                    return self.error_type();
                }
            }
        }

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
                let t = self.element_access_result_type(node, m, arg_expr, &effective_arg, false);
                if !t.flags.contains(TypeFlags::Any) {
                    elem_types.push(t);
                }
            }
            if !elem_types.is_empty() {
                return self.get_union_type(elem_types);
            }
            return self.get_any_type();
        }
        self.element_access_result_type(node, &obj_type, arg_expr, &effective_arg, true)
    }

    fn index_access_with_no_unchecked_undefined(
        &mut self,
        t: Arc<Type>,
        from_index_signature: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("index_access_with_no_unchecked_undefined"); 
        if self.no_unchecked_indexed_access && from_index_signature {
            return self.get_union_type(vec![t, self.undefined_type()]);
        }
        t
    }

    fn member_allows_dynamic_index(&self, m: &Arc<Type>, want_string: bool) -> bool { ::tsox_core::fntrace::enter("member_allows_dynamic_index");         if m.flags.intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never) {
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
        report_nia: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("element_access_result_type"); 
        // Go getIndexedAccessTypeOrUndefined：命名属性访问（含知名符号
        // `[Symbol.iterator]`、字面量下标命中的成员）先于数组/元组下标捷径
        let early_prop_name = self
            .property_name_from_index(&effective_arg)
            .or_else(|| self.literal_element_access_name(arg_expr));
        let named_member_hit = early_prop_name
            .as_ref()
            .and_then(|name| self.get_property_of_type(obj_type, name));

        if named_member_hit.is_none() {
            if self.is_tuple_type(obj_type) {
                if let Some(index) = self.get_constant_numeric_value(arg_expr) {
                    if let Some(t) = self.get_tuple_element_type(obj_type, index as usize) {
                        return t;
                    }
                }

                return self.get_any_type();
            }

            if self.is_array_type(obj_type) {
                // Go getPropertyTypeForIndexType（checker.go:28697-28740）：数组对象
                // 仅携带 number 索引签名，string-like/symbol-like 键经
                // getApplicableIndexInfo 不适用、string 签名兜底为空，noImplicitAny
                // 关闭时不报错返回 nil，checkElementAccessExpression（checker.go:8656）
                // OrElse 收 errorType；number-like/any/enum-literal 键适用 number
                // 签名保持元素型直通
                if !effective_arg.flags.intersects(
                    TypeFlags::Number
                        | TypeFlags::NumberLiteral
                        | TypeFlags::Any
                        | TypeFlags::EnumLiteral,
                ) {
                    if report_nia {
                        self.report_element_access_implicit_any(
                            node,
                            obj_type,
                            arg_expr,
                            effective_arg,
                        );
                    }
                    return self.error_type();
                }
                let element = self.get_array_element_type(obj_type);
                if self.no_unchecked_indexed_access {
                    let union = self.get_union_type(vec![element, self.undefined_type()]);
                    return self.flow_type_of_access_expression(node, None, union);
                }
                return element;
            }
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
                let assignment_kind = get_assignment_target_kind(node);
                if assignment_kind != AssignmentKind::None
                    && self.is_assignment_to_readonly_entity(node, &sym, assignment_kind)
                {
                    self.error_message(
                        arg_expr,
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                        &[sym.name.clone()],
                    );
                    return self.error_type();
                }
                let from_index_signature = !matches!(&obj_type.data, crate::checker::types::TypeData::Tuple(_))
                    && element_name_resolved_from_index(obj_type, &member_name);
                // Go getPropertyTypeForIndexType（checker.go:28670-28674）：写入位
                //（assignmentTargetKind != None 即 AccessFlagsWriting）命名属性命中
                // 取 getWriteTypeOfSymbol——element access 赋值目标的 accessor 呈
                // setter 注解型
                let writing = assignment_kind != AssignmentKind::None;
                if !writing {
                    if let Some(substituted) = self.instantiate_array_member_type(obj_type, &sym) {
                        let t = self.index_access_with_no_unchecked_undefined(
                            substituted,
                            from_index_signature,
                        );
                        return self.flow_type_of_access_expression(node, Some(&sym), t);
                    }
                }
                let prop_type = if writing {
                    self.substituted_write_member_type_of(obj_type, &sym)
                } else {
                    self.get_type_of_symbol(&sym)
                };
                let t =
                    self.index_access_with_no_unchecked_undefined(prop_type, from_index_signature);
                return self.flow_type_of_access_expression(node, Some(&sym), t);
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
                // Go getPropertyTypeForIndexType（checker.go:28722-28733）：枚举对象
                // 自身枚举成员字面量索引（E[E.A]）不并 undefined，其余数字向索引
                // 在 noUncheckedIndexedAccess 读位并 undefined
                let own_enum_member_index = effective_arg.flags.intersects(TypeFlags::EnumLiteral)
                    && effective_arg
                        .symbol
                        .as_ref()
                        .is_some_and(|i| {
                            self.get_parent_of_symbol(i)
                                .is_some_and(|p| Arc::ptr_eq(&p, &sym))
                        });
                if self.no_unchecked_indexed_access && !own_enum_member_index {
                    let union = self.get_union_type(vec![s, self.undefined_type()]);
                    return self.flow_type_of_access_expression(node, None, union);
                }
                return self.flow_type_of_access_expression(node, None, s);
            }
        }

        // Go getPropertyTypeForIndexType（checker.go:28697-28700）：索引签名经
        // getApplicableIndexInfo 按索引键适用性匹配，无适用者退 string 键签名；
        // 键不适用不得吞并命中（如 string 字面量键不得经 number 签名取值）
        let index_info = self
            .get_applicable_index_info(obj_type, effective_arg)
            .or_else(|| {
                // Go getPropertyTypeForIndexType（checker.go:28696-28700）：
                // string 键签名回落仅当索引键可作 string 使用
                //（isTypeAssignableToKind StringLike；TS2538 分支
                // checker.go:28720-28722 同判据）。symbol/unique symbol 索引
                //（modularizeLibrary_UsingES5LibES6ArrayLibES6WellknownSymbolLib：
                // a[Symbol.isConcatSpreadable] 对 string[]）无适用签名时不吞
                // string 签名——indexInfo=nil 落 noImplicitAny 块，
                // getIndexedAccessTypeOrUndefined 返 nil → errorType
                if effective_arg.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
                    self.get_index_info_of_type(obj_type, &self.string_type())
                } else {
                    None
                }
            });
        if let Some(val_type) = index_info.and_then(|info| info.value_type.clone()) {
            if self.no_unchecked_indexed_access {
                let union = self.get_union_type(vec![val_type, self.undefined_type()]);
                return self.flow_type_of_access_expression(node, None, union);
            }
            return self.flow_type_of_access_expression(node, None, val_type);
        }

        if report_nia {
            // Go getPropertyTypeForIndexType（checker.go:28746/28771-28793）：
            // const enum 对象类型跳过 noImplicitAny 错误块，落尾按索引类型分派
            if is_const_enum_object_type(obj_type) {
                return self.const_enum_element_access_fallthrough(obj_type, arg_expr, effective_arg);
            }
            self.report_element_access_implicit_any(node, obj_type, arg_expr, effective_arg);
        }
        self.get_any_type()
    }
}

fn element_name_resolved_from_index(obj_type: &Arc<Type>, name: &str) -> bool { ::tsox_core::fntrace::enter("element_name_resolved_from_index"); 
    let Some(structured) = obj_type.as_structured() else {
        return false;
    };
    if structured.members.get(name).is_some() {
        return false;
    }
    let numeric = name.parse::<f64>().is_ok();
    structured.index_infos.iter().any(|info| {
        let Some(key) = &info.key_type else {
            return false;
        };
        let applicable = if numeric {
            key.flags.contains(TypeFlags::Number)
        } else {
            key.flags.contains(TypeFlags::String)
        };
        applicable && info.value_type.is_some()
    })
}
