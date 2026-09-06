use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, NodeData,
    NodeFlags, SymbolFlags, SyntaxKind,
};
use crate::diagnostics::messages_generated::*;







use super::*;


impl Checker {
    pub fn check_statement(&mut self, node: &Arc<Node>) {
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
            && node.parent.as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::SourceFile
                )
            })
            && !Self::inside_function_body(node)
        {
            let block_id = node.parent.as_ref().unwrap().id();
            if !self.ambient_ts1036_reported_blocks.contains(&block_id) {
                self.ambient_ts1036_reported_blocks.insert(block_id);
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                    Vec::new(),
                ));
            }
        }
        match node.kind {
            SyntaxKind::ExpressionStatement => {
                if let crate::ast::NodeData::ExpressionStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::VariableStatement => {
                if let crate::ast::NodeData::VariableStatement(data) = &node.data {

                    self.check_grammar_variable_declaration_list(&data.declaration_list);
                    self.check_variable_declaration_list(&data.declaration_list);

                    self.check_grammar_modifiers(node);

                    if let crate::ast::NodeData::VariableDeclarationList(list) =
                        &data.declaration_list.data
                    {
                        let decls = list.declarations.clone();
                        for d in decls.iter() {
                            if let crate::ast::NodeData::VariableDeclaration(vd) = &d.data {
                                self.check_cjs_reserved_top_level_name(d, &vd.name);
                            }
                        }
                    }

                    self.check_declaration_nameability(node);
                }
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.check_statement(&data.then_statement);
                    if let Some(else_stmt) = &data.else_statement {
                        self.check_statement(else_stmt);
                    }
                }
            }
            SyntaxKind::WhileStatement => {
                if let crate::ast::NodeData::WhileStatement(data) = &node.data {
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
                if let crate::ast::NodeData::DoStatement(data) = &node.data {
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
                if let crate::ast::NodeData::ForStatement(data) = &node.data {
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
                if let crate::ast::NodeData::ForInOrOfStatement(data) = &node.data {
                    if node.kind == SyntaxKind::ForOfStatement && data.await_modifier.is_none() {
                        self.check_for_of_iterated_type(node, &data.expression);
                    }
                    self.check_for_initializer(&data.initializer);
                    self.check_expression(&data.expression);
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

                if self.function_scope_count == 0 && self.arrow_function_scope_count == 0 {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            A_RETURN_STATEMENT_CAN_ONLY_BE_USED_WITHIN_A_FUNCTION_BODY,
                        Vec::new(),
                    ));
                }
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);

                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            let actual = self.get_type_of_node(expr);

                            if !actual.flags.contains(TypeFlags::Any)
                                && !self.is_type_assignable_to(&actual, &expected)
                            {

                                let display_type =
                                    if crate::checker::is_literal_type(&actual) {
                                        self.get_base_type_of_literal_type(&actual)
                                    } else {
                                        actual.clone()
                                    };
                                let ok = self.check_type_related_to_and_optionally_elaborate(
                                    &display_type,
                                    &expected,
                                    crate::checker::relater::RelationKind::Assignable,
                                    Some(node),
                                    Some(expr),
                                    None,
                                    None,
                                );
                                if ok {

                                }
                            }
                        }
                    } else {

                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            if !expected.flags.contains(TypeFlags::Void)
                                && !expected.flags.contains(TypeFlags::Undefined)
                                && !expected.flags.contains(TypeFlags::Any)
                            {
                                let expected_str = self.type_to_string(&expected);
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                    vec!["undefined".to_string(), expected_str],
                                ));
                            }
                        }
                    }
                }
            }
            SyntaxKind::Block => {
                self.push_scope(node);
                if let crate::ast::NodeData::Block(data) = &node.data {

                    let mut after_terminator = false;
                    for stmt in data.statements.iter() {

                        let is_hoistable_decl = matches!(
                            stmt.kind,
                            SyntaxKind::EnumDeclaration
                                | SyntaxKind::FunctionDeclaration
                                | SyntaxKind::ClassDeclaration
                        );
                        if after_terminator && !is_hoistable_decl {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                stmt.loc,
                                UNREACHABLE_CODE_DETECTED,
                                vec![],
                            ));
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
                if let crate::ast::NodeData::ThrowStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Switch,
                            label: None,
                            is_iteration: false,
                        });

                    if let crate::ast::NodeData::CaseBlock(case_block) = &data.case_block.data {
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

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        self.check_cjs_reserved_top_level_name(node, name);
                    }
                }

                self.check_duplicate_function_implementations(node);

                self.check_overload_implementation_follows(node);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(tps) = &data.type_parameters {
                        let _ = tps;
                    }
                    self.check_grammar_parameter_list(&data.parameters);

                    self.check_parameter_property_modifiers(&data.parameters, false);

                    self.check_parameter_implicit_any(node, &data.parameters, 0);
                    for p in data.parameters.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &data.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && data.type_node.is_none()
                        && data.body.is_none()
                        && let Some(name) = &data.name
                        && name.kind == SyntaxKind::Identifier
                    {
                        let file = self.current_file.clone();
                        let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }

                self.check_unmatched_jsdoc_parameters(node);

                let fn_type = self.get_type_of_function_like(node);

                let fn_symbol = match &node.data {
                    crate::ast::NodeData::FunctionDeclaration(data) => data
                        .name
                        .as_ref()
                        .and_then(|n| self.resolve_identifier(n)),
                    _ => None,
                };
                let fn_type = match &fn_symbol {
                    Some(sym) => self.attach_function_expando_type(sym, fn_type),
                    None => fn_type,
                };
                self.type_node_links.get_or_default(node).resolved_type = Some(fn_type.clone());
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {

                            let symbol_type = match self.build_overload_function_type(&symbol) {
                                Some(overload_type) => overload_type,
                                None => fn_type.clone(),
                            };
                            self.value_symbol_links
                                .get_or_default(&symbol)
                                .resolved_type = Some(symbol_type.clone());
                            self.type_node_links.get_or_default(name).resolved_type =
                                Some(symbol_type);
                        }
                    }
                }

                self.push_function_scope(node);
                self.break_continue_context_stack
                    .push(BreakContinueContext {
                        kind: BreakContinueContextKind::Function,
                        label: None,
                        is_iteration: false,
                    });

                let declared_return = match &node.data {
                    crate::ast::NodeData::FunctionDeclaration(data) => {
                        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                        data.type_node
                            .as_ref()
                            .map(|tn| self.get_type_from_type_node(tn))
                            .map(|t| self.unwrap_async_return_type(t, is_async))
                    }
                    _ => None,
                };
                self.return_type_stack.push(declared_return.clone());
                self.in_ctor_body_stack.push(false);

                self.this_container_stack
                    .push(ThisContainerKind::PlainFunction);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.this_container_stack.pop();

                if let Some(ret_type) = &declared_return {
                    if !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                    {
                        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                            if let Some(body) = &data.body {
                                if !self.function_body_definitely_returns(body) {
                                    if !Self::function_body_has_explicit_return(body) {

                                        let loc = data
                                            .type_node
                                            .as_ref()
                                            .map_or(node.loc, |tn| tn.loc);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            loc,
                                            A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                            vec![],
                                        ));
                                    } else {

                                        let loc = data
                                            .type_node
                                            .as_ref()
                                            .map_or(node.loc, |tn| tn.loc);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            loc,
                                            FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                            vec![],
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                self.return_type_stack.pop();
                self.in_ctor_body_stack.pop();
                self.break_continue_context_stack.pop();
                self.pop_function_scope();
            }
            SyntaxKind::ClassDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        self.check_reserved_type_name(
                            name,
                            &crate::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                        );

                        self.check_cjs_reserved_top_level_name(node, name);
                    }
                }

                self.push_scope(node);

                let this_type = self.build_class_instance_type_with_base(node);
                self.this_type_stack.push(this_type);

                self.enclosing_class_stack.push(Arc::clone(node));

                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            self.check_heritage_clause(clause);
                        }
                    }

                    if !node.has_syntactic_modifier(ModifierFlags::Ambient)
                        && self.ambient_context_depth == 0
                        && !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file)
                    {
                        self.check_class_member_overloads(&data.members);
                    }

                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }

                    if let Some(this_type) = self.this_type_stack.last().cloned() {
                        self.check_index_constraints(&this_type, node);
                    }
                    self.check_class_heritage_members(node);

                    self.check_property_initialization(node);
                }
                self.pop_scope();
                self.this_type_stack.pop();
                self.enclosing_class_stack.pop();

                let class_type = self.get_type_of_class_declaration(node);
                self.type_node_links.get_or_default(node).resolved_type = Some(class_type.clone());
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {
                            self.value_symbol_links
                                .get_or_default(&symbol)
                                .resolved_type = Some(class_type);
                        }
                    }
                }
            }
            SyntaxKind::InterfaceDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::InterfaceDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::INTERFACE_NAME_CANNOT_BE_0,
                    );
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

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration
                ) && self.ambient_context_depth == 0
                    && self
                        .current_file
                        .as_ref()
                        .is_none_or(|f| !f.file_name.starts_with("bundled://"))
                {
                    self.check_module_specifier_members(node);
                    self.check_module_export_names(node);
                }

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration
                        | SyntaxKind::ExportDeclaration
                        | SyntaxKind::ImportEqualsDeclaration
                ) && self.ambient_context_depth == 0
                    && self
                        .current_file
                        .as_ref()
                        .is_none_or(|f| !f.file_name.starts_with("bundled://"))
                {
                    self.check_module_format_mismatch(node);
                }

                if node.kind == SyntaxKind::TypeAliasDeclaration
                    && let crate::ast::NodeData::TypeAliasDeclaration(d) = &node.data
                {
                    self.check_type_annotation(&d.type_node);

                    if !self.current_file.as_ref().is_some_and(|f| {
                        f.file_name.starts_with("bundled://")
                    }) {
                        let _ = self.get_type_from_type_node(&d.type_node);
                    }
                }

                {
                    use crate::core::compiler_options::ModuleKind;
                    let module_ok = matches!(
                        self.compiler_options.module,
                        ModuleKind::ESNext
                            | ModuleKind::Node18
                            | ModuleKind::Node20
                            | ModuleKind::NodeNext
                            | ModuleKind::Preserve
                    );
                    let attributes = match &node.data {
                        crate::ast::NodeData::ImportDeclaration(d) => d.attributes.clone(),
                        crate::ast::NodeData::ExportDeclaration(d) => d.attributes.clone(),
                        _ => None,
                    };
                    if let Some(attrs) = attributes {
                        let file_has_parse_errors = self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.has_parse_diagnostics);
                        if !file_has_parse_errors {
                            let file = self.current_file.clone();
                            let is_type_only = match &node.data {
                                crate::ast::NodeData::ImportDeclaration(d) => d
                                    .import_clause
                                    .as_ref()
                                    .is_some_and(|c| {
                                        matches!(
                                            &c.data,
                                            crate::ast::NodeData::ImportClause(ic)
                                                if ic.phase_modifier
                                                    == Some(SyntaxKind::TypeKeyword)
                                        )
                                    }),
                                crate::ast::NodeData::ExportDeclaration(d) => {
                                    d.is_type_only
                                }
                                _ => false,
                            };
                            let override_mode =
                                self.get_resolution_mode_override(&attrs, is_type_only);
                            let exempt = is_type_only && override_mode.is_some();
                            if !exempt {

                                let emit_commonjs = file
                                    .as_ref()
                                    .map(|f| {
                                        self.program
                                            .get_emit_module_format_of_file(&f.file_name)
                                            == ModuleKind::CommonJS
                                    })
                                    .unwrap_or(false);
                                if !module_ok {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_ARE_ONLY_SUPPORTED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ESNEXT_NODE18_NODE20_NODENEXT_OR_PRESERVE,
                                        Vec::new(),
                                    ));
                                } else if emit_commonjs {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_ARE_NOT_ALLOWED_ON_STATEMENTS_THAT_COMPILE_TO_COMMONJS_REQUIRE_CALLS,
                                        Vec::new(),
                                    ));
                                } else if is_type_only {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_CANNOT_BE_USED_WITH_TYPE_ONLY_IMPORTS_OR_EXPORTS,
                                        Vec::new(),
                                    ));
                                } else if override_mode.is_some() {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            X_RESOLUTION_MODE_CAN_ONLY_BE_SET_FOR_TYPE_ONLY_IMPORTS,
                                        Vec::new(),
                                    ));
                                }
                            }
                        }
                    }
                }

                if self.ambient_context_depth == 0 {
                    let emit_format_cjs = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| {
                            self.program
                                .get_emit_module_format_of_file(&f.file_name)
                                < crate::core::compiler_options::ModuleKind::System
                        });
                    let interop = self.compiler_options.es_module_interop.is_true_or_unknown();
                    if emit_format_cjs {
                        match &node.data {
                            crate::ast::NodeData::ExportDeclaration(d)
                                if d.module_specifier.is_some() =>
                            {
                                match d.export_clause.as_ref().map(|c| c.kind) {

                                    Some(SyntaxKind::NamespaceExport) if interop => {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                        );
                                    }

                                    None => {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_EXPORT_STAR,
                                        );
                                    }

                                    Some(SyntaxKind::NamedImports | SyntaxKind::NamedExports) => {
                                        let elements = d.export_clause.as_ref().and_then(|c| {
                                            match &c.data {
                                                crate::ast::NodeData::NamedExports(ne) => {
                                                    Some(ne.elements.clone())
                                                }
                                                crate::ast::NodeData::NamedImports(ni) => {
                                                    Some(ni.elements.clone())
                                                }
                                                _ => None,
                                            }
                                        });
                                        if interop
                                            && let Some(elements) = elements
                                        {
                                            for spec in elements.nodes.iter() {
                                                if let crate::ast::NodeData::ExportSpecifier(es) =
                                                    &spec.data
                                                {
                                                    let pn = es
                                                        .property_name
                                                        .as_ref()
                                                        .unwrap_or(&es.name);
                                                    if pn.kind == SyntaxKind::DefaultKeyword
                                                        || pn.text() == "default"
                                                    {
                                                        self.check_external_emit_helpers(
                                                            spec,
                                                            EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            crate::ast::NodeData::ImportDeclaration(d) => {
                                if let Some(clause) = &d.import_clause
                                    && let crate::ast::NodeData::ImportClause(ic) = &clause.data
                                {

                                    if interop
                                        && matches!(
                                            ic.named_bindings.as_ref().map(|b| b.kind),
                                            Some(SyntaxKind::NamespaceImport)
                                        )
                                    {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                        );
                                    }

                                    if interop
                                        && let Some(nb) = &ic.named_bindings
                                        && let crate::ast::NodeData::NamedImports(ni) = &nb.data
                                    {
                                        for spec in ni.elements.nodes.iter() {
                                            if let crate::ast::NodeData::ImportSpecifier(is) =
                                                &spec.data
                                            {
                                                let pn =
                                                    is.property_name.as_ref().unwrap_or(&is.name);
                                                if pn.kind == SyntaxKind::DefaultKeyword
                                                    || pn.text() == "default"
                                                {
                                                    self.check_external_emit_helpers(
                                                        spec,
                                                        EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration
                ) && self.ambient_context_depth > 0
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let spec = match &node.data {
                        crate::ast::NodeData::ImportDeclaration(d) => {
                            Some(d.module_specifier.text().to_string())
                        }
                        crate::ast::NodeData::ImportEqualsDeclaration(d) => {
                            if let crate::ast::NodeData::ExternalModuleReference(ext) =
                                &d.module_reference.data
                            {
                                Some(ext.expression.text().to_string())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(spec) = spec {
                        let relative = spec.starts_with("./")
                            || spec.starts_with("../")
                            || spec.starts_with(".\\")
                            || spec.starts_with("..\\");
                        if relative {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    IMPORT_OR_EXPORT_DECLARATION_IN_AN_AMBIENT_MODULE_DECLARATION_CANNOT_REFERENCE_MODULE_THROUGH_RELATIVE_MODULE_NAME,
                                vec![],
                            ));

                            let spec_loc = match &node.data {
                                crate::ast::NodeData::ImportDeclaration(d) => {
                                    d.module_specifier.loc
                                }
                                crate::ast::NodeData::ImportEqualsDeclaration(d) => {

                                    if let crate::ast::NodeData::ExternalModuleReference(ext) =
                                        &d.module_reference.data
                                    {
                                        ext.expression.loc
                                    } else {
                                        d.module_reference.loc
                                    }
                                }
                                _ => node.loc,
                            };
                            let spec_trimmed = spec.trim_matches(['"', '\'', '`']).to_string();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                spec_loc,
                                crate::diagnostics::messages_generated::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                                vec![spec_trimmed],
                            ));
                        }
                    }
                }

                if node.kind == SyntaxKind::ImportEqualsDeclaration
                    && let crate::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
                    && matches!(
                        d.module_reference.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {
                    let entity_ok = match &d.module_reference.data {
                        crate::ast::NodeData::Identifier(id) => {
                            is_valid_identifier_text(&id.text)
                                && !matches!(id.text.as_str(), "null" | "true" | "false")
                        }
                        _ => true,
                    };

                    let ns_hit = self
                        .resolve_identifier_with_meaning(
                            &base_identifier_of(&d.module_reference),
                            SymbolFlags::NAMESPACE,
                        )
                        .map(|s| self.resolve_alias_base(s));
                    let base_is_namespace = match &d.module_reference.data {
                        crate::ast::NodeData::Identifier(_) => ns_hit
                            .as_ref()
                            .is_some_and(|b| b.flags.intersects(SymbolFlags::NAMESPACE)),
                        _ => true,
                    };
                    let traced_err = if entity_ok && !base_is_namespace {

                        let base = base_identifier_of(&d.module_reference);
                        let any_hit = self
                            .resolve_identifier(&base)
                            .map(|s| self.resolve_alias_base(s));
                        let masked = any_hit.as_ref().is_some_and(|s| {
                            !s.flags.intersects(SymbolFlags::NAMESPACE)
                                && ns_hit
                                    .as_ref()
                                    .is_some_and(|n| n.flags.intersects(SymbolFlags::VALUE))
                        });
                        if masked {
                            ImportEntityError::HiddenByLocal(base)
                        } else if any_hit
                            .as_ref()
                            .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                        {
                            ImportEntityError::TypeAsNamespace(base)
                        } else {
                            ImportEntityError::NamespaceNotFound(base)
                        }
                    } else if entity_ok {
                        match self.resolve_qualified_symbol_traced(&d.module_reference) {
                            Err((segment, ns_path, _member)) if ns_path.is_empty() => {

                                let any_hit = self
                                    .resolve_identifier(&segment)
                                    .map(|s| self.resolve_alias_base(s));
                                if any_hit
                                    .as_ref()
                                    .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                                {
                                    ImportEntityError::TypeAsNamespace(segment)
                                } else {
                                    ImportEntityError::NamespaceNotFound(segment)
                                }
                            }
                            Err(e) => ImportEntityError::MissingMember(e),
                            Ok(_) => ImportEntityError::None,
                        }
                    } else {
                        ImportEntityError::None
                    };
                    if !matches!(traced_err, ImportEntityError::None)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        match traced_err {
                            ImportEntityError::None => {}
                            ImportEntityError::NamespaceNotFound(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::TypeAsNamespace(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::HiddenByLocal(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::MissingMember((seg, ns_path, member)) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                    vec![ns_path, member],
                                ));
                            }
                        }
                    }

                    if entity_ok
                        && let Some(ns) = ns_hit.as_ref()
                        && ns.flags.intersects(SymbolFlags::VALUE)
                    {
                        let base = base_identifier_of(&d.module_reference);
                        let masked = self
                            .resolve_identifier_with_meaning(
                                &base,
                                SymbolFlags::VALUE | SymbolFlags::NAMESPACE,
                            )

                            .map(|s| self.resolve_alias_base(s))
                            .is_some_and(|s| !s.flags.intersects(SymbolFlags::NAMESPACE));
                        if masked {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                base.loc,
                                crate::diagnostics::messages_generated::
                                    MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                                vec![base.text().to_string()],
                            ));
                        }
                    }

                    if node.kind == SyntaxKind::ImportEqualsDeclaration
                        && let crate::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
                    {
                        if let Some(alias_sym) =
                            self.program.symbol_map().symbol_of(node).cloned()
                        {
                            let target = self.resolve_alias_base(Arc::clone(&alias_sym));

                            let target_resolved = !Arc::ptr_eq(&target, &alias_sym)
                                || !target.flags.intersects(SymbolFlags::Alias);
                            if target_resolved && target.flags.intersects(SymbolFlags::TYPE) {
                                self.check_reserved_type_name(
                                    &d.name,
                                    &crate::diagnostics::messages_generated::IMPORT_NAME_CANNOT_BE_0,
                                );
                            }

                            let non_alias_flags =
                                alias_sym.flags.difference(SymbolFlags::Alias);
                            let has_local_conflict = target_resolved
                                && alias_sym
                                    .declarations
                                    .iter()
                                    .any(|dd| dd.id() != node.id())
                                && !non_alias_flags.is_empty()
                                && {
                                    let value_side =
                                        non_alias_flags.intersects(SymbolFlags::VALUE);
                                    let type_side =
                                        non_alias_flags.intersects(SymbolFlags::TYPE);
                                    (value_side && target.flags.intersects(SymbolFlags::VALUE))
                                        || (type_side && target.flags.intersects(SymbolFlags::TYPE))
                                };
                            if has_local_conflict {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        IMPORT_DECLARATION_CONFLICTS_WITH_LOCAL_DECLARATION_OF_0,
                                    vec![d.name.text().to_string()],
                                ));
                            }
                        }
                    }
                }
            }
            SyntaxKind::EnumDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
                    );

                    if let Some(sym) = self.program.symbol_map().symbol_of(node) {
                        let enum_decls: Vec<&Arc<Node>> = sym
                            .declarations
                            .iter()
                            .filter(|d| d.kind == SyntaxKind::EnumDeclaration)
                            .collect();
                        if enum_decls.len() > 1 {
                            let is_first_decl =
                                enum_decls.first().is_some_and(|d| Arc::ptr_eq(d, &node));

                            let first_decl_starts_uninit = enum_decls.first().and_then(|d| {
                                let NodeData::EnumDeclaration(ed) = &d.data else {
                                    return None;
                                };
                                ed.members.iter().next().and_then(|m| {
                                    matches!(&m.data, crate::ast::NodeData::EnumMember(em) if em.initializer.is_none())
                                        .then_some(())
                                })
                            }) == Some(());
                            if !is_first_decl && first_decl_starts_uninit {
                                let first_member = data.members.iter().next();
                                let uninit = first_member.is_some_and(|m| {
                                    matches!(
                                        &m.data,
                                        crate::ast::NodeData::EnumMember(em)
                                            if em.initializer.is_none()
                                    )
                                });
                                if uninit {
                                    let loc = first_member
                                        .and_then(|m| m.name())
                                        .map(|n| n.loc)
                                        .unwrap_or(node.loc);
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        loc,
                                        crate::diagnostics::messages_generated::
                                            IN_AN_ENUM_WITH_MULTIPLE_DECLARATIONS_ONLY_ONE_DECLARATION_CAN_OMIT_AN_INITIALIZER_FOR_ITS_FIRST_ENUM_ELEMENT,
                                        Vec::new(),
                                    ));
                                }
                            }
                        }
                    }
                }

                self.push_scope(node);
                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    for member in data.members.iter() {
                        self.check_enum_member(member);
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ExportAssignment => {

                if let crate::ast::NodeData::ExportAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ModuleDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::Identifier
                    && !is_valid_identifier_text(data.name.text())
                {
                    if let Some(msg) = Self::cannot_find_name_message_for("module") {
                        let file = self.current_file.clone();
                        let kw = crate::core::text::TextRange::new(
                            node.loc.pos(),
                            (node.loc.pos() + 6).min(node.loc.end()),
                        );
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            kw,
                            *msg,
                            vec!["module".to_string()],
                        ));
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::StringLiteral
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let raw = data.name.text();
                    let module_name = raw.trim_matches(['"', '\'']);
                    let relative = module_name.starts_with("./")
                        || module_name.starts_with("../")
                        || module_name.starts_with(".\\")
                        || module_name.starts_with("..\\");
                    let ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file);

                    if relative && ambient {

                        let is_decl_name_direct = !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.external_module_indicator.is_some());
                        if is_decl_name_direct {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                data.name.loc,
                                crate::diagnostics::messages_generated::
                                    AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME,
                                vec![],
                            ));
                        }
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::StringLiteral
                    && self.current_file.as_ref().is_some_and(|f| {
                        f.external_module_indicator.is_some()
                            && !f.file_name.starts_with("bundled://")
                    })
                {
                    let module_name = data.name.text().trim_matches(['"', '\'']).to_string();
                    let resolvable = self.resolve_module_file_symbol(&module_name).is_some();
                    if !resolvable {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                INVALID_MODULE_NAME_IN_AUGMENTATION_MODULE_0_CANNOT_BE_FOUND,
                            vec![module_name],
                        ));
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(mdd) = &node.data
                    && mdd.name.kind == SyntaxKind::Identifier
                    && !node.has_syntactic_modifier(ModifierFlags::Ambient)
                    && self.ambient_context_depth == 0
                    && !self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(sym) = self.program.symbol_map().symbol_of(node)
                {
                if sym.flags.contains(SymbolFlags::ValueModule)
                    && sym.declarations.len() > 1
                    && module_is_instantiated(
                        node,
                        self.compiler_options.should_preserve_const_enums(),
                    )
                {

                    let first_non_ambient = sym.declarations.iter().find(|d| {
                        let bodied_fn = matches!(
                            &d.data,
                            crate::ast::NodeData::FunctionDeclaration(fd)
                                if fd.body.is_some()
                        );
                        (matches!(d.kind, SyntaxKind::ClassDeclaration) || bodied_fn)
                            && !d.has_syntactic_modifier(ModifierFlags::Ambient)
                            && !self
                                .get_source_file_of_node(d)
                                .is_some_and(|f| f.is_declaration_file)
                    });
                    if let Some(fc) = first_non_ambient
                        && node.loc.pos() < fc.loc.pos()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            mdd.name.loc,
                            crate::diagnostics::messages_generated::
                                A_NAMESPACE_DECLARATION_CANNOT_BE_LOCATED_PRIOR_TO_A_CLASS_OR_FUNCTION_WITH_WHICH_IT_IS_MERGED,
                            Vec::new(),
                        ));
                    }
                }
                }

                let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient);
                if is_ambient {
                    self.ambient_context_depth += 1;
                }
                self.push_scope(node);
                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.pop_scope();
                if is_ambient {
                    self.ambient_context_depth -= 1;
                }
            }
            SyntaxKind::EmptyStatement => {

            }
            SyntaxKind::LabeledStatement => {

                if let crate::ast::NodeData::LabeledStatement(data) = &node.data {
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
                if let crate::ast::NodeData::ModuleBlock(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
                    }
                }
            }
            _ => {

                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }

    fn declaration_is_ambient(&self, node: &Arc<Node>) -> bool {
        if self.ambient_context_depth > 0 {
            return true;
        }

        if self
            .get_source_file_of_node(node)
            .or(self.current_file.clone())
            .is_some_and(|f| f.is_declaration_file)
        {
            return true;
        }
        let mut cur = Some(node);
        while let Some(n) = cur {
            if n.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }

            if matches!(n.kind, SyntaxKind::VariableStatement | SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration) {
                break;
            }
            cur = n.parent.as_ref();
        }
        false
    }

    fn check_cjs_reserved_top_level_name(&mut self, node: &Arc<Node>, name: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(name.kind, SyntaxKind::Identifier) {
            return;
        }

        if self.compiler_options.no_emit.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let is_module = file.external_module_indicator.is_some()
            || file.common_js_module_indicator.is_some();
        if !is_module {
            return;
        }

        let mut top_level = false;
        let mut p = node.parent.as_ref();
        while let Some(parent) = p {
            match parent.kind {
                SyntaxKind::SourceFile => {
                    top_level = true;
                    break;
                }
                SyntaxKind::VariableDeclarationList | SyntaxKind::VariableStatement => {
                    p = parent.parent.as_ref();
                }
                _ => break,
            }
        }
        if !top_level {
            return;
        }

        if self.declaration_is_ambient(node) {
            return;
        }
        let emit_format = self.program.get_emit_module_format_of_file(&file.file_name);
        let text = name.text().to_string();
        if text == "require" || text == "exports" {

            if emit_format >= ModuleKind::ES2015 {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE,
                vec![text.clone(), text],
            ));
        } else if text == "__esModule" {

            let var_stmt = node
                .parent
                .as_ref()
                .and_then(|list| list.parent.as_ref())
                .filter(|stmt| stmt.kind == SyntaxKind::VariableStatement);
            let exported = var_stmt.is_some_and(|stmt| stmt.has_syntactic_modifier(ModifierFlags::Export));
            if !exported || emit_format >= ModuleKind::System {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    IDENTIFIER_EXPECTED_ESMODULE_IS_RESERVED_AS_AN_EXPORTED_MARKER_WHEN_TRANSFORMING_ECMASCRIPT_MODULES,
                Vec::new(),
            ));
        } else if text == "Object" && node.kind == SyntaxKind::ClassDeclaration {

            if emit_format != ModuleKind::CommonJS {
                return;
            }
            let module_str = match self.compiler_options.module {
                ModuleKind::Node16 => "Node16".to_string(),
                ModuleKind::Node18 => "Node18".to_string(),
                ModuleKind::Node20 => "Node20".to_string(),
                ModuleKind::NodeNext => "NodeNext".to_string(),
                ModuleKind::CommonJS => "CommonJS".to_string(),
                ModuleKind::AMD => "AMD".to_string(),
                ModuleKind::UMD => "UMD".to_string(),
                ModuleKind::System => "System".to_string(),
                ModuleKind::ES2015 => "es2015".to_string(),
                ModuleKind::ES2020 => "es2020".to_string(),
                ModuleKind::ES2022 => "es2022".to_string(),
                _ => "esnext".to_string(),
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CLASS_NAME_CANNOT_BE_OBJECT_WHEN_TARGETING_ES5_AND_ABOVE_WITH_MODULE_0,
                vec![module_str],
            ));
        }
    }

    fn check_for_of_iterated_type(&mut self, statement: &Arc<Node>, expression: &Arc<Node>) {
        let readonly_array_exists = match self.globals.get("ReadonlyArray") {
            Some(sym) => !sym.members.is_empty(),
            None => false,
        };
        if !readonly_array_exists {
            return;
        }
        let t = self.get_type_of_node(expression);
        if t.flags.contains(TypeFlags::Any | TypeFlags::Never) {
            return;
        }
        let mut parts: Vec<Arc<Type>> = Vec::new();
        if t.is_union() {
            parts = self.constituent_types(&t);
        } else {
            parts.push(t.clone());
        }
        for part in &parts {
            let is_string_like = part
                .flags
                .intersects(TypeFlags::String | TypeFlags::StringLiteral);
            if !(self.is_array_type(part) || self.is_tuple_type(part) || is_string_like) {
                let type_str = self.type_to_string(&t);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    expression.loc,
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_NOT_AN_ARRAY_TYPE_OR_A_STRING_TYPE,
                    vec![type_str],
                ));
                return;
            }
        }
        let _ = statement;
    }

    fn check_for_initializer(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::VariableDeclarationList => {
                self.check_variable_declaration_list(node);
            }
            _ => self.check_expression(node),
        }
    }

    fn check_binding_pattern_computed_names(&mut self, name: &Arc<Node>) {
        if !matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            return;
        }
        let mut stack = vec![Arc::clone(name)];
        while let Some(n) = stack.pop() {
            match &n.data {
                crate::ast::NodeData::BindingPattern(data) => {
                    for element in data.elements.iter() {
                        stack.push(Arc::clone(element));
                    }
                }
                crate::ast::NodeData::BindingElement(data) => {
                    if let Some(pn) = &data.property_name {
                        if pn.kind == SyntaxKind::ComputedPropertyName {

                            self.check_computed_property_name(pn);
                            if let crate::ast::NodeData::ComputedPropertyName(cd) = &pn.data {
                                self.check_expression(&cd.expression);

                                let expr_type = self.get_type_of_node(&cd.expression);
                                let is_any = match &expr_type.data {
                                    crate::checker::types::TypeData::Union(u) => u
                                        .union_or_intersection
                                        .types
                                        .iter()
                                        .any(|t| t.flags.contains(TypeFlags::Any)),
                                    _ => expr_type.flags.contains(TypeFlags::Any),
                                };
                                if is_any {
                                    let file = self.current_file.clone();
                                    let type_str = self.type_to_string(&expr_type);
                                    let diagnostic = crate::ast::Diagnostic::new(
                                        file,
                                        cd.expression.loc,
                                        crate::diagnostics::messages_generated::
                                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                                        vec![type_str],
                                    );
                                    self.diagnostics.add(diagnostic);
                                }
                            }
                        }
                    }
                    if let Some(inner) = &data.name {
                        if matches!(
                            inner.kind,
                            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                        ) {
                            stack.push(Arc::clone(inner));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn check_variable_declaration_list(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclarationList(data) = &node.data {
            for decl in data.declarations.iter() {

                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && let Some(init) = &vd.initializer
                    && (node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || node.parent.as_ref().is_some_and(|p| {
                            p.has_syntactic_modifier(ModifierFlags::Ambient)
                        })
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file))
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let is_const = node.flags.contains(NodeFlags::Const);
                    let is_simple_literal = match &init.data {
                        crate::ast::NodeData::StringLiteral(_)
                        | crate::ast::NodeData::NumericLiteral(_)
                        | crate::ast::NodeData::BigIntLiteral(_)
                        | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_) => true,
                        _ if matches!(init.kind, SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword) => {
                            true
                        }

                        crate::ast::NodeData::PropertyAccessExpression(_)
                        | crate::ast::NodeData::ElementAccessExpression(_) => true,
                        _ => false,
                    };
                    let message = if is_const && vd.type_node.is_none() {
                        if is_simple_literal {
                            None
                        } else {
                            Some(
                                crate::diagnostics::messages_generated::
                                    A_CONST_INITIALIZER_IN_AN_AMBIENT_CONTEXT_MUST_BE_A_STRING_OR_NUMERIC_LITERAL_OR_LITERAL_ENUM_REFERENCE,
                            )
                        }
                    } else {
                        Some(
                            crate::diagnostics::messages_generated::
                                INITIALIZERS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                        )
                    };
                    if let Some(message) = message {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            init.loc,
                            message,
                            vec![],
                        ));
                    }
                }

                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && vd.name.kind == SyntaxKind::Identifier
                    && matches!(vd.name.text(), "eval" | "arguments")
                    && self.in_strict_context()
                {
                    let is_module = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                    let message = if is_module {
                        crate::diagnostics::messages_generated::
                            INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
                    } else {
                        crate::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE
                    };
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        vd.name.loc,
                        message,
                        vec![vd.name.text().to_string()],
                    ));
                }
                self.check_variable_declaration(decl);
            }
        }
    }

    fn in_strict_context(&self) -> bool {
        if self.program.options().always_strict.is_true() {
            return true;
        }
        self.current_file.as_ref().is_some_and(|f| {
            f.external_module_indicator.is_some()
                || f.text.trim_start().starts_with("\"use strict\"")
                || f.text.trim_start().starts_with("'use strict'")
        })
    }

    pub(crate) fn report_abstract_property_access_in_ctor(
        &mut self,
        name_node: &Arc<Node>,
        prop_text: &str,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let Some(member_symbol) = structured.members.get(prop_text) else {
            return;
        };
        let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
            d.kind == SyntaxKind::PropertyDeclaration
                && d.has_syntactic_modifier(ModifierFlags::Abstract)
        }) else {
            return;
        };
        let Some(parent) = &abstract_decl.parent else { return };
        let Some(class_name) = class_declaration_name(parent) else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            name_node.loc,
            crate::diagnostics::messages_generated::
                ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
            vec![prop_text.to_string(), class_name],
        ));
    }

    pub(crate) fn access_in_property_initializer(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            match a.kind {
                SyntaxKind::PropertyDeclaration => return true,
                SyntaxKind::Constructor
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression => return false,
                _ => {}
            }
            cur = a.parent.as_ref();
        }
        false
    }

    fn check_this_destructuring_abstract_properties(
        &mut self,
        pattern: &Arc<Node>,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let crate::ast::NodeData::BindingPattern(data) = &pattern.data else {
            return;
        };
        for element in data.elements.iter() {
            let crate::ast::NodeData::BindingElement(el) = &element.data else {
                continue;
            };

            let Some(prop_name_node) = el
                .property_name
                .as_ref()
                .or(el.name.as_ref())
                .filter(|n| n.kind == SyntaxKind::Identifier)
            else {
                continue;
            };
            let prop_text = prop_name_node.text();
            let Some(member_symbol) = structured.members.get(prop_text) else {
                continue;
            };
            let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                d.kind == SyntaxKind::PropertyDeclaration
                    && d.has_syntactic_modifier(ModifierFlags::Abstract)
            }) else {
                continue;
            };
            let Some(parent) = &abstract_decl.parent else { continue };
            let Some(class_name) = class_declaration_name(parent) else {
                continue;
            };
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                prop_name_node.loc,
                crate::diagnostics::messages_generated::
                    ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                vec![prop_text.to_string(), class_name],
            );
            self.diagnostics.add(diagnostic);
        }
    }

    fn check_variable_declaration(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclaration(data) = &node.data {

            if data.initializer.is_none() {
                let is_const = node
                    .parent
                    .as_ref()
                    .is_some_and(|list| list.flags.contains(NodeFlags::Const));
                let in_for_in_of = node.parent.as_ref().and_then(|l| l.parent.as_ref())
                    .is_some_and(|g| {
                        matches!(
                            g.kind,
                            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                        )
                    });
                let is_ambient = self.ambient_context_depth > 0
                    || node.flags.contains(NodeFlags::Ambient)
                    || node
                        .parent
                        .as_ref()
                        .and_then(|p| p.parent.as_ref())
                        .is_some_and(|stmt| {
                            stmt.has_syntactic_modifier(ModifierFlags::Ambient)
                        })
                    || {

                        let mut anc = node.parent.as_ref();
                        let mut found = false;
                        while let Some(a) = anc {
                            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                                found = true;
                                break;
                            }
                            anc = a.parent.as_ref();
                        }
                        found
                    }
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if is_const && !in_for_in_of && !is_ambient {
                    let file = self.current_file.clone();
                    let name_loc = data.name.loc;
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        name_loc,
                        crate::diagnostics::messages_generated::X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                        vec!["const".to_string()],
                    ));
                }
            }

            if data.initializer.is_some() && data.name.kind == SyntaxKind::Identifier {
                let list_is_var = node.parent.as_ref().is_none_or(|l| {
                    !(l.flags.contains(NodeFlags::Let) || l.flags.contains(NodeFlags::Const))
                });
                let is_param = node
                    .parent
                    .as_ref()
                    .is_some_and(|l| l.kind == SyntaxKind::Parameter);
                if list_is_var && !is_param {
                    let own = self.program.symbol_map().symbol_of(node).cloned();
                    if let Some(local) = self.resolve_identifier(&data.name)
                        && own.as_ref().is_none_or(|o| !Arc::ptr_eq(o, &local))
                        && local.flags.contains(SymbolFlags::BlockScopedVariable)
                        && let Some(vd) = local.value_declaration.clone()
                        && vd.kind == SyntaxKind::VariableDeclaration
                        && let Some(list) = vd.parent.as_ref()
                        && list.kind == SyntaxKind::VariableDeclarationList
                    {
                        let container = list.parent.as_ref().and_then(|s| s.parent.as_ref());
                        let names_share_scope = container.is_some_and(|c| {
                            c.kind == SyntaxKind::ModuleBlock
                                || c.kind == SyntaxKind::ModuleDeclaration
                                || c.kind == SyntaxKind::SourceFile
                                || (c.kind == SyntaxKind::Block
                                    && c.parent.as_ref().is_some_and(|p| {
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
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_INITIALIZE_OUTER_SCOPED_VARIABLE_0_IN_THE_SAME_SCOPE_AS_BLOCK_SCOPED_DECLARATION_1,
                                vec![name_text.clone(), name_text],
                            ));
                        }
                    }
                }
            }

            self.check_binding_pattern_computed_names(&data.name);

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

                    if let Some(excess_name) =
                        self.get_excess_property_name(&init_type, &annotation_type)
                    {
                        let file = self.current_file.clone();
                        let annot_str = self.type_to_string(&annotation_type);

                        let loc = self
                            .find_object_literal_property_name_node(init, &excess_name)
                            .unwrap_or(node.loc);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            loc,
                            OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                            vec![excess_name, annot_str],
                        ));
                        reported_error = true;
                    }

                    if !assignable && !reported_error {

                        self.check_type_assignable_to_and_optionally_elaborate(
                            &init_type,
                            &annotation_type,
                            Some(node),
                            Some(init),
                            None,
                            None,
                        );
                    }
                    annotation_type
                }
                (Some(type_node), None) => self.get_type_from_type_node(type_node),
                (None, Some(init)) => {

                    if data.name.kind == SyntaxKind::ArrayBindingPattern {
                        let init_type = if init.kind == SyntaxKind::Identifier
                            && let Some(sym) = self.resolve_identifier(init)
                        {
                            let flow = self
                                .program
                                .symbol_map()
                                .flow_node_of(init)
                                .map(Arc::clone);
                            self.get_narrowed_type_of_symbol(&sym, flow.as_ref())
                        } else {
                            self.get_type_of_node(init)
                        };
                        if init_type.flags.contains(TypeFlags::Never) {
                            let type_str = self.type_to_string(&init_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                data.name.loc,
                                crate::diagnostics::messages_generated::
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
                        let widened_literal =
                            self.get_widened_literal_type_for_initializer(node, &init_type);
                        let regularized = self.get_regular_type_of_literal_type(&widened_literal);
                        self.widen_initializer_type(&regularized)
                    }
                }
                (None, None) => {

                    match self.initial_type_of_declaration(node) {
                        Some(t) => t,
                        None => self.auto_type(),
                    }
                }
            };

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                let primary = symbol.value_declaration.clone();
                if let Some(primary) = primary
                    && !Arc::ptr_eq(&primary, node)
                    && symbol.declarations.len() > 1
                    && primary.kind == SyntaxKind::VariableDeclaration
                    && symbol
                        .flags
                        .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                {
                    let auto_to_any = |t: &Arc<Type>| -> Arc<Type> {
                        if t.intrinsic_name() == Some("auto") {
                            self.get_any_type()
                        } else {
                            Arc::clone(t)
                        }
                    };
                    let primary_type = self
                        .type_node_links
                        .get(&primary)
                        .and_then(|l| l.resolved_type.clone())
                        .map(|t| auto_to_any(&t));
                    let this_type = auto_to_any(&resolved_type);
                    if let Some(primary_type) = primary_type
                        && !matches!(primary_type.intrinsic_name(), Some("error"))
                        && !matches!(this_type.intrinsic_name(), Some("error"))
                        && !self
                            .compare_types_identical(&primary_type, &this_type)
                            .is_true()
                    {
                        let name_text = data.name.text().to_string();
                        let first_str = self.type_to_string(&primary_type);
                        let next_str = self.type_to_string(&this_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_VARIABLE_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                            vec![name_text, first_str, next_str],
                        ));
                    }
                }
            }

            self.type_node_links.get_or_default(node).resolved_type = Some(resolved_type.clone());

            self.type_node_links
                .get_or_default(&data.name)
                .resolved_type = Some(resolved_type.clone());

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                self.value_symbol_links
                    .get_or_default(&symbol)
                    .resolved_type = Some(resolved_type);
            }
        }
    }

    fn check_case_clause(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::CaseOrDefaultClause(data) = &node.data {
            if data.expression.kind != SyntaxKind::UnknownKeyword {
                self.check_expression(&data.expression);
            }
            for stmt in data.statements.iter() {
                self.check_statement(stmt);
            }
        }
    }
}
