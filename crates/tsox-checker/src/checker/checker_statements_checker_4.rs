#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub(crate) fn check_variable_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_variable_declaration(node);
        self.check_exports_on_merged_declarations(node);
        if let Some(name) = node.name() {
            self.check_cjs_reserved_top_level_name(node, &name);
        }
        if let tsox_frontend::ast::NodeData::VariableDeclaration(data) = &node.data {
            if data.initializer.is_none() {
                let is_const = node
                    .parent()
                    .as_ref()
                    .is_some_and(|list| list.flags.contains(NodeFlags::Const));
                let in_for_in_of = node
                    .parent()
                    .as_ref()
                    .and_then(|l| l.parent())
                    .is_some_and(|g| {
                        matches!(
                            g.kind,
                            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                        )
                    });
                let is_ambient = self.ambient_context_depth > 0
                    || node.flags.contains(NodeFlags::Ambient)
                    || node
                        .parent()
                        .as_ref()
                        .and_then(|p| p.parent())
                        .is_some_and(|stmt| stmt.has_syntactic_modifier(ModifierFlags::Ambient))
                    || {
                        let mut anc = node.parent();
                        let mut found = false;
                        while let Some(a) = anc {
                            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                                found = true;
                                break;
                            }
                            anc = a.parent();
                        }
                        found
                    }
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if is_ambient
                    && self.no_implicit_any
                    && data.type_node.is_none()
                    && data.name.kind == SyntaxKind::Identifier
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        data.name.loc,
                        tsox_core::diagnostics::messages_generated::
                            VARIABLE_0_IMPLICITLY_HAS_AN_1_TYPE,
                        vec![data.name.text().to_string(), "any".to_string()],
                    ));
                }
                if is_const
                    && !in_for_in_of
                    && !is_ambient
                    && !tsox_frontend::ast::node_data_generated::is_binding_pattern(&data.name)
                    && !self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.has_parse_diagnostics)
                {
                    let file = self.current_file.clone();
                    let name_loc = data.name.loc;
                    let already = self.diagnostics.get_all().iter().any(|d| {
                        d.code == 1155
                            && d.loc.pos() == name_loc.pos()
                            && d.file.as_ref().map(|f| f.file_name.as_str())
                                == file.as_ref().map(|f| f.file_name.as_str())
                    });
                    if !already {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            vec!["const".to_string()],
                        ));
                    }
                }
            }

            if data.name.kind == SyntaxKind::Identifier {
                let list_is_var = node.parent().as_ref().is_none_or(|l| {
                    !(l.flags.contains(NodeFlags::Let) || l.flags.contains(NodeFlags::Const))
                });
                let is_param = node
                    .parent()
                    .as_ref()
                    .is_some_and(|l| l.kind == SyntaxKind::Parameter);
                if list_is_var && !is_param {
                    let own = self.program.symbol_map().symbol_of(node).cloned();
                    if let Some(local) = self.resolve_identifier(&data.name)
                        && own.as_ref().is_none_or(|o| !Arc::ptr_eq(o, &local))
                        && local.flags.contains(SymbolFlags::BlockScopedVariable)
                        && let Some(vd) = local.value_declaration.clone()
                        && vd.kind == SyntaxKind::VariableDeclaration
                        && let Some(list) = vd.parent().as_ref()
                        && list.kind == SyntaxKind::VariableDeclarationList
                    {
                        // Go：container 仅在 VariableStatement 路径取（for-in/of 头
                        // 声明不在 VariableStatement 下 → container 为空 → 必报）
                        let container = list
                            .parent()
                            .and_then(|s| {
                                (s.kind == SyntaxKind::VariableStatement).then(|| s.clone())
                            })
                            .and_then(|s| s.parent());
                        let names_share_scope = container.is_some_and(|c| {
                            c.kind == SyntaxKind::ModuleBlock
                                || c.kind == SyntaxKind::ModuleDeclaration
                                || c.kind == SyntaxKind::SourceFile
                                || (c.kind == SyntaxKind::Block
                                    && c.parent().as_ref().is_some_and(|p| {
                                        matches!(
                                            p.kind,
                                            SyntaxKind::FunctionDeclaration
                                                | SyntaxKind::FunctionExpression
                                                | SyntaxKind::ArrowFunction
                                                | SyntaxKind::MethodDeclaration
                                                | SyntaxKind::Constructor
                                                | SyntaxKind::GetAccessor
                                                | SyntaxKind::SetAccessor
                                        )
                                    }))
                        });
                        if !names_share_scope {
                            let name_text = data.name.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                node.loc,
                                tsox_core::diagnostics::messages_generated::
                                    CANNOT_INITIALIZE_OUTER_SCOPED_VARIABLE_0_IN_THE_SAME_SCOPE_AS_BLOCK_SCOPED_DECLARATION_1,
                                vec![name_text.clone(), name_text],
                            ));
                        }
                    }
                }
            }

            // Go checkVariableLikeDeclaration：数组 binding pattern 无命名元素时按
            // widened 类型做迭代检查，元素类型为 never 报 TS2488；ambient 上下文跳过
            let in_ambient = self.ambient_context_depth > 0 || node.flags.contains(NodeFlags::Ambient);
            if !in_ambient && data.name.kind == SyntaxKind::ArrayBindingPattern {
                let has_named_element = match &data.name.data {
                    tsox_frontend::ast::NodeData::BindingPattern(bp) => bp
                        .elements
                        .nodes
                        .iter()
                        .any(|e| {
                            matches!(&e.data, tsox_frontend::ast::NodeData::BindingElement(be) if be.name.is_some())
                        }),
                    _ => false,
                };
                if !has_named_element
                    && let Some(widened) = self.initial_type_of_declaration(node)
                    && widened.flags.contains(TypeFlags::Never)
                {
                    let type_str = self.type_to_string(&widened);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        data.name.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR,
                        vec![type_str],
                    ));
                }
            }

            self.check_binding_pattern_computed_names(&data.name);
            self.check_binding_pattern_element_initializers(&data.name);

            if data.name.kind == SyntaxKind::ObjectBindingPattern
                && self.in_ctor_body_stack.last() == Some(&true)
                && let Some(init) = &data.initializer
                && init.kind == SyntaxKind::ThisKeyword
            {
                let this_type = self.get_type_of_node(init);
                self.check_this_destructuring_abstract_properties(&data.name, &this_type);
            }
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }

            let resolved_type = match (&data.type_node, &data.initializer) {
                (Some(type_node), Some(init)) => {
                    let annotation_type = self.get_type_from_type_node(type_node);

                    if init.kind == SyntaxKind::ArrayLiteralExpression {
                        let at = Arc::clone(&annotation_type);
                        self.check_contextual_elements(init, &at, init.loc);
                    }
                    let init_type = self.get_type_of_node(init);
                    let assignable = self.is_type_assignable_to(&init_type, &annotation_type);
                    let mut reported_error = false;

                    let mut init_node: &Arc<Node> = init;
                    while init_node.kind == SyntaxKind::ParenthesizedExpression {
                        let inner = match &init_node.data {
                            tsox_frontend::ast::NodeData::ParenthesizedExpression(p) => {
                                Some(&p.expression)
                            }
                            _ => None,
                        };
                        match inner {
                            Some(i) => init_node = i,
                            None => break,
                        }
                    }
                    if init_node.kind == SyntaxKind::ObjectLiteralExpression
                        && let Some(excess_name) =
                            self.get_excess_property_name(&init_type, &annotation_type)
                    {
                        let file = self.current_file.clone();
                        let filtered_annot = self.excess_check_error_target(&annotation_type);
                        let annot_str = self.type_to_string(&filtered_annot);

                        let loc = self
                            .find_object_literal_property_name_node(init, &excess_name)
                            .unwrap_or(node.loc);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            loc,
                            OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                            vec![
                                crate::checker::property_name_for_display(&excess_name),
                                annot_str,
                            ],
                        ));
                        reported_error = true;
                    }

                    if !assignable && !reported_error {
                        let name_node = Arc::clone(&data.name);
                        self.check_type_assignable_to_and_optionally_elaborate(
                            &init_type,
                            &annotation_type,
                            Some(&name_node),
                            Some(init),
                            None,
                            None,
                        );
                    }
                    annotation_type
                }
                (Some(type_node), None) => {
                    // Go checkVariableDeclaration：注解类型走完整检查
                    //（计算名 TS2304/TS2464 等）
                    self.check_type_annotation(type_node);
                    self.get_type_from_type_node(type_node)
                }
                (None, Some(init)) => {
                    if data.name.kind == SyntaxKind::ObjectBindingPattern && !in_ambient {
                        self.check_binding_pattern_initializer_excess(&data.name, init);
                    }
                    if data.name.kind == SyntaxKind::ArrayBindingPattern {
                        let init_type = if init.kind == SyntaxKind::Identifier
                            && let Some(sym) = self.resolve_identifier(init)
                        {
                            let flow = self.program.symbol_map().flow_node_of(init).map(Arc::clone);
                            self.get_narrowed_type_of_symbol(&sym, flow.as_ref())
                        } else {
                            self.get_type_of_node(init)
                        };
                        if init_type.flags.contains(TypeFlags::Never) {
                            let type_str = self.type_to_string(&init_type);
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                data.name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR,
                                vec![type_str],
                            ));
                        }
                    }

                    let is_const_decl = self
                        .get_combined_node_flags(node)
                        .intersects(NodeFlags::Constant);
                    if !is_const_decl
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        self.auto_type()
                    } else if self.is_empty_array_literal(init) {
                        self.auto_array_type()
                    } else {
                        let init_type = self.get_type_of_node(init);
                        // 自引用初始化式（var a = { f: a }）：环期成员类型为
                        // in-flight error 时整体回退 any（Go 循环初始化的隐式 any）
                        let circular = init_type.as_structured().is_some_and(|s| {
                            s.properties.iter().any(|p| {
                                self.value_symbol_links
                                    .get(p)
                                    .and_then(|l| l.resolved_type.clone())
                                    .is_some_and(|t| {
                                        crate::checker::utilities::is_type_error(&t)
                                    })
                            })
                        });
                        if circular {
                            self.get_any_type()
                        } else {
                            let widened_literal =
                                self.get_widened_literal_type_for_initializer(node, &init_type);
                            let regularized =
                                self.get_regular_type_of_literal_type(&widened_literal);
                            self.widen_initializer_type(&regularized)
                        }
                    }
                }
                (None, None) => match self.initial_type_of_declaration(node) {
                    Some(t) => t,
                    None => self.auto_type(),
                },
            };

            if let Some(mut symbol) = self.resolve_identifier(&data.name) {
                // Go createGlobals：script 文件顶层声明并入全局符号（跨文件
                // 合并视图挂在 globals 表中的首个文件符号上）
                let at_script_top_level = node
                    .parent()
                    .and_then(|l| l.parent())
                    .is_some_and(|g| g.kind == SyntaxKind::VariableStatement)
                    && node
                        .parent()
                        .and_then(|l| l.parent())
                        .and_then(|s| s.parent())
                        .is_some_and(|sf| sf.kind == SyntaxKind::SourceFile)
                    && self
                        .current_file
                        .as_ref()
                        .is_none_or(|f| f.external_module_indicator.is_none());
                if at_script_top_level
                    && let Some(merged) = self.globals.get(data.name.text())
                    && merged
                        .declarations
                        .iter()
                        .any(|d| Arc::ptr_eq(d, node))
                {
                    symbol = Arc::clone(merged);
                }
                // Go 分表语义：exports 与 locals 各有独立 ValueDeclaration，
                // 仅同导出性的首个声明（任意 variable-like 形态，含参数）
                // 参与二级声明类型一致性比较
                let node_exported = self.effective_export_default_flags(node).0;
                let primary = symbol
                    .declarations
                    .iter()
                    .find(|d| self.effective_export_default_flags(d).0 == node_exported)
                    .cloned();
                if let Some(primary) = primary
                    && !Arc::ptr_eq(&primary, node)
                    && symbol.declarations.len() > 1
                    && symbol.flags.intersects(
                        SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable,
                    )
                    && !symbol.flags.intersects(SymbolFlags::Assignment)
                {
                    let primary_type_raw = self.get_type_of_symbol(&symbol);
                    // Go getTypeForVariableLikeDeclaration(includeOptionality=true)：
                    // 可选参数的符号类型在 strictNullChecks 下补 | undefined
                    let primary_type_raw = if primary.kind == SyntaxKind::Parameter
                        && self.strict_null_checks
                        && matches!(
                            &primary.data,
                            tsox_frontend::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some()
                        ) {
                        self.get_union_type(vec![primary_type_raw, self.undefined_type()])
                    } else {
                        primary_type_raw
                    };
                    fn auto_to_any(c: &Checker, t: &Arc<Type>) -> Arc<Type> {
                        if t.intrinsic_name() == Some("auto") {
                            c.get_any_type()
                        } else {
                            Arc::clone(t)
                        }
                    }
                    let primary_type = auto_to_any(self, &primary_type_raw);
                    let this_type = auto_to_any(self, &resolved_type);
                    if !matches!(primary_type.intrinsic_name(), Some("error"))
                        && !matches!(this_type.intrinsic_name(), Some("error"))
                        && !self
                            .compare_types_identical(&primary_type, &this_type)
                            .is_true()
                    {
                        let name_text = data.name.text().to_string();
                        let first_str = self.type_to_string(&primary_type);
                        let next_str = self.type_to_string(&this_type);
                        let file = self.current_file.clone();
                        let mut diag = tsox_frontend::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            tsox_core::diagnostics::messages_generated::
                                SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_VARIABLE_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                            vec![name_text.clone(), first_str, next_str],
                        );
                        diag.related_information.push(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            primary.loc,
                            tsox_core::diagnostics::messages_generated::X_0_WAS_ALSO_DECLARED_HERE,
                            vec![name_text],
                        ));
                        self.diagnostics.add(diag);
                    }
                }
            }

            let resolved_type = self
                .attach_expando_if_fn_initialized(node, resolved_type);

            self.type_node_links.get_or_default(node).resolved_type = Some(resolved_type.clone());

            self.type_node_links
                .get_or_default(&data.name)
                .resolved_type = Some(resolved_type.clone());

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                // Go getTypeOfVariableOrParameterOrPropertyWorker：符号类型取
                // value_declaration（首声明）计算，二级 var 声明不覆盖符号类型
                let is_primary = symbol
                    .value_declaration
                    .as_ref()
                    .is_none_or(|vd| Arc::ptr_eq(vd, node));
                if is_primary {
                    self.value_symbol_links
                        .get_or_default(&symbol)
                        .resolved_type = Some(resolved_type);
                }
            }

            // Go checkVariableLikeDeclaration：绑定模式名逐元素急切解析
            //（TS2339/TS2488 等在声明检查期发出）
            if matches!(
                data.name.kind,
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
            ) {
                self.check_binding_pattern_element_types(&data.name);
            }
        }
    }

    pub(crate) fn check_binding_pattern_element_initializers(&mut self, pattern: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::BindingPattern(bp) = &pattern.data else {
            return;
        };
        for element in bp.elements.iter() {
            if let tsox_frontend::ast::NodeData::BindingElement(be) = &element.data {
                if let Some(default) = &be.initializer {
                    self.check_expression(default);
                }
                if let Some(inner) = &be.name
                    && matches!(
                        inner.kind,
                        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                    )
                {
                    self.check_binding_pattern_element_initializers(inner);
                }
            }
        }
    }

    pub(crate) fn check_binding_pattern_element_types(&mut self, pattern: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::BindingPattern(bp) = &pattern.data else {
            return;
        };
        for element in bp.elements.iter() {
            if let tsox_frontend::ast::NodeData::BindingElement(be) = &element.data {
                // rest 元素类型是“剩余属性”对象，不做属性查找，也不产生 TS2339
                if be.dot_dot_dot_token.is_some() {
                    continue;
                }
                if let Some(name) = &be.name
                    && matches!(
                        name.kind,
                        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                    )
                {
                    self.check_binding_pattern_element_types(name);
                }
            }
            let _ = self.binding_element_type(&element);
        }
    }

    pub(crate) fn check_case_clause(
        &mut self,
        node: &Arc<Node>,
        switch_expression_type: &Arc<Type>,
    ) {
        if let tsox_frontend::ast::NodeData::CaseOrDefaultClause(data) = &node.data {
            if data.expression.kind != SyntaxKind::UnknownKeyword {
                self.check_expression(&data.expression);
                if node.kind == SyntaxKind::CaseClause {
                    let case_type = self.get_type_of_node(&data.expression);
                    if !self.is_type_equality_comparable_to(switch_expression_type, &case_type) {
                        self.check_type_related_to_and_optionally_elaborate(
                            &case_type,
                            switch_expression_type,
                            crate::checker::relater::RelationKind::Comparable,
                            Some(&data.expression),
                            None,
                            Some(
                                &tsox_core::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_COMPARABLE_TO_TYPE_1,
                            ),
                            None,
                        );
                    }
                }
            }
            for stmt in data.statements.iter() {
                self.check_statement(stmt);
            }
        }
    }

    fn is_type_equality_comparable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        target.flags.contains(TYPE_FLAGS_NULLABLE) || self.is_type_comparable_to(source, target)
    }

    pub(crate) fn attach_expando_if_fn_initialized(
        &mut self,
        node: &Arc<Node>,
        t: Arc<Type>,
    ) -> Arc<Type> {
        let is_fn_init = matches!(
            &node.data,
            tsox_frontend::ast::NodeData::VariableDeclaration(d)
                if d.initializer.as_ref().is_some_and(|i| matches!(
                    i.kind,
                    SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
                ))
        );
        if !is_fn_init {
            return t;
        }
        let name = node.name();
        let Some(name) = name else {
            return t;
        };
        match self.resolve_identifier(&name) {
            Some(symbol) if !symbol.exports.is_empty() => {
                self.attach_function_expando_type(&symbol, t)
            }
            _ => t,
        }
    }
}
