#![allow(unused_imports)]

use crate::checker::checker_statements::*;
use tsox_core::core::tristate::Tristate;
use tsox_core::diagnostics::Category;

impl Checker {
    pub fn check_statement(&mut self, node: &Arc<Node>) {
        // Go checkSourceElement：within_unreachable_code 按语句子树保存/恢复
        let saved_within_unreachable = self.within_unreachable_code;
        if !self.within_unreachable_code
            && !matches!(self.allow_unreachable_code, Tristate::True)
            && self.check_source_element_unreachable(node)
        {
            self.within_unreachable_code = true;
        }
        self.check_statement_inner(node);
        self.within_unreachable_code = saved_within_unreachable;
    }

    fn check_statement_inner(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;

        if self.ambient_context_depth > 0
            && !matches!(
                node.kind,
                SyntaxKind::VariableStatement
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::ImportDeclaration
                    | SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::ExportDeclaration
                    | SyntaxKind::ExportAssignment
                    | SyntaxKind::NamespaceExportDeclaration
            )
            && node.parent().as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::SourceFile
                )
            })
            && !Self::inside_function_body(node)
        {
            let block_id = node.parent().as_ref().unwrap().id();
            if !self.ambient_ts1036_reported_blocks.contains(&block_id) {
                self.ambient_ts1036_reported_blocks.insert(block_id);
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                    Vec::new(),
                ));
            }
        }
        match node.kind {
            SyntaxKind::ExpressionStatement => {
                if let tsox_frontend::ast::NodeData::ExpressionStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::VariableStatement => {
                if let tsox_frontend::ast::NodeData::VariableStatement(data) = &node.data {
                    self.check_grammar_modifiers(node);
                    let list_has_grammar_error =
                        self.check_grammar_variable_declaration_list(&data.declaration_list);
                    if !list_has_grammar_error {
                        self.check_grammar_for_disallowed_block_scoped_variable_statement(node);
                    }
                    self.check_variable_declaration_list(&data.declaration_list);

                    if let tsox_frontend::ast::NodeData::VariableDeclarationList(list) =
                        &data.declaration_list.data
                    {
                        let decls = list.declarations.clone();
                        for d in decls.iter() {
                            if let tsox_frontend::ast::NodeData::VariableDeclaration(vd) = &d.data {
                                self.check_cjs_reserved_top_level_name(d, &vd.name);
                            }
                        }
                    }

                    self.check_declaration_nameability(node);
                }
            }
            SyntaxKind::IfStatement => {
                if let tsox_frontend::ast::NodeData::IfStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.check_statement(&data.then_statement);
                    if let Some(else_stmt) = &data.else_statement {
                        self.check_statement(else_stmt);
                    }
                }
            }
            SyntaxKind::WhileStatement => {
                if let tsox_frontend::ast::NodeData::WhileStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::DoStatement => {
                if let tsox_frontend::ast::NodeData::DoStatement(data) = &node.data {
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                }
            }
            SyntaxKind::ForStatement => {
                self.push_scope(node);
                if let tsox_frontend::ast::NodeData::ForStatement(data) = &node.data {
                    if let Some(init) = &data.initializer {
                        self.check_for_initializer(init);
                    }
                    if let Some(cond) = &data.condition {
                        self.check_expression(cond);
                        self.check_truthiness_of_type(cond);
                    }
                    if let Some(incr) = &data.incrementor {
                        self.check_expression(incr);
                    }
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.push_scope(node);
                if let tsox_frontend::ast::NodeData::ForInOrOfStatement(data) = &node.data {
                    self.check_grammar_for_in_or_for_of_statement(node);
                    if node.kind == SyntaxKind::ForOfStatement {
                        if let Some(await_modifier) = &data.await_modifier {
                            let container = crate::checker::utilities_get_assignment_target::
                                get_containing_function_or_class_static_block(node);
                            if container
                                .as_ref()
                                .is_some_and(|c| c.kind == SyntaxKind::ClassStaticBlockDeclaration)
                            {
                                self.diagnostics.add(
                                    tsox_frontend::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        await_modifier.loc,
                                        tsox_core::diagnostics::messages_generated::
                                            X_FOR_AWAIT_LOOPS_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
                                        Vec::new(),
                                    ),
                                );
                            }
                        }
                        self.check_expression(&data.expression);
                        let iterated = self.check_right_hand_side_of_for_of(node);
                        if data.initializer.kind == SyntaxKind::VariableDeclarationList {
                            self.check_variable_declaration_list(&data.initializer);
                        } else {
                            let var_expr = Arc::clone(&data.initializer);
                            if matches!(
                                var_expr.kind,
                                SyntaxKind::ArrayLiteralExpression
                                    | SyntaxKind::ObjectLiteralExpression
                            ) {
                                let source =
                                    iterated.unwrap_or_else(|| self.error_type());
                                self.check_destructuring_assignment(&var_expr, &source);
                            } else {
                                self.check_expression(&var_expr);
                                let left_type = self.get_type_of_node(&var_expr);
                                self.check_for_of_reference_expression(&var_expr);
                                if let Some(iterated) = &iterated {
                                    self.check_type_assignable_to_and_optionally_elaborate(
                                        iterated,
                                        &left_type,
                                        Some(&var_expr),
                                        Some(&data.expression),
                                        None,
                                        None,
                                    );
                                }
                            }
                        }
                    } else {
                        self.check_for_initializer(&data.initializer);
                        self.check_expression(&data.expression);
                    }
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ReturnStatement => {
                self.check_return_statement(node);
            }
            SyntaxKind::Block => {
                self.push_scope(node);
                if let tsox_frontend::ast::NodeData::Block(data) = &node.data {
                    let mut after_terminator = false;
                    for stmt in data.statements.iter() {
                        let is_hoistable_decl = matches!(
                            stmt.kind,
                            SyntaxKind::EnumDeclaration
                                | SyntaxKind::FunctionDeclaration
                                | SyntaxKind::ClassDeclaration
                        );
                        if after_terminator
                            && !is_hoistable_decl
                            && !node_flags_contains_unreachable(stmt)
                        {
                            // Go addErrorOrSuggestion：allowUnreachableCode 为 True 跳过，
                            // False 报 Error，未设置降级 Suggestion（LSP 侧映射 Hint）
                            if !matches!(self.allow_unreachable_code, Tristate::True) {
                                let category = if matches!(self.allow_unreachable_code, Tristate::False)
                                {
                                    Category::Error
                                } else {
                                    Category::Suggestion
                                };
                                let mut diag = tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    stmt.loc,
                                    UNREACHABLE_CODE_DETECTED,
                                    vec![],
                                );
                                diag.category = category;
                                self.diagnostics.add(diag);
                            }
                        }
                        self.check_statement(stmt);
                        if Self::is_block_terminating_statement(stmt) {
                            after_terminator = true;
                        }
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ThrowStatement => {
                if let tsox_frontend::ast::NodeData::ThrowStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SwitchStatement => {
                if let tsox_frontend::ast::NodeData::SwitchStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Switch,
                            label: None,
                            is_iteration: false,
                        });

                    if let tsox_frontend::ast::NodeData::CaseBlock(case_block) =
                        &data.case_block.data
                    {
                        self.push_scope(&data.case_block);
                        for case in case_block.clauses.iter() {
                            self.check_case_clause(case);
                        }
                        self.pop_scope();
                    }
                    self.break_continue_context_stack.pop();
                }
            }

            SyntaxKind::FunctionDeclaration => {
                self.check_function_declaration(node);
            }
            SyntaxKind::ClassDeclaration => {
                self.check_class_declaration(node);
            }
            SyntaxKind::InterfaceDeclaration => {
                self.check_grammar_modifiers(node);

                if let tsox_frontend::ast::NodeData::InterfaceDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &tsox_core::diagnostics::messages_generated::INTERFACE_NAME_CANNOT_BE_0,
                    );
                    self.check_class_type_for_duplicate_declarations(node);
                    self.check_interface_members(&data.members);
                }

                let iface_sym = self.program.symbol_map().symbol_of(node).cloned();
                if let Some(sym) = iface_sym {
                    let iface_type = self.resolve_interface_type(&sym, None);

                    self.check_index_constraints(&iface_type, node);
                }
            }
            SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {
                self.check_type_alias_and_specifiers(node);
                self.check_import_ambient_rules(node);
                self.check_import_equals_conflicts(node);
                self.check_alias_symbol_bindings(node);
                self.check_node_next_extension_rules(node);
            }
            SyntaxKind::EnumDeclaration => {
                self.check_enum_declaration(node);
            }
            SyntaxKind::ExportAssignment => {
                if let tsox_frontend::ast::NodeData::ExportAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ModuleDeclaration => {
                self.check_module_declaration(node);
            }
            SyntaxKind::EmptyStatement => {}
            SyntaxKind::LabeledStatement => {
                if let tsox_frontend::ast::NodeData::LabeledStatement(data) = &node.data {
                    // Go checkLabeledStatement：binder 标记未引用的标签报
                    // Unused label（allowUnusedLabels False→Error，否则 Suggestion）
                    if data
                        .label
                        .flags
                        .contains(tsox_frontend::ast::NodeFlags::Unreachable)
                        && !matches!(self.allow_unused_labels, Tristate::True)
                    {
                        let mut diag = tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.label.loc,
                            UNUSED_LABEL,
                            vec![],
                        );
                        diag.category =
                            if matches!(self.allow_unused_labels, Tristate::False) {
                                Category::Error
                            } else {
                                Category::Suggestion
                            };
                        self.diagnostics.add(diag);
                    }
                    let label_text = data.label.text().to_string();
                    let is_iteration = matches!(
                        data.statement.kind,
                        SyntaxKind::WhileStatement
                            | SyntaxKind::DoStatement
                            | SyntaxKind::ForStatement
                            | SyntaxKind::ForInStatement
                            | SyntaxKind::ForOfStatement
                    );
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Labeled,
                            label: Some(label_text),
                            is_iteration,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement => {
                self.check_grammar_break_or_continue_statement(node);
            }
            SyntaxKind::VariableDeclaration => {
                self.check_variable_declaration(node);
            }

            SyntaxKind::ModuleBlock => {
                if let tsox_frontend::ast::NodeData::ModuleBlock(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
                    }
                }
            }
            SyntaxKind::WithStatement => {
                // Go checkWithStatement：body 不检查（with 块内一切符号
                // 按 any，TS2410）；span 从 with 关键字到 body 起点
                if let tsox_frontend::ast::NodeData::WithStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    if !self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.has_parse_diagnostics)
                    {
                        let loc = tsox_core::core::text::TextRange::new(
                            node.loc.pos(),
                            data.statement.loc.pos(),
                        );
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            loc,
                            tsox_core::diagnostics::messages_generated::
                                THE_WITH_STATEMENT_IS_NOT_SUPPORTED_ALL_SYMBOLS_IN_A_WITH_BLOCK_WILL_HAVE_TYPE_ANY,
                            Vec::new(),
                        ));
                    }
                }
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }
}

fn node_flags_contains_unreachable(node: &Arc<Node>) -> bool {
    node.flags
        .contains(tsox_frontend::ast::NodeFlags::Unreachable)
}

fn parent_statements_of(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> {
    let parent = node.parent()?;
    match &parent.data {
        NodeData::Block(data) => Some(data.statements.iter().cloned().collect()),
        NodeData::SourceFile(data) => Some(data.statements.iter().cloned().collect()),
        NodeData::ModuleBlock(data) => Some(data.statements.iter().cloned().collect()),
        _ => None,
    }
}

impl Checker {
    /// Go checkSourceElementUnreachable：binder 已打标的语句报
    /// Unreachable code detected（合并连续不可达语句为一条）
    pub(crate) fn check_source_element_unreachable(&mut self, node: &Arc<Node>) -> bool {
        if !tsox_frontend::ast::is_potentially_executable_node(node) {
            return false;
        }
        let key = Arc::as_ptr(node) as usize;
        if self.reported_unreachable_nodes.contains(&key) {
            return true;
        }
        if !self.is_source_element_unreachable(node) {
            return false;
        }
        self.reported_unreachable_nodes.insert(key);

        let mut start_node = Arc::clone(node);
        let mut end_node = Arc::clone(node);
        if let Some(statements) = parent_statements_of(node) {
            if let Some(offset) = statements.iter().position(|s| Arc::ptr_eq(s, node)) {
                let mut last = offset;
                for next in statements.iter().skip(offset + 1) {
                    if !tsox_frontend::ast::is_potentially_executable_node(next)
                        || !self.is_source_element_unreachable(next)
                    {
                        break;
                    }
                    last += 1;
                    self.reported_unreachable_nodes
                        .insert(Arc::as_ptr(next) as usize);
                }
                start_node = Arc::clone(&statements[offset]);
                end_node = Arc::clone(&statements[last]);
            }
        }

        let diagnostic = tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            start_node.loc,
            UNREACHABLE_CODE_DETECTED,
            vec![],
        );
        let mut diagnostic = diagnostic;
        diagnostic.loc = tsox_core::core::text::TextRange::new(
            start_node.loc.pos(),
            end_node.loc.end(),
        );
        diagnostic.category = if matches!(self.allow_unreachable_code, Tristate::False) {
            Category::Error
        } else {
            Category::Suggestion
        };
        self.diagnostics.add(diagnostic);
        true
    }

    /// Go isSourceElementUnreachable：旗标分支（const enum / 非实例化
    /// module 除外）
    pub(crate) fn is_source_element_unreachable(&self, node: &Arc<Node>) -> bool {
        if !node
            .flags
            .contains(tsox_frontend::ast::NodeFlags::Unreachable)
        {
            return false;
        }
        match node.kind {
            SyntaxKind::EnumDeclaration => !node.has_syntactic_modifier(
                tsox_frontend::ast::ModifierFlags::Const,
            ),
            SyntaxKind::ModuleDeclaration => {
                crate::checker::checker_attach_explicit_type_arguments::module_is_instantiated(
                    node, false,
                )
            }
            _ => true,
        }
    }
}

impl Checker {
    /// Go resolveExternalModule 的 node16/nodenext ESM 分支：implied ESM 文件
    /// 的相对无扩展名说明符报 TS2835（建议 ./x.ts）/ TS2834
    pub(crate) fn check_node_next_extension_rules(&mut self, node: &Arc<Node>) {
        use tsox_core::core::compiler_options::ModuleKind;
        let NodeData::ImportDeclaration(d) = &node.data else {
            return;
        };
        let spec = &d.module_specifier;
        if spec.kind != SyntaxKind::StringLiteral {
            return;
        }
        let text = spec.text();
        if !(text.starts_with("./") || text.starts_with("../")) {
            return;
        }
        if tsox_core::tspath::has_extension(&text) {
            return;
        }
        let module_kind = self.compiler_options.module;
        if !matches!(
            module_kind,
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext
        ) {
            return;
        }
        let Some(file) = self.current_file.clone() else { return };
        let implied = tsox_tsoptions::tsoptions::implied_node_format_of_file(
            &file.file_name,
            &|p| self.program.read_file(p),
        );
        if implied != ModuleKind::ESNext {
            return;
        }
        // Go getSuggestedImportExtension：按存在性探测建议扩展名（.mts→.mjs、
        // .ts→.js、.cts→.cjs、原生 .mjs/.js/.cjs、.tsx→.jsx(preserve)/.js）
        let dir = tsox_core::tspath::get_directory_path(&file.file_name);
        let absolute = tsox_core::tspath::get_normalized_absolute_path(&text, &dir);
        let exists = |ext: &str| self.program.read_file(&format!("{absolute}{ext}")).is_some();
        let suggested = if exists(".mts") {
            Some(".mjs")
        } else if exists(".ts") {
            Some(".js")
        } else if exists(".cts") {
            Some(".cjs")
        } else if exists(".mjs") {
            Some(".mjs")
        } else if exists(".js") {
            Some(".js")
        } else if exists(".cjs") {
            Some(".cjs")
        } else if exists(".tsx") {
            Some(if self.compiler_options.jsx == tsox_core::core::compiler_options::JsxEmit::Preserve {
                ".jsx"
            } else {
                ".js"
            })
        } else if exists(".jsx") {
            Some(".jsx")
        } else if exists(".json") {
            Some(".json")
        } else {
            None
        };
        let file = self.current_file.clone();
        let message = match suggested {
            Some(ext) => {
                let args = vec![format!("{text}{ext}")];
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    spec.loc,
                    tsox_core::diagnostics::messages_generated::
                        RELATIVE_IMPORT_PATHS_NEED_EXPLICIT_FILE_EXTENSIONS_IN_ECMASCRIPT_IMPORTS_WHEN_MODULERESOLUTION_IS_NODE16_OR_NODENEXT_DID_YOU_MEAN_0,
                    args,
                )
            }
            None => tsox_frontend::ast::Diagnostic::new(
                file,
                spec.loc,
                tsox_core::diagnostics::messages_generated::
                    RELATIVE_IMPORT_PATHS_NEED_EXPLICIT_FILE_EXTENSIONS_IN_ECMASCRIPT_IMPORTS_WHEN_MODULERESOLUTION_IS_NODE16_OR_NODENEXT_CONSIDER_ADDING_AN_EXTENSION_TO_THE_IMPORT_PATH,
                vec![],
            ),
        };
        self.diagnostics.add(message);
    }

    /// 动态 import() 恒以 ESM 模式解析（含 CJS 文件内），node16+ 下相对
    /// 无扩展名说明符同样报 TS2835/TS2834
    pub(crate) fn check_dynamic_import_extension_rules(&mut self, node: &Arc<Node>) {
        use tsox_core::core::compiler_options::ModuleKind;
        let NodeData::CallExpression(call) = &node.data else {
            return;
        };
        if call.expression.kind != SyntaxKind::ImportKeyword {
            return;
        }
        let Some(arg0) = call.arguments.nodes.first() else {
            return;
        };
        if arg0.kind != SyntaxKind::StringLiteral {
            return;
        }
        let text = arg0.text();
        if !(text.starts_with("./") || text.starts_with("../")) {
            return;
        }
        if tsox_core::tspath::has_extension(&text) {
            return;
        }
        if !matches!(
            self.compiler_options.module,
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext
        ) {
            return;
        }
        let Some(file) = self.current_file.clone() else { return };
        let dir = tsox_core::tspath::get_directory_path(&file.file_name);
        let absolute = tsox_core::tspath::get_normalized_absolute_path(&text, &dir);
        let exists = |ext: &str| self.program.read_file(&format!("{absolute}{ext}")).is_some();
        let suggested = if exists(".mts") {
            Some(".mjs")
        } else if exists(".ts") {
            Some(".js")
        } else if exists(".cts") {
            Some(".cjs")
        } else if exists(".mjs") {
            Some(".mjs")
        } else if exists(".js") {
            Some(".js")
        } else if exists(".cjs") {
            Some(".cjs")
        } else if exists(".tsx") {
            Some(if self.compiler_options.jsx == tsox_core::core::compiler_options::JsxEmit::Preserve {
                ".jsx"
            } else {
                ".js"
            })
        } else if exists(".jsx") {
            Some(".jsx")
        } else if exists(".json") {
            Some(".json")
        } else {
            None
        };
        let message = match suggested {
            Some(ext) => {
                let args = vec![format!("{text}{ext}")];
                tsox_frontend::ast::Diagnostic::new(
                    Some(file),
                    arg0.loc,
                    tsox_core::diagnostics::messages_generated::
                        RELATIVE_IMPORT_PATHS_NEED_EXPLICIT_FILE_EXTENSIONS_IN_ECMASCRIPT_IMPORTS_WHEN_MODULERESOLUTION_IS_NODE16_OR_NODENEXT_DID_YOU_MEAN_0,
                    args,
                )
            }
            None => tsox_frontend::ast::Diagnostic::new(
                Some(file),
                arg0.loc,
                tsox_core::diagnostics::messages_generated::
                    RELATIVE_IMPORT_PATHS_NEED_EXPLICIT_FILE_EXTENSIONS_IN_ECMASCRIPT_IMPORTS_WHEN_MODULERESOLUTION_IS_NODE16_OR_NODENEXT_CONSIDER_ADDING_AN_EXTENSION_TO_THE_IMPORT_PATH,
                vec![],
            ),
        };
        self.diagnostics.add(message);
    }
}
