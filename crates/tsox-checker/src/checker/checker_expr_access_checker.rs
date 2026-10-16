#![allow(unused_imports)]

use crate::checker::checker_expr_access::*;
use crate::checker::checker_resolve_access::is_write_only_access;
use crate::checker::types_type_id::{
    TYPE_FLAGS_ANY_OR_UNKNOWN, TYPE_FLAGS_BIG_INT_LIKE, TYPE_FLAGS_NUMBER_LIKE,
    TYPE_FLAGS_STRING_LIKE,
};
use crate::checker::utilities_has_only_expression_initialization::get_assignment_target_kind;
use crate::checker::utilities_token_is_identifier_or_keyword::AssignmentKind;
use tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY;

impl Checker {
    pub(crate) fn get_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_binary_expression"); 
        use tsox_frontend::ast::SyntaxKind::*;
        if let tsox_frontend::ast::NodeData::BinaryExpression(data) = &node.data {
            match data.operator_token.kind {
                PlusToken => {
                    let mut lt = self.get_type_of_node(&data.left);
                    let mut rt = self.get_type_of_node(&data.right);
                    let silent_never = self.silent_never_type();
                    if Arc::ptr_eq(&lt, &silent_never) || Arc::ptr_eq(&rt, &silent_never) {
                        return silent_never;
                    }
                    if !self.is_type_assignable_to_kind(&lt, TYPE_FLAGS_STRING_LIKE)
                        && !self.is_type_assignable_to_kind(&rt, TYPE_FLAGS_STRING_LIKE)
                    {
                        lt = self.check_non_null_type(&lt, &data.left);
                        rt = self.check_non_null_type(&rt, &data.right);
                    }
                    if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_NUMBER_LIKE, true)
                        && self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_NUMBER_LIKE, true)
                    {
                        self.number_type()
                    } else if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_BIG_INT_LIKE, true)
                        && self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_BIG_INT_LIKE, true)
                    {
                        self.bigint_type()
                    } else if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_STRING_LIKE, true)
                        || self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_STRING_LIKE, true)
                    {
                        self.string_type()
                    } else if lt.flags.intersects(TypeFlags::Any)
                        || rt.flags.intersects(TypeFlags::Any)
                    {
                        if self.is_error_type(&lt) || self.is_error_type(&rt) {
                            self.error_type()
                        } else {
                            self.get_any_type()
                        }
                    } else {
                        self.get_any_type()
                    }
                }

                MinusToken
                | AsteriskToken
                | SlashToken
                | PercentToken
                | AsteriskAsteriskToken
                | LessThanLessThanToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanToken
                | AmpersandToken
                | BarToken
                | CaretToken => {
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if (self.is_type_assignable_to_kind(&left_type, TYPE_FLAGS_ANY_OR_UNKNOWN)
                        && self.is_type_assignable_to_kind(&right_type, TYPE_FLAGS_ANY_OR_UNKNOWN))
                        || (!self.maybe_type_of_kind(&left_type, TYPE_FLAGS_BIG_INT_LIKE)
                            && !self.maybe_type_of_kind(&right_type, TYPE_FLAGS_BIG_INT_LIKE))
                    {
                        self.number_type()
                    } else if self.both_are_big_int_like(&left_type, &right_type) {
                        self.bigint_type()
                    } else {
                        self.error_type()
                    }
                }

                LessThanToken
                | GreaterThanToken
                | LessThanEqualsToken
                | GreaterThanEqualsToken
                | EqualsEqualsToken
                | ExclamationEqualsToken
                | EqualsEqualsEqualsToken
                | ExclamationEqualsEqualsToken
                | InKeyword
                | InstanceOfKeyword => self.boolean_type(),

                AmpersandAmpersandToken | AmpersandAmpersandEqualsToken => {
                    // Go checkBinaryExpressionWorker：&& 结果 =
                    // union(extractDefinitelyFalsy(left), right)，
                    // 左操作数无真值事实时结果为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if self
                        .constituent_types(&left_type)
                        .iter()
                        .any(|c| self.has_truthy_fact(c))
                    {
                        // Go checkBinaryLikeExpression（checker.go:13163-13171）：
                        // t = left（strictNullChecks）/ getBaseTypeOfLiteralType(right)（非严格），
                        // falsy 部分取 extractDefinitelyFalsyTypes（checker.go:30820-30823）
                        // = mapType(getDefinitelyFalsyPartOfType)（checker.go:30825-30842），
                        // number/string/bigint 基类映为零字面量 0/""/0n，对象成员落 never
                        let t = if self.strict_null_checks {
                            Arc::clone(&left_type)
                        } else {
                            self.get_base_type_of_literal_type(&right_type)
                        };
                        let falsy = self.extract_definitely_falsy_types(&t);
                        self.get_union_type(vec![falsy, right_type])
                    } else {
                        left_type
                    }
                }

                BarBarToken | BarBarEqualsToken => {
                    // Go checkBinaryExpressionWorker：|| 结果 =
                    // union(nonNullable(removeDefinitelyFalsy(left)), right)，
                    // 左操作数无假值事实时结果为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if self
                        .constituent_types(&left_type)
                        .iter()
                        .any(|c| self.has_falsy_fact(c))
                    {
                        let removed = self.remove_definitely_falsy_constituents(&left_type);
                        let non_null = self.get_non_nullable_type(&removed);
                        let reduced = self.remove_subtype_redundant_members(vec![
                            non_null,
                            Arc::clone(&right_type),
                        ]);
                        self.get_union_type(reduced)
                    } else {
                        left_type
                    }
                }

                QuestionQuestionToken | QuestionQuestionEqualsToken => {
                    // Go checkBinaryExpressionWorker：?? 结果 =
                    // union(nonNullable(left), right)，左操作数不可空时为左类型
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    let may_be_nullish = self.constituent_types(&left_type).iter().any(|c| {
                        c.flags.intersects(
                            TypeFlags::Undefined
                                | TypeFlags::Null
                                | TypeFlags::Any
                                | TypeFlags::Unknown
                                | TypeFlags::TypeParameter,
                        )
                    });
                    if may_be_nullish {
                        let non_null = self.get_non_nullable_type(&left_type);
                        let reduced =
                            self.remove_subtype_redundant_members(vec![non_null, right_type]);
                        self.get_union_type(reduced)
                    } else {
                        left_type
                    }
                }

                CommaToken => self.get_type_of_node(&data.right),

                EqualsToken => self.get_type_of_node(&data.right),

                PlusEqualsToken => {
                    let mut lt = self.get_type_of_node(&data.left);
                    let mut rt = self.get_type_of_node(&data.right);
                    let silent_never = self.silent_never_type();
                    if Arc::ptr_eq(&lt, &silent_never) || Arc::ptr_eq(&rt, &silent_never) {
                        return silent_never;
                    }
                    if !self.is_type_assignable_to_kind(&lt, TYPE_FLAGS_STRING_LIKE)
                        && !self.is_type_assignable_to_kind(&rt, TYPE_FLAGS_STRING_LIKE)
                    {
                        lt = self.check_non_null_type(&lt, &data.left);
                        rt = self.check_non_null_type(&rt, &data.right);
                    }
                    if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_NUMBER_LIKE, true)
                        && self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_NUMBER_LIKE, true)
                    {
                        self.number_type()
                    } else if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_BIG_INT_LIKE, true)
                        && self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_BIG_INT_LIKE, true)
                    {
                        self.bigint_type()
                    } else if self.is_type_assignable_to_kind_ex(&lt, TYPE_FLAGS_STRING_LIKE, true)
                        || self.is_type_assignable_to_kind_ex(&rt, TYPE_FLAGS_STRING_LIKE, true)
                    {
                        self.string_type()
                    } else if lt.flags.intersects(TypeFlags::Any)
                        || rt.flags.intersects(TypeFlags::Any)
                    {
                        if self.is_error_type(&lt) || self.is_error_type(&rt) {
                            self.error_type()
                        } else {
                            self.get_any_type()
                        }
                    } else {
                        self.get_any_type()
                    }
                }

                MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken => {
                    let left_type = self.get_type_of_node(&data.left);
                    let right_type = self.get_type_of_node(&data.right);
                    if (self.is_type_assignable_to_kind(&left_type, TYPE_FLAGS_ANY_OR_UNKNOWN)
                        && self.is_type_assignable_to_kind(&right_type, TYPE_FLAGS_ANY_OR_UNKNOWN))
                        || (!self.maybe_type_of_kind(&left_type, TYPE_FLAGS_BIG_INT_LIKE)
                            && !self.maybe_type_of_kind(&right_type, TYPE_FLAGS_BIG_INT_LIKE))
                    {
                        self.number_type()
                    } else if self.both_are_big_int_like(&left_type, &right_type) {
                        self.bigint_type()
                    } else {
                        self.error_type()
                    }
                }
                _ => self.get_any_type(),
            }
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn get_type_of_property_access(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_property_access"); 
        let (obj_expr, name) = match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name)
            }
            _ => return self.get_any_type(),
        };

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let name_text = name.text();
                let member = base
                    .exports
                    .get(name_text)
                    .or_else(|| base.members.get(name_text))
                    .cloned()
                    .or_else(|| self.ambient_namespace_local(&base, name_text));

                if member.is_none() && !self.ambient_namespace_locals_visible(&base) {
                    return self.error_type();
                }
                if let Some(member) = member {
                    let assignment_kind = get_assignment_target_kind(node);
                    if assignment_kind != AssignmentKind::None
                        && self.is_assignment_to_readonly_entity(node, &member, assignment_kind)
                    {
                        self.error_message(
                            name,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                            &[name_text.to_string()],
                        );
                        return self.error_type();
                    }
                    if let Some(t) = self
                        .value_symbol_links
                        .get(&member)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        return t;
                    }
                    for decl in &member.declarations {
                        match decl.kind {
                            SyntaxKind::FunctionDeclaration => {
                                // Go getTypeOfSymbol 经匿名壳 +
                                // resolveDeclaredMembers 聚合符号全部过载声明；
                                // 多声明成员（lib.d.ts 的 namespace 重载组）须建
                                // 过载集而非首个声明的单签名型
                                return match self.build_overload_function_type(&member) {
                                    Some(t) => t,
                                    None => self.get_type_of_function_like(decl),
                                };
                            }
                            SyntaxKind::ClassDeclaration => {
                                return self.get_type_of_class_declaration(decl);
                            }

                            SyntaxKind::ImportEqualsDeclaration => {
                                let t = self.type_of_imported_symbol(&member);
                                let resolved = match t {
                                    Some(t)
                                        if !(t.flags.contains(TypeFlags::Any)
                                            && t.intrinsic_name() == Some("any")) =>
                                    {
                                        Some(t)
                                    }
                                    _ => {
                                        let base = self.resolve_alias_base(Arc::clone(&member));
                                        base.declarations
                                            .iter()
                                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                                            .map(|cd| self.get_type_of_class_declaration(cd))
                                    }
                                };
                                if let Some(t) = resolved {
                                    return t;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        let left_type = self.get_type_of_node(obj_expr);
        if left_type.intrinsic_name() == Some("error") {
            return self.error_type();
        }
        // Go checkPropertyAccessChain（checker.go:11866-11871）：链根表达式按
        // GetNonNullableType 参加成员解析，访问结果按 wasOptional 回补 undefined
        let non_optional_type = self.get_optional_expression_type(&left_type, obj_expr);
        let was_optional = !Arc::ptr_eq(&non_optional_type, &left_type);
        let obj_type = non_optional_type;
        let name_text = name.text();

        // Go checkPropertyAccessExpressionOrQualifiedName（checker.go:11997-12001）：
        // 写目标解析出 readonly 属性即返回 errorType（.types 基线渲染 any）；
        // TS2540 报告位在 check_binary_expression 的 is_readonly_property_write，
        // 本处只补 Go 的 errorType 返回
        let assignment_kind = get_assignment_target_kind(node);
        if assignment_kind != AssignmentKind::None
            && let Some(prop) = self.get_property_of_type(&obj_type, &name_text)
            && self.is_assignment_to_readonly_entity(node, &prop, assignment_kind)
        {
            return self.error_type();
        }

        if let Some(sym) = self.get_property_of_type(&obj_type, &name_text) {
            // Go checkPropertyAccessExpressionOrQualifiedName（checker.go:12000-12002）：
            // 只写访问（赋值目标）取 getWriteTypeOfSymbol，divergent accessors
            // （get/set 异型）呈 set 参数型而非 get 返回型；实例化成员经
            // getWriteTypeOfInstantiatedSymbol（checker.go:17508-17512）按 mapper
            // 实例化（divergentAccessorsTypes7：泛型类写位 setter 注解按实参
            // 替换）。与 element_access_result_type 写位同一分派，维持 a.value
            // 与 a['value'] 两访问通道目标型一致
            if is_write_only_access(node) && sym.flags.contains(SymbolFlags::SetAccessor) {
                let write_type = self.substituted_write_member_type_of(&obj_type, &sym);
                let flowed = self.flow_type_of_access_expression(node, Some(&sym), write_type);
                return self.propagate_optional_type_marker(&flowed, node, was_optional);
            }
            if let Some(substituted) = self.instantiate_array_member_type(&obj_type, &sym) {
                let flowed = self.flow_type_of_access_expression(node, Some(&sym), substituted);
                return self.propagate_optional_type_marker(&flowed, node, was_optional);
            }

            if obj_type
                .as_object()
                .is_some_and(|o| !o.type_arguments.is_empty())
            {
                let substituted = self.substituted_member_type_of(&obj_type, &sym);
                let flowed = self.flow_type_of_access_expression(node, Some(&sym), substituted);
                return self.propagate_optional_type_marker(&flowed, node, was_optional);
            }
            // 类实例型成员是急建合成符号（无注解方法返回 any 驻缓存）：
            // 回源 binder 声明符号走惰性体推断（this 返回型等）
            // Go checkPropertyAccessExpressionOrQualifiedName（checker.go:11996-12000）：
            // 写-only 访问（赋值唯一目标）的成员型取 getWriteTypeOfSymbol——
            // accessor 的写位呈 setter 注解型，checkExpressionEx 驻留该型供
            // .types 基线与赋值可赋值性消费
            let prop_type = if crate::checker::checker_resolve_access::is_write_only_access(node) {
                self.substituted_write_member_type_of(&obj_type, &sym)
            } else {
                self.member_decl_symbol_type(&sym)
            };
            let flowed = self.flow_type_of_access_expression(node, Some(&sym), prop_type);
            return self.propagate_optional_type_marker(&flowed, node, was_optional);
        }

        if name_text == "length" && self.is_array_type(&obj_type) {
            let number = self.number_type();
            return self.propagate_optional_type_marker(&number, node, was_optional);
        }
        // 无具名成员：适用的字符串索引签名值（tsc getApplicableIndexInfo）
        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                let key_matches = info.key_type.as_ref().is_some_and(|k| {
                    k.flags.contains(TypeFlags::String)
                        || k.flags.contains(TypeFlags::Number)
                });
                if key_matches {
                    if let Some(value) = &info.value_type {
                        let flowed =
                            self.flow_type_of_access_expression(node, None, Arc::clone(value));
                        return self.propagate_optional_type_marker(&flowed, node, was_optional);
                    }
                }
            }
        }
        let any = self.get_any_type();
        self.propagate_optional_type_marker(&any, node, was_optional)
    }

    pub(crate) fn flow_type_of_access_expression(
        &mut self,
        node: &Arc<Node>,
        prop: Option<&Arc<Symbol>>,
        prop_type: Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("flow_type_of_access_expression");
        if Self::is_definite_assignment_target(node) {
            return prop_type;
        }
        if let Some(prop) = prop {
            // Go checker.go:12017：SymbolFlagsMethod（ast/symbolflags.go:22）为独立
            // 位，binder 方法成员不带 Property 位，方法访问非 union 型直接返回
            // 成员类型不进流收窄。本侧类成员急建合成符号带 Property|Method 双位
            //（typenode_references_interface_members_properties.rs），按 Method 位
            // 判定保持与 Go 方法成员同口径，Property 位不单独授予流资格
            let eligible = if prop.flags.contains(SymbolFlags::Method) {
                prop_type.is_union()
            } else {
                prop.flags
                    .intersects(SymbolFlags::VARIABLE | SymbolFlags::Property | SymbolFlags::ACCESSOR)
            };
            if !eligible {
                return prop_type;
            }
        }
        // Go getFlowTypeOfAccessExpression（checker.go:12023）：约束位置或
        // 无泛型语境型时 getNarrowableTypeForReference 先行（链根宿主、
        // 调用宿主等处把 { [P in K]: E }[X] 换成基约束 E[P:=X] 再入流）
        let prop_type = self.get_narrowable_type_for_reference(&prop_type, node);
        self.get_flow_type_of_reference(node, &prop_type)
    }

    pub(crate) fn is_definite_assignment_target(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_definite_assignment_target"); 
        let Some(parent) = &node.parent() else {
            return false;
        };
        match &parent.data {
            NodeData::BinaryExpression(bin) => {
                Self::is_assignment_operator(bin.operator_token.kind)
                    && Arc::ptr_eq(&bin.left, node)
            }
            NodeData::PostfixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && Arc::ptr_eq(&unary.operand, node)
            }
            NodeData::PrefixUnaryExpression(unary) => {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && Arc::ptr_eq(&unary.operand, node)
            }
            _ => false,
        }
    }

    pub(crate) fn is_assignment_operator(kind: tsox_frontend::ast::SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_assignment_operator"); 
        use tsox_frontend::ast::SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | BarBarEqualsToken
                | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken
        )
    }

    pub(crate) fn is_block_terminating_statement(stmt: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_block_terminating_statement"); 
        matches!(
            stmt.kind,
            SyntaxKind::ReturnStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
        )
    }
}
