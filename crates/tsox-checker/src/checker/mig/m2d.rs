#![allow(invalid_reference_casting)]
#![allow(unused_imports)]

use crate::binder::referenceresolver_hooks::ReferenceResolverHooks;
use crate::binder::referenceresolver_reference_resolver::ReferenceResolver;
use crate::binder::referenceresolver_resolver_impl::{new_reference_resolver, ReferenceResolverImpl};
use crate::checker::grammarchecks_is_this_parameter_2::is_optional_declaration;
use crate::checker::mig::wc3_2::is_const_enum_symbol;
use crate::checker::utilities_is_private_within_ambient::contains_non_missing_undefined_type;
use crate::checker::checker_checker::*;
use crate::checker::types::{SymbolAccessibilityResult, Type};
use std::sync::Arc;
use tsox_frontend::ast::mig::m3e_4::get_first_identifier;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::ast::{
    get_source_file_of_node, has_syntactic_modifier, is_identifier,
    Node, NodeData, Symbol, SymbolFlags, SyntaxKind,
};

#[derive(Clone)]
pub struct EmitResolver {
    pub(crate) checker: *mut Checker,
    pub(crate) reference_resolver: Option<Arc<ReferenceResolverImpl>>,
}

pub fn new_emit_resolver(checker: &mut Checker) -> Arc<EmitResolver> { ::tsox_core::fntrace::enter("new_emit_resolver"); 
    Arc::new(EmitResolver {
        checker,
        reference_resolver: None,
    })
}

pub fn noop_add_visible_alias(_declaration: &Arc<Node>, _aliasing_statement: &Arc<Node>) { ::tsox_core::fntrace::enter("noop_add_visible_alias"); }

pub fn is_const_enum_or_const_enum_only_module(s: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_const_enum_or_const_enum_only_module"); 
    is_const_enum_symbol(s) || s.flags.contains(SymbolFlags::ConstEnumOnlyModule)
}

impl EmitResolver {
    pub fn declared_parameter_type_contains_undefined(
        &self,
        checker: &mut Checker,
        parameter: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("declared_parameter_type_contains_undefined"); 
        let Some(type_node) = parameter.type_() else {
            return false;
        };
        let t = checker.get_type_from_type_node(&type_node);
        checker.is_error_type(&t) || checker.contains_undefined_type(&t)
    }

    pub fn get_reference_resolver(
        &mut self,
        checker: &mut Checker,
    ) -> Arc<ReferenceResolverImpl> { ::tsox_core::fntrace::enter("get_reference_resolver"); 
        if self.reference_resolver.is_none() {
            let checker_ptr: *mut Checker = checker;
            let mut hooks = ReferenceResolverHooks::new();
            hooks.resolve_name_fn = Some(Box::new(
                move |location: &Arc<Node>,
                      name: &str,
                      meaning: SymbolFlags,
                      _not_found_message: Option<&tsox_core::diagnostics::Message>,
                      _is_use: bool,
                      exclude_globals: bool|
                      -> Option<Arc<Symbol>> {
                    unsafe {
                        (*checker_ptr).resolve_name(name, location, meaning, exclude_globals)
                    }
                },
            ));
            hooks.get_resolved_symbol_fn = Some(Box::new(move |node: &Arc<Node>| {
                unsafe { (*checker_ptr).get_resolved_symbol_or_nil(node) }
            }));
            hooks.get_merged_symbol_fn = Some(Box::new(move |symbol: &Arc<Symbol>| {
                unsafe { Some((*checker_ptr).get_merged_symbol(symbol)) }
            }));
            hooks.get_parent_of_symbol_fn = Some(Box::new(move |symbol: &Arc<Symbol>| {
                unsafe { (*checker_ptr).get_parent_of_symbol(symbol) }
            }));
            hooks.get_symbol_of_declaration_fn = Some(Box::new(move |node: &Arc<Node>| {
                unsafe { (*checker_ptr).get_symbol_of_declaration(node) }
            }));
            hooks.get_type_only_alias_declaration_fn =
                Some(Box::new(move |symbol: &Arc<Symbol>, meaning: SymbolFlags| {
                    unsafe { (*checker_ptr).get_type_only_alias_declaration_ex(symbol, meaning) }
                }));
            hooks.get_export_symbol_of_value_symbol_if_exported_fn =
                Some(Box::new(move |symbol: &Arc<Symbol>| {
                    unsafe {
                        Some((*checker_ptr).get_export_symbol_of_value_symbol_if_exported(symbol))
                    }
                }));
            hooks.get_element_access_expression_name_fn =
                Some(Box::new(move |node: &Arc<Node>| {
                    unsafe { (*checker_ptr).try_get_element_access_expression_name(node) }
                }));
            self.reference_resolver = Some(Arc::new(new_reference_resolver(
                Some(Arc::clone(&checker.compiler_options)),
                hooks,
            )));
        }
        Arc::clone(self.reference_resolver.as_ref().unwrap())
    }

    pub fn is_alias_resolved_to_value(
        &self,
        checker: &mut Checker,
        symbol: Option<&Arc<Symbol>>,
        exclude_type_only_values: bool,
    ) -> bool { ::tsox_core::fntrace::enter("is_alias_resolved_to_value"); 
        let Some(symbol) = symbol else {
            return false;
        };
        if let Some(value_declaration) = &symbol.value_declaration {
            let container = get_source_file_of_node(value_declaration);
            if let Some(container) = container {
                if let Some(file_symbol) = checker.get_symbol_of_declaration(&container) {
                    checker.resolve_external_module_symbol(&file_symbol, false);
                }
            }
        }
        let resolved_alias = checker.resolve_alias(symbol);
        let target = checker.get_export_symbol_of_value_symbol_if_exported(&resolved_alias);
        if checker
            .unknown_symbol
            .as_ref()
            .is_some_and(|unknown| Arc::ptr_eq(&target, unknown))
        {
            return !exclude_type_only_values
                || checker.get_type_only_alias_declaration(symbol).is_none();
        }
        checker
            .get_symbol_flags_ex(&target, exclude_type_only_values, true)
            .contains(SymbolFlags::VALUE)
            && (checker.compiler_options.should_preserve_const_enums()
                || !is_const_enum_or_const_enum_only_module(&target))
    }

    pub fn is_optional_uninitialized_parameter_property(
        &self,
        checker: &Checker,
        parameter: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_optional_uninitialized_parameter_property"); 
        checker.strict_null_checks
            && checker.is_optional_parameter(parameter)
            && parameter.initializer().is_none()
            && has_syntactic_modifier(
                parameter,
                tsox_frontend::ast::ModifierFlags::ParameterPropertyModifier,
            )
    }

    pub fn is_required_initialized_parameter(
        &self,
        checker: &Checker,
        parameter: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("is_required_initialized_parameter"); 
        if !checker.strict_null_checks
            || checker.is_optional_parameter(parameter)
            || parameter.initializer().is_none()
        {
            return false;
        }
        if has_syntactic_modifier(
            parameter,
            tsox_frontend::ast::ModifierFlags::ParameterPropertyModifier,
        ) {
            return enclosing_declaration
                .is_some_and(|d| tsox_frontend::ast::is_function_like_declaration(d));
        }
        true
    }

    pub fn is_value_alias_declaration_worker(
        &self,
        checker: &mut Checker,
        node: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_value_alias_declaration_worker"); 
        match node.kind {
            SyntaxKind::ImportEqualsDeclaration => self.is_alias_resolved_to_value(
                checker,
                checker.get_symbol_of_declaration(node).as_ref(),
                false,
            ),
            SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                let Some(symbol) = checker.get_symbol_of_declaration(node) else {
                    return false;
                };
                self.is_alias_resolved_to_value(checker, Some(&symbol), true)
            }
            SyntaxKind::ExportDeclaration => {
                let NodeData::ExportDeclaration(decl) = &node.data else {
                    return false;
                };
                let Some(export_clause) = &decl.export_clause else {
                    return false;
                };
                tsox_frontend::ast::is_namespace_export(export_clause)
                    || tsox_frontend::ast::mig::m3b::elements(export_clause)
                        .iter()
                        .any(|element| self.is_value_alias_declaration_worker(checker, element))
            }
            SyntaxKind::ExportAssignment => {
                if let Some(expression) = node.expression() {
                    if expression.kind == SyntaxKind::Identifier {
                        return self.is_alias_resolved_to_value(
                            checker,
                            checker.get_symbol_of_declaration(node).as_ref(),
                            true,
                        );
                    }
                }
                true
            }
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(binary) = &node.data else {
                    return false;
                };
                if Checker::is_common_js_module_exports(node) && is_identifier(&binary.right) {
                    return self.is_alias_resolved_to_value(
                        checker,
                        checker.get_symbol_of_declaration(node).as_ref(),
                        true,
                    );
                }
                false
            }
            _ => false,
        }
    }

    pub fn requires_adding_implicit_undefined(
        &self,
        checker: &mut Checker,
        declaration: &Arc<Node>,
        symbol: Option<&Arc<Symbol>>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("requires_adding_implicit_undefined"); 
        if !is_parse_tree_node(declaration) {
            return false;
        }
        match declaration.kind {
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::JSDocPropertyTag => {
                let symbol = match symbol {
                    Some(s) => Arc::clone(s),
                    None => match checker.get_symbol_of_declaration(declaration) {
                        Some(s) => s,
                        None => return false,
                    },
                };
                let t = checker.get_type_of_symbol(&symbol);
                symbol.flags.contains(SymbolFlags::Property)
                    && symbol.flags.contains(SymbolFlags::Optional)
                    && is_optional_declaration(declaration)
                    && checker.reverse_mapped_symbol_links.get(&symbol).is_some()
                    && checker
                        .reverse_mapped_symbol_links
                        .get(&symbol)
                        .and_then(|links| links.mapped_type.clone())
                        .is_some()
                    && contains_non_missing_undefined_type(&t)
            }
            SyntaxKind::Parameter | SyntaxKind::JSDocParameterTag => self
                .requires_adding_implicit_undefined_worker(
                    checker,
                    declaration,
                    enclosing_declaration,
                ),
            _ => panic!("Node cannot possibly require adding undefined"),
        }
    }

    pub fn requires_adding_implicit_undefined_worker(
        &self,
        checker: &mut Checker,
        parameter: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("requires_adding_implicit_undefined_worker"); 
        (self.is_required_initialized_parameter(checker, parameter, enclosing_declaration)
            || self.is_optional_uninitialized_parameter_property(checker, parameter))
            && !self.declared_parameter_type_contains_undefined(checker, parameter)
    }

    fn emit_reference_resolver(&self) -> Arc<ReferenceResolverImpl> { ::tsox_core::fntrace::enter("emit_reference_resolver"); 
        // Go emitresolver.go getReferenceResolver: 惰性初始化并把结果写回缓存字段。
        let this = unsafe { &mut *(self as *const EmitResolver as *mut EmitResolver) };
        let checker = unsafe { &mut *this.checker };
        this.get_reference_resolver(checker)
    }
}

impl ReferenceResolver for EmitResolver {
    fn get_referenced_export_container(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_export_container"); 
        if !is_parse_tree_node(node) {
            return None;
        }
        self.emit_reference_resolver()
            .get_referenced_export_container(node, prefix_locals)
    }

    fn get_referenced_import_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_import_declaration"); 
        if !is_parse_tree_node(node) {
            return None;
        }
        self.emit_reference_resolver().get_referenced_import_declaration(node)
    }

    fn get_referenced_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_value_declaration"); 
        if !is_parse_tree_node(node) {
            return None;
        }
        self.emit_reference_resolver().get_referenced_value_declaration(node)
    }

    fn get_referenced_value_declarations(&self, node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_value_declarations"); 
        if !is_parse_tree_node(node) {
            return Vec::new();
        }
        self.emit_reference_resolver().get_referenced_value_declarations(node)
    }

    fn get_element_access_expression_name(&self, expression: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_element_access_expression_name"); 
        if !is_parse_tree_node(expression) {
            return String::new();
        }
        self.emit_reference_resolver().get_element_access_expression_name(expression)
    }

    fn get_referenced_member_value_declaration(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_member_value_declaration"); 
        if !is_parse_tree_node(node) {
            return None;
        }
        self.emit_reference_resolver()
            .get_referenced_member_value_declaration(node)
    }
}
