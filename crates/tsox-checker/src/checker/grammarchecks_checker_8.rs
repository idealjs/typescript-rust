#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub fn check_grammar_jsx_element(&mut self, node: &Arc<Node>) -> bool {
        let tag_name = match crate::checker::jsx::jsx_tag_name(node) {
            Some(t) => t,
            None => return false,
        };

        if self.check_grammar_jsx_name(&tag_name) {
            return true;
        }

        let type_args: Option<Vec<Arc<Node>>> = match &node.data {
            NodeData::JsxOpeningElement(data) => data
                .type_arguments
                .as_ref()
                .map(|l| l.iter().cloned().collect()),
            NodeData::JsxSelfClosingElement(data) => data
                .type_arguments
                .as_ref()
                .map(|l| l.iter().cloned().collect()),
            _ => None,
        };
        if let Some(args) = type_args {
            if !args.is_empty() {
                let count = args.len().to_string();
                return self.grammar_error_on_node_with_args(
                    node,
                    &EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    &["0".to_string(), count],
                );
            }
        }

        let attrs = match crate::checker::jsx::jsx_attributes(node) {
            Some(a) => a,
            None => return false,
        };
        let properties: Vec<Arc<Node>> = match &attrs.data {
            NodeData::JsxAttributes(data) => data.properties.iter().cloned().collect(),
            _ => return false,
        };

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for attr in &properties {
            if attr.kind == SyntaxKind::JsxSpreadAttribute {
                continue;
            }
            let (name_node, initializer) = match &attr.data {
                NodeData::JsxAttribute(d) => (Arc::clone(&d.name), d.initializer.clone()),
                _ => continue,
            };
            let text = name_node.text().to_string();
            if !seen.insert(text.clone()) {
                return self.grammar_error_on_node(
                    &name_node,
                    &JSX_ELEMENTS_CANNOT_HAVE_MULTIPLE_ATTRIBUTES_WITH_THE_SAME_NAME,
                );
            }
            if let Some(init) = initializer {
                if init.kind == SyntaxKind::JsxExpression {
                    if let NodeData::JsxExpression(d) = &init.data {
                        if d.expression.is_none() {
                            return self.grammar_error_on_node(
                                &init,
                                &JSX_ATTRIBUTES_MUST_ONLY_BE_ASSIGNED_A_NON_EMPTY_EXPRESSION,
                            );
                        }
                    }
                }
            }
        }

        false
    }

    pub fn check_grammar_jsx_name(&mut self, node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::PropertyAccessExpression {
            if let NodeData::PropertyAccessExpression(data) = &node.data {
                let expr = &data.expression;
                if is_jsx_namespaced_name(expr) {
                    return self.grammar_error_on_node(
                        expr,
                        &JSX_PROPERTY_ACCESS_EXPRESSIONS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                    );
                }
            }
        }

        if is_jsx_namespaced_name(node) && self.is_jsx_transform_enabled() {
            let namespace_text = match &node.data {
                NodeData::JsxNamespacedName(data) => data.namespace.text().to_string(),
                _ => String::new(),
            };
            if !crate::checker::jsx::is_intrinsic_jsx_name(&namespace_text) {
                return self.grammar_error_on_node(
                    node,
                    &REACT_COMPONENTS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                );
            }
        }
        false
    }

    pub fn check_grammar_jsx_expression(&mut self, node: &Arc<Node>) -> bool {
        let expr = match &node.data {
            NodeData::JsxExpression(data) => &data.expression,
            _ => return false,
        };
        let Some(expr) = expr else { return false };

        if is_comma_sequence(expr) {
            return self.grammar_error_on_node(
                expr,
                &JSX_EXPRESSIONS_MAY_NOT_USE_THE_COMMA_OPERATOR_DID_YOU_MEAN_TO_WRITE_AN_ARRAY,
            );
        }
        false
    }

    pub(crate) fn is_jsx_transform_enabled(&self) -> bool {
        self.compiler_options.jsx != tsox_core::core::compiler_options::JsxEmit::None
    }

    pub fn grammar_error_on_node_skipped_on_no_emit(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
    ) -> bool {
        self.grammar_error_on_node(node, message)
    }

    pub fn check_grammar_regular_expression_literal(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_private_identifier_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_mapped_type(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_decorator(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_export_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_module_element_context(
        &mut self,
        _node: &Arc<Node>,
        _error_message: &Message,
    ) -> bool {
        false
    }

    pub fn report_obvious_modifier_errors(&mut self, node: &Arc<Node>) -> bool {
        let Some(modifier) = self.find_first_illegal_modifier(node) else {
            return false;
        };
        self.grammar_error_on_first_token(&modifier, &MODIFIERS_CANNOT_APPEAR_HERE)
    }

    fn modifier_nodes_of(node: &Arc<Node>) -> Vec<Arc<Node>> {
        node.modifiers()
            .map(|ml| ml.list.nodes.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn first_modifier(node: &Arc<Node>) -> Option<Arc<Node>> {
        Self::modifier_nodes_of(node)
            .into_iter()
            .find(|m| m.kind != SyntaxKind::Decorator)
    }

    pub fn find_first_modifier_except(
        &self,
        node: &Arc<Node>,
        allowed_modifier: SyntaxKind,
    ) -> Option<Arc<Node>> {
        Self::modifier_nodes_of(node)
            .into_iter()
            .find(|m| m.kind != SyntaxKind::Decorator && m.kind != allowed_modifier)
    }

    pub fn find_first_illegal_modifier(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::Parameter
            | SyntaxKind::TypeParameter
            | SyntaxKind::JSTypeAliasDeclaration => None,
            SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::MissingDeclaration => Self::first_modifier(node),
            _ => {
                if node.parent().is_some_and(|p| {
                    matches!(p.kind, SyntaxKind::ModuleBlock | SyntaxKind::SourceFile)
                }) {
                    return None;
                }
                match node.kind {
                    SyntaxKind::FunctionDeclaration => {
                        self.find_first_modifier_except(node, SyntaxKind::AsyncKeyword)
                    }
                    SyntaxKind::ClassDeclaration | SyntaxKind::ConstructorType => {
                        self.find_first_modifier_except(node, SyntaxKind::AbstractKeyword)
                    }
                    SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration => Self::first_modifier(node),
                    SyntaxKind::VariableStatement => {
                        let is_using = matches!(
                            &node.data,
                            NodeData::VariableStatement(d)
                                if d.declaration_list.flags.contains(NodeFlags::Using)
                        );
                        if is_using {
                            self.find_first_modifier_except(node, SyntaxKind::AwaitKeyword)
                        } else {
                            Self::first_modifier(node)
                        }
                    }
                    SyntaxKind::EnumDeclaration => {
                        self.find_first_modifier_except(node, SyntaxKind::ConstKeyword)
                    }
                    _ => None,
                }
            }
        }
    }

    pub fn report_obvious_decorator_errors(&mut self, node: &Arc<Node>) -> bool {
        let Some(decorator) = self.find_first_illegal_decorator(node) else {
            return false;
        };
        self.grammar_error_on_first_token(&decorator, &DECORATORS_ARE_NOT_VALID_HERE)
    }

    pub fn find_first_illegal_decorator(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        // Go ast.CanHaveIllegalDecorators
        let can_have_illegal = matches!(
            node.kind,
            SyntaxKind::PropertyAssignment
                | SyntaxKind::ShorthandPropertyAssignment
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::IndexSignature
                | SyntaxKind::ClassStaticBlockDeclaration
                | SyntaxKind::MissingDeclaration
                | SyntaxKind::VariableStatement
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::ModuleDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::ImportDeclaration
                | SyntaxKind::JSImportDeclaration
        );
        if !can_have_illegal {
            return None;
        }
        Self::modifier_nodes_of(node)
            .into_iter()
            .find(|m| m.kind == SyntaxKind::Decorator)
    }

    pub fn check_grammar_async_modifier(
        &mut self,
        _node: &Arc<Node>,
        _async_modifier: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_disallowed_trailing_comma(
        &mut self,
        _list: &tsox_frontend::ast::NodeList,
        _diag: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_type_parameter_list(
        &mut self,
        _type_parameters: &tsox_frontend::ast::NodeList,
        _file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_use_strict_simple_parameter_list(
        &mut self,
        _node: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_function_like_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_class_like_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_arrow_function(
        &mut self,
        _node: &Arc<Node>,
        _file: &Arc<tsox_frontend::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_index_signature_parameters(&mut self, node: &Arc<Node>) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let NodeData::IndexSignatureDeclaration(data) = &node.data else {
            return false;
        };
        let params = &data.parameters;
        if params.nodes.is_empty() {
            return self.grammar_error_on_node(node, &msg::AN_INDEX_SIGNATURE_MUST_HAVE_EXACTLY_ONE_PARAMETER);
        }
        let parameter = Arc::clone(&params.nodes[0]);
        if params.nodes.len() != 1 {
            let name = parameter.name().cloned().unwrap_or_else(|| Arc::clone(&parameter));
            return self.grammar_error_on_node(&name, &msg::AN_INDEX_SIGNATURE_MUST_HAVE_EXACTLY_ONE_PARAMETER);
        }
        let NodeData::ParameterDeclaration(pd) = &parameter.data else {
            return false;
        };
        if let Some(rest) = &pd.dot_dot_dot_token {
            return self.grammar_error_on_node(rest, &msg::AN_INDEX_SIGNATURE_CANNOT_HAVE_A_REST_PARAMETER);
        }
        if let Some(modifiers) = &pd.modifiers
            && modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            )
        {
            // Go checkParameter：非构造器实现的参数属性先报 TS2369（param 节点位）
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                parameter.loc,
                msg::A_PARAMETER_PROPERTY_IS_ONLY_ALLOWED_IN_A_CONSTRUCTOR_IMPLEMENTATION,
                Vec::new(),
            ));
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_CANNOT_HAVE_AN_ACCESSIBILITY_MODIFIER,
            );
        }
        if pd.modifiers.is_some() {
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_CANNOT_HAVE_AN_ACCESSIBILITY_MODIFIER,
            );
        }
        if let Some(question) = &pd.question_token {
            return self.grammar_error_on_node(
                question,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_CANNOT_HAVE_A_QUESTION_MARK,
            );
        }
        if pd.initializer.is_some() {
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
            );
        }
        let Some(type_node) = &pd.type_node else {
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_MUST_HAVE_A_TYPE_ANNOTATION,
            );
        };
        let t = self.get_type_from_type_node(type_node);
        let literal_or_generic = if t.is_union() {
            t.types().is_some_and(|parts| {
                parts.iter().any(|c| {
                    c.flags.intersects(crate::checker::types::TYPE_FLAGS_LITERAL)
                        || c.flags.contains(crate::checker::types::TypeFlags::UniqueESSymbol)
                        || self.is_generic_type(c)
                })
            })
        } else {
            t.flags.intersects(crate::checker::types::TYPE_FLAGS_LITERAL)
                || t.flags.contains(crate::checker::types::TypeFlags::UniqueESSymbol)
                || self.is_generic_type(&t)
        };
        if literal_or_generic {
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_TYPE_CANNOT_BE_A_LITERAL_TYPE_OR_GENERIC_TYPE_CONSIDER_USING_A_MAPPED_OBJECT_TYPE_INSTEAD,
            );
        }
        if data.type_node.kind == SyntaxKind::MissingDeclaration {
            return self.grammar_error_on_node(
                node,
                &msg::AN_INDEX_SIGNATURE_MUST_HAVE_A_TYPE_ANNOTATION,
            );
        }
        let valid_key = if t.is_union() {
            t.types().is_some_and(|parts| {
                parts.iter().all(|c| self.index_key_type_is_valid(c))
            })
        } else {
            self.index_key_type_is_valid(&t)
        };
        if !valid_key {
            return self.grammar_error_on_node(
                &pd.name,
                &msg::AN_INDEX_SIGNATURE_PARAMETER_TYPE_MUST_BE_STRING_NUMBER_SYMBOL_OR_A_TEMPLATE_LITERAL_TYPE,
            );
        }
        false
    }

    fn index_key_type_is_valid(&self, t: &Arc<crate::checker::types::Type>) -> bool {
        t.flags.intersects(crate::checker::types::TYPE_FLAGS_STRING_LIKE) || t.intrinsic_name() == Some("string")
            || t.intrinsic_name() == Some("number")
            || t.intrinsic_name() == Some("symbol")
    }

    pub fn check_grammar_index_signature(&mut self, node: &Arc<Node>) -> bool {
        if self.check_grammar_modifiers(node) {
            return true;
        }
        self.check_grammar_index_signature_parameters(node)
    }

    pub fn check_grammar_for_at_least_one_type_argument(
        &mut self,
        _node: &Arc<Node>,
        _type_arguments: &tsox_frontend::ast::NodeList,
    ) -> bool {
        false
    }

    pub fn check_grammar_type_arguments(
        &mut self,
        _node: &Arc<Node>,
        _type_arguments: &tsox_frontend::ast::NodeList,
    ) -> bool {
        false
    }

    pub fn check_grammar_tagged_template_chain(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_heritage_clause(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_expression_with_type_arguments(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_class_declaration_heritage_clauses(&mut self, node: &Arc<Node>) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let heritage = match &node.data {
            NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
            NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
            _ => return false,
        };
        let Some(clauses) = heritage else {
            return false;
        };
        let mut seen_extends = false;
        let mut seen_implements = false;
        for clause in clauses.iter() {
            let NodeData::HeritageClause(h) = &clause.data else {
                continue;
            };
            if h.token == SyntaxKind::ExtendsKeyword {
                if seen_extends {
                    return self.grammar_error_on_node(clause, &msg::X_EXTENDS_CLAUSE_ALREADY_SEEN);
                }
                if seen_implements {
                    return self
                        .grammar_error_on_node(clause, &msg::X_EXTENDS_CLAUSE_MUST_PRECEDE_IMPLEMENTS_CLAUSE);
                }
                if h.types.nodes.len() > 1 {
                    return self.grammar_error_on_node(
                        &h.types.nodes[1],
                        &msg::CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
                    );
                }
                seen_extends = true;
            } else if h.token == SyntaxKind::ImplementsKeyword {
                if seen_implements {
                    return self
                        .grammar_error_on_node(clause, &msg::X_IMPLEMENTS_CLAUSE_ALREADY_SEEN);
                }
                seen_implements = true;
            }
        }
        false
    }

    pub fn check_grammar_interface_declaration(&mut self, node: &Arc<Node>) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let heritage = match &node.data {
            NodeData::InterfaceDeclaration(d) => d.heritage_clauses.as_ref(),
            _ => return false,
        };
        let Some(clauses) = heritage else {
            return false;
        };
        let mut seen_extends = false;
        for clause in clauses.iter() {
            let NodeData::HeritageClause(h) = &clause.data else {
                continue;
            };
            match h.token {
                SyntaxKind::ExtendsKeyword => {
                    if seen_extends {
                        return self
                            .grammar_error_on_node(clause, &msg::X_EXTENDS_CLAUSE_ALREADY_SEEN);
                    }
                    seen_extends = true;
                }
                SyntaxKind::ImplementsKeyword => {
                    return self.grammar_error_on_node(
                        clause,
                        &msg::INTERFACE_DECLARATION_CANNOT_HAVE_IMPLEMENTS_CLAUSE,
                    );
                }
                _ => {}
            }
        }
        false
    }

    pub fn check_grammar_computed_property_name(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_generator(&mut self, _node: &Arc<Node>) -> bool {
        false
    }
}
