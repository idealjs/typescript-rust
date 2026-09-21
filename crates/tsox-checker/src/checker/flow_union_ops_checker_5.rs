#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;

impl Checker {
    pub(crate) fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> {
        if evolving {
            return Arc::clone(assigned);
        }

        if declared.flags.contains(TypeFlags::Null)
            && (self.is_auto_array_type(assigned)
                || assigned.object_flags.contains(ObjectFlags::EvolvingArray))
        {
            return Arc::clone(assigned);
        }
        if !declared.is_union() {
            return Arc::clone(declared);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    pub(crate) fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> {
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
    ) -> bool {
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

    pub(crate) fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {
                    if matches!(
                        &init.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        return Some(self.auto_array_type());
                    }
                    if matches!(
                        init.kind,
                        tsox_frontend::ast::SyntaxKind::NullKeyword
                            | tsox_frontend::ast::SyntaxKind::UndefinedKeyword
                    ) {
                        return Some(self.auto_type());
                    }
                    let t = self.get_type_of_node(init);
                    // Go widenTypeInferredFromInitializer：可变变量初始化式含 widening
                    // 成员时拓宽（如推断 T | undefinedWidening → any）
                    if type_contains_widening_member(&t) {
                        return Some(self.get_widened_type(&t));
                    }
                    return Some(t);
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
                let parent_type = self.initial_type_of_declaration(&pattern_parent);
                let diagnostics_allowed = !self.in_ambient_declaration_context();
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        // rest 元素：剩余属性对象，不做属性查找、不产生 TS2339
                        if be.dot_dot_dot_token.is_some() {
                            return Some(self.rest_element_type(expr, &parent_type));
                        }
                        match Self::binding_element_property_name(expr) {
                            Some(name) => {
                                let t = self.get_property_type_of_type(parent_type, &name);
                                if t.is_none()
                                    && diagnostics_allowed
                                    && !parent_type.flags.intersects(
                                        TypeFlags::Any
                                            | TypeFlags::Unknown
                                            | TypeFlags::Never,
                                    )
                                    && !crate::checker::utilities::is_type_error(parent_type)
                                {
                                    let display =
                                        self.boxed_declared_type_for_display(parent_type);
                                    let type_str = self.type_to_string(&display);
                                    let name_node = Self::binding_element_name_node(expr);
                                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        name_node.unwrap_or_else(|| Arc::clone(&pattern)).loc,
                                        tsox_core::diagnostics::messages_generated::
                                            PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                                        vec![name.clone(), type_str],
                                    ));
                                }
                                t
                            }
                            None => None,
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

    pub(crate) fn binding_element_property_name(element: &Arc<Node>) -> Option<String> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    pub(crate) fn binding_element_name_node(element: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        be.property_name.clone().or_else(|| be.name.clone())
    }

    pub(crate) fn in_ambient_declaration_context(&self) -> bool {
        self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file)
    }

    /// Go getApparentType：原始类型装箱显示为全局接口声明型（渲染名 Number 等）
    pub(crate) fn boxed_declared_type_for_display(&mut self, t: &Arc<Type>) -> Arc<Type> {
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

    pub(crate) fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
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
    ) -> Option<Arc<Type>> {
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

    pub(crate) fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
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

    pub(crate) fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
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
    ) -> Option<Arc<Symbol>> {
        if t.is_union() || t.is_intersection() {
            return self.get_property_of_union_or_intersection_type(t, name);
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
        // 重建型）：经实例化重取成员（Go 结构化成员解析：实例引用的成员来自
        // 实例化 target；成员 owner 与当前型一致才视为已实例化）
        if let Some(obj) = t.as_object()
            && !obj.type_arguments.is_empty()
            && let Some(sym) = t.symbol.as_ref()
            && sym.flags.contains(SymbolFlags::Interface)
            && !obj.structured.members.entries.is_empty()
            && obj
                .structured
                .members
                .entries
                .values()
                .next()
                .is_some_and(|m| {
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
        if let Some(interface_name) = self.primitive_interface_name(t)
            && let Some(sym) = self.globals.get(interface_name).cloned()
        {
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
        // 类型参数（含多态 this）成员经约束解析（tsc resolveStructuredTypeMembers 语义）
        if let crate::checker::types::TypeData::TypeParameter(tp) = &t.data {
            if let Some(constraint) = tp.constraint.clone()
                && !constraint.flags.contains(TypeFlags::Unknown)
            {
                let member = self.get_property_of_type(&constraint, name);
                if member.is_some() {
                    return member;
                }
            }
            if self.strict_null_checks {
                return None;
            }
        }
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
        // Go getPropertyOfTypeEx：普通成员未命中时回退全局 Object 接口成员
        //（对象字面量查 toString 等由此命中，缺失属性报告因此不含 Object 原型成员）
        self.global_object_member(name)
    }

    fn global_object_member(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let obj_sym = self.globals.get("Object").cloned()?;
        let obj_type = self.get_declared_type_of_symbol(&obj_sym);
        obj_type
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    pub(crate) fn unresolved_interface_symbol_of(&self, t: &Arc<Type>) -> Option<Arc<Symbol>> {
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
    fn synthetic_property_of_type(&mut self, name: &str, t: Arc<Type>) -> Arc<Symbol> {
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


fn type_contains_widening_member(t: &Arc<Type>) -> bool {
    if t
        .object_flags
        .intersects(crate::checker::types::ObjectFlags::ContainsWideningType)
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
