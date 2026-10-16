#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;

impl Checker {
    pub(crate) fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("reduced_assignment_type"); 
        if evolving {
            return Arc::clone(assigned);
        }

        if declared.flags.contains(TypeFlags::Null)
            && (self.is_auto_array_type(assigned)
                || assigned.object_flags.contains(ObjectFlags::EvolvingArray))
        {
            return Arc::clone(assigned);
        }
        if Arc::ptr_eq(declared, &self.auto_type()) || self.is_auto_array_type(declared) {
            let widened = self.get_base_type_of_literal_type(assigned);
            if self.is_type_assignable_to(&widened, declared) {
                return widened;
            }
            return self.any_array_type();
        }
        if !declared.is_union() {
            // Go checker.go:1013 booleanType 即 union{regularFalse, regularTrue}，
            // flow.go:268 的 TypeFlagsUnion 赋值收窄门对其放行；本仓 boolean 是
            // 内建单例种别，按 union{false, true} 拆分后走同一归约
            if !declared.flags.contains(TypeFlags::Boolean) {
                return Arc::clone(declared);
            }
            // Go flow.go:2515-2517 getAssignmentReducedType 的
            // declaredType == assignedType 快路径：初始式与声明同为 boolean
            // 单例时归约恒等，原样返回声明型不做拆分
            if Arc::ptr_eq(declared, assigned) {
                return Arc::clone(declared);
            }
            let split = self.split_intrinsic_boolean(declared);
            return self.get_assignment_reduced_type(&split, assigned);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    pub(crate) fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_assignment_reduced_type"); 
        if Arc::ptr_eq(declared, assigned) {
            return Arc::clone(declared);
        }
        if assigned.flags.contains(TypeFlags::Never) {
            return Arc::clone(assigned);
        }
        let constituents = self.constituent_types(declared);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| self.type_maybe_assignable_to(assigned, t))
            .collect();
        let reduced = self.rebuild_union_or_never(declared, kept);
        if self.is_type_assignable_to(assigned, &reduced) {
            reduced
        } else {
            Arc::clone(declared)
        }
    }

    pub(crate) fn type_maybe_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_maybe_assignable_to"); 
        if !source.is_union() {
            return self.is_type_assignable_to(source, target);
        }
        let constituents = self.constituent_types(source);
        if constituents.iter().any(|t| Arc::ptr_eq(t, target)) {
            return true;
        }
        constituents
            .iter()
            .any(|t| self.is_type_assignable_to(t, target))
    }

    pub(crate) fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("initial_type_of_declaration");
        self.initial_type_of_declaration_ex(expr, true)
    }

    pub(crate) fn initial_type_of_declaration_ex(
        &mut self,
        expr: &Arc<Node>,
        widen: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("initial_type_of_declaration_ex");
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {
                    if matches!(
                        &init.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        return Some(self.auto_array_type());
                    }
                    if self.is_null_or_undefined(init) {
                        return Some(self.auto_type());
                    }
                    let t = self.get_type_of_node(init);
                    if !widen {
                        return Some(t);
                    }
                    // Go getTypeForVariableLikeDeclaration（checker.go:17726-17728）：
                    // 初始化式推断型先过 widenTypeInferredFromInitializer——非 const
                    // 声明的 fresh 字面量拓宽（'' → string、1 → number）
                    let widened = self.get_widened_literal_type_for_initializer(expr, &t);
                    // Go widenTypeInferredFromInitializer（checker.go:17867-17876）：
                    // JS 文件加宽结果为 implicit never（strict）/undefinedWidening
                    //（非 strict）时报告隐式 any 并整体转 any
                    if tsox_frontend::ast::is_in_js_file(expr) {
                        if self.is_empty_literal_type(&widened) {
                            self.report_implicit_any(
                                expr,
                                &self.get_any_type(),
                                crate::checker::types_alias_symbol_links::WideningKind::Normal,
                            );
                            return Some(self.get_any_type());
                        }
                        if self.is_empty_array_literal_type(&widened) {
                            let any_array = self.any_array_type();
                            self.report_implicit_any(
                                expr,
                                &any_array,
                                crate::checker::types_alias_symbol_links::WideningKind::Normal,
                            );
                            return Some(any_array);
                        }
                    }
                    if type_contains_widening_member(&widened) {
                        return Some(self.get_widened_type(&widened));
                    }
                    return Some(widened);
                }
                let for_stmt = Self::for_in_or_of_statement_of(expr)?;
                let NodeData::ForInOrOfStatement(data) = &for_stmt.data else {
                    return None;
                };
                match for_stmt.kind {
                    SyntaxKind::ForInStatement => Some(self.string_type()),
                    SyntaxKind::ForOfStatement => {
                        let for_await = data.await_modifier.is_some();
                        let rhs = self.get_type_of_node(&data.expression);
                        Some(self.check_iterated_type_or_element_type(
                            crate::checker::checker_iteration::IterationUse::ForOf { for_await },
                            &rhs,
                            None,
                        ))
                    }
                    _ => None,
                }
            }
            NodeData::BindingElement(be) => {
                let pattern = Arc::clone(expr.parent().as_ref()?);
                let pattern_parent = Arc::clone(pattern.parent().as_ref()?);
                let parent_type = self.initial_type_of_declaration_ex(&pattern_parent, widen);
                let diagnostics_allowed = !self.in_ambient_declaration_context();
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        // rest 元素：剩余属性对象，不做属性查找、不产生 TS2339
                        if be.dot_dot_dot_token.is_some() {
                            return Some(self.rest_element_type(expr, &parent_type));
                        }
                        // Go getInitialTypeOfBindingElement（flow.go:2373-2391）：
                        // 键名经 getLiteralTypeFromPropertyName（计算名求值内表达
                        // 式），查找走 getTypeOfDestructuredProperty（属性落空回落
                        // getApplicableIndexInfoForName）——失败尾臂 errorType
                        //（flow.go:2485），静默不发 TS2339；报告位在声明期
                        //（get_binding_element_type_from_parent_type 的
                        // reportErrors 通道，checker_symbol_types_checker_2.rs）
                        // TS2454 链路：流赋值归约对 errorType≡any 全保留成分回落
                        // 声明型（flow.go:269-273 仅联合才归约），流型不含
                        // undefined，本例 TS2339/TS2454 双消
                        let Some(name) = self.destructured_property_key_name(expr) else {
                            return Some(self.error_type());
                        };
                        match self.destructured_property_type_of(parent_type, &name) {
                            Some(t) => Some(t),
                            None => Some(self.error_type()),
                        }
                    }
                    (Some(parent_type), SyntaxKind::ArrayBindingPattern)
                        if be.dot_dot_dot_token.is_none() =>
                    {
                        match Self::binding_element_index(&pattern, expr) {
                            Some(index) => self.destructured_array_element_type(
                                parent_type,
                                index,
                                diagnostics_allowed.then(|| &pattern),
                            ),
                            None => None,
                        }
                    }
                    (Some(parent_type), SyntaxKind::ArrayBindingPattern) => {
                        let element = self.check_iterated_type_or_element_type(
                            crate::checker::checker_iteration::IterationUse::Destructuring,
                            parent_type,
                            None,
                        );
                        if self.is_tuple_type(parent_type) {
                            let mut rest: Vec<Arc<Type>> = Vec::new();
                            let mut index = Self::binding_element_index(&pattern, expr)
                                .unwrap_or(0);
                            while let Some(t) = self.get_tuple_element_type(parent_type, index) {
                                rest.push(t);
                                index += 1;
                            }
                            let element_t = if rest.is_empty() {
                                self.never_type()
                            } else {
                                self.get_union_type(rest)
                            };
                            Some(self.create_array_type(element_t))
                        } else {
                            Some(self.create_array_type(element))
                        }
                    }
                    _ => None,
                };
                if let Some(default_expr) = &be.initializer {
                    let default_type = self.get_type_of_node(default_expr);

                    t = match t {
                        Some(t) => {
                            let non_undefined =
                                self.remove_flags_from_union(&t, TypeFlags::Undefined);
                            Some(self.get_union_type(vec![non_undefined, default_type]))
                        }
                        None => Some(default_type),
                    };
                }
                t
            }
            _ => None,
        }
    }

    // Go getLiteralTypeFromPropertyName（checker.go:28363-28375）+
    // getPropertyNameFromType（getTypeOfDestructuredProperty flow.go:2477-2480）：
    // 键名取值，ComputedPropertyName 求值内表达式取字面量名（[itemId] →
    // 'some-id'，itemId 常量折叠），不可用名恒 None；其余键按静态文本
    //（GetPropertyNameForPropertyNameNode → 字符串字面量型）。静态 text()
    // 对计算名恒空串，属性与索引查找双双落空
    fn destructured_property_key_name(&mut self, element: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("destructured_property_key_name");
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        match &be.property_name {
            Some(pn) if pn.kind == SyntaxKind::ComputedPropertyName => {
                let name_type = self.check_computed_property_name_type(pn);
                use crate::checker::utilities_token_is_identifier_or_keyword::{
                    get_property_name_from_type, is_type_usable_as_property_name,
                };
                if is_type_usable_as_property_name(&name_type) {
                    Some(get_property_name_from_type(&name_type))
                } else {
                    None
                }
            }
            Some(pn) => Some(pn.text().to_string()),
            None => be.name.as_ref().map(|n| n.text().to_string()),
        }
    }

    // Go getTypeOfDestructuredProperty（flow.go:2474-2485）：属性查找失败回落
    // getApplicableIndexInfoForName——字符串索引签名对任意字符串字面量键
    // 生效（{ [s: string]: AB } 解构 [itemId] 键取 AB），noUncheckedIndexedAccess
    // 下按 includeUndefinedInIndexSignature（flow.go:2444-2452）并 missing
    fn destructured_property_type_of(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("destructured_property_type_of");
        if let Some(sym) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&sym));
        }
        let info = self.get_applicable_index_info_for_name(t, name)?;
        let value = info.value_type.clone()?;
        if self.no_unchecked_indexed_access {
            return Some(self.get_union_type(vec![value, self.missing_type()]));
        }
        Some(value)
    }

    pub(crate) fn binding_element_property_name(element: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("binding_element_property_name");
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    pub(crate) fn binding_element_name_node(element: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binding_element_name_node"); 
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        be.property_name.clone().or_else(|| be.name.clone())
    }

    pub(crate) fn in_ambient_declaration_context(&self) -> bool { ::tsox_core::fntrace::enter("in_ambient_declaration_context"); 
        self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file)
    }

    /// Go getApparentType：原始类型装箱显示为全局接口声明型（渲染名 Number 等）
    pub(crate) fn boxed_declared_type_for_display(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("boxed_declared_type_for_display"); 
        use crate::checker::types::TYPE_FLAGS_ENUM_LIKE;
        let name = if t.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::Index
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral | TypeFlags::EnumLiteral)
            || (t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) && !t.flags.intersects(TypeFlags::String))
        {
            "Number"
        } else if t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral) {
            "Boolean"
        } else if t.flags.intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral) {
            "BigInt"
        } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
            "Symbol"
        } else {
            return Arc::clone(t);
        };
        match self.globals.get(name).cloned() {
            Some(sym) => {
                let declared = self.get_declared_type_of_symbol(&sym);
                if declared.symbol.is_some() {
                    declared
                } else {
                    Arc::clone(t)
                }
            }
            None => Arc::clone(t),
        }
    }

    pub(crate) fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> { ::tsox_core::fntrace::enter("binding_element_index"); 
        let NodeData::BindingPattern(data) = &pattern.data else {
            return None;
        };
        data.elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
    }

    pub(crate) fn destructured_array_element_type(
        &mut self,
        parent_type: &Arc<Type>,
        index: usize,
        error_node: Option<&Arc<Node>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("destructured_array_element_type"); 
        if parent_type.is_union() {
            // Go 解构位元素逐联合成分按下标取（联合元组各成分各自取元素），
            // 整体走 array-like 会拍平全部位置的元素
            let mut results: Vec<Arc<Type>> = Vec::new();
            for t in self.constituent_types(parent_type) {
                results.push(self.destructured_array_element_type(&t, index, error_node)?);
            }
            if results.len() == 1 {
                return results.into_iter().next();
            }
            return Some(self.get_union_type(results));
        }
        if self.is_tuple_type(parent_type) {
            return self.get_tuple_element_type(parent_type, index);
        }
        if self.is_array_like_type(parent_type) {
            return Some(self.get_array_element_type(parent_type));
        }
        Some(self.check_iterated_type_or_element_type(
            crate::checker::checker_iteration::IterationUse::Destructuring,
            parent_type,
            error_node,
        ))
    }

    pub(crate) fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("for_in_or_of_statement_of"); 
        let list = decl.parent()?;
        if list.kind != SyntaxKind::VariableDeclarationList {
            return None;
        }
        let stmt = list.parent()?;
        if matches!(
            stmt.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        ) {
            Some(Arc::clone(&stmt))
        } else {
            None
        }
    }

    pub(crate) fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("for_in_expression_of"); 
        let stmt = Self::for_in_or_of_statement_of(decl)?;
        if stmt.kind != SyntaxKind::ForInStatement {
            return None;
        }
        match &stmt.data {
            NodeData::ForInOrOfStatement(d) => Some(Arc::clone(&d.expression)),
            _ => None,
        }
    }

    pub fn get_property_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_property_of_type");
        // Go getPropertyOfTypeEx（checker.go:20019）入口 getReducedApparentType
        // 的类型参数臂：this 型等类型参数经约束映射后取成员。本仓 INSTANTIABLE
        // 不含 Intersection 且交集 apparent 助手与 Go 语义不同，交集仍走下方
        // union/intersection 分支，故此处仅收敛 TypeParameter
        if t.flags.contains(TypeFlags::TypeParameter) {
            let apparent = self.get_apparent_type(t);
            if !Arc::ptr_eq(&apparent, t) {
                return self.get_property_of_type(&apparent, name);
            }
            return None;
        }
        if self.is_no_infer_type(t)
            && let TypeData::Substitution(sub) = &t.data
            && let Some(base) = sub.base_type.clone()
            && !Arc::ptr_eq(&base, t)
        {
            return self.get_property_of_type(&base, name);
        }
        // Go getPropertyOfTypeEx（checker.go:20029）Object 臂先经
        // resolveStructuredTypeMembers 分派（Mapped 臂 resolveMappedTypeMembers）
        // 再按名查找；本仓 resolve_structured_type_members 为恒等，Mapped 壳
        // 成员在此按需解析
        self.resolve_mapped_type_members_if_needed(t);
        if t.is_union() || t.is_intersection() {
            return self.get_property_of_union_or_intersection_type(t, name);
        }
        // Go getPropertyOfTypeEx（checker.go:20019-20022）Mapped 臂经
        // resolveStructuredTypeMembers（checker.go:20185-20208）查询前按需
        // resolveMappedTypeMembers 解析成员，未命中按索引信息合成属性
        //（checker.go:22182-22195）；创建处对游离类型参数约束维持惰性的映射
        //（typenode_template_mapped_checker 创建点 skip）在此现算，经
        // resolved_mapped_members 驻留侧表（环守卫+幂等）；未命中落空即
        // 沿函数尾段继续（与已解析型同径：结构化成员查找、缓存与增补链）
        if matches!(&t.data, TypeData::Mapped(_))
            && !t.object_flags.contains(ObjectFlags::MembersResolved)
        {
            let (members, index_infos) = self.resolved_mapped_members(t);
            if let Some(sym) = members.get(name) {
                return Some(Arc::clone(sym));
            }
            if !self.property_lookup_skips_index_synthesis {
                let numeric = name.parse::<f64>().is_ok();
                for info in &index_infos {
                    let Some(key) = &info.key_type else { continue };
                    let applicable = if numeric {
                        key.flags.contains(TypeFlags::Number)
                    } else {
                        key.flags.contains(TypeFlags::String)
                    };
                    if !applicable {
                        continue;
                    }
                    let vt = info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    return Some(self.synthetic_property_of_type(name, vt));
                }
            }
        }
        // Go globalThisSymbol.Exports 即 globals 表：typeof globalThis 的成员
        // 直接取 globals
        if let Some(sym) = self.global_this_export_of_type(t, name) {
            return Some(sym);
        }
        // 挂起的条件类型（checkType 已具体时）先解析再取成员
        if matches!(&t.data, crate::checker::types::TypeData::Conditional(_))
            && let Some(resolved) = self.resolve_conditional_type(t)
        {
            if !Arc::ptr_eq(&resolved, t) {
                return self.get_property_of_type(&resolved, name);
            }
        }
        // 带实参的接口引用但成员是声明形式或来自其它实参集的克隆（substitute
        // 重建型、跨文件同名接口合并后补入的增广成员）：经实例化重取成员
        //（Go resolveTypeReferenceMembers：实例引用的成员一律来自
        // instantiateSymbolTable(declaredMembers)；成员 owner 与当前型一致才
        // 视为已实例化——按被查成员名判定，不得抽样首成员）
        if let Some(obj) = t.as_object()
            && !obj.type_arguments.is_empty()
            && let Some(sym) = t.symbol.as_ref()
            && sym.flags.contains(SymbolFlags::Interface)
            && obj.structured.members.get(name).is_some_and(|m| {
                !m.check_flags
                    .contains(tsox_frontend::ast::CheckFlags::Instantiated)
                    || self
                        .instantiated_member_owner
                        .get(&(Arc::as_ptr(m) as usize))
                        .is_none_or(|owner| *owner != u64::from(t.id))
            })
        {
            let inst = self.resolve_interface_type_ex(sym, Some(obj.type_arguments.clone()));
            if let Some(member) = inst
                .as_structured()
                .and_then(|s| s.members.get(name).cloned())
            {
                return Some(member);
            }
        }
        // Go getPropertyOfTypeEx：原始类型成员走对应全局接口的声明类型
        //（apparent type 即接口声明型；String/Number 等符号的声明型在此惰性补建）
        // 成员未命中即 Go Object 分支尾 getPropertyOfObjectType(globalObjectType)
        //（checker.go:19248，成员查找语义 checker.go:21724）；原始种别无调用/
        // 构造签名，Function 增补不可达
        if let Some(interface_name) = self.primitive_interface_name(t) {
            let Some(sym) = self.globals.get(interface_name).cloned() else {
                // Go getReducedApparentType（checker.go:20013）：原始种别全局接口
                // 缺失时 apparent type 降为 emptyObjectType，成员查找落 Object
                // 分支尾增补（如无 es2020 lib 时 bigint 的 valueOf/toString）
                let apparent = self.get_apparent_type(t);
                return self.get_property_of_type(&apparent, name);
            };
            let declared = self
                .type_alias_links
                .get(&sym)
                .and_then(|l| l.declared_type.clone())
                .unwrap_or_else(|| self.resolve_interface_type(&sym, None));
            if let Some(member) = declared
                .as_structured()
                .and_then(|s| s.members.get(name).cloned())
            {
                return Some(member);
            }
            if let Some(member) = self.global_object_member(name) {
                return Some(member);
            }
        }
        // Go createTupleTargetType：无 rest/variadic 元素的元组 length 成员
        // 是固定长度的数字字面量（判别式收窄依赖它），带 rest 回落 number
        if let crate::checker::types::TypeData::Tuple(tuple) = &t.data
            && name == "length"
            && !tuple
                .element_infos
                .iter()
                .any(|e| e.flags.intersects(crate::checker::types::ELEMENT_FLAGS_VARIABLE))
        {
            let len_type = self.get_number_literal_type(tsox_core::jsnum::Number::from(
                tuple.fixed_length as f64,
            ));
            return Some(self.synthetic_property_of_type(name, len_type));
        }
        // Go getBaseTypes→getTupleBaseType：元组结构化成员经
        // Array/ReadonlyArray<元素并集> 基类型实例化解析，声明型泛型参数
        // 不得外泄（inferFromProperties 会以外层类型参数污染候选集）
        if let crate::checker::types::TypeData::Tuple(tuple) = &t.data
            && let Some(member) = self.tuple_base_type_member(t, tuple, name)
        {
            return Some(member);
        }
        // Go getPropertyOfTypeEx（checker.go:20022-20052）：自有成员先行；未命中
        // 按签名种类增补 Function 族——call 签名取 globalCallableFunctionType、
        // construct 签名取 globalNewableFunctionType（strictBindCallApply 时为
        // CallableFunction/NewableFunction，否则 Function）。strict 下 `apply`
        // 的泛型重载来自 CallableFunction 声明，直接取 Function 接口会遮蔽该
        // 增补（Go 只对 anyFunctionType 特判 Function，checker.go:20040-20041）
        if let Some(structured) = t.as_structured()
            && let Some(sym) = structured.members.get(name)
        {
            return Some(Arc::clone(sym));
        }
        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Anonymous)
        {
            let augment = if !self.get_signatures_of_type(t, SignatureKind::Call).is_empty() {
                self.global_callable_function_type()
            } else if !self
                .get_signatures_of_type(t, SignatureKind::Construct)
                .is_empty()
            {
                self.global_newable_function_type()
            } else {
                None
            };
            if let Some(member) = augment
                .as_ref()
                .and_then(|ft| ft.as_structured())
                .and_then(|s| s.members.get(name))
                .cloned()
            {
                return Some(member);
            }
        }
        if let Some(sym) = self.get_property_of_type_cached(t, name) {
            return Some(sym);
        }
        // Go 元组的数字名成员来自位置元素符号（createTupleTargetType 的
        // "0"/"1" 成员），不可落到索引签名合成（那是全部元素的并集）
        if let crate::checker::types::TypeData::Tuple(_) = &t.data
            && let Ok(index) = name.parse::<usize>()
        {
            if let Some(elem) = self.get_tuple_element_type(t, index) {
                return Some(self.synthetic_property_of_type(name, elem));
            }
        }
        // Go getPropertyOfObjectType：声明成员未命中时按索引签名合成属性
        //（数字名配 number 索引，非数字名配 string 索引）
        if !self.property_lookup_skips_index_synthesis
            && let Some(structured) = t.as_structured()
            && structured.members.get(name).is_none()
        {
            let numeric = name.parse::<f64>().is_ok();
            for info in &structured.index_infos {
                let Some(key) = &info.key_type else { continue };
                let applicable = if numeric {
                    key.flags.contains(TypeFlags::Number)
                } else {
                    key.flags.contains(TypeFlags::String)
                };
                if !applicable {
                    continue;
                }
                let vt = info
                    .value_type
                    .clone()
                    .unwrap_or_else(|| self.any_type());
                return Some(self.synthetic_property_of_type(name, vt));
            }
        }
        if let Some(interface_sym) = self.unresolved_interface_symbol_of(t) {
            // 壳（自引用/实例化重建）：按 owner 的 type_arguments 取实例成员符号，
            // 成员类型已在实例中替换；实例仍为空壳时视为未解析（防重入循环）
            let args = t.as_object().map(|o| o.type_arguments.clone());
            let inst = self.resolve_interface_type_ex(&interface_sym, args);
            if let Some(member) = inst
                .as_structured()
                .and_then(|s| s.members.get(name))
                .cloned()
            {
                return Some(member);
            }
        }
        // 类型参数（含多态 this）成员经约束解析；Go getReducedApparentType 在
        // getPropertyOfTypeEx 开头完成映射：有约束取约束，无约束时非 strict
        // 映射 emptyObjectType，strict 保持 unknown 返回 nil
        if let crate::checker::types::TypeData::TypeParameter(tp) = &t.data {
            return match tp
                .constraint
                .clone()
                .filter(|c| !c.flags.contains(TypeFlags::Unknown))
            {
                Some(constraint) => self.get_property_of_type(&constraint, name),
                None => {
                    if self.strict_null_checks {
                        None
                    } else {
                        self.get_property_of_type(&self.empty_object_type(), name)
                    }
                }
            };
        }
        // Go getApparentType（checker.go:22094-22103）：Object 分支前置的种别
        // 映射——StringLike（模板字面量/字符串映射）、NonPrimitive、Index、
        // 非 strict 的 unknown 先行映射，再落下方 Object 分支增补
        if t.flags
            .intersects(TypeFlags::TemplateLiteral | TypeFlags::StringMapping)
        {
            let mapped = self.global_string_type();
            return self.get_property_of_type(&mapped, name);
        }
        if t.flags.contains(TypeFlags::NonPrimitive) {
            return self.get_property_of_type(&self.empty_object_type(), name);
        }
        if t.flags.contains(TypeFlags::Index) {
            let mapped = self.string_number_symbol_type();
            return self.get_property_of_type(&mapped, name);
        }
        if t.flags.contains(TypeFlags::Unknown) && !self.strict_null_checks {
            return self.get_property_of_type(&self.empty_object_type(), name);
        }
        // Go getPropertyOfTypeEx：Function/Callable/Newable 与全局 Object 接口
        // 的成员增补仅在 TypeFlagsObject 分支内；其余类型种别（unknown、未解析
        // 条件型、替换型等）一律 nil，缺失属性报告由此落到 reportNonexistentProperty
        if t.flags.contains(crate::checker::types::TypeFlags::Object) {
            let call_sigs = self.get_signatures_of_type(t, SignatureKind::Call);
            let construct_sigs = if call_sigs.is_empty() {
                self.get_signatures_of_type(t, SignatureKind::Construct)
            } else {
                Vec::new()
            };
            let augment_type = if self.any_function_type.get().is_some_and(|f| Arc::ptr_eq(f, t)) {
                self.global_function_type_of("Function")
            } else if !call_sigs.is_empty() {
                self.global_callable_function_type()
            } else if !construct_sigs.is_empty() {
                self.global_newable_function_type()
            } else {
                None
            };
            if let Some(ft) = augment_type
                && let Some(member) = self.get_property_of_type(&ft, name)
            {
                return Some(member);
            }
            // Go getPropertyOfObjectType(globalObjectType)：普通成员未命中时回退
            // 全局 Object 接口成员（对象字面量查 toString 等由此命中）
            return self.global_object_member(name);
        }
        None
    }

    fn global_object_member(&mut self, name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("global_object_member"); 
        let obj_sym = self.globals.get("Object").cloned()?;
        let obj_type = self.get_declared_type_of_symbol(&obj_sym);
        obj_type
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    fn tuple_base_type_member(
        &mut self,
        t: &Arc<Type>,
        tuple: &crate::checker::types::TupleTypeData,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("tuple_base_type_member");
        let mut elements: Vec<Arc<Type>> = Vec::new();
        for e in &tuple.element_infos {
            let Some(ty) = &e.type_ else { continue };
            if e.flags.intersects(
                crate::checker::types::ElementFlags::Rest
                    | crate::checker::types::ElementFlags::Variadic,
            ) && self.is_array_type(ty)
            {
                elements.push(self.get_array_element_type(ty));
            } else {
                elements.push(Arc::clone(ty));
            }
        }
        let union = if elements.is_empty() {
            self.never_type()
        } else {
            self.get_union_type(elements)
        };
        let iface = if tuple.readonly {
            "ReadonlyArray"
        } else {
            "Array"
        };
        let sym = self.globals.get(iface).cloned()?;
        let base = self.resolve_interface_type_ex(&sym, Some(vec![union]));
        let member = base
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())?;
        // 收窄门：逐引用克隆仅在「同 (接口,实参) 驻留键已命中第二个 tuple
        // reference」的多元组场景激活（base 为该驻留键的共享实例，base.id 即
        // 驻留键身份），单引用场景维持共享驻留路径，波及面压到
        // sliceResultCast 形态
        let multi_tuple = if let Some(&first) = self.tuple_base_member_hits.get(&base.id) {
            first != t.id
        } else {
            self.tuple_base_member_hits.insert(base.id, t.id);
            false
        };
        if !multi_tuple {
            return Some(member);
        }
        // Go resolveTypeReferenceMembers（checker.go:20228-20242）：多元组场景下
        // 继承成员逐引用实例化——getTypeWithThisArgument（checker.go:20276-20278）
        // 以引用自身为 this 实参令两个元组（元素并集归约后相同，unionTypes 驻留
        // 共享同一并集对象）仍各得独立基类型实例，instantiateSymbolTable 每引用
        // 产出新符号对象，createUnionOrIntersectionProperty（checker.go:22744-
        // 22753）的 propSet 收得两个符号（compareTypesEqual checker.go:29340-
        // 29345 仅指针判等）呈双签名并集。按元组类型身份克隆：同元组幂等驻留、
        // 异元组各异；成员型经 instantiate_array_member_type 以克隆符号为键
        // （(元素并集指针, 克隆符号指针)）逐元组实例化，不触碰共享成员符号的
        // 既有缓存键
        let key = (t.id, name.to_string());
        if let Some(cached) = self.tuple_base_member_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        let mut result =
            Symbol::new(member.flags | SymbolFlags::Transient, member.name.clone());
        result.check_flags = tsox_frontend::ast::CheckFlags::Instantiated
            | member.check_flags
                & (tsox_frontend::ast::CheckFlags::Readonly
                    | tsox_frontend::ast::CheckFlags::Late
                    | tsox_frontend::ast::CheckFlags::OptionalParameter
                    | tsox_frontend::ast::CheckFlags::RestParameter);
        result.declarations = member.declarations.clone();
        if let Some(parent) = member.parent() {
            result.set_parent(&parent);
        }
        result.value_declaration = member.value_declaration.clone();
        let result = Arc::new(result);
        let member_type = self
            .instantiate_array_member_type(t, &result)
            .unwrap_or_else(|| self.get_type_of_symbol(&member));
        let result_links = self.value_symbol_links.get_or_default(&result);
        result_links.target = Some(Arc::clone(&member));
        result_links.resolved_type = Some(member_type);
        self.tuple_base_member_cache
            .insert(key, Arc::clone(&result));
        Some(result)
    }

    pub(crate) fn unresolved_interface_symbol_of(&self, t: &Arc<Type>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("unresolved_interface_symbol_of"); 
        if !t.flags.contains(crate::checker::types::TypeFlags::Object) {
            return None;
        }
        let sym = t.symbol.as_ref()?;
        let has_interface_decl = sym
            .declarations
            .iter()
            .any(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)));
        if !has_interface_decl {
            return None;
        }
        if let Some(structured) = t.as_structured()
            && !structured.members.entries.is_empty()
        {
            return None;
        }
        Some(Arc::clone(sym))
    }

    // 索引签名命中的合成属性（Go getPropertySymbolForIndexInfo）
    fn synthetic_property_of_type(&mut self, name: &str, t: Arc<Type>) -> Arc<Symbol> { ::tsox_core::fntrace::enter("synthetic_property_of_type"); 
        let sym = Arc::new(Symbol::new(SymbolFlags::Property, name.to_string()));
        self.value_symbol_links.insert(
            &sym,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(t),
                ..Default::default()
            },
        );
        sym
    }
}


fn type_contains_widening_member(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_contains_widening_member");
    if t
        .object_flags
        .intersects(crate::checker::types::OBJECT_FLAGS_REQUIRES_WIDENING)
    {
        return true;
    }
    if let TypeData::Union(u) = &t.data {
        return u
            .union_or_intersection
            .types
            .iter()
            .any(type_contains_widening_member);
    }
    false
}
