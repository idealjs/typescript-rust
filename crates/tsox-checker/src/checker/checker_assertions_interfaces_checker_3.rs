#![allow(unused_imports)]

use crate::checker::checker_assertions_interfaces::*;

impl Checker {
    pub(crate) fn report_function_impl_expected(&mut self, statements: &[Arc<Node>], idx: usize) {
        let node = Arc::clone(&statements[idx]);
        let (name_text, name_loc) = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                Some(n) => (n.text().to_string(), n.loc),
                None => return,
            },
            _ => return,
        };
        if let Some(sib) = statements.get(idx + 1) {
            if sib.kind == SyntaxKind::FunctionDeclaration {
                let sib_name = match &sib.data {
                    tsox_frontend::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                        Some(n) => (n.text().to_string(), n.loc, d.body.is_some()),
                        None => (String::new(), sib.loc, false),
                    },
                    _ => (String::new(), sib.loc, false),
                };
                if sib_name.0 == name_text {
                    return;
                }
                if sib_name.2 {
                    let file = self.current_file.clone();
                    let diagnostic = tsox_frontend::ast::Diagnostic::new(
                        file,
                        sib_name.1,
                        tsox_core::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name_text],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }
        let file = self.current_file.clone();
        let diagnostic = tsox_frontend::ast::Diagnostic::new(
            file,
            name_loc,
            tsox_core::diagnostics::messages_generated::
                FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
            Vec::new(),
        );
        self.diagnostics.add(diagnostic);
    }

    pub(crate) fn check_export_assignment_conflicts(&mut self, statements: &[Arc<Node>]) {
        // Go checkExportAssignment/checkSourceFile：export= 所在容器（文件或
        // ambient 模块）逐个跑 checkExternalModuleExports 的 export= 段
        if let Some(file_sym) = self.current_file_symbol.clone() {
            self.check_external_module_export_equals(&file_sym);
        }
        for m in Self::collect_module_declarations(statements) {
            if let Some(sym) = self.program.symbol_map().symbol_of(&m).cloned() {
                self.check_external_module_export_equals(&sym);
            }
        }
    }

    fn collect_module_declarations(statements: &[Arc<Node>]) -> Vec<Arc<Node>> {
        let mut out = Vec::new();
        let mut stack: Vec<Arc<Node>> = statements.iter().rev().cloned().collect();
        while let Some(n) = stack.pop() {
            if n.kind == SyntaxKind::ModuleDeclaration {
                out.push(Arc::clone(&n));
            }
            let children: Vec<Arc<Node>> = match &n.data {
                tsox_frontend::ast::NodeData::SourceFile(d) => {
                    d.statements.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::ModuleDeclaration(d) => match &d.body {
                    Some(b)
                        if b.kind == SyntaxKind::ModuleBlock =>
                    {
                        match &b.data {
                            tsox_frontend::ast::NodeData::ModuleBlock(mb) => {
                                mb.statements.iter().cloned().collect()
                            }
                            _ => Vec::new(),
                        }
                    }
                    _ => Vec::new(),
                },
                tsox_frontend::ast::NodeData::ModuleBlock(d) => {
                    d.statements.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::Block(d) => d.statements.iter().cloned().collect(),
                _ => Vec::new(),
            };
            stack.extend(children.into_iter().rev());
        }
        out
    }

    // Go checkExternalModuleExports 的 export= 段：模块导出含值成员，或
    // export= 指向的类型/命名空间被模块自身同名遮蔽时，报 TS2309
    fn check_external_module_export_equals(&mut self, module_symbol: &Arc<Symbol>) {
        let Some(export_equals) = module_symbol
            .exports
            .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            .cloned()
        else {
            return;
        };

        let mut has_value = false;
        for (name, sym) in module_symbol.exports.iter() {
            if name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS {
                continue;
            }
            // Go 的 exports 表不含 `export as namespace` 别名（binder 入
            // locals）与 bind 期挂入的 JS 赋值增广成员（Go 在 check 期后合）
            let excluded_from_exports = !sym.declarations.is_empty()
                && sym.declarations.iter().all(|d| {
                    d.kind == SyntaxKind::NamespaceExportDeclaration
                        || matches!(
                            d.kind,
                            SyntaxKind::BinaryExpression | SyntaxKind::CallExpression
                        ) && crate::binder::get_assignment_declaration_kind(d)
                            != crate::binder::bind_js_assignment_declarations::JsDeclarationKind::None
                });
            if excluded_from_exports {
                continue;
            }
            // Go getSymbolFlags：别名链断（unknownSymbol）返回全标志
            let (flags, complete) = self.symbol_flags_with_alias_chain_ex(sym);
            if !complete || flags.intersects(SymbolFlags::VALUE) {
                has_value = true;
                break;
            }
        }
        let mut has_shadowed_namespace = false;
        if !has_value
            && export_equals.flags.contains(SymbolFlags::NamespaceModule)
            && export_equals.flags.contains(SymbolFlags::Alias)
        {
            let target = self.resolve_export_equals_target(&export_equals);
            if target.flags.intersects(SymbolFlags::NAMESPACE)
                && Self::module_exports_have_kind(
                    &target,
                    SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                )
            {
                has_shadowed_namespace = true;
            }
        }
        if !has_value && !has_shadowed_namespace {
            return;
        }
        let declaration = export_equals
            .declarations
            .iter()
            .rev()
            .find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::ImportClause
                        | SyntaxKind::ImportSpecifier
                        | SyntaxKind::NamespaceImport
                        | SyntaxKind::ExportSpecifier
                        | SyntaxKind::ImportEqualsDeclaration
                        | SyntaxKind::NamespaceExport
                        | SyntaxKind::ExportAssignment
                )
            })
            .cloned()
            .or_else(|| export_equals.value_declaration.clone());
        let Some(declaration) = declaration else { return };
        if crate::checker::utilities_get_assignment_target::is_top_level_in_external_module_augmentation(&declaration) {
            return;
        }
        let loc = declaration.loc;
        let file = self
            .get_source_file_of_node(&declaration)
            .or_else(|| self.current_file.clone());
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            loc,
            tsox_core::diagnostics::messages_generated::
                AN_EXPORT_ASSIGNMENT_CANNOT_BE_USED_IN_A_MODULE_WITH_OTHER_EXPORTED_ELEMENTS,
            Vec::new(),
        ));
    }

    fn module_exports_have_kind(module: &Arc<Symbol>, kind: SymbolFlags) -> bool {
        module.exports.iter().any(|(name, sym)| {
            name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                && sym.flags.intersects(kind)
        })
    }


    pub(crate) fn check_reserved_type_name(
        &mut self,
        name: &Arc<Node>,
        message: &'static tsox_core::diagnostics::Message,
    ) {
        const RESERVED: &[&str] = &[
            "any",
            "unknown",
            "never",
            "number",
            "bigint",
            "boolean",
            "string",
            "symbol",
            "void",
            "object",
            "undefined",
        ];
        let text = name.text();
        if RESERVED.contains(&text) {
            let file = self.current_file.clone();
            let diagnostic = tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                *message,
                vec![text.to_string()],
            );
            self.diagnostics.add(diagnostic);
        }
    }

    pub(crate) fn is_type_assignable_to_kind_snf(
        &mut self,
        source: &Arc<Type>,
        kind: TypeFlags,
    ) -> bool {
        if source.flags.intersects(kind) {
            return true;
        }
        let number = self.number_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_NUMBER_LIKE)
            && self.is_type_assignable_to(source, &number)
        {
            return true;
        }
        let string = self.string_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_STRING_LIKE)
            && self.is_type_assignable_to(source, &string)
        {
            return true;
        }
        let symbol = self.es_symbol_type();
        if kind.intersects(TypeFlags::ESSymbol) && self.is_type_assignable_to(source, &symbol) {
            return true;
        }
        false
    }

    pub(crate) fn check_computed_property_name(&mut self, name: &Arc<Node>) {
        self.check_computed_property_name_type(name);
    }

    // Go checkComputedPropertyName：返回表达式类型（isLateBindableName 复用）
    pub(crate) fn check_computed_property_name_type(
        &mut self,
        name: &Arc<Node>,
    ) -> std::sync::Arc<crate::checker::types::Type> {
        if name.kind != SyntaxKind::ComputedPropertyName {
            return self.error_type();
        }
        let first_visit = self
            .computed_property_name_checked
            .insert(Arc::as_ptr(name));
        let expr = match &name.data {
            tsox_frontend::ast::NodeData::ComputedPropertyName(data) => {
                Arc::clone(&data.expression)
            }
            _ => return self.error_type(),
        };

        let invalid_in_form = matches!(&expr.data, tsox_frontend::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::InKeyword)
            && name.parent().as_ref().is_some_and(|member| {
                !matches!(
                    member.kind,
                    SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                ) && member.parent().as_ref().is_some_and(|container| {
                    matches!(
                        container.kind,
                        SyntaxKind::TypeLiteral
                            | SyntaxKind::ClassDeclaration
                            | SyntaxKind::ClassExpression
                            | SyntaxKind::InterfaceDeclaration
                    )
                })
            });
        if invalid_in_form {
            return self.error_type();
        }

        self.check_expression(&expr);
        let t = self.get_type_of_node(&expr);

        let kind = crate::checker::types::TYPE_FLAGS_STRING_LIKE
            | crate::checker::types::TYPE_FLAGS_NUMBER_LIKE
            | crate::checker::types::TYPE_FLAGS_ES_SYMBOL_LIKE;
        let bad = t
            .flags
            .intersects(crate::checker::types::TYPE_FLAGS_NULLABLE)
            || (!self.is_type_assignable_to_kind_snf(&t, kind) && {
                let target = self.get_union_type(vec![
                    self.string_type(),
                    self.number_type(),
                    self.es_symbol_type(),
                ]);
                !self.is_type_assignable_to(&t, &target)
            });
        if bad && first_visit {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                tsox_core::diagnostics::messages_generated::
                    A_COMPUTED_PROPERTY_NAME_MUST_BE_OF_TYPE_STRING_NUMBER_SYMBOL_OR_ANY,
                vec![],
            ));
        }
        t
    }

    // Go checkGrammarProperty/checkGrammarMethod 的动态名分支：
    // 按成员容器选消息（class property/method-overload/ambient/interface）
    pub(crate) fn check_member_dynamic_name_grammar(&mut self, member: &Arc<Node>) {
        let Some(name) = Self::member_name_node(member) else {
            return;
        };
        if name.kind != SyntaxKind::ComputedPropertyName {
            return;
        }
        let Some(container) = member.parent() else {
            return;
        };
        match container.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => match member.kind {
                SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                    self.check_grammar_for_invalid_dynamic_name(
                        &name,
                        &tsox_core::diagnostics::messages_generated::
                            A_COMPUTED_PROPERTY_NAME_IN_A_CLASS_PROPERTY_DECLARATION_MUST_HAVE_A_SIMPLE_LITERAL_TYPE_OR_A_UNIQUE_SYMBOL_TYPE,
                    );
                }
                SyntaxKind::MethodDeclaration => {
                    let ambient = member.flags.contains(
                        tsox_frontend::ast::NodeFlags::Ambient,
                    ) || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                    let has_body = matches!(
                        &member.data,
                        tsox_frontend::ast::NodeData::MethodDeclaration(m) if m.body.is_some()
                    );
                    if ambient {
                        self.check_grammar_for_invalid_dynamic_name(
                            &name,
                            &tsox_core::diagnostics::messages_generated::
                                A_COMPUTED_PROPERTY_NAME_IN_AN_AMBIENT_CONTEXT_MUST_REFER_TO_AN_EXPRESSION_WHOSE_TYPE_IS_A_LITERAL_TYPE_OR_A_UNIQUE_SYMBOL_TYPE,
                        );
                    } else if !has_body {
                        self.check_grammar_for_invalid_dynamic_name(
                            &name,
                            &tsox_core::diagnostics::messages_generated::
                                A_COMPUTED_PROPERTY_NAME_IN_A_METHOD_OVERLOAD_MUST_REFER_TO_AN_EXPRESSION_WHOSE_TYPE_IS_A_LITERAL_TYPE_OR_A_UNIQUE_SYMBOL_TYPE,
                        );
                    }
                }
                _ => {}
            },
            SyntaxKind::InterfaceDeclaration => {
                self.check_grammar_for_invalid_dynamic_name(
                    &name,
                    &tsox_core::diagnostics::messages_generated::
                        A_COMPUTED_PROPERTY_NAME_IN_AN_INTERFACE_MUST_REFER_TO_AN_EXPRESSION_WHOSE_TYPE_IS_A_LITERAL_TYPE_OR_A_UNIQUE_SYMBOL_TYPE,
                );
            }
            _ => {}
        }
    }

    pub(crate) fn member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::MethodSignatureDeclaration(d) => {
                Some(Arc::clone(&d.name))
            }
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::PropertyDeclaration(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => {
                Some(Arc::clone(&d.name))
            }
            tsox_frontend::ast::NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.name)),
            tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(d) => {
                Some(Arc::clone(&d.name))
            }
            _ => None,
        }
    }

    pub(crate) fn property_name_key_type(&mut self, name: &Arc<Node>) -> Option<Arc<Type>> {
        match &name.data {
            tsox_frontend::ast::NodeData::ComputedPropertyName(data) => {
                let expr = &data.expression;
                match &expr.data {
                    tsox_frontend::ast::NodeData::StringLiteral(s) => {
                        Some(self.literal_type_for_property_name_text(&s.text))
                    }
                    tsox_frontend::ast::NodeData::NumericLiteral(n) => Some(
                        self.get_number_literal_type(tsox_core::jsnum::Number::from_string(
                            &n.text,
                        )),
                    ),
                    _ => Some(self.get_type_of_node(expr)),
                }
            }
            tsox_frontend::ast::NodeData::Identifier(data) => {
                Some(self.literal_type_for_property_name_text(&data.text))
            }
            tsox_frontend::ast::NodeData::StringLiteral(data) => {
                Some(self.literal_type_for_property_name_text(&data.text))
            }
            tsox_frontend::ast::NodeData::NumericLiteral(data) => Some(
                self.get_number_literal_type(tsox_core::jsnum::Number::from_string(&data.text)),
            ),
            _ => None,
        }
    }

    // Go isNumericLiteralName：ToString(ToNumber(text)) == text 时属性名是
    // 数字名（含 Infinity/-Infinity/NaN，不含 +Infinity 前缀形式），
    // 索引约束按数字字面量键参与
    fn literal_type_for_property_name_text(&mut self, text: &str) -> Arc<Type> {
        if tsox_core::jsnum::Number::from_string(text).to_string() == text {
            self.get_number_literal_type(tsox_core::jsnum::Number::from_string(text))
        } else {
            self.get_string_literal_type(text)
        }
    }

    pub(crate) fn property_name_display(&self, name: &Arc<Node>) -> String {
        if name.kind == SyntaxKind::ComputedPropertyName {
            if let Some(text) = self.node_source_text(name) {
                let inner = text
                    .strip_prefix('[')
                    .and_then(|t| t.strip_suffix(']'))
                    .unwrap_or(&text);
                return format!("[{inner}]");
            }
        }
        let text = name.text().to_string();
        // Go symbolToString：源文本为字符串字面量的属性名带引号展示
        if name.kind == SyntaxKind::StringLiteral {
            format!("\"{text}\"")
        } else {
            text
        }
    }

    pub(crate) fn node_source_text(&self, node: &Arc<Node>) -> Option<String> {
        let mut root = Arc::clone(node);
        while let Some(p) = root.parent() {
            root = p;
        }
        for f in &self.files {
            if Arc::ptr_eq(&f.node, &root) {
                return f
                    .text
                    .get(node.loc.pos()..node.loc.end())
                    .map(|s| s.to_string());
            }
        }
        None
    }
}

impl Checker {
    // Go checkExternalModuleExports 的 export * 冲突段：同一导出名多个
    // 非重载声明时报 TS2323（namespace/enum/接口合并/类型别名合并除外）
    fn check_default_export_duplicates(&mut self, statements: &[Arc<Node>]) {
        let is_not_overload = |d: &Arc<Node>| {
            !matches!(
                d.kind,
                SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration
            ) || body_of(d).is_some()
        };
        let table_decls = check_default_export_duplicates_inner(&is_not_overload, statements);
        let count = table_decls
            .iter()
            .filter(|d| {
                is_not_overload(d)
                    && !matches!(
                        d.kind,
                        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                    )
                    && d.kind != SyntaxKind::InterfaceDeclaration
            })
            .count();
        if count <= 1 {
            return;
        }
        for declaration in &table_decls {
            if is_not_overload(declaration)
                && let Some(loc) = declaration_name_loc(declaration)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        CANNOT_REDECLARE_EXPORTED_VARIABLE_0,
                    vec!["default".to_string()],
                ));
            }
        }
    }

    pub(crate) fn check_external_module_export_duplicates(&mut self, statements: &[Arc<Node>]) {
        let Some(module_symbol) = self.current_file_symbol.clone() else {
            return;
        };
        self.check_default_export_duplicates(statements);
        let exports = self.get_exports_of_module_table(&module_symbol);
        for (name, symbol) in exports.entries.iter() {
            if name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_STAR
                || name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }
            let flags = self.get_symbol_flags(symbol);
            if flags.intersects(SymbolFlags::NAMESPACE | SymbolFlags::ENUM) {
                continue;
            }
            let is_not_overload = |d: &Arc<Node>| {
                !matches!(d.kind, SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration)
                    || body_of(d).is_some()
            };
            let exported_declarations_count = symbol
                .declarations
                .iter()
                .filter(|d| {
                    is_not_overload(d)
                        && !matches!(
                            d.kind,
                            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                        )
                        && d.kind != SyntaxKind::InterfaceDeclaration
                })
                .count();
            if flags.intersects(SymbolFlags::TypeAlias) && exported_declarations_count <= 2 {
                continue;
            }
            if exported_declarations_count > 1
                && !symbol.declarations.iter().all(|d| {
                    crate::binder::get_assignment_declaration_kind(d)
                        == crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ExportsProperty
                })
            {
                for declaration in symbol.declarations.iter() {
                    if is_not_overload(declaration)
                        && let Some(loc) = declaration_name_loc(declaration)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            loc,
                            tsox_core::diagnostics::messages_generated::
                                CANNOT_REDECLARE_EXPORTED_VARIABLE_0,
                            vec![name.clone()],
                        ));
                    }
                }
            }
        }
    }
}

// Go checkExternalModuleExports 的 default 声明段：default 声明按
// declareSymbol(exports, "default", kind, kindExcludes) 顺序合并，冲突时
// 表外另立符号；承表组按 countWhere（除重载/访问器/接口）计数 >1 时逐
// 非重载声明报 TS2323
fn check_default_export_duplicates_inner(
    is_not_overload: &dyn Fn(&Arc<Node>) -> bool,
    statements: &[Arc<Node>],
) -> Vec<Arc<Node>> {
    let mut table_decls: Vec<Arc<Node>> = Vec::new();
    let mut table_flags = SymbolFlags::None;
    let mut sealed = false;
    for s in statements {
        let Some((includes, excludes)) = default_export_binding(s) else {
            continue;
        };
        if table_decls.is_empty() {
            table_flags = includes;
            table_decls.push(Arc::clone(s));
            continue;
        }
        if sealed || table_flags.intersects(excludes) {
            sealed = true;
            continue;
        }
        table_flags |= includes;
        table_decls.push(Arc::clone(s));
    }
    table_decls
}

fn default_export_binding(s: &Arc<Node>) -> Option<(SymbolFlags, SymbolFlags)> {
    match &s.data {
        tsox_frontend::ast::NodeData::ExportAssignment(d) if !d.is_export_equals => {
            let is_alias = matches!(
                d.expression.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::QualifiedName
                    | SyntaxKind::PropertyAccessExpression
                    | SyntaxKind::ClassExpression
            );
            Some((
                if is_alias {
                    SymbolFlags::Alias
                } else {
                    SymbolFlags::Property
                },
                SymbolFlags::all(),
            ))
        }
        _ if s.has_syntactic_modifier(ModifierFlags::Default) => match s.kind {
            SyntaxKind::FunctionDeclaration => Some((
                SymbolFlags::Function,
                SymbolFlags::FunctionExcludes,
            )),
            SyntaxKind::ClassDeclaration => Some((SymbolFlags::Class, SymbolFlags::ClassExcludes)),
            SyntaxKind::InterfaceDeclaration => Some((
                SymbolFlags::Interface,
                SymbolFlags::InterfaceExcludes,
            )),
            _ => None,
        },
        _ => None,
    }
}

fn body_of(d: &Arc<Node>) -> Option<Arc<Node>> {
    match &d.data {
        tsox_frontend::ast::NodeData::FunctionDeclaration(f) => f.body.clone(),
        tsox_frontend::ast::NodeData::MethodDeclaration(m) => m.body.clone(),
        _ => None,
    }
}

// Go scanner.GetErrorRangeForNode：报错定位到声明名（变量/函数/类等），
// ExportAssignment 用整节点
fn declaration_name_loc(d: &Arc<Node>) -> Option<tsox_core::core::text::TextRange> {
    if matches!(d.kind, SyntaxKind::ExportAssignment) {
        return Some(d.loc);
    }
    Some(d.name().map(|n| n.loc).unwrap_or(d.loc))
}
