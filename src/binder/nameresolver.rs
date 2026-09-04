//! Name resolver, ported from `internal/binder/nameresolver.go`.
//!
//! The name resolver walks up the AST scope chain to resolve an
//! identifier to a symbol, applying TypeScript's lexical scope,
//! visibility, and module/namespace export rules. It is the core of
//! the checker's name lookup (`resolveName`/`resolveEntityName`).
//!
//! Mirrors `binder.NameResolver` in Go.

#![allow(dead_code)]

use crate::ast::*;
use crate::core::compiler_options::{CompilerOptions, ScriptTarget};
use crate::core::tristate::Tristate;
use crate::diagnostics::Message;
use std::sync::Arc;

/// The name resolver.
///
/// Mirrors `binder.NameResolver` in Go. Fields are optional callbacks
/// that allow the checker to override the default behavior (e.g. to
/// follow aliases, merge symbols, or report diagnostics). When a
/// callback is `None`, the resolver uses a conservative default.
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
    /// Create a new name resolver with no callbacks set.
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

    /// Resolve `name` starting from `location`, walking up the scope chain.
    ///
    /// Mirrors `NameResolver.Resolve` in Go. Returns the resolved
    /// symbol, if any.
    ///
    /// **TODO**: the full scope-walk depends on many node accessors
    /// (`Locals()`, `Body()`, `Parameters()`, `AsConditionalTypeNode()`,
    /// `AsFunctionExpression()`, `Symbol()`, …) that are not yet ported
    /// to the Rust AST side tables. The structure below mirrors the Go
    /// control flow; the per-kind handling is stubbed until those
    /// accessors are available.
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
        // Needed for did-you-mean error reporting, which gathers candidates
        // starting from the original location.
        let original_location = Arc::clone(location);
        let name_is_const = name == "const";

        let mut current = Some(Arc::clone(location));
        'outer: while let Some(node) = current {
            if name_is_const && is_const_assertion(&node) {
                // `const` in an `as const` has no symbol, but issues no
                // error because there is no *actual* lookup of the type
                // (it refers to the constant type of the expression instead).
                return None;
            }
            if is_module_or_enum_declaration(&node)
                && last_location.as_ref().map_or(false, |last| {
                    node.name().map(|n| Arc::ptr_eq(n, last)).unwrap_or(false)
                })
            {
                // If lastLocation is the name of a namespace or enum, skip
                // the parent since it will have its own locals that could
                // conflict.
                let parent = node.parent.clone();
                last_location = Some(node);
                current = parent;
                continue 'outer;
            }
            // Locals of a source file are not in scope (because they get
            // merged into the global symbol table).
            // TODO: locals lookup requires NodeSymbolMap access
            // (`location.Locals()` in Go).
            let locals: Option<&SymbolTable> = None;
            if let Some(locals) = locals {
                if !is_global_source_file(&node) {
                    result = self.lookup(locals, name, meaning);
                    if result.is_some() {
                        let mut use_result = true;
                        // TODO: function-like scope restrictions and
                        // conditional-type branch handling depend on node
                        // accessors (`Body()`, `Type()`,
                        // `AsConditionalTypeNode().TrueType`).
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
                    // TODO: `ast.IsExternalOrCommonJSModule(location.AsSourceFile())`
                    // requires the SourceFile downcast; module-export
                    // resolution requires `getSymbolOfDeclaration` and
                    // `moduleSymbol.Exports`.
                    self.resolve_module_exports_case(&node, name, meaning, &mut result);
                }
                SyntaxKind::ModuleDeclaration => {
                    // TODO: same module-export handling as SourceFile above.
                    self.resolve_module_exports_case(&node, name, meaning, &mut result);
                }
                SyntaxKind::EnumDeclaration => {
                    // TODO: enum member resolution requires
                    // `getSymbolOfDeclaration` and `enumSymbol.Exports`.
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
                        // TODO: find constructor declaration and its locals.
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
                    // TODO: class/interface member lookup requires
                    // `getSymbolOfDeclaration().Members` and type-parameter
                    // container checks.
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
                    // TODO: base-class expression type-parameter reference
                    // check requires `Expression()`, heritage-clause, and
                    // parent traversal.
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
                    // It is not legal to reference a class's own type
                    // parameters from a computed property name.
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
                        // TODO: function-expression name lookup requires
                        // `AsFunctionExpression().Name()` and `Symbol()`.
                        if self.resolve_function_expression_name_case(&node, name) {
                            result = node_symbol(&node);
                            break 'outer;
                        }
                    }
                }
                SyntaxKind::Decorator => {
                    // Decorators are resolved at the class declaration.
                    // Resolving at the parameter or member would result in
                    // looking up locals in the method.
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
                    // TODO: track associated declaration for initializer/
                    // binding-name deferred-context handling.
                    self.track_parameter_initializer(
                        &node,
                        last_location.as_ref(),
                        &mut associated_declaration_for_containing_initializer_or_binding_name,
                    );
                }
                SyntaxKind::BindingElement => {
                    // TODO: same tracking as Parameter for binding elements
                    // that are part of a parameter declaration.
                    self.track_binding_element_initializer(
                        &node,
                        last_location.as_ref(),
                        &mut associated_declaration_for_containing_initializer_or_binding_name,
                    );
                }
                SyntaxKind::InferType => {
                    if meaning.intersects(SymbolFlags::TypeParameter) {
                        // TODO: infer-type parameter name match requires
                        // `AsInferTypeNode().TypeParameter.AsTypeParameterDeclaration().Name()`.
                        if self.resolve_infer_type_case(&node, name) {
                            result = node_symbol(&node);
                            break 'outer;
                        }
                    }
                }
                SyntaxKind::ExportSpecifier => {
                    // TODO: export-specifier module-specifier climb requires
                    // `AsExportSpecifier().PropertyName` and parent traversal.
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
            // !!! In Strada, JSDocTemplateTag/JSDocParameterTag/JSDocReturnTag
            // locations skip to getEffectiveContainerForJSDocTemplateTag/
            // getHostSignatureFromJSDoc instead of location.parent. This is a
            // no-op currently because JSDoc nodes have no locals.
            current = last_location.as_ref().and_then(|n| n.parent.clone());
        }
        // We just climbed up parents looking for the name. If
        // `result === lastSelfReferenceLocation.symbol`, this is a
        // self-reference and shouldn't count when considering whether
        // `lastLocation` is used.
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
                    // TODO: `ast.IsRequireCall(originalLocation.Parent, …)`
                    // requires the require-call predicate.
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

    /// Whether an outer variable referenced from within a parameter
    /// initializer should use the outer-variable scope (rather than the
    /// hoisted parameter scope).
    ///
    /// Mirrors `NameResolver.useOuterVariableScopeInParameter` in Go.
    pub fn use_outer_variable_scope_in_parameter(
        &self,
        result: &Arc<Symbol>,
        location: &Arc<Node>,
        last_location: Option<&Arc<Node>>,
    ) -> bool {
        if let Some(last) = last_location {
            if is_parameter_declaration(last) {
                // TODO: `location.Body()` requires a node accessor.
                let body: Option<&Arc<Node>> = None;
                if let Some(body) = body {
                    if let Some(value_decl) = &result.value_declaration {
                        if value_decl.pos() >= body.pos() && value_decl.end() <= body.end() {
                            // Check for cases where we introduce temporaries
                            // that require moving the name/initializer of the
                            // parameter to the body.
                            let function_location = Arc::clone(location);
                            let mut declaration_requires_scope_change = Tristate::Unknown;
                            if let Some(get_cache) = &self.get_requires_scope_change_cache_fn {
                                declaration_requires_scope_change = get_cache(&function_location);
                            }
                            if declaration_requires_scope_change.is_unknown() {
                                // TODO: `core.Some(functionLocation.Parameters(),
                                // r.requiresScopeChange)` requires the
                                // parameters accessor and `Some`.
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

    /// Whether a parameter declaration requires a scope change (its
    /// initializer or binding name references constructs that must be
    /// moved into the function body during emit).
    ///
    /// Mirrors `NameResolver.requiresScopeChange` in Go.
    pub fn requires_scope_change(&self, _node: &Arc<Node>) -> bool {
        // TODO: `node.AsParameterDeclaration()` accessor + `.Name()` / `.Initializer`.
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

    /// Worker for [`Self::requires_scope_change`].
    ///
    /// Mirrors `NameResolver.requiresScopeChangeWorker` in Go.
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
                // TODO: `node.Name()` accessor.
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
                // TODO: `node.AsPropertyDeclaration().Name()` accessor.
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
                    // TODO: `node.AsBindingElement().DotDotDotToken` and
                    // `node.Parent` object-binding-pattern check.
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
                // TODO: `node.ForEachChild(r.requiresScopeChangeWorker)`
                // requires a child visitor that returns bool.
                false
            }
        }
    }

    /// Report a diagnostic via the `Error` callback (if any).
    ///
    /// Mirrors `NameResolver.error` in Go. The default implementation
    /// does not report errors.
    pub fn error(
        &self,
        location: &Arc<Node>,
        message: &Message,
        args: &[String],
    ) -> Option<Diagnostic> {
        if let Some(callback) = &self.error_fn {
            return callback(location, message, args);
        }
        // Default implementation does not report errors.
        None
    }

    /// Get the symbol of a declaration node via the override callback
    /// (if any), falling back to the node's own symbol.
    ///
    /// Mirrors `NameResolver.getSymbolOfDeclaration` in Go. The default
    /// implementation does not support merged symbols.
    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        if let Some(callback) = &self.get_symbol_of_declaration_fn {
            return callback(node);
        }
        // Default: use the node's own symbol.
        node_symbol(node)
    }

    /// Look up `name` in `symbols` restricted to `meaning` via the
    /// override callback (if any), falling back to a direct table lookup.
    ///
    /// Mirrors `NameResolver.lookup` in Go. The default implementation
    /// does not support following aliases or merged symbols.
    pub fn lookup(
        &self,
        symbols: &SymbolTable,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        if let Some(callback) = &self.lookup_fn {
            return callback(symbols, name, meaning);
        }
        // Default implementation does not support following aliases or
        // merged symbols.
        if !meaning.is_empty() {
            if let Some(symbol) = symbols.get(name) {
                if symbol.flags.intersects(meaning) {
                    return Some(Arc::clone(symbol));
                }
            }
        }
        None
    }

    /// Get (lazily synthesizing) the transient `arguments` symbol.
    ///
    /// Mirrors `NameResolver.argumentsSymbol` in Go.
    pub fn arguments_symbol(&mut self) -> Arc<Symbol> {
        if self.arguments_symbol.is_none() {
            // Default implementation synthesizes a transient symbol for
            // `arguments`.
            self.arguments_symbol = Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "arguments",
            )));
        }
        Arc::clone(self.arguments_symbol.as_ref().unwrap())
    }

    // ───────────────────────────────────────────────────────────────
    // Private helpers that encapsulate the per-kind TODO stubs of
    // `resolve`. Each mirrors the corresponding arm in Go's switch.
    // ───────────────────────────────────────────────────────────────

    fn resolve_module_exports_case(
        &self,
        _location: &Arc<Node>,
        _name: &str,
        _meaning: SymbolFlags,
        _result: &mut Option<Arc<Symbol>>,
    ) {
        // TODO: requires `getSymbolOfDeclaration`, `moduleSymbol.Exports`,
        // `InternalSymbolNameDefault`, `GetLocalSymbolForExportDefault`,
        // `GetDeclarationOfKind`, and the CommonJS module indicator check.
    }

    fn resolve_enum_case(
        &self,
        original_location: &Arc<Node>,
        _location: &Arc<Node>,
        name: &str,
        name_not_found_message: Option<&Message>,
        _result: &mut Option<Arc<Symbol>>,
    ) {
        // TODO: enum member resolution via `getSymbolOfDeclaration` and
        // `enumSymbol.Exports`. The isolated-modules diagnostic below is
        // reproduced for fidelity.
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
                            // enumSymbol.Name + "." + name
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
        // TODO: `ast.FindConstructorDeclaration(location.Parent)` and
        // `ctor.Locals()` lookup.
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
        // TODO: class/interface member lookup via
        // `getSymbolOfDeclaration(location).Members`; type-parameter
        // container and static-member checks. The static-members
        // diagnostic is gated on the (TODO) member lookup.
        if name_not_found_message.is_some() {
            self.error(
                original_location,
                &crate::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                &[],
            );
        }
    }

    /// Returns `true` when the helper determined the lookup is an error
    /// and `resolve` should `return nil`.
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
        // TODO: requires heritage-clause and container member lookups.
        if name_not_found_message.is_some() {
            self.error(
                original_location,
                &crate::diagnostics::messages_generated::BASE_CLASS_EXPRESSIONS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                &[],
            );
        }
        false
    }

    /// Returns `true` when the helper found a type-parameter reference
    /// (an error) and `resolve` should `return nil`.
    fn resolve_computed_property_name_case(
        &self,
        original_location: &Arc<Node>,
        _grandparent: &Option<Arc<Node>>,
        _name: &str,
        _meaning: SymbolFlags,
        name_not_found_message: Option<&Message>,
        _result: &mut Option<Arc<Symbol>>,
    ) -> bool {
        // TODO: requires `IsClassLike(grandparent)` /
        // `IsInterfaceDeclaration` and grandparent member lookups.
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
        // TODO: `location.AsFunctionExpression().Name()` text match.
        false
    }

    fn track_parameter_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {
        // TODO: `AsParameterDeclaration().Initializer` / `.Name()` checks.
    }

    fn track_binding_element_initializer(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
        _associated: &mut Option<Arc<Node>>,
    ) {
        // TODO: `AsBindingElement().Initializer` / `.Name()` and
        // `IsPartOfParameterDeclaration` checks.
    }

    fn resolve_infer_type_case(&self, _location: &Arc<Node>, _name: &str) -> bool {
        // TODO: `AsInferTypeNode().TypeParameter.AsTypeParameterDeclaration().Name()`.
        false
    }

    fn resolve_export_specifier_case(
        &self,
        _location: &Arc<Node>,
        _last_location: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {
        // TODO: `AsExportSpecifier().PropertyName` and parent module-
        // specifier climb. Returns the new location to continue from.
        None
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Free functions
// ────────────────────────────────────────────────────────────────────────────

/// Get the local symbol for an export-default symbol, if any.
///
/// Mirrors `binder.GetLocalSymbolForExportDefault` in Go.
pub fn get_local_symbol_for_export_default(symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
    if !is_export_default_symbol(symbol) || symbol.declarations.is_empty() {
        return None;
    }
    for decl in &symbol.declarations {
        // TODO: `decl.LocalSymbol()` requires a node accessor for the
        // local-symbol side table entry.
        if let Some(local) = node_local_symbol(decl) {
            return Some(local);
        }
    }
    None
}

/// Whether `symbol` is an export-default declaration.
///
/// Mirrors `binder.isExportDefaultSymbol` in Go.
pub fn is_export_default_symbol(symbol: &Arc<Symbol>) -> bool {
    !symbol.declarations.is_empty()
        && has_syntactic_modifier(&symbol.declarations[0], ModifierFlags::Default)
}

/// Whether `location` is a deferred context (its body is not executed
/// synchronously at the point of declaration).
///
/// Mirrors `binder.getIsDeferredContext` in Go.
pub fn get_is_deferred_context(location: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
    if location.kind != SyntaxKind::ArrowFunction && location.kind != SyntaxKind::FunctionExpression
    {
        // Initializers in instance property declarations of class-like
        // entities are executed in the constructor and thus deferred.
        // A name is evaluated within the enclosing scope — so it
        // shouldn't count as deferred.
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
    // Generator functions and async functions are not inlined in control
    // flow when immediately invoked.
    // TODO: `location.BodyData().AsteriskToken` and
    // `ast.GetImmediatelyInvokedFunctionExpression(location)` accessors.
    false
}

/// Whether a type-parameter symbol was declared directly in `container`.
///
/// Mirrors `binder.isTypeParameterSymbolDeclaredInContainer` in Go.
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

/// Whether `node` at `last_location` is a self-reference location (the
/// declaration of the name being resolved).
///
/// Mirrors `binder.isSelfReferenceLocation` in Go.
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

// ────────────────────────────────────────────────────────────────────────────
// Stubs for AST helpers that are not yet ported
// ────────────────────────────────────────────────────────────────────────────

/// TODO: port `ast.IsConstAssertion` — whether `node` is an `as const`
/// assertion expression.
fn is_const_assertion(_node: &Arc<Node>) -> bool {
    false
}

/// TODO: port `ast.IsGlobalSourceFile` — whether `node` is a
/// non-module source file (script).
fn is_global_source_file(_node: &Arc<Node>) -> bool {
    false
}

/// TODO: port `ast.IsTypeQueryNode` — whether `node` is a `typeof`
/// type query.
fn is_type_query_node(_node: &Arc<Node>) -> bool {
    false
}

/// TODO: port `ast.IsRequireCall` — whether `node` is a CommonJS
/// `require(...)` call.
fn is_require_call(_node: &Arc<Node>, _require_string_literal_like_argument: bool) -> bool {
    false
}

/// TODO: port `node.Symbol()` access — look up the symbol side-table
/// entry for `node`. Currently no side table is threaded through.
fn node_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

/// TODO: port `node.LocalSymbol()` access.
fn node_local_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

/// Pointer-equality helper comparing a node against a declaration's
/// `Name()` child (mirrors Go's `lastLocation == location.Name()`).
fn ptr_eq_name(node: &Arc<Node>, name: Option<&Arc<Node>>) -> bool {
    name.map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
}
