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
        let export_equals = statements.iter().find(|s| {
            matches!(
                &s.data,
                tsox_frontend::ast::NodeData::ExportAssignment(d) if d.is_export_equals
            )
        });
        let Some(eq_decl) = export_equals else { return };
        let has_other_value_export = statements.iter().any(|s| {
            if Arc::ptr_eq(s, eq_decl) {
                return false;
            }
            let value_declaring = match s.kind {
                SyntaxKind::ModuleDeclaration => {
                    tsox_frontend::ast::utilities::get_module_instance_state(s)
                        != tsox_frontend::ast::utilities::ModuleInstanceState::NonInstantiated
                }
                SyntaxKind::ClassDeclaration
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::VariableStatement => true,
                _ => false,
            };
            value_declaring && s.has_syntactic_modifier(ModifierFlags::Export)
        });
        if has_other_value_export {
            let file = self.current_file.clone();
            let diagnostic = tsox_frontend::ast::Diagnostic::new(
                file,
                eq_decl.loc,
                tsox_core::diagnostics::messages_generated::
                    AN_EXPORT_ASSIGNMENT_CANNOT_BE_USED_IN_A_MODULE_WITH_OTHER_EXPORTED_ELEMENTS,
                Vec::new(),
            );
            self.diagnostics.add(diagnostic);
            return;
        }

        // Go hasShadowedNamespace：export= 指向含类型/命名空间成员的命名空间，
        // 且模块自身也导出类型/命名空间成员 → 同样报 TS2309
        let eq_symbol = self
            .program
            .symbol_map()
            .symbol_of(eq_decl)
            .cloned();
        let target = eq_symbol.map(|s| self.resolve_export_equals_target(&s));
        let target_has_type = target.as_ref().is_some_and(|t| {
            t.flags.intersects(SymbolFlags::NAMESPACE)
                && t
                    .exports
                    .iter()
                    .chain(t.members.iter())
                    .any(|(_, m)| m.flags.intersects(SymbolFlags::TYPE | SymbolFlags::NAMESPACE))
        });
        if !target_has_type {
            return;
        }
        let module_has_type = statements.iter().any(|s| {
            if Arc::ptr_eq(s, eq_decl) || !s.has_syntactic_modifier(ModifierFlags::Export) {
                return false;
            }
            matches!(
                s.kind,
                SyntaxKind::TypeAliasDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ModuleDeclaration
            )
        });
        if module_has_type {
            let file = self.current_file.clone();
            let diagnostic = tsox_frontend::ast::Diagnostic::new(
                file,
                eq_decl.loc,
                tsox_core::diagnostics::messages_generated::
                    AN_EXPORT_ASSIGNMENT_CANNOT_BE_USED_IN_A_MODULE_WITH_OTHER_EXPORTED_ELEMENTS,
                Vec::new(),
            );
            self.diagnostics.add(diagnostic);
        }
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
        let default_locs: Vec<tsox_core::core::text::TextRange> = statements
            .iter()
            .filter(|s| {
                s.kind != SyntaxKind::InterfaceDeclaration
                    && (matches!(&s.data, tsox_frontend::ast::NodeData::ExportAssignment(d) if !d.is_export_equals)
                        || s.has_syntactic_modifier(ModifierFlags::Default))
            })
            .map(|s| match &s.data {
                tsox_frontend::ast::NodeData::ExportAssignment(d) if !d.is_export_equals => {
                    if d.expression.kind == SyntaxKind::Identifier {
                        d.expression.loc
                    } else {
                        s.loc
                    }
                }
                _ => declaration_name_loc(s).unwrap_or(s.loc),
            })
            .collect();
        if default_locs.len() > 1 {
            for loc in default_locs {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    loc,
                    tsox_core::diagnostics::messages_generated::
                        A_MODULE_CANNOT_HAVE_MULTIPLE_DEFAULT_EXPORTS,
                    Vec::new(),
                ));
            }
        }
        let exports = self.get_exports_of_module_table(&module_symbol);
        for (name, symbol) in exports.entries.iter() {
            if name == "export*" || name == "export=" {
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
    d.name().map(|n| n.loc)
}
