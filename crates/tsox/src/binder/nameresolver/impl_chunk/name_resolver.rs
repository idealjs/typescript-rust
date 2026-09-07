#![allow(unused_imports)]

use super::*;

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
}
