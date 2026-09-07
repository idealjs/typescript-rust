#![allow(unused_imports)]

use super::*;

impl NameResolver {
    pub fn use_outer_variable_scope_in_parameter(
        &self,
        result: &Arc<Symbol>,
        location: &Arc<Node>,
        last_location: Option<&Arc<Node>>,
    ) -> bool {
        if let Some(last) = last_location {
            if is_parameter_declaration(last) {
                let body: Option<&Arc<Node>> = None;
                if let Some(body) = body {
                    if let Some(value_decl) = &result.value_declaration {
                        if value_decl.pos() >= body.pos() && value_decl.end() <= body.end() {
                            let function_location = Arc::clone(location);
                            let mut declaration_requires_scope_change = Tristate::Unknown;
                            if let Some(get_cache) = &self.get_requires_scope_change_cache_fn {
                                declaration_requires_scope_change = get_cache(&function_location);
                            }
                            if declaration_requires_scope_change.is_unknown() {
                                declaration_requires_scope_change = Tristate::False;
                                if let Some(set_cache) = &self.set_requires_scope_change_cache_fn {
                                    set_cache(
                                        &function_location,
                                        declaration_requires_scope_change,
                                    );
                                }
                            }
                            return !declaration_requires_scope_change.is_true();
                        }
                    }
                }
            }
        }
        false
    }

    pub fn requires_scope_change(&self, _node: &Arc<Node>) -> bool {
        let name: Option<&Arc<Node>> = None;
        let initializer: Option<&Arc<Node>> = None;
        let name_change = name
            .map(|n| self.requires_scope_change_worker(n))
            .unwrap_or(false);
        let init_change = initializer
            .map(|i| self.requires_scope_change_worker(i))
            .unwrap_or(false);
        name_change || init_change
    }

    pub fn requires_scope_change_worker(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor => false,
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::PropertyAssignment => {
                let name: Option<&Arc<Node>> = None;
                name.map(|n| self.requires_scope_change_worker(n))
                    .unwrap_or(false)
            }
            SyntaxKind::PropertyDeclaration => {
                if has_static_modifier(node) {
                    return self
                        .compiler_options
                        .as_ref()
                        .map(|o| !o.get_emit_standard_class_fields())
                        .unwrap_or(false);
                }

                let name: Option<&Arc<Node>> = None;
                name.map(|n| self.requires_scope_change_worker(n))
                    .unwrap_or(false)
            }
            _ => {
                if is_nullish_coalesce(node) || is_optional_chain(node) {
                    return self
                        .compiler_options
                        .as_ref()
                        .map(|o| o.get_emit_script_target() < ScriptTarget::ES2020)
                        .unwrap_or(false);
                }
                if is_binding_element(node) {
                    let is_dotdotdot_in_object_pattern = false;
                    if is_dotdotdot_in_object_pattern {
                        return self
                            .compiler_options
                            .as_ref()
                            .map(|o| o.get_emit_script_target() < ScriptTarget::ES2017)
                            .unwrap_or(false);
                    }
                }
                if is_type_node(node) {
                    return false;
                }

                false
            }
        }
    }

    pub fn error(
        &self,
        location: &Arc<Node>,
        message: &Message,
        args: &[String],
    ) -> Option<Diagnostic> {
        if let Some(callback) = &self.error_fn {
            return callback(location, message, args);
        }

        None
    }

    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        if let Some(callback) = &self.get_symbol_of_declaration_fn {
            return callback(node);
        }

        node_symbol(node)
    }

    pub fn lookup(
        &self,
        symbols: &SymbolTable,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        if let Some(callback) = &self.lookup_fn {
            return callback(symbols, name, meaning);
        }

        if !meaning.is_empty() {
            if let Some(symbol) = symbols.get(name) {
                if symbol.flags.intersects(meaning) {
                    return Some(Arc::clone(symbol));
                }
            }
        }
        None
    }

    pub fn arguments_symbol(&mut self) -> Arc<Symbol> {
        if self.arguments_symbol.is_none() {
            self.arguments_symbol = Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "arguments",
            )));
        }
        Arc::clone(self.arguments_symbol.as_ref().unwrap())
    }

    pub(crate) fn resolve_module_exports_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        _result: &mut Option<Arc<Symbol>>,
    ) {
    }

    pub(crate) fn resolve_enum_case(
        &self,
        original_location: &Arc<Node>,
        _location: &Arc<Node>,
        name: &str,
        name_not_found_message: Option<&Message>,
        _result: &mut Option<Arc<Symbol>>,
    ) {
        if let Some(_message) = name_not_found_message {
            if let Some(opts) = &self.compiler_options {
                if opts.get_isolated_modules() {
                    let isolated_modules_like_flag_name =
                        if opts.verbatim_module_syntax == Tristate::True {
                            "verbatimModuleSyntax"
                        } else {
                            "isolatedModules"
                        };
                    self.error(
                        original_location,
                        &crate::diagnostics::messages_generated::CANNOT_ACCESS_0_FROM_ANOTHER_FILE_WITHOUT_QUALIFICATION_WHEN_1_IS_ENABLED_USE_2_INSTEAD,
                        &[
                            name.to_string(),
                            isolated_modules_like_flag_name.to_string(),

                            name.to_string(),
                        ],
                    );
                }
            }
        }
    }

    pub(crate) fn resolve_property_declaration_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        _property_with_invalid_initializer: &mut Option<Arc<Node>>,
    ) {
    }

    pub(crate) fn resolve_class_or_interface_case(
        &self,
        original_location: &Arc<Node>,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        name_not_found_message: Option<&Message>,
        _last_location: Option<&Arc<Node>>,
        _result: &mut Option<Arc<Symbol>>,
    ) {
        if name_not_found_message.is_some() {
            self.error(
                original_location,
                &crate::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                &[],
            );
        }
    }

    pub(crate) fn resolve_expression_with_type_arguments_case(
        &self,
        original_location: &Arc<Node>,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        name_not_found_message: Option<&Message>,
        _last_location: Option<&Arc<Node>>,
        _result: &mut Option<Arc<Symbol>>,
    ) -> bool {
        if name_not_found_message.is_some() {
            self.error(
                original_location,
                &crate::diagnostics::messages_generated::BASE_CLASS_EXPRESSIONS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                &[],
            );
        }
        false
    }

    pub(crate) fn resolve_computed_property_name_case(
        &self,
        original_location: &Arc<Node>,
        _grandparent: &Option<Arc<Node>>,
        _name: &str,
        _meaning: SymbolFlags,
        name_not_found_message: Option<&Message>,
        _result: &mut Option<Arc<Symbol>>,
    ) -> bool {
        if name_not_found_message.is_some() {
            self.error(
                original_location,
                &crate::diagnostics::messages_generated::A_COMPUTED_PROPERTY_NAME_CANNOT_REFERENCE_A_TYPE_PARAMETER_FROM_ITS_CONTAINING_TYPE,
                &[],
            );
        }
        false
    }

    pub(crate) fn resolve_function_expression_name_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
    ) -> bool {
        false
    }

    pub(crate) fn track_parameter_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {
    }

    pub(crate) fn track_binding_element_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {
    }

    pub(crate) fn resolve_infer_type_case(&self, _location: &Arc<Node>, _name: &str) -> bool {
        false
    }
}
