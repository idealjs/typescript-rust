#![allow(dead_code)]

use crate::ast::*;
use crate::core::compiler_options::{CompilerOptions, ScriptTarget};
use crate::core::tristate::Tristate;
use crate::diagnostics::Message;
use std::sync::Arc;

pub struct NameResolver {
    pub compiler_options: Option<Arc<CompilerOptions>>,
    pub get_symbol_of_declaration_fn: Option<Box<dyn Fn(&Arc<Node>) -> Option<Arc<Symbol>>>>,
    pub error_fn: Option<Box<dyn Fn(&Arc<Node>, &Message, &[String]) -> Option<Diagnostic>>>,
    pub globals: SymbolTable,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub lookup_fn: Option<Box<dyn Fn(&SymbolTable, &str, SymbolFlags) -> Option<Arc<Symbol>>>>,
    pub symbol_referenced_fn: Option<Box<dyn Fn(&Arc<Symbol>, SymbolFlags)>>,
    pub set_requires_scope_change_cache_fn: Option<Box<dyn Fn(&Arc<Node>, Tristate)>>,
    pub get_requires_scope_change_cache_fn: Option<Box<dyn Fn(&Arc<Node>) -> Tristate>>,
    pub on_property_with_invalid_initializer_fn:
        Option<Box<dyn Fn(&Arc<Node>, &str, &Arc<Node>, Option<&Arc<Symbol>>) -> bool>>,
    pub on_failed_to_resolve_symbol_fn:
        Option<Box<dyn Fn(&Arc<Node>, &str, SymbolFlags, &Message)>>,
    pub on_successfully_resolved_symbol_fn: Option<
        Box<
            dyn Fn(
                &Arc<Node>,
                &Arc<Symbol>,
                SymbolFlags,
                Option<&Arc<Node>>,
                Option<&Arc<Node>>,
                bool,
            ),
        >,
    >,
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl NameResolver {

    pub fn new() -> Self {
        Self {
            compiler_options: None,
            get_symbol_of_declaration_fn: None,
            error_fn: None,
            globals: SymbolTable::new(),
            arguments_symbol: None,
            require_symbol: None,
            lookup_fn: None,
            symbol_referenced_fn: None,
            set_requires_scope_change_cache_fn: None,
            get_requires_scope_change_cache_fn: None,
            on_property_with_invalid_initializer_fn: None,
            on_failed_to_resolve_symbol_fn: None,
            on_successfully_resolved_symbol_fn: None,
        }
    }

    pub fn resolve(
        &mut self,
        location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
        name_not_found_message: Option<&Message>,
        is_use: bool,
        exclude_globals: bool,
    ) -> Option<Arc<Symbol>> {
        let mut result: Option<Arc<Symbol>> = None;
        let mut last_location: Option<Arc<Node>> = None;
        let mut last_self_reference_location: Option<Arc<Node>> = None;
        let mut property_with_invalid_initializer: Option<Arc<Node>> = None;
        let mut associated_declaration_for_containing_initializer_or_binding_name: Option<
            Arc<Node>,
        > = None;
        let mut within_deferred_context = false;
        let mut grandparent: Option<Arc<Node>>;

        let original_location = Arc::clone(location);
        let name_is_const = name == "const";

        let mut current = Some(Arc::clone(location));
        'outer: while let Some(node) = current {
            if name_is_const && is_const_assertion(&node) {

                return None;
            }
            if is_module_or_enum_declaration(&node)
                && last_location.as_ref().map_or(false, |last| {
                    node.name().map(|n| Arc::ptr_eq(n, last)).unwrap_or(false)
                })
            {

                let parent = node.parent.clone();
                last_location = Some(node);
                current = parent;
                continue 'outer;
            }

            let locals: Option<&SymbolTable> = None;
            if let Some(locals) = locals {
                if !is_global_source_file(&node) {
                    result = self.lookup(locals, name, meaning);
                    if result.is_some() {
                        let mut use_result = true;

                        let _ = &mut use_result;
                        if use_result {
                            break 'outer;
                        }
                        result = None;
                    }
                }
            }
            within_deferred_context =
                within_deferred_context || get_is_deferred_context(&node, last_location.as_ref());
            match node.kind {
                SyntaxKind::SourceFile => {

                    self.resolve_module_exports_case(&node, name, meaning, &mut result);
                }
                SyntaxKind::ModuleDeclaration => {

                    self.resolve_module_exports_case(&node, name, meaning, &mut result);
                }
                SyntaxKind::EnumDeclaration => {

                    self.resolve_enum_case(
                        &original_location,
                        &node,
                        name,
                        name_not_found_message,
                        &mut result,
                    );
                    if result.is_some() {
                        break 'outer;
                    }
                }
                SyntaxKind::PropertyDeclaration => {
                    if !is_static(&node) {

                        self.resolve_property_declaration_case(
                            &node,
                            name,
                            meaning,
                            &mut property_with_invalid_initializer,
                        );
                    }
                }
                SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression
                | SyntaxKind::InterfaceDeclaration => {

                    self.resolve_class_or_interface_case(
                        &original_location,
                        &node,
                        name,
                        meaning,
                        name_not_found_message,
                        last_location.as_ref(),
                        &mut result,
                    );
                    if result.is_some() {
                        break 'outer;
                    }
                }
                SyntaxKind::ExpressionWithTypeArguments => {

                    let should_return_nil = self.resolve_expression_with_type_arguments_case(
                        &original_location,
                        &node,
                        name,
                        meaning,
                        name_not_found_message,
                        last_location.as_ref(),
                        &mut result,
                    );
                    if should_return_nil {
                        return None;
                    }
                }
                SyntaxKind::ComputedPropertyName => {

                    grandparent = node.parent.as_ref().and_then(|p| p.parent.clone());
                    let should_return_nil = self.resolve_computed_property_name_case(
                        &original_location,
                        &grandparent,
                        name,
                        meaning,
                        name_not_found_message,
                        &mut result,
                    );
                    if should_return_nil {
                        return None;
                    }
                }
                SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::FunctionDeclaration => {
                    if meaning.intersects(SymbolFlags::VARIABLE) && name == "arguments" {
                        result = Some(self.arguments_symbol());
                        break 'outer;
                    }
                }
                SyntaxKind::FunctionExpression => {
                    if meaning.intersects(SymbolFlags::VARIABLE) && name == "arguments" {
                        result = Some(self.arguments_symbol());
                        break 'outer;
                    }
                    if meaning.intersects(SymbolFlags::Function) {

                        if self.resolve_function_expression_name_case(&node, name) {
                            result = node_symbol(&node);
                            break 'outer;
                        }
                    }
                }
                SyntaxKind::Decorator => {

                    let mut next = Arc::clone(&node);
                    if let Some(parent) = &node.parent {
                        if parent.kind == SyntaxKind::Parameter {
                            next = Arc::clone(parent);
                        }
                    }
                    if let Some(parent) = &node.parent {
                        if is_class_element(parent) || parent.kind == SyntaxKind::ClassDeclaration {
                            next = Arc::clone(parent);
                        }
                    }
                    last_location = Some(node);
                    current = next.parent.clone();
                    continue 'outer;
                }
                SyntaxKind::Parameter => {

                    self.track_parameter_initializer(
                        &node,
                        last_location.as_ref(),
                        &mut associated_declaration_for_containing_initializer_or_binding_name,
                    );
                }
                SyntaxKind::BindingElement => {

                    self.track_binding_element_initializer(
                        &node,
                        last_location.as_ref(),
                        &mut associated_declaration_for_containing_initializer_or_binding_name,
                    );
                }
                SyntaxKind::InferType => {
                    if meaning.intersects(SymbolFlags::TypeParameter) {

                        if self.resolve_infer_type_case(&node, name) {
                            result = node_symbol(&node);
                            break 'outer;
                        }
                    }
                }
                SyntaxKind::ExportSpecifier => {

                    if let Some(new_loc) =
                        self.resolve_export_specifier_case(&node, last_location.as_ref())
                    {
                        last_location = Some(node);
                        current = Some(new_loc);
                        continue 'outer;
                    }
                }
                _ => {}
            }
            if is_self_reference_location(&node, last_location.as_ref()) {
                last_self_reference_location = Some(Arc::clone(&node));
            }
            last_location = Some(node);

            current = last_location.as_ref().and_then(|n| n.parent.clone());
        }

        if is_use {
            if let Some(result_sym) = &result {
                let is_self_ref = match &last_self_reference_location {
                    None => true,
                    Some(self_loc) => !node_symbol(self_loc)
                        .map(|s| Arc::ptr_eq(&s, result_sym))
                        .unwrap_or(false),
                };
                if is_self_ref {
                    if let Some(callback) = &self.symbol_referenced_fn {
                        callback(result_sym, meaning);
                    }
                }
            }
        }
        if result.is_none() && !exclude_globals {
            result = self.lookup(&self.globals, name, meaning | SymbolFlags::GlobalLookup);
        }
        if result.is_none() {
            if is_in_js_file(&original_location) {
                if let Some(orig_parent) = &original_location.parent {

                    if is_require_call(orig_parent, false) {
                        return self.require_symbol.clone();
                    }
                }
            }
        }
        if let Some(message) = name_not_found_message {
            if let Some(property) = &property_with_invalid_initializer {
                if let Some(callback) = &self.on_property_with_invalid_initializer_fn {
                    if callback(&original_location, name, property, result.as_ref()) {
                        return None;
                    }
                }
            }
            match &result {
                None => {
                    if let Some(callback) = &self.on_failed_to_resolve_symbol_fn {
                        callback(&original_location, name, meaning, message);
                    }
                }
                Some(sym) => {
                    if let Some(callback) = &self.on_successfully_resolved_symbol_fn {
                        callback(
                            &original_location,
                            sym,
                            meaning,
                            last_location.as_ref(),
                            associated_declaration_for_containing_initializer_or_binding_name
                                .as_ref(),
                            within_deferred_context,
                        );
                    }
                }
            }
        }
        result
    }

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

    fn resolve_module_exports_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        _result: &mut Option<Arc<Symbol>>,
    ) {

    }

    fn resolve_enum_case(
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

    fn resolve_property_declaration_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        _property_with_invalid_initializer: &mut Option<Arc<Node>>,
    ) {

    }

    fn resolve_class_or_interface_case(
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

    fn resolve_expression_with_type_arguments_case(
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

    fn resolve_computed_property_name_case(
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

    fn resolve_function_expression_name_case(&self, _location: &Arc<Node>, _name: &str) -> bool {

        false
    }

    fn track_parameter_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {

    }

    fn track_binding_element_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {

    }

    fn resolve_infer_type_case(&self, _location: &Arc<Node>, _name: &str) -> bool {

        false
    }

    fn resolve_export_specifier_case(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {

        None
    }
}

pub fn get_local_symbol_for_export_default(symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
    if !is_export_default_symbol(symbol) || symbol.declarations.is_empty() {
        return None;
    }
    for decl in &symbol.declarations {

        if let Some(local) = node_local_symbol(decl) {
            return Some(local);
        }
    }
    None
}

pub fn is_export_default_symbol(symbol: &Arc<Symbol>) -> bool {
    !symbol.declarations.is_empty()
        && has_syntactic_modifier(&symbol.declarations[0], ModifierFlags::Default)
}

pub fn get_is_deferred_context(location: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
    if location.kind != SyntaxKind::ArrowFunction && location.kind != SyntaxKind::FunctionExpression
    {

        return is_type_query_node(location)
            || ((is_function_like_declaration(location)
                || (location.kind == SyntaxKind::PropertyDeclaration && !is_static(location)))
                && last_location
                    .map(|l| !ptr_eq_name(l, location.name()))
                    .unwrap_or(true));
    }
    if let Some(last) = last_location {
        if ptr_eq_name(last, location.name()) {
            return false;
        }
    }

    false
}

pub fn is_type_parameter_symbol_declared_in_container(
    symbol: &Arc<Symbol>,
    container: &Arc<Node>,
) -> bool {
    for decl in &symbol.declarations {
        if decl.kind == SyntaxKind::TypeParameter {
            if let Some(parent) = &decl.parent {
                if Arc::ptr_eq(parent, container) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn is_self_reference_location(node: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
    match node.kind {
        SyntaxKind::Parameter => last_location
            .map(|l| ptr_eq_name(l, node.name()))
            .unwrap_or(false),
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::ModuleDeclaration => true,
        _ => false,
    }
}

fn is_const_assertion(_node: &Arc<Node>) -> bool {
    false
}

fn is_global_source_file(_node: &Arc<Node>) -> bool {
    false
}

fn is_type_query_node(_node: &Arc<Node>) -> bool {
    false
}

fn is_require_call(_node: &Arc<Node>, _require_string_literal_like_argument: bool) -> bool {
    false
}

fn node_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

fn node_local_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

fn ptr_eq_name(node: &Arc<Node>, name: Option<&Arc<Node>>) -> bool {
    name.map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
}
