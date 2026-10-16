#![allow(unused_imports)]

use crate::checker::checker_assignment2::*;
use tsox_core::diagnostics::messages_generated::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1;

impl Checker {
    pub(crate) fn check_assignment_compat(
        &mut self,
        node: &Arc<Node>,
        data: &tsox_frontend::ast::node_data_generated::BinaryExpressionData,
    ) { ::tsox_core::fntrace::enter("check_assignment_compat"); 
        use tsox_frontend::ast::SyntaxKind::*;

        if data.operator_token.kind == EqualsToken
            && matches!(
                data.left.kind,
                ObjectLiteralExpression | ArrayLiteralExpression
            )
        {
            return;
        }

        let mut target: &Arc<Node> = &data.left;
        loop {
            match &target.data {
                tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                    target = &p.expression;
                }
                tsox_frontend::ast::NodeData::NonNullExpression(n) => {
                    target = &n.expression;
                }
                _ => break,
            }
        }

        let optional_chain = match &target.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                pa.question_dot_token.is_some()
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_some()
            }
            _ => false,
        };
        let is_reference = matches!(
            target.kind,
            Identifier | PropertyAccessExpression | ElementAccessExpression
        );
        if !is_reference || optional_chain {
            let message = if optional_chain {
                tsox_core::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS
            } else {
                tsox_core::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS
            };

            let loc = if data.left.kind == SyntaxKind::ParenthesizedExpression {
                node.loc
            } else {
                target.loc
            };
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                Vec::new(),
            ));

            self.check_expression(&data.left);
            return;
        }
        // Go checkBinaryLikeExpression（checker.go:13008）：赋值可赋值性的目标
        // 型即 checkExpressionEx(left) 的驻留型——写-only access 经写位分派
        //（checker.go:11996-12000/28670-28674）驻留 setter 注解型，Identifier
        // 走 flow narrow；本函数原以 assignment_target_type 独立重算，accessor
        // 场景与驻留型分叉（computedPropertiesWithSetterAssignment：foo.k 目标
        // 型算成 getter 注解 Set<string> 误报 TS2322）。复合赋值左目标是读写
        // 访问（is_write_only_access=false），Go checkAssignmentOperator
        //（checker.go:13452-13454）对 property access 显式按 writeOnly 重取——
        // get/set 异型时对 setter 注解型判定（getterSetterSubtypeAssignment：
        // get x(): number / set x(v: number|string) 的 this.x += '1' 不因
        // getter 型 number 误报 string→number）
        let left_type = if tsox_frontend::ast::is_compound_assignment(data.operator_token.kind)
            && tsox_frontend::ast::is_property_access_expression(target)
        {
            self.check_property_access_expression(
                target,
                crate::checker::types_alias_symbol_links::CheckMode::Normal,
                true,
            )
        } else if target.kind == SyntaxKind::Identifier {
            self.assignment_target_type(target)
                .unwrap_or_else(|| self.get_type_of_node(target))
        } else {
            self.get_type_of_node(target)
        };

        if target.kind == Identifier {
            if let Some(sym) = self.resolve_identifier(target) {
                let base = self.resolve_alias_base(sym);
                if base.flags.contains(SymbolFlags::NamespaceModule)
                    && !base.flags.contains(SymbolFlags::ValueModule)
                {
                    return;
                }
                if base
                    .flags
                    .intersects(SymbolFlags::Class | SymbolFlags::ENUM | SymbolFlags::ValueModule)
                    && !base.flags.intersects(
                        SymbolFlags::VARIABLE
                            | SymbolFlags::PROPERTY_OR_ACCESSOR
                            | SymbolFlags::Function,
                    )
                {
                    return;
                }

                if self.symbol_is_const_variable(&base) {
                    return;
                }
            }
        }

        if left_type.flags.contains(TypeFlags::Any) && left_type.intrinsic_name() == Some("error") {
            return;
        }

        if self.assignment_target_is_readonly(target) {
            return;
        }
        let right_type = match data.operator_token.kind {
            EqualsToken => self.get_type_of_node(&data.right),

            AmpersandAmpersandEqualsToken | BarBarEqualsToken | QuestionQuestionEqualsToken => {
                match self.logical_rhs_frame(data.operator_token.kind, target) {
                    Some((sym, t)) => {
                        self.logical_rhs_narrowing_frames.push((sym, t));
                        let rt = self.get_type_of_node(&data.right);
                        self.logical_rhs_narrowing_frames.pop();
                        rt
                    }
                    None => self.get_type_of_node(&data.right),
                }
            }

            _ => {
                if self
                    .arith_operand_error_nodes
                    .contains(&(Arc::as_ptr(node) as *const tsox_frontend::ast::Node))
                {
                    return;
                }
                self.get_type_of_node(node)
            }
        };

        if data.operator_token.kind == EqualsToken
            && data.right.kind == SyntaxKind::ObjectLiteralExpression
        {
            if let Some(excess_name) = self.get_excess_property_name(&right_type, &left_type) {
                let loc = self
                    .find_object_literal_property_name_node(&data.right, &excess_name)
                    .unwrap_or(data.right.loc);
                let filtered_left = self.excess_check_error_target(&left_type);
                let annot_str = self.type_to_string(&filtered_left);
                // Go reportUnmatchedPropertyForExcessProperty：字面量自身元素命中
                // 拼写建议时换 TS2561 文案
                if let Some(sugg) = self.suggestion_for_nonexistent_property(&excess_name, &filtered_left) {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        loc,
                        tsox_core::diagnostics::messages_generated::
                            OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2,
                        vec![
                            crate::checker::property_name_for_display(&excess_name),
                            annot_str,
                            sugg,
                        ],
                    ));
                } else {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        loc,
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                        vec![
                            crate::checker::property_name_for_display(&excess_name),
                            annot_str,
                        ],
                    ));
                }
                return;
            }
        }

        if !self.is_type_assignable_to(&right_type, &left_type) {
            let report_type = self.assignment_report_type(target, &left_type);
            let head_message = self.exact_optional_mismatch_head_message(target, &right_type);
            let _ = self.check_type_assignable_to_and_optionally_elaborate(
                &right_type,
                &report_type,
                Some(target),
                Some(&data.right),
                head_message.as_ref(),
                None,
            );
        }
    }

    // Go checkAssignmentOperator：EOPT 下写 undefined 到属性访问目标且
    // 目标属性类型含 missing（Rust 以符号 Optional 位近似）时换 TS2412 文案
    fn exact_optional_mismatch_head_message(
        &mut self,
        target: &Arc<Node>,
        right_type: &Arc<Type>,
    ) -> Option<tsox_core::diagnostics::Message> { ::tsox_core::fntrace::enter("exact_optional_mismatch_head_message"); 
        if !self.exact_optional_property_types {
            return None;
        }
        let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &target.data else {
            return None;
        };
        if !self.maybe_type_of_kind(right_type, TypeFlags::Undefined) {
            return None;
        }
        let obj_type = self.get_type_of_node(&pa.expression);
        let prop = self.get_property_of_type(&obj_type, &pa.name.text())?;
        if !prop.flags.contains(SymbolFlags::Optional) {
            return None;
        }
        Some(
            tsox_core::diagnostics::messages_generated::
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_WITH_EXACTOPTIONALPROPERTYTYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPE_OF_THE_TARGET,
        )
    }

    pub(crate) fn write_type_of_property_symbol(
        &mut self,
        owner: Option<&Arc<Type>>,
        prop: &Arc<tsox_frontend::ast::Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("write_type_of_property_symbol"); 
        let mut t = None;
        if prop.flags.contains(SymbolFlags::SetAccessor)
            && let Some(setter) = prop
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor)
            && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &setter.data
            && let Some(param) = sd.parameters.iter().next()
            && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            let raw = self.get_type_from_type_node(tn);
            let mapped = self
                .value_symbol_links
                .get(prop)
                .and_then(|l| l.mapper.clone())
                .map(|m| m.map(&raw))
                .unwrap_or(raw);
            t = Some(mapped);
        }
        let base = t.unwrap_or_else(|| self.get_type_of_symbol(prop));
        let Some(owner) = owner else {
            return base;
        };
        let Some(obj) = owner.as_object() else {
            return base;
        };
        if obj.type_arguments.is_empty() {
            return base;
        }
        let Some(owner_sym) = owner.symbol.clone() else {
            return base;
        };
        let decl_tps = self.declared_type_parameter_types(&owner_sym);
        if decl_tps.is_empty() || decl_tps.len() != obj.type_arguments.len() {
            return base;
        }
        self.substitute_infer_type_parameters(&base, &decl_tps, &obj.type_arguments)
    }

    // setter 目标的赋值报错文案：联合写类型去掉 undefined 成员
    // （Go getFlowTypeOfAccessExpression 对确定性写路径的 removeMissingType）
    fn assignment_report_type(&mut self, target: &Arc<Node>, write_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("assignment_report_type"); 
        let has_setter = match &target.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);
                self.get_property_of_type(&obj_type, &pa.name.text())
                    .is_some_and(|s| s.flags.contains(SymbolFlags::SetAccessor))
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(ea) => {
                match self.element_access_property_key(&ea.argument_expression) {
                    Some(name) => {
                        let obj_type = self.get_type_of_node(&ea.expression);
                        self.get_property_of_type(&obj_type, &name)
                            .is_some_and(|s| s.flags.contains(SymbolFlags::SetAccessor))
                    }
                    None => false,
                }
            }
            _ => false,
        };
        if !has_setter {
            return Arc::clone(write_type);
        }
        if let crate::checker::types::TypeData::Union(u) = &write_type.data {
            let kept: Vec<&Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .filter(|t| !t.flags.contains(TypeFlags::Undefined))
                .collect();
            if kept.len() == 1 && kept.len() < u.union_or_intersection.types.len() {
                return Arc::clone(kept[0]);
            }
        }
        Arc::clone(write_type)
    }

    pub(crate) fn assignment_target_type(&mut self, target: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("assignment_target_type"); 
        match &target.data {
            tsox_frontend::ast::NodeData::Identifier(_) => {
                let sym = self.resolve_identifier(target)?;
                let declared = self.get_type_of_symbol(&sym);

                let target_kind = get_assignment_target_kind(target);
                let compound_like = target_kind == AssignmentKind::Definite
                    && is_in_compound_like_assignment(target);
                if compound_like || target_kind == AssignmentKind::Compound {
                    Some(self.get_base_type_of_literal_type(&declared))
                } else {
                    Some(declared)
                }
            }
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);

                self.get_property_of_type(&obj_type, &pa.name.text())
                    .map(|sym| self.write_type_of_property_symbol(Some(&obj_type), &sym))
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(ea) => {
                if let Some(name) = self.element_access_property_key(&ea.argument_expression) {
                    let obj_type = self.get_type_of_node(&ea.expression);
                    if let Some(prop) = self.get_property_of_type(&obj_type, &name) {
                        return Some(self.write_type_of_property_symbol(Some(&obj_type), &prop));
                    }
                }
                let obj_type = self.get_type_of_node(&ea.expression);
                let index_type = self.get_type_of_node(&ea.argument_expression);
                Some(self.get_indexed_access_type(&obj_type, &index_type))
            }
            _ => None,
        }
    }

    pub(crate) fn assignment_target_is_readonly(&mut self, target: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("assignment_target_is_readonly"); 
        match &target.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);
                if let Some(sym) = self.get_property_of_type(&obj_type, &pa.name.text())
                    && (sym
                        .check_flags
                        .contains(tsox_frontend::ast::CheckFlags::Readonly)
                        || sym
                            .declarations
                            .iter()
                            .any(|d| d.has_syntactic_modifier(ModifierFlags::Readonly)))
                {
                    return true;
                }
                self.namespace_const_member(&pa.expression, &pa.name.text())
                    .is_some()
            }

            tsox_frontend::ast::NodeData::ElementAccessExpression(ea)
                if ea.argument_expression.kind == SyntaxKind::StringLiteral =>
            {
                self.namespace_const_member(&ea.expression, ea.argument_expression.text())
                    .is_some()
            }
            _ => false,
        }
    }

    /// 元素访问的属性键：字符串/数字字面量按文本，`Symbol.<well-known>` 按内部名，
    /// 其余计算名按求值型取名（Go getPropertyNameFromIndex 同序：isTypeUsableAsPropertyName
    /// 的字面量型即取值）——const enum 成员引用（foo[Props.k]）求值后为字符串
    /// 字面量型，按语法非字面量跳过符号臂会落泛型 indexed access 读型臂
    /// （computedPropertiesWithSetterAssignment：写目标呈 getter 读型 Set<string>
    /// 报 TS2322，Go 取 setter 写型 Iterable<string>）
    fn element_access_property_key(&mut self, arg: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("element_access_property_key");
        match arg.kind {
            SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                Some(arg.text().to_string())
            }
            _ => {
                if let Some(internal) =
                    crate::binder::symbols_binder_4::well_known_symbol_member_name(arg)
                {
                    return Some(internal);
                }
                let index_type = self.get_type_of_node(arg);
                if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&index_type) {
                    let name = crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&index_type);
                    (!name.is_empty()).then_some(name)
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn namespace_const_member(
        &mut self,
        obj_expr: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> { ::tsox_core::fntrace::enter("namespace_const_member"); 
        if obj_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(obj_expr)?;
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return None;
        }
        let member = base
            .exports
            .get(name)
            .or_else(|| base.members.get(name))
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
                            .and_then(|l| l.get(name).cloned())
                    })
            });
        member.filter(|m| self.symbol_is_const_variable(m))
    }
}
