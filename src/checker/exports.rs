#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Arc;

use crate::ast::utilities::get_combined_modifier_flags;
use crate::ast::{CheckFlags, ModifierFlags, Node, Symbol, SymbolFlags, SyntaxKind};
use crate::core::compiler_options::ResolutionMode;
use crate::diagnostics::Message;

use super::checker::Checker;
use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnionReduction {
    #[default]
    None,
    Literal,
    Subtype,
}

pub fn get_declaration_modifier_flags_from_symbol(s: &Symbol) -> ModifierFlags {
    get_declaration_modifier_flags_from_symbol_ex(s, false )
}

pub fn get_declaration_modifier_flags_from_symbol_ex(s: &Symbol, is_write: bool) -> ModifierFlags {

    let base_decl = s
        .value_declaration
        .as_ref()
        .or_else(|| s.declarations.first());
    if let Some(value_declaration) = base_decl {
        let mut declaration: Option<&Arc<Node>> = None;
        if is_write {
            declaration = s
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor);
        }
        if declaration.is_none() && s.flags.contains(SymbolFlags::GetAccessor) {
            declaration = s
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::GetAccessor);
        }
        let declaration = declaration
            .map(Arc::clone)
            .unwrap_or_else(|| Arc::clone(value_declaration));
        let flags = get_combined_modifier_flags(&declaration);

        if let Some(parent) = &s.parent {
            if !parent.flags.contains(SymbolFlags::Class) {
                return flags.difference(ModifierFlags::AccessibilityModifier);
            }
        }
        return flags;
    }
    if s.check_flags.contains(CheckFlags::SYNTHETIC) {
        let access_modifier = if s.check_flags.contains(CheckFlags::ContainsPrivate) {
            ModifierFlags::Private
        } else if s.check_flags.contains(CheckFlags::ContainsPublic) {
            ModifierFlags::Public
        } else {
            ModifierFlags::Protected
        };
        let static_modifier = if s.check_flags.contains(CheckFlags::ContainsStatic) {
            ModifierFlags::Static
        } else {
            ModifierFlags::empty()
        };
        return access_modifier.union(static_modifier);
    }
    if s.flags.contains(SymbolFlags::Prototype) {
        return ModifierFlags::Public.union(ModifierFlags::Static);
    }
    ModifierFlags::empty()
}

impl Checker {

    pub fn get_unknown_signature(&self) -> Option<Arc<Signature>> {
        self.unknown_signature.get().cloned()
    }

    pub fn get_name_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        self.value_symbol_links
            .get(symbol)
            .and_then(|links| links.name_type.clone())
    }

    pub fn get_global_symbol(
        &self,
        name: &str,
        meaning: SymbolFlags,
        diagnostic: Option<&Message>,
    ) -> Option<Arc<Symbol>> {

        self.globals.get(name).cloned()
    }

    pub fn get_global_symbol_by_name(
        &self,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    pub fn get_global_type_by_name(&self, name: &str) -> Option<Arc<Type>> {

        let _symbol = self.globals.get(name)?;
        None
    }

    pub fn get_symbol_by_name(&self, name: &str, meaning: SymbolFlags) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    pub fn get_merged_symbol_public(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {

        Some(Arc::clone(symbol))
    }

    pub fn try_find_ambient_module(&self, module_name: &str) -> Option<Arc<Symbol>> {

        None
    }

    pub fn get_immediate_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {

        None
    }

    pub fn get_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> {

        None
    }

    pub fn resolve_external_module_name(
        &self,
        module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {

        None
    }

    pub fn get_declared_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Type> {

        self.any_type()
    }

    pub fn get_resolution_mode_override(
        &mut self,
        attrs: &Arc<Node>,
        report_errors: bool,
    ) -> Option<ResolutionMode> {
        use crate::ast::SyntaxKind;
        let data = match &attrs.data {
            crate::ast::NodeData::ImportAttributes(d) => d,
            _ => return None,
        };
        let is_assertions = data.token == SyntaxKind::AssertKeyword;
        if data.attributes.len() != 1 {
            if report_errors {
                let msg = if is_assertions {
                    crate::diagnostics::messages_generated::
                        TYPE_IMPORT_ASSERTIONS_SHOULD_HAVE_EXACTLY_ONE_KEY_RESOLUTION_MODE_WITH_VALUE_IMPORT_OR_REQUIRE
                } else {
                    crate::diagnostics::messages_generated::
                        TYPE_IMPORT_ATTRIBUTES_SHOULD_HAVE_EXACTLY_ONE_KEY_RESOLUTION_MODE_WITH_VALUE_IMPORT_OR_REQUIRE
                };
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    attrs.loc,
                    msg,
                    Vec::new(),
                ));
            }
            return None;
        }
        let elem = &data.attributes.nodes[0];
        let (name, value) = match &elem.data {
            crate::ast::NodeData::ImportAttribute(d) => (d.name.clone(), d.value.clone()),
            _ => return None,
        };
        if !matches!(name.kind, SyntaxKind::StringLiteral) {
            return None;
        }
        if name.text() != "resolution-mode" {
            if report_errors {
                let msg = if is_assertions {
                    crate::diagnostics::messages_generated::
                        X_RESOLUTION_MODE_IS_THE_ONLY_VALID_KEY_FOR_TYPE_IMPORT_ASSERTIONS
                } else {
                    crate::diagnostics::messages_generated::
                        X_RESOLUTION_MODE_IS_THE_ONLY_VALID_KEY_FOR_TYPE_IMPORT_ATTRIBUTES
                };
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    msg,
                    Vec::new(),
                ));
            }
            return None;
        }
        if !matches!(value.kind, SyntaxKind::StringLiteral) {
            return None;
        }
        match value.text() {
            "import" => Some(ResolutionMode::ESNext),
            "require" => Some(ResolutionMode::CommonJS),
            _ => {
                if report_errors {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        value.loc,
                        crate::diagnostics::messages_generated::
                            X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
                        Vec::new(),
                    ));
                }
                None
            }
        }
    }

    pub fn type_predicate_to_string(&self, t: &TypePredicate) -> String {

        String::new()
    }

    pub fn get_expanded_parameters(
        &self,
        signature: &Arc<Signature>,
        skip_union_expanding: bool,
    ) -> Vec<Vec<Arc<Symbol>>> {

        Vec::new()
    }

    pub fn get_resolved_signature(&self, node: &Arc<Node>) -> Option<Arc<Signature>> {

        None
    }

    pub fn get_contextual_type_for_argument_at_index(
        &self,
        node: &Arc<Node>,
        arg_index: usize,
    ) -> Option<Arc<Type>> {

        None
    }

    pub fn get_index_signatures_at_location(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {

        Vec::new()
    }

    pub fn get_resolved_symbol(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {

        None
    }

    pub fn get_jsx_fragment_factory(&self, location: &Arc<Node>) -> String {

        String::new()
    }

    pub fn resolve_name(
        &self,
        name: &str,
        location: &Arc<Node>,
        meaning: SymbolFlags,
        exclude_globals: bool,
    ) -> Option<Arc<Symbol>> {

        None
    }

    pub fn get_symbol_flags(&self, symbol: &Arc<Symbol>) -> SymbolFlags {

        symbol.flags
    }

    pub fn get_base_types(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {

        Vec::new()
    }

    pub fn get_base_constructor_type_of_class(&self, t: &Arc<Type>) -> Option<Arc<Type>> {

        None
    }

    pub fn get_rest_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {

        None
    }

    pub fn is_context_sensitive(&self, node: &Arc<Node>) -> bool {

        false
    }

    pub fn fill_missing_type_arguments(
        &self,
        type_arguments: &[Arc<Type>],
        type_parameters: &[Arc<Type>],
        min_type_argument_count: usize,
        is_java_script_implicit_any: bool,
    ) -> Vec<Arc<Type>> {

        type_arguments.to_vec()
    }

    pub fn get_min_type_argument_count(&self, type_parameters: &[Arc<Type>]) -> usize {

        type_parameters.len()
    }

    pub fn get_union_type_ex(
        &self,
        types: Vec<Arc<Type>>,
        union_reduction: UnionReduction,
    ) -> Arc<Type> {

        self.build_union_from_types(types)
    }

    pub fn requires_adding_implicit_undefined(&self, node: &Arc<Node>) -> bool {

        false
    }

    pub fn remove_missing_or_undefined_type(&self, t: &Arc<Type>) -> Arc<Type> {

        Arc::clone(t)
    }

    pub fn compare_symbols(&self, s1: &Arc<Symbol>, s2: &Arc<Symbol>) -> i32 {

        0
    }

    pub fn get_default_keyword_type(&self) -> Option<Arc<Type>> {

        self.get_global_type_by_name("default")
    }

    pub fn get_promise_type(&self) -> Option<Arc<Type>> {
        self.global_promise_type.get().cloned()
    }

    pub fn get_promise_like_type(&self) -> Option<Arc<Type>> {

        self.get_global_type_by_name("PromiseLike")
    }

    pub fn create_type_checker_cache(&self) {

    }

    pub fn clear_possible_type_requests(&mut self) {

    }
}
