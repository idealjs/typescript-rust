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
            // file.GlobalExports，与模块内同名 namespace 不合并）；bind 期挂入的
            // JS 赋值增广成员 Go 在 check 期后合
            let umd_global = sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::NamespaceExportDeclaration);
            let js_assignment_only = !sym.declarations.is_empty()
                && sym.declarations.iter().all(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::BinaryExpression | SyntaxKind::CallExpression
                    ) && crate::binder::get_assignment_declaration_kind(d)
                        != crate::binder::bind_js_assignment_declarations::JsDeclarationKind::None
                });
            if umd_global || js_assignment_only {
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
    pub(crate) fn check_external_module_export_duplicates(&mut self, statements: &[Arc<Node>]) {
        let Some(module_symbol) = self.current_file_symbol.clone() else {
            return;
        };
        // Go binder declareSymbol 重放：顶层 default 声明按各形式的
        // includes/excludes 合并；仅 export default EA（excludes=ALL）与
        // 既有 default 冲突时对每处声明报 2528 且不入表（表保旧符号）
        let mut default_table: Option<(SymbolFlags, Vec<Arc<Node>>)> = None;
        for stmt in statements {
            for (includes, excludes, decl_node) in default_decl_semantics(stmt) {
                match &mut default_table {
                    Some((flags, decls)) if flags.intersects(excludes) => {
                        for d in decls.iter() {
                            let file = self.current_file.clone();
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                default_export_name_loc(d),
                                tsox_core::diagnostics::messages_generated::
                                    A_MODULE_CANNOT_HAVE_MULTIPLE_DEFAULT_EXPORTS,
                                Vec::new(),
                            ));
                        }
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            default_export_name_loc(&decl_node),
                            tsox_core::diagnostics::messages_generated::
                                A_MODULE_CANNOT_HAVE_MULTIPLE_DEFAULT_EXPORTS,
                            Vec::new(),
                        ));
                    }
                    Some((flags, decls)) => {
                        *flags |= includes;
                        decls.push(Arc::clone(&decl_node));
                    }
                    None => {
                        default_table = Some((includes, vec![Arc::clone(&decl_node)]));
                    }
                }
            }
        }
        // 存活 default 符号走 exports 循环同款 2323 规则（Go checkExternalModuleExports
        // 的 getExportsOfModule 含 "default" 符号，与普通导出名一视同仁）
        if let Some((flags, decls)) = &default_table {
            self.report_export_declarations_2323("default".to_string(), *flags, decls);
        }
        let exports = self.get_exports_of_module_table(&module_symbol);
        for (name, symbol) in exports.entries.iter() {
            if name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_STAR
                || name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                || name == "default"
            {
                continue;
            }
            let umd_global = symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::NamespaceExportDeclaration);
            if umd_global {
                continue;
            }
            let flags = self.get_symbol_flags(symbol);
            if flags.intersects(SymbolFlags::NAMESPACE | SymbolFlags::ENUM) {
                continue;
            }
            // Go binder 不把本模块 ExportSpecifier 并入导出符号声明集，
            // 计数只含本地声明；specifier 归并是本地绑定形态的差异
            let local_declarations: Vec<Arc<Node>> = symbol
                .declarations
                .iter()
                .filter(|d| {
                    !matches!(
                        d.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExport
                    )
                })
                .cloned()
                .collect();
            self.report_export_declarations_2323(name.clone(), flags, &local_declarations);
        }
    }

    // Go checkExternalModuleExports 的 export 冲突段：同一导出名多个
    // 非重载声明时报 TS2323（namespace/enum/接口合并/类型别名合并除外）
    fn report_export_declarations_2323(
        &mut self,
        name: String,
        flags: SymbolFlags,
        declarations: &[Arc<Node>],
    ) {
        let is_not_overload = |d: &Arc<Node>| {
            !matches!(d.kind, SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration)
                || body_of(d).is_some()
        };
        let exported_declarations_count = declarations
            .iter()
            .filter(|d| {
                is_not_overload(d)
                    && !matches!(d.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
                    && d.kind != SyntaxKind::InterfaceDeclaration
            })
            .count();
        if flags.intersects(SymbolFlags::TypeAlias) && exported_declarations_count <= 2 {
            return;
        }
        if exported_declarations_count > 1
            && !declarations.iter().all(|d| {
                crate::binder::get_assignment_declaration_kind(d)
                    == crate::binder::bind_js_assignment_declarations::JsDeclarationKind::ExportsProperty
            })
        {
            for declaration in declarations.iter() {
                if is_not_overload(declaration) {
                    let loc = declaration_name_loc(declaration).unwrap_or(declaration.loc);
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

// Go bindExportAssignment/bindFunctionDeclaration/bindClassLikeDeclaration/
// bindBlockScopedDeclaration 的 includes/excludes（default 顶层声明）；
// re-export 的 default 命名 specifier 走 declareSymbol(Alias, AliasExcludes)
fn default_decl_semantics(stmt: &Arc<Node>) -> Vec<(SymbolFlags, SymbolFlags, Arc<Node>)> {
    if let tsox_frontend::ast::NodeData::ExportAssignment(d) = &stmt.data {
        if !d.is_export_equals {
            let includes = if crate::binder::bind_js_assignment_declarations::expression_is_alias(
                &d.expression,
            ) {
                SymbolFlags::Alias
            } else {
                SymbolFlags::Property
            };
            return vec![(includes, SymbolFlags::all(), Arc::clone(stmt))];
        }
        return Vec::new();
    }
    if let tsox_frontend::ast::NodeData::ExportDeclaration(ed) = &stmt.data {
        let mut out = Vec::new();
        if let Some(clause) = &ed.export_clause
            && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
        {
            for el in ne.elements.iter() {
                if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data
                    && spec.name.text().trim_matches(['"', '\'', '`']) == "default"
                {
                    out.push((SymbolFlags::Alias, SymbolFlags::Alias, Arc::clone(el)));
                }
            }
        }
        return out;
    }
    if !stmt.has_syntactic_modifier(ModifierFlags::Default) {
        return Vec::new();
    }
    let pair = match stmt.kind {
        SyntaxKind::FunctionDeclaration => {
            Some((SymbolFlags::Function, SymbolFlags::FunctionExcludes))
        }
        SyntaxKind::ClassDeclaration => Some((SymbolFlags::Class, SymbolFlags::ClassExcludes)),
        SyntaxKind::InterfaceDeclaration => {
            Some((SymbolFlags::Interface, SymbolFlags::InterfaceExcludes))
        }
        _ => None,
    };
    pair.map(|(i, e)| (i, e, Arc::clone(stmt)))
        .into_iter()
        .collect()
}

// Go GetNameOfDeclaration 或节点本身（2528 的声明名定位）
fn default_export_name_loc(d: &Arc<Node>) -> tsox_core::core::text::TextRange {
    if let tsox_frontend::ast::NodeData::ExportAssignment(data) = &d.data {
        if !data.is_export_equals {
            return data.expression.loc;
        }
    }
    if d.kind == SyntaxKind::ExportSpecifier {
        return d.name().map(|n| n.loc).unwrap_or(d.loc);
    }
    d.name().map(|n| n.loc).unwrap_or(d.loc)
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
