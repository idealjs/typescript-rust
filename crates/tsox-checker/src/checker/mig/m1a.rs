#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use super::r27k_defs::set_literal_fresh_type;
use std::collections::HashSet;
use std::sync::Arc;
use tsox_frontend::ast::{
    is_ambient_module_symbol_name, is_identifier, is_jsx_tag_name, is_private_identifier, Diagnostic,
    Node, SourceFile, Symbol, SyntaxKind,
};
use tsox_frontend::ast::mig::m3b::is_write_access;
use tsox_frontend::ast::mig::m3e::get_function_flags;
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use tsox_frontend::ast::mig::m3e_4::{for_each_return_statement, get_declaration_of_kind, get_jsdoc_deprecated_tag};
use tsox_frontend::ast::mig::m3f::get_reparsed_node_for_node;
use tsox_frontend::ast::mig::m3f_3::is_assignment_target;
use tsox_frontend::ast::mig::m3f_4::{is_declaration_name, is_deprecated_declaration_with_cached_flags};
use tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access;
use tsox_frontend::scanner::mig::m3i::{declaration_name_to_string, get_text_of_node};
use crate::checker::nodebuilder_checker_13::MAX_SERIALIZATION_LEVEL;
use crate::checker::utilities_has_only_expression_initialization::{compare_types, is_optional_declaration};
use crate::checker::utilities_is_optional_symbol::{get_selected_modifier_flags, is_type_alias};
use crate::checker::utilities_get_assignment_target::{is_in_type_query, is_type_reference_identifier};
use crate::checker::mig::m1c::create_file_index_map;
use crate::checker::mig::m1c_3::create_diagnostic_for_node;
use crate::checker::mig::m2a::{is_primitive_type_name, is_rest_parameter};
use crate::checker::mig::m2b_2::TupleNormalizer;
use crate::checker::mig::m2c_3::signature_has_rest_parameter;
use crate::checker::mig::m2d::{new_emit_resolver, EmitResolver};
use crate::checker::mig::m3a_2::new_diagnostic_for_node;
#[path = "r19k2_defs.rs"]
pub mod r19k2_defs;
#[path = "r28k2_defs.rs"]
pub mod r28k2_defs;
pub use r19k2_defs::*;
pub use r28k2_defs::*;

#[path = "r20k9_defs.rs"]
pub mod r20k9_defs;
pub use r20k9_defs::*;

#[path = "r24k18_defs.rs"]
pub mod r24k18_defs;
use r24k18_defs::for_each_yield_expression;
use crate::checker::mig::wc3::CallState;
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_frontend::ast::node_data_generated::{TypeParameterDeclarationData, YieldExpressionData};

impl Checker {
    pub fn get_ambient_modules(&mut self) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_ambient_modules"); 
        let mut modules: Vec<Arc<Symbol>> = Vec::new();
        let mut seen: HashSet<*const Symbol> = HashSet::new();
        for (name, global) in self.globals.entries.iter() {
            if is_ambient_module_symbol_name(name) {
                modules.push(Arc::clone(global));
                seen.insert(Arc::as_ptr(global));
            }
        }
        modules
    }

    pub fn get_emit_resolver(&mut self) -> Arc<EmitResolver> { ::tsox_core::fntrace::enter("get_emit_resolver"); 
        new_emit_resolver(self)
    }

    pub fn get_global_diagnostics(&mut self) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_global_diagnostics"); 
        if self.was_canceled() {
            panic!("Checker was previously cancelled");
        }
        self.diagnostics.get_global_diagnostics()
    }

    pub fn get_type_at_location(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_at_location");
        // Go 基线/LS 的名字可达性按查询位置判定（tsbaseline writeTypeOrSymbol 的
        // TypeToTypeNode(t, node.Parent, …) → useFullyQualifiedType →
        // IsSymbolAccessible(symbol, enclosingDeclaration)），查询入口记录位置，
        // 供 namespace_qualifier_of 判「符号在位置所属容器内可不加限定」
        self.access_location = Some(Arc::clone(node));
        let reparsed = tsox_frontend::ast::mig::m3f::get_reparsed_node_for_node(node);
        self.get_type_of_node(&reparsed)
    }

    pub fn get_type_of_symbol_at_location(
        &mut self,
        symbol: &Arc<Symbol>,
        location: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_symbol_at_location"); 
        let symbol = self.get_export_symbol_of_value_symbol_if_exported(symbol);
        let mut location = Arc::clone(location);
        let parent = location.parent();
        let parent_is_jsx_attribute = parent
            .as_ref()
            .is_some_and(|p| tsox_frontend::ast::is_jsx_attribute(p));
        let parent_is_jsx_namespaced_name = parent
            .as_ref()
            .is_some_and(|p| tsox_frontend::ast::is_jsx_namespaced_name(p));
        if (is_identifier(&location) || is_private_identifier(&location))
            && !(is_jsx_tag_name(&location) || parent_is_jsx_attribute || parent_is_jsx_namespaced_name)
        {
            if tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access(&location) {
                if let Some(p) = location.parent() {
                    location = p;
                }
            }
            if crate::checker::mig::m1f::r24k9_defs::is_expression_node(&location)
                && (!tsox_frontend::ast::mig::m3f_3::is_assignment_target(&location)
                    || is_write_access(&location))
            {
                let t = if is_write_access(&location)
                    && location.kind == SyntaxKind::PropertyAccessExpression
                {
                    self.check_property_access_expression(&location, CheckMode::Normal, true)
                } else {
                    self.get_type_of_expression(&location)
                };
                let resolved = self
                    .symbol_node_links
                    .get(&location)
                    .and_then(|l| l.resolved_symbol.clone());
                if let Some(resolved) = resolved {
                    if Arc::ptr_eq(
                        &self.get_export_symbol_of_value_symbol_if_exported(&resolved),
                        &symbol,
                    ) {
                        return self.remove_optional_type_marker(&t);
                    }
                }
            }
        }
        if let Some(parent) = location.parent() {
            if tsox_frontend::ast::mig::m3f_4::is_declaration_name(&location)
                && tsox_frontend::ast::is_set_accessor_declaration(&parent)
                && self.get_annotated_accessor_type_node(&parent).is_some()
            {
                if let Some(accessor_symbol) = self.symbol_of_node(&parent) {
                    return self.get_write_type_of_accessors(&accessor_symbol);
                }
            }
        }
        if is_right_side_of_access_expression(&location)
            && location
                .parent()
                .is_some_and(|p| is_write_access(&p))
        {
            return self.get_write_type_of_symbol(&symbol);
        }
        self.get_non_missing_type_of_symbol(&symbol)
    }

    pub fn is_deprecated_declaration(&mut self, declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_deprecated_declaration"); 
        let flags = self.get_combined_node_flags_cached(declaration);
        tsox_frontend::ast::mig::m3f_4::is_deprecated_declaration_with_cached_flags(declaration, flags)
    }

    pub fn union_types(&self) -> impl Iterator<Item = Arc<Type>> + '_ { ::tsox_core::fntrace::enter("union_types"); 
        self.union_types.values().cloned()
    }

    pub fn add_declaration_to_late_bound_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        member: &Arc<Node>,
        symbol_flags: SymbolFlags,
    ) { ::tsox_core::fntrace::enter("add_declaration_to_late_bound_symbol"); 
        debug_assert!(
            symbol.check_flags.contains(CheckFlags::Late),
            "Expected a late-bound symbol."
        );
        if let Some(member_symbol) = self.symbol_of_node(member) {
            self.late_bound_links
                .get_or_default(&member_symbol)
                .late_symbol = Some(Arc::clone(symbol));
        }
        let member_symbol_flags = self.symbol_of_node(member).map(|s| s.flags).unwrap_or_default();
        if symbol.declarations.is_empty()
            || !member_symbol_flags.contains(SymbolFlags::ReplaceableByMethod)
        {
            let mut new_flags = symbol.flags | symbol_flags;
            let mut declarations = symbol.declarations.clone();
            declarations.push(Arc::clone(member));
            symbol.update_flags_and_declarations(new_flags, declarations);
        } else if symbol.flags.contains(SymbolFlags::ReplaceableByMethod)
            && member_symbol_flags.contains(SymbolFlags::Method)
        {
            let mut declarations: Vec<Arc<Node>> = symbol
                .declarations
                .iter()
                .filter(|d| {
                    self.symbol_of_node(d)
                        .is_some_and(|s| !s.flags.contains(SymbolFlags::ReplaceableByMethod))
                })
                .cloned()
                .collect();
            declarations.push(Arc::clone(member));
            let mut new_flags = SymbolFlags::empty();
            for d in &declarations {
                if let Some(s) = self.symbol_of_node(d) {
                    new_flags |= s.flags;
                }
            }
            if symbol.flags.contains(SymbolFlags::ACCESSOR) {
                new_flags |= SymbolFlags::ACCESSOR;
            }
            symbol.update_flags_and_declarations(new_flags, declarations);
        }
        if symbol_flags.contains(SymbolFlags::VALUE) {
            let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).value_declaration = Some(Arc::clone(member));
            }
        }
    }

    pub fn add_deferred_diagnostic(&mut self, callback: Box<dyn FnOnce() + Send>) { ::tsox_core::fntrace::enter("add_deferred_diagnostic"); 
        self.deferred_diagnostic_callbacks.push(callback);
    }

    pub fn add_deprecated_suggestion(
        &mut self,
        location: &Arc<Node>,
        declarations: &[Arc<Node>],
        deprecated_entity: &str,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("add_deprecated_suggestion"); 
        let diagnostic = new_diagnostic_for_node(
            Some(location),
            tsox_core::diagnostics::messages_generated::X_0_IS_DEPRECATED.clone(),
            vec![deprecated_entity.to_string()],
        );
        self.add_deprecated_suggestion_worker(declarations, diagnostic)
    }

    pub fn add_deprecated_suggestion_with_signature(
        &mut self,
        location: &Arc<Node>,
        declaration: &Arc<Node>,
        deprecated_entity: &str,
        signature_string: &str,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("add_deprecated_suggestion_with_signature"); 
        let message = if !deprecated_entity.is_empty() {
            &tsox_core::diagnostics::messages_generated::THE_SIGNATURE_0_OF_1_IS_DEPRECATED
        } else {
            &tsox_core::diagnostics::messages_generated::X_0_IS_DEPRECATED
        };
        let diagnostic = new_diagnostic_for_node(
            Some(location),
            message.clone(),
            vec![signature_string.to_string(), deprecated_entity.to_string()],
        );
        self.add_deprecated_suggestion_worker(&[Arc::clone(declaration)], diagnostic)
    }

    pub fn add_deprecated_suggestion_worker(
        &mut self,
        declarations: &[Arc<Node>],
        mut diagnostic: Diagnostic,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("add_deprecated_suggestion_worker"); 
        for declaration in declarations {
            if let Some(deprecated_tag) = tsox_frontend::ast::mig::m3e_4::get_jsdoc_deprecated_tag(declaration) {
                diagnostic.add_related_info(new_diagnostic_for_node(
                    Some(&deprecated_tag),
                    tsox_core::diagnostics::messages_generated::THE_DECLARATION_WAS_MARKED_AS_DEPRECATED_HERE.clone(),
                    vec![],
                ));
                break;
            }
        }
        self.add_suggestion_diagnostic(diagnostic)
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> Diagnostic { ::tsox_core::fntrace::enter("add_diagnostic"); 
        if self.serialization_level < MAX_SERIALIZATION_LEVEL {
            self.diagnostics.add(diagnostic.clone());
            return diagnostic;
        }
        diagnostic
    }

    pub fn add_duplicate_declaration_error(
        &mut self,
        node: &Arc<Node>,
        message: &tsox_core::diagnostics::Message,
        symbol_name: &str,
        related_nodes: &[Arc<Node>],
    ) { ::tsox_core::fntrace::enter("add_duplicate_declaration_error"); 
        let error_node = get_adjusted_node_for_error(node).unwrap_or_else(|| Arc::clone(node));
        let Some(mut err) = self.lookup_or_issue_error(
            &error_node,
            message,
            &[Box::new(symbol_name.to_string()) as Box<dyn std::fmt::Display>],
        ) else {
            return;
        };
        for related_node in related_nodes {
            let adjusted_node = get_adjusted_node_for_error(related_node)
                .unwrap_or_else(|| Arc::clone(related_node));
            if Arc::ptr_eq(&adjusted_node, &error_node) {
                continue;
            }
            let leading_message = new_diagnostic_for_node(
                Some(&adjusted_node),
                tsox_core::diagnostics::messages_generated::X_0_WAS_ALSO_DECLARED_HERE.clone(),
                vec![symbol_name.to_string()],
            );
            let follow_on_message = new_diagnostic_for_node(
                Some(&adjusted_node),
                tsox_core::diagnostics::messages_generated::X_AND_HERE.clone(),
                vec![],
            );
            let related = err.related_information();
            if related.len() >= 5
                || related.iter().any(|d| {
                    compare_diagnostics(d, &follow_on_message) == 0
                        || compare_diagnostics(d, &leading_message) == 0
                })
            {
                continue;
            }
            if related.is_empty() {
                err.add_related_info(leading_message);
            } else {
                err.add_related_info(follow_on_message);
            }
        }
    }

    pub fn add_duplicate_declaration_errors_for_symbols(
        &mut self,
        target: &Arc<Symbol>,
        message: &tsox_core::diagnostics::Message,
        symbol_name: &str,
        source: &Arc<Symbol>,
    ) { ::tsox_core::fntrace::enter("add_duplicate_declaration_errors_for_symbols"); 
        for node in &target.declarations {
            self.add_duplicate_declaration_error(node, message, symbol_name, &source.declarations);
        }
    }
}

pub fn accepts_void(t: &Type) -> bool { ::tsox_core::fntrace::enter("accepts_void"); 
    t.flags.contains(TypeFlags::Void)
}

pub fn is_right_side_of_access_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_right_side_of_access_expression"); 
    if let Some(parent) = node.parent() {
        match parent.kind {
            SyntaxKind::PropertyAccessExpression => {
                if let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) = &parent.data {
                    return Arc::ptr_eq(&d.name, node);
                }
            }
            SyntaxKind::QualifiedName => {
                if let tsox_frontend::ast::NodeData::QualifiedName(d) = &parent.data {
                    return Arc::ptr_eq(&d.right, node);
                }
            }
            _ => {}
        }
    }
    false
}

pub fn new_checker(program: Arc<dyn Program>, tracer: Arc<Tracer>) -> (Box<Checker>, std::sync::Mutex<()>) { ::tsox_core::fntrace::enter("new_checker"); 
    program.bind_source_files();

    let c = Checker::new(Arc::clone(&program), Arc::clone(&tracer));
    let mu = std::sync::Mutex::new(());
    (Box::new(c), mu)
}

impl Checker {
    pub fn add_error_or_suggestion(&mut self, is_error: bool, diagnostic: Diagnostic) { ::tsox_core::fntrace::enter("add_error_or_suggestion"); 
        if is_error {
            self.add_diagnostic(diagnostic);
        } else {
            let mut suggestion = diagnostic.clone();
            suggestion.set_category(tsox_core::diagnostics::Category::Suggestion);
            self.add_suggestion_diagnostic(suggestion);
        }
    }

    pub fn add_implementation_success_elaboration(
        &mut self,
        s: &CallState,
        failed: &Signature,
        diagnostic: &mut Diagnostic,
    ) { ::tsox_core::fntrace::enter("add_implementation_success_elaboration"); 
        if let Some(declaration) = &failed.declaration {
            if let Some(decl_symbol) = self.symbol_of_node(declaration) {
                let declarations = &decl_symbol.declarations;
                if declarations.len() > 1 {
                    let implementation = declarations
                        .iter()
                        .find(|d| {
                            tsox_frontend::ast::is_function_like_declaration(*d)
                                && d.body()
                                    .is_some_and(|b| tsox_frontend::ast::node_is_present(Some(&b)))
                        })
                        .cloned();
                    if let Some(implementation) = implementation {
                        let Some(candidate) =
                            self.get_signature_from_declaration(&implementation)
                        else {
                            return;
                        };
                        let mut local_state = s.clone();
                        local_state.candidates = vec![Arc::clone(&candidate)];
                        local_state.is_single_non_generic_candidate =
                            candidate.type_parameters.is_empty();
                        let chosen =
                            self.choose_overload(&mut local_state, &self.assignable_relation());
                        if chosen.is_some() {
                            diagnostic.add_related_info(new_diagnostic_for_node(
                                Some(&implementation),
                                tsox_core::diagnostics::messages_generated::THE_CALL_WOULD_HAVE_SUCCEEDED_AGAINST_THIS_IMPLEMENTATION_BUT_IMPLEMENTATION_SIGNATURES_OF_OVERLOADS_ARE_NOT_EXTERNALLY_VISIBLE.clone(),
                                vec![],
                            ));
                        }
                    }
                }
            }
        }
    }

    pub fn add_inherited_members(
        &mut self,
        mut symbols: SymbolTable,
        base_symbols: &[Arc<Symbol>],
    ) -> SymbolTable { ::tsox_core::fntrace::enter("add_inherited_members"); 
        for base in base_symbols {
            if !is_static_private_identifier_property(base) {
                let replace = match symbols.get(&base.name) {
                    Some(s) => !s.flags.contains(SymbolFlags::VALUE),
                    None => true,
                };
                if replace {
                    symbols.insert(base.name.clone(), Arc::clone(base));
                }
            }
        }
        symbols
    }

    pub fn add_named_unions(
        &mut self,
        mut named_unions: Vec<Arc<Type>>,
        types: &[Arc<Type>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("add_named_unions"); 
        for t in types {
            if t.flags.contains(TypeFlags::Union) {
                if let TypeData::Union(u) = &t.data {
                    let named = t.alias.is_some()
                        || u.origin
                            .as_ref()
                            .is_some_and(|o| !o.flags.contains(TypeFlags::Union));
                    if named {
                        if !named_unions
                            .iter()
                            .any(|n| Arc::ptr_eq(n, t))
                        {
                            named_unions.push(Arc::clone(t));
                        }
                    } else if let Some(origin) = &u.origin {
                        if origin.flags.contains(TypeFlags::Union) {
                            let origin_types = type_types_list(origin);
                            named_unions = self.add_named_unions(named_unions, &origin_types);
                        }
                    }
                }
            }
        }
        named_unions
    }

    pub fn add_optional_type_marker(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("add_optional_type_marker"); 
        if self.strict_null_checks {
            let optional_type = r28k2_defs::optional_type_of(self);
            return self.get_union_type(vec![Arc::clone(t), optional_type]);
        }
        Arc::clone(t)
    }

    pub fn add_optionality_ex(
        &mut self,
        t: &Arc<Type>,
        is_property: bool,
        is_optional: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("add_optionality_ex"); 
        if self.strict_null_checks && is_optional {
            return self.get_optional_type(Arc::clone(t));
        }
        Arc::clone(t)
    }

    pub fn add_suggestion_diagnostic(&mut self, diagnostic: Diagnostic) -> Diagnostic { ::tsox_core::fntrace::enter("add_suggestion_diagnostic"); 
        if self.serialization_level < MAX_SERIALIZATION_LEVEL {
            self.suggestion_diagnostics.add(diagnostic.clone());
            return diagnostic;
        }
        diagnostic
    }

    pub fn add_type_only_declaration_related_info(
        &mut self,
        mut diagnostic: Diagnostic,
        type_only_declaration: Option<&Arc<Node>>,
        name: &str,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("add_type_only_declaration_related_info"); 
        let Some(type_only_declaration) = type_only_declaration else {
            return diagnostic;
        };
        let is_export = tsox_frontend::ast::is_export_specifier(type_only_declaration)
            || tsox_frontend::ast::is_export_declaration(type_only_declaration)
            || tsox_frontend::ast::is_namespace_export(type_only_declaration);
        let message = if is_export {
            &tsox_core::diagnostics::messages_generated::X_0_WAS_EXPORTED_HERE
        } else {
            &tsox_core::diagnostics::messages_generated::X_0_WAS_IMPORTED_HERE
        };
        diagnostic.add_related_info(new_diagnostic_for_node(
            Some(type_only_declaration),
            message.clone(),
            vec![name.to_string()],
        ));
        diagnostic
    }

    pub fn add_type_to_intersection(
        &mut self,
        type_set: &mut OrderedSet<Arc<Type>>,
        mut includes: TypeFlags,
        t: &Arc<Type>,
    ) -> TypeFlags { ::tsox_core::fntrace::enter("add_type_to_intersection"); 
        let flags = t.flags;
        if flags.contains(TypeFlags::Intersection) {
            return self.add_types_to_intersection(type_set, includes, &type_types_list(t));
        }
        if self.is_empty_anonymous_object_type(t) {
            if !includes.contains(TYPE_FLAGS_INCLUDES_EMPTY_OBJECT) {
                includes |= TYPE_FLAGS_INCLUDES_EMPTY_OBJECT;
                type_set.add(Arc::clone(t));
            }
        } else if flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
            if Arc::ptr_eq(t, &self.wildcard_type()) {
                includes |= TYPE_FLAGS_INCLUDES_WILDCARD;
            }
            if self.is_error_type(t) {
                includes |= TYPE_FLAGS_INCLUDES_ERROR;
            }
        } else if self.strict_null_checks || !flags.intersects(TYPE_FLAGS_NULLABLE) {
            let mut t = Arc::clone(t);
            if Arc::ptr_eq(&t, &self.missing_type()) {
                includes |= TYPE_FLAGS_INCLUDES_MISSING_TYPE;
                t = self.undefined_type();
            }
            if !type_set.contains(&t) {
                if t.flags.contains(TYPE_FLAGS_UNIT) && includes.contains(TYPE_FLAGS_UNIT) {
                    includes |= TypeFlags::NonPrimitive;
                }
                type_set.add(t);
            }
        }
        includes | (flags & TYPE_FLAGS_INCLUDES_MASK)
    }

    pub fn add_types_to_intersection(
        &mut self,
        type_set: &mut OrderedSet<Arc<Type>>,
        includes: TypeFlags,
        types: &[Arc<Type>],
    ) -> TypeFlags { ::tsox_core::fntrace::enter("add_types_to_intersection"); 
        let mut includes = includes;
        for t in types {
            let regular = self.get_regular_type_of_literal_type(t);
            includes = self.add_type_to_intersection(type_set, includes, &regular);
        }
        includes
    }

    pub fn add_types_to_union(
        &mut self,
        source_types: &[Arc<Type>],
    ) -> (Vec<Arc<Type>>, TypeFlags) { ::tsox_core::fntrace::enter("add_types_to_union"); 
        let mut types: Vec<Arc<Type>> = Vec::with_capacity(source_types.len());
        let mut includes = TypeFlags::empty();
        let mut add_type = |types: &mut Vec<Arc<Type>>, includes: &mut TypeFlags, t: &Arc<Type>| {
            let flags = t.flags;
            if flags.contains(TypeFlags::Never) {
                return;
            }
            *includes |= flags & TYPE_FLAGS_INCLUDES_MASK;
            if flags.intersects(r19k2_defs::TYPE_FLAGS_INSTANTIABLE) {
                *includes |= TYPE_FLAGS_INCLUDES_INSTANTIABLE;
            }
            if flags.contains(TypeFlags::Intersection)
                && t
                    .object_flags
                    .contains(ObjectFlags::IsConstrainedTypeVariable)
            {
                *includes |= TYPE_FLAGS_INCLUDES_CONSTRAINED_TYPE_VARIABLE;
            }
            if Arc::ptr_eq(t, &self.wildcard_type()) {
                *includes |= TYPE_FLAGS_INCLUDES_WILDCARD;
            }
            if self.is_error_type(t) {
                *includes |= TYPE_FLAGS_INCLUDES_ERROR;
            }
            if !self.strict_null_checks && flags.intersects(TYPE_FLAGS_NULLABLE) {
                if !t
                    .object_flags
                    .contains(ObjectFlags::ContainsWideningType)
                {
                    *includes |= TYPE_FLAGS_INCLUDES_NON_WIDENING_TYPE;
                }
                return;
            }
            types.push(Arc::clone(t));
        };
        let mut last_type: Option<Arc<Type>> = None;
        for t in source_types {
            if last_type.as_ref().is_some_and(|l| Arc::ptr_eq(l, t)) {
                continue;
            }
            if t.flags.contains(TypeFlags::Union) {
                if let TypeData::Union(u) = &t.data {
                    if t.alias.is_some() || u.origin.is_some() {
                        includes |= TypeFlags::Union;
                    }
                    for s in &u.union_or_intersection.types {
                        add_type(&mut types, &mut includes, s);
                    }
                }
            } else {
                add_type(&mut types, &mut includes, t);
            }
            last_type = Some(Arc::clone(t));
        }
        if types.len() >= 2 {
            let mut sorted = types.clone();
            sorted.sort_by(|a, b| self.compare_types_ordered(a, b));
            types.clear();
            for t in sorted {
                if types
                    .last()
                    .is_none_or(|l| !Arc::ptr_eq(l, &t))
                {
                    types.push(t);
                }
            }
        }
        (types, includes)
    }

    pub fn all_types_assignable_to_kind(&mut self, source: &Arc<Type>, kind: TypeFlags) -> bool { ::tsox_core::fntrace::enter("all_types_assignable_to_kind"); 
        self.all_types_assignable_to_kind_ex(source, kind, false)
    }

    pub fn all_types_assignable_to_kind_ex(
        &mut self,
        source: &Arc<Type>,
        kind: TypeFlags,
        strict: bool,
    ) -> bool { ::tsox_core::fntrace::enter("all_types_assignable_to_kind_ex"); 
        if source.flags.contains(TypeFlags::Union) {
            return type_types_list(source)
                .iter()
                .all(|sub_type| self.all_types_assignable_to_kind_ex(sub_type, kind, strict));
        }
        self.is_type_assignable_to_kind_ex(source, kind, strict)
    }

    pub fn append_contextual_property_type_constituent(
        &mut self,
        mut types: Vec<Arc<Type>>,
        t: Option<&Arc<Type>>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("append_contextual_property_type_constituent"); 
        let Some(t) = t else {
            return types;
        };
        if t.flags.contains(TypeFlags::Any) {
            types.push(self.unknown_type());
        } else {
            types.push(Arc::clone(t));
        }
        types
    }

    pub fn append_index_info(
        &mut self,
        mut index_infos: Vec<Arc<IndexInfo>>,
        new_info: &Arc<IndexInfo>,
        union: bool,
    ) -> Vec<Arc<IndexInfo>> { ::tsox_core::fntrace::enter("append_index_info"); 
        for i in 0..index_infos.len() {
            let info = &index_infos[i];
            let key_type_matches = info
                .key_type
                .as_ref()
                .zip(new_info.key_type.as_ref())
                .is_some_and(|(a, b)| Arc::ptr_eq(a, b));
            if key_type_matches {
                let (value_type, is_readonly) = if union {
                    let vt = self.get_union_type(vec![
                        info.value_type.clone().unwrap(),
                        new_info.value_type.clone().unwrap(),
                    ]);
                    (vt, info.is_readonly || new_info.is_readonly)
                } else {
                    let vt = self.get_intersection_type(vec![
                        info.value_type.clone().unwrap(),
                        new_info.value_type.clone().unwrap(),
                    ]);
                    (vt, info.is_readonly && new_info.is_readonly)
                };
                index_infos[i] = self.new_index_info(
                    info.key_type.as_ref().unwrap(),
                    &value_type,
                    is_readonly,
                    None,
                    &[],
                );
                return index_infos;
            }
        }
        index_infos.push(Arc::clone(new_info));
        index_infos
    }

    pub fn append_local_type_parameters_of_class_or_interface_or_type_alias(
        &mut self,
        mut types: Vec<Arc<Type>>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("append_local_type_parameters_of_class_or_interface_or_type_alias"); 
        for node in &symbol.declarations {
            if matches!(
                node.kind,
                SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
            ) || is_type_alias(node)
            {
                types = self.append_type_parameters(
                    types,
                    node.type_parameters()
                        .map(|l| l.nodes.clone())
                        .unwrap_or_default(),
                );
            }
        }
        types
    }

    pub fn append_signatures(
        &mut self,
        mut signatures: Vec<Arc<Signature>>,
        new_signatures: &[Arc<Signature>],
    ) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("append_signatures"); 
        for sig in new_signatures {
            let none_identical = signatures.iter().all(|s| {
                self.compare_signatures_identical(s, sig, false, false, false) == Ternary::False
            });
            if signatures.is_empty() || none_identical {
                signatures.push(Arc::clone(sig));
            }
        }
        signatures
    }

    pub fn append_type_parameters(
        &mut self,
        mut types: Vec<Arc<Type>>,
        parameters: Vec<Arc<Node>>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("append_type_parameters"); 
        for parameter in &parameters {
            // Go appendTypeParameters（checker.go:25237）经 getSymbolOfDeclaration
            // 取 binder 挂的声明符号；symbol_node_links 只记引用位解析，声明节点恒空
            if let Some(symbol) = self.get_symbol_of_declaration(parameter) {
                types.push(self.get_declared_type_of_type_parameter(&symbol));
            }
        }
        types
    }

    pub fn are_all_outer_type_parameters_applied(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("are_all_outer_type_parameters_applied"); 
        let Some(data) = t.as_interface_type() else {
            return true;
        };
        if data.outer_type_parameter_count > 0 {
            let last = data.outer_type_parameter_count - 1;
            let outer_last = &data.all_type_parameters[last];
            let type_arguments = self.get_type_arguments(t);
            return !Arc::ptr_eq(outer_last, &type_arguments[last]);
        }
        true
    }

    pub fn are_declaration_flags_identical(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("are_declaration_flags_identical"); 
        if (tsox_frontend::ast::is_parameter_declaration(left)
            && tsox_frontend::ast::is_variable_declaration(right))
            || (tsox_frontend::ast::is_variable_declaration(left)
                && tsox_frontend::ast::is_parameter_declaration(right))
        {
            return true;
        }
        if is_optional_declaration(left) != is_optional_declaration(right) {
            return false;
        }
        let interesting_flags = ModifierFlags::Private
            | ModifierFlags::Protected
            | ModifierFlags::Async
            | ModifierFlags::Abstract
            | ModifierFlags::Readonly
            | ModifierFlags::Static;
        get_selected_modifier_flags(left, interesting_flags)
            == get_selected_modifier_flags(right, interesting_flags)
    }

    pub fn are_type_parameters_identical<F>(
        &mut self,
        declarations: &[Arc<Node>],
        target_parameters: &[Arc<Type>],
        get_type_parameter_declarations: F,
    ) -> bool
    where
        F: Fn(&Arc<Node>) -> Vec<Arc<Node>>,
    { ::tsox_core::fntrace::enter("are_type_parameters_identical"); 
        let max_type_argument_count = target_parameters.len();
        let min_type_argument_count = self.get_min_type_argument_count(target_parameters);
        for declaration in declarations {
            let source_parameters = get_type_parameter_declarations(declaration);
            if source_parameters.len() < min_type_argument_count
                || source_parameters.len() > max_type_argument_count
            {
                return false;
            }
            for (i, source) in source_parameters.iter().enumerate() {
                let target = &target_parameters[i];
                let source_text = source.name().map(|n| n.text()).unwrap_or_default();
                let target_name = target
                    .symbol
                    .as_ref()
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                if source_text != target_name {
                    return false;
                }
                let (constraint_node, default_node) = match &source.data {
                    tsox_frontend::ast::NodeData::TypeParameterDeclaration(d) => {
                        (d.constraint.clone(), d.default_type.clone())
                    }
                    _ => (None, None),
                };
                let target_constraint = self.get_constraint_of_type_parameter(target);
                if let (Some(constraint_node), Some(target_constraint)) =
                    (&constraint_node, &target_constraint)
                {
                    let constraint_type = self.get_type_from_type_node(constraint_node);
                    if !self.is_type_identical_to(&constraint_type, target_constraint) {
                        return false;
                    }
                }
                let target_default = self.get_default_from_type_parameter(target);
                if let (Some(default_node), Some(target_default)) = (&default_node, &target_default)
                {
                    let default_type = self.get_type_from_type_node(default_node);
                    if !self.is_type_identical_to(&default_type, target_default) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Checker {
    pub fn assign_binding_element_types(&mut self, pattern: &Arc<Node>, parent_type: &Arc<Type>) { ::tsox_core::fntrace::enter("assign_binding_element_types"); 
        let elements = pattern
            .elements()
            .map(|l| l.nodes.clone())
            .unwrap_or_default();
        for element in elements {
            if let Some(name) = element.name() {
                let t = self.get_binding_element_type_from_parent_type(&element, parent_type, false);
                if is_identifier(name) {
                    if let Some(sym) = self.get_symbol_of_declaration(&element) {
                        self.value_symbol_links
                            .get_or_default(&sym)
                            .resolved_type = Some(t);
                    }
                } else {
                    self.assign_binding_element_types(name, &t);
                }
            }
        }
    }

    pub fn assign_contextual_parameter_types(
        &mut self,
        sig: &mut Signature,
        context: &Signature,
    ) { ::tsox_core::fntrace::enter("assign_contextual_parameter_types"); 
        let context = Arc::new(context.clone());
        if !context.type_parameters.is_empty() {
            if !sig.type_parameters.is_empty() {
                return;
            }
            sig.type_parameters = context.type_parameters.clone();
        }
        if let Some(context_this) = &context.this_parameter {
            let parameter = sig.this_parameter.clone();
            let needs_assign = match &parameter {
                None => {
                    let with_type = self.create_symbol_with_type(
                        context_this,
                        &self.unknown_type(),
                    );
                    if let Some(links) = self.value_symbol_links.get_mut(&with_type) {
                        links.resolved_type = None;
                    }
                    sig.this_parameter = Some(with_type);
                    true
                }
                Some(p) => match &p.value_declaration {
                    Some(d) => d.type_node().is_none(),
                    None => true,
                },
            };
            if needs_assign {
                if let Some(this_parameter) = &sig.this_parameter {
                    let t = self.get_type_of_symbol(context_this);
                    self.assign_parameter_type(this_parameter, Some(&t));
                }
            }
        }
        let length = sig
            .parameters
            .len()
            .saturating_sub(if signature_has_rest_parameter(&Arc::new(sig.clone())) {
                1
            } else {
                0
            });
        for i in 0..length {
            let parameter = Arc::clone(&sig.parameters[i]);
            let declaration = parameter.value_declaration.clone();
            if let Some(declaration) = declaration {
                if declaration.type_node().is_none() {
                    let mut t = self.try_get_type_at_position(&context, i);
                    if let Some(tt) = &t {
                        if declaration.initializer().is_some() {
                            let mut initializer_type =
                                self.check_declaration_initializer(&declaration, CheckMode::Normal, None);
                            if !self.is_type_assignable_to(&initializer_type, tt) {
                                initializer_type =
                                    self.widen_type_inferred_from_initializer(&declaration, &initializer_type);
                                if self.is_type_assignable_to(tt, &initializer_type) {
                                    t = Some(initializer_type);
                                }
                            }
                        }
                    }
                    self.assign_parameter_type(&parameter, t.as_ref());
                }
            }
        }
        if signature_has_rest_parameter(&Arc::new(sig.clone())) {
            let Some(parameter) = sig.parameters.last() else {
                return;
            };
            let assign = match &parameter.value_declaration {
                Some(d) => d.type_node().is_none(),
                None => parameter
                    .check_flags
                    .contains(CheckFlags::DeferredType),
            };
            if assign {
                let contextual_parameter_type = self.get_rest_type_at_position(&context, length);
                self.assign_parameter_type(parameter, contextual_parameter_type.as_ref());
            }
        }
    }

    pub fn assign_non_contextual_parameter_types(&mut self, signature: &Signature) { ::tsox_core::fntrace::enter("assign_non_contextual_parameter_types"); 
        if let Some(this_parameter) = &signature.this_parameter {
            self.assign_parameter_type(this_parameter, None);
        }
        for parameter in &signature.parameters {
            self.assign_parameter_type(parameter, None);
        }
    }

    pub fn assign_parameter_type(
        &mut self,
        parameter: &Arc<Symbol>,
        contextual_type: Option<&Arc<Type>>,
    ) { ::tsox_core::fntrace::enter("assign_parameter_type"); 
        if let Some(existing) = self
            .value_symbol_links
            .get(parameter)
            .and_then(|l| l.resolved_type.as_ref())
        {
            // 解析窗口内的 in-flight error 占位不是已定型：Go 的
            // resolvedType != nil 守卫下该状态不存在（解析期不污染缓存），
            // 此时仍需执行赋值（嵌套 assignContextualParameterTypes 经此
            // 写入位置型 + 初始化器回退的结果）
            let in_flight = crate::checker::utilities::is_type_error(existing)
                && self.is_resolving(
                    Arc::as_ptr(parameter) as *const Symbol,
                    crate::checker::TypeResolutionProperty::Type,
                );
            if !in_flight {
                return;
            }
        }
        let declaration = parameter.value_declaration.clone();
        let t: Option<Arc<Type>> = match contextual_type {
            Some(t) => Some(Arc::clone(t)),
            None => match &declaration {
                Some(d) => self.get_widened_type_for_variable_like_declaration(d, true),
                None => Some(self.get_type_of_symbol(parameter)),
            },
        };
        let optional = declaration
            .as_ref()
            .is_some_and(|d| d.initializer().is_none() && is_optional_declaration(d));
        let resolved = self.add_optionality_ex(t.as_ref().unwrap(), false, optional);
        if let Some(d) = &declaration {
            if let Some(name) = d.name() {
                if !is_identifier(&name) {
                    let mut resolved_type = resolved;
                    if Arc::ptr_eq(&resolved_type, &self.unknown_type()) {
                        resolved_type = self.get_type_from_binding_pattern(&name, false, false);
                    }
                    self.assign_binding_element_types(&name, &resolved_type);
                    if let Some(links) = self.value_symbol_links.get_mut(parameter) {
                        links.resolved_type = Some(resolved_type);
                    }
                    return;
                }
            }
        }
        if let Some(links) = self.value_symbol_links.get_mut(parameter) {
            links.resolved_type = Some(resolved);
        }
    }

    pub fn both_are_big_int_like(&mut self, left: &Arc<Type>, right: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("both_are_big_int_like"); 
        self.is_type_assignable_to_kind(left, TYPE_FLAGS_BIG_INT_LIKE)
            && self.is_type_assignable_to_kind(right, TYPE_FLAGS_BIG_INT_LIKE)
    }

    pub fn can_get_type_parameters_of_class_or_interface(&mut self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("can_get_type_parameters_of_class_or_interface"); 
        self.get_class_or_interface_like_declaration(symbol).is_some()
    }

    pub fn check_accessor_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_accessor_declaration"); 
        if !self.check_grammar_function_like_declaration(node)
            && !self.check_grammar_accessor(node)
        {
            if let Some(name) = node.name() {
                self.check_grammar_computed_property_name(&name);
            }
        }
        let name = node.name();
        if let Some(name) = &name {
            let is_constructor_name = match &name.data {
                NodeData::Identifier(d) => d.text == "constructor",
                _ => false,
            };
            if is_constructor_name
                && node
                    .parent()
                    .is_some_and(|p| tsox_frontend::ast::is_class_like(&p))
            {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::CLASS_CONSTRUCTOR_MAY_NOT_BE_AN_ACCESSOR,
                    vec![],
                ));
            }
        }
        self.check_decorators(node);
        self.check_signature_declaration(node);
        if tsox_frontend::ast::is_get_accessor_declaration(node)
            && !node.flags.contains(NodeFlags::Ambient)
            && node
                .body()
                .is_some_and(|b| tsox_frontend::ast::node_is_present(Some(&b)))
            && node.flags.contains(NodeFlags::HasImplicitReturn)
            && !node.flags.contains(NodeFlags::HasExplicitReturn)
        {
            if let Some(name) = &name {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::A_GET_ACCESSOR_MUST_RETURN_A_VALUE,
                    vec![],
                ));
            }
        }
        if let Some(name) = &name {
            if tsox_frontend::ast::is_computed_property_name(name) {
                self.check_computed_property_name(name);
            }
        }
        if self.has_bindable_name(node) {
            let symbol = self.get_symbol_of_declaration(node);
            if let Some(symbol) = symbol {
                let getter = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(
                    &symbol,
                    SyntaxKind::GetAccessor,
                );
                let setter = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(
                    &symbol,
                    SyntaxKind::SetAccessor,
                );
                if let (Some(getter), Some(setter)) = (&getter, &setter) {
                    if !self
                        .node_links
                        .get(getter)
                        .is_some_and(|l| l.flags.contains(NodeCheckFlags::TypeChecked))
                    {
                        self.node_links
                            .get_or_default(getter)
                            .flags |= NodeCheckFlags::TypeChecked;
                        let getter_flags =
                            get_selected_modifier_flags(getter, ModifierFlags::all());
                        let setter_flags =
                            get_selected_modifier_flags(setter, ModifierFlags::all());
                        if (getter_flags & ModifierFlags::Abstract)
                            != (setter_flags & ModifierFlags::Abstract)
                        {
                            for accessor in [getter, setter] {
                                if let Some(accessor_name) = accessor.name() {
                                    self.diagnostics.add(Diagnostic::new(
                                        self.current_file.clone(),
                                        accessor_name.loc,
                                        tsox_core::diagnostics::messages_generated::ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                        vec![],
                                    ));
                                }
                            }
                        }
                        if (getter_flags.contains(ModifierFlags::Protected)
                            && !(setter_flags
                                .intersects(ModifierFlags::Protected | ModifierFlags::Private)))
                            || (getter_flags.contains(ModifierFlags::Private)
                                && !setter_flags.contains(ModifierFlags::Private))
                        {
                            for accessor in [getter, setter] {
                                if let Some(accessor_name) = accessor.name() {
                                    self.diagnostics.add(Diagnostic::new(
                                        self.current_file.clone(),
                                        accessor_name.loc,
                                        tsox_core::diagnostics::messages_generated::A_GET_ACCESSOR_MUST_BE_AT_LEAST_AS_ACCESSIBLE_AS_THE_SETTER,
                                        vec![],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(symbol) = self.get_symbol_of_declaration(node) {
            let return_type = self.get_type_of_accessors(&symbol);
            if node.kind == SyntaxKind::GetAccessor {
                self.check_all_code_paths_in_non_void_function_return_or_throw(node, Some(&return_type));
            }
        }
        if let Some(body) = node.body() {
            self.check_source_element(&body);
        }
        self.set_node_links_for_private_identifier_scope(node);
    }

    pub fn check_all_code_paths_in_non_void_function_return_or_throw(
        &mut self,
        fn_node: &Arc<Node>,
        return_type: Option<&Arc<Type>>,
    ) { ::tsox_core::fntrace::enter("check_all_code_paths_in_non_void_function_return_or_throw"); 
        let function_flags = tsox_frontend::ast::mig::m3e::get_function_flags(Some(fn_node));
        let t = return_type.map(|rt| {
            if function_flags
                .contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC)
            {
                let awaited = self.check_awaited_type(rt, false, Some(fn_node), None);
                self.unwrap_awaited_type(&awaited)
            } else {
                Arc::clone(rt)
            }
        });
        if let Some(t) = &t {
            if self.maybe_type_of_kind(t, TypeFlags::Void)
                || t.flags.intersects(TypeFlags::Any | TypeFlags::Undefined)
            {
                return;
            }
        }
        let body = fn_node.body();
        if tsox_frontend::ast::is_method_signature_declaration(fn_node)
            || body
                .as_ref()
                .is_some_and(|b| tsox_frontend::ast::node_is_missing(Some(b)))
            || !body.as_ref().is_some_and(|b| tsox_frontend::ast::is_block(b))
            || !self.function_has_implicit_return(fn_node)
        {
            return;
        }
        let has_explicit_return = fn_node.flags.contains(NodeFlags::HasExplicitReturn);
        let mut error_node = fn_node.type_node();
        if error_node.is_none() {
            error_node = fn_node.type_node();
        }
        let error_node = error_node.unwrap_or(fn_node);
        if let Some(t) = &t {
            if t.flags.contains(TypeFlags::Never) {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_node.loc,
                    tsox_core::diagnostics::messages_generated::A_FUNCTION_RETURNING_NEVER_CANNOT_HAVE_A_REACHABLE_END_POINT,
                    vec![],
                ));
            } else if !has_explicit_return {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_node.loc,
                    tsox_core::diagnostics::messages_generated::A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                    vec![],
                ));
            } else if self.strict_null_checks
                && !self.is_type_assignable_to(&self.undefined_type(), t)
            {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_node.loc,
                    tsox_core::diagnostics::messages_generated::FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                    vec![],
                ));
            } else if self.compiler_options.no_implicit_returns == tsox_core::core::tristate::Tristate::True {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_node.loc,
                    tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
                    vec![],
                ));
            }
        } else if self.compiler_options.no_implicit_returns == tsox_core::core::tristate::Tristate::True {
            if !has_explicit_return {
                return;
            }
            let Some(signature) = self.get_signature_from_declaration(fn_node) else {
                return;
            };
            let inferred_return_type = self
                .get_return_type_of_signature(&signature)
                .unwrap_or_else(|| self.unknown_type());
            if !self.is_unwrapped_return_type_undefined_void_or_any(fn_node, &inferred_return_type)
            {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_node.loc,
                    tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
                    vec![],
                ));
            }
        }
    }

    pub fn check_and_aggregate_return_expression_types(
        &mut self,
        fn_node: &Arc<Node>,
        check_mode: CheckMode,
    ) -> (Option<Vec<Arc<Type>>>, bool) { ::tsox_core::fntrace::enter("check_and_aggregate_return_expression_types"); 
        let function_flags = tsox_frontend::ast::mig::m3e::get_function_flags(Some(fn_node));
        let mut aggregated_types: Vec<Arc<Type>> = Vec::new();
        let mut has_return_with_no_expression = self.function_has_implicit_return(fn_node);
        let mut has_return_of_type_never = false;
        let fn_symbol = self.symbol_of_node(fn_node).map(|s| self.get_merged_symbol(&s));
        if let Some(body) = fn_node.body() {
        tsox_frontend::ast::mig::m3e_4::for_each_return_statement(body, &mut |return_statement: &Arc<Node>| {
            let Some(expr) = return_statement.expression() else {
                has_return_with_no_expression = true;
                return false;
            };
            let mut expr = Checker::skip_parentheses(&expr);
            if function_flags.contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC)
                && tsox_frontend::ast::is_await_expression(&expr)
            {
                if let Some(inner) = expr.expression() {
                    expr = Checker::skip_parentheses(&inner);
                }
            }
            if tsox_frontend::ast::is_call_expression(&expr)
                && expr
                    .expression()
                    .is_some_and(|e| is_identifier(&e))
            {
                let callee = expr.expression().unwrap();
                let checked_symbol = self
                    .check_expression_cached(&callee)
                    .symbol
                    .clone();
                let is_self_call = match (&checked_symbol, &fn_symbol) {
                    (Some(cs), Some(fs)) => Arc::ptr_eq(cs, fs),
                    _ => false,
                };
                let not_fn_expr = !fn_symbol
                    .as_ref()
                    .and_then(|s| s.value_declaration.clone())
                    .is_some_and(|d| tsox_frontend::ast::is_function_expression_or_arrow_function(&d));
                if is_self_call && (not_fn_expr || self.is_constant_reference(&callee)) {
                    has_return_of_type_never = true;
                    return false;
                }
            }
            let mut t = self.check_expression_cached_ex(
                &expr,
                check_mode - CheckMode::SkipGenericFunctions,
            );
            if function_flags.contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC) {
                let awaited = self.check_awaited_type(
                    &t,
                    false,
                    Some(fn_node),
                    Some(&tsox_core::diagnostics::messages_generated::THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER),
                );
                t = self.unwrap_awaited_type(&awaited);
            }
            if t.flags.contains(TypeFlags::Never) {
                has_return_of_type_never = true;
            }
            if self.is_const_context(&expr) {
                t = self.get_regular_type_of_literal_type(&t);
            }
            if !aggregated_types.iter().any(|a| Arc::ptr_eq(a, &t)) {
                aggregated_types.push(t);
            }
            false
        });
        }
        if aggregated_types.is_empty()
            && !has_return_with_no_expression
            && (has_return_of_type_never || Checker::may_return_never(fn_node))
        {
            return (None, true);
        }
        if self.strict_null_checks
            && !aggregated_types.is_empty()
            && has_return_with_no_expression
        {
            let undefined = self.undefined_type();
            if !aggregated_types.iter().any(|a| Arc::ptr_eq(a, &undefined)) {
                aggregated_types.push(undefined);
            }
        }
        (Some(aggregated_types), false)
    }

    pub fn check_and_aggregate_yield_operand_types(
        &mut self,
        fn_node: &Arc<Node>,
        check_mode: CheckMode,
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) { ::tsox_core::fntrace::enter("check_and_aggregate_yield_operand_types"); 
        let is_async = tsox_frontend::ast::mig::m3e::get_function_flags(Some(fn_node))
            .contains(tsox_frontend::ast::mig::m3e::FunctionFlags::ASYNC);
        let mut yield_types: Vec<Arc<Type>> = Vec::new();
        let mut next_types: Vec<Arc<Type>> = Vec::new();
        for_each_yield_expression(fn_node.body(), &mut |yield_expr| {
            let mut yield_expr_type = self.undefined_widening_type();
            if let Some(expr) = yield_expr.expression() {
                yield_expr_type = self.check_expression_ex(
                    &expr,
                    check_mode - CheckMode::SkipGenericFunctions,
                );
                if self.is_const_context(&expr) {
                    yield_expr_type = self.get_regular_type_of_literal_type(&yield_expr_type);
                }
            }
            let yielded = self.get_yielded_type_of_yield_expression(
                yield_expr,
                &yield_expr_type,
                &self.any_type(),
                is_async,
            );
            if !yield_types.iter().any(|a| Arc::ptr_eq(a, &yielded)) {
                yield_types.push(yielded);
            }
            let next_type = if let tsox_frontend::ast::NodeData::YieldExpression(d) =
                &yield_expr.data
            {
                if d.asterisk_token.is_some() {
                    let use_ = IterationUse::YieldStar { is_async };
                    let iteration_types =
                        self.iteration_types_of_iterable(use_, &yield_expr_type, yield_expr.expression());
                    iteration_types.next_type
                } else {
                    self.get_contextual_type(yield_expr, ContextFlags::None)
                }
            } else {
                None
            };
            if let Some(next_type) = next_type {
                if !next_types.iter().any(|a| Arc::ptr_eq(a, &next_type)) {
                    next_types.push(next_type);
                }
            }
            false
        });
        (yield_types, next_types)
    }
}

impl CacheHashKey {
    pub fn is_zero(&self) -> bool { ::tsox_core::fntrace::enter("is_zero"); 
        self.hi == 0 && self.lo == 0
    }
}

impl Checker {
    pub fn check_and_report_error_for_exporting_primitive_type(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_exporting_primitive_type"); 
        if is_primitive_type_name(name)
            && error_location
                .parent()
                .is_some_and(|p| p.kind == SyntaxKind::ExportSpecifier)
        {
            self.diagnostics.add(Diagnostic::new(
                self.current_file.clone(),
                error_location.loc,
                tsox_core::diagnostics::messages_generated::CANNOT_EXPORT_0_ONLY_LOCAL_DECLARATIONS_CAN_BE_EXPORTED_FROM_A_MODULE,
                vec![name.to_string()],
            ));
            return true;
        }
        false
    }

    fn get_entity_name_for_extending_interface_arc(
        &self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_entity_name_for_extending_interface_arc"); 
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::QualifiedName
            | SyntaxKind::PropertyAccessExpression => node
                .parent()
                .and_then(|p| self.get_entity_name_for_extending_interface_arc(&p)),
            SyntaxKind::TypeReference => {
                if let NodeData::TypeReferenceNode(tr) = &node.data {
                    return Some(Arc::clone(&tr.type_name));
                }
                None
            }
            SyntaxKind::ExpressionWithTypeArguments => {
                let e = node.expression()?;
                if matches!(
                    e.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) {
                    Some(Arc::clone(e))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn check_and_report_error_for_extending_interface(
        &mut self,
        error_location: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_extending_interface"); 
        let expression = self.get_entity_name_for_extending_interface_arc(error_location);
        if let Some(expression) = expression {
            let resolved = self.resolve_entity_name(&expression, SymbolFlags::Interface, false, false, None);
            if resolved.is_some() {
                let text = tsox_frontend::scanner::mig::m3i::get_text_of_node(&expression);
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_location.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS,
                    vec![text],
                ));
                return true;
            }
        }
        false
    }

    pub fn check_and_report_error_for_invalid_initializer(
        &mut self,
        error_location: Option<&Arc<Node>>,
        name: &str,
        property_with_invalid_initializer: &Arc<Node>,
        result: Option<&Arc<Symbol>>,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_invalid_initializer"); 
        if !self.compiler_options.get_emit_standard_class_fields() {
            if let Some(error_location) = error_location {
                if result.is_none()
                    && self.check_and_report_error_for_missing_prefix(error_location, name)
                {
                    return true;
                }
            }
            let prop = match &property_with_invalid_initializer.data {
                NodeData::PropertyDeclaration(d) => d,
                _ => return true,
            };
            let type_annotated = error_location.is_some_and(|loc| {
                prop.type_node.as_ref().is_some_and(|t| {
                    t.loc.contains_inclusive(loc.pos())
                })
            });
            let message = if type_annotated {
                &tsox_core::diagnostics::messages_generated::TYPE_OF_INSTANCE_MEMBER_VARIABLE_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_IN_THE_CONSTRUCTOR
            } else {
                &tsox_core::diagnostics::messages_generated::INITIALIZER_OF_INSTANCE_MEMBER_VARIABLE_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_IN_THE_CONSTRUCTOR
            };
            let prop_name = tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(
                &prop.name,
            ));
            if let Some(loc) = error_location {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    loc.loc,
                    message.clone(),
                    vec![prop_name, name.to_string()],
                ));
            }
            return true;
        }
        false
    }

    pub fn check_and_report_error_for_missing_prefix(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_missing_prefix"); 
        let is_target = match &error_location.data {
            NodeData::Identifier(d) => d.text == name,
            _ => false,
        };
        if !is_target
            || is_type_reference_identifier(error_location)
            || is_in_type_query(error_location)
        {
            return false;
        }
        let container = crate::checker::checker_this_container::get_this_container(
            error_location,
            false,
            false,
        );
        let mut location = container.clone();
        while let Some(parent) = location.parent() {
            if tsox_frontend::ast::is_class_like(&parent) {
                let Some(class_symbol) = self.get_symbol_of_declaration(&parent) else {
                    break;
                };
                let constructor_type = self.get_type_of_symbol(&class_symbol);
                if self
                    .get_property_type_of_type(&constructor_type, name)
                    .is_some()
                {
                    let class_name = self.symbol_to_string(&class_symbol);
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0,
                        vec![name.to_string(), format!("{}.{name}", class_name)],
                    ));
                    return true;
                }
                if Arc::ptr_eq(&location, &container)
                    && !location.has_syntactic_modifier(ModifierFlags::Static)
                {
                    let declared = self.get_declared_type_of_symbol(&class_symbol);
                    let this_type =
                        declared.as_interface_type().and_then(|d| d.this_type.clone());
                    if let Some(this_type) = &this_type {
                        if self.get_property_type_of_type(this_type, name).is_some() {
                        self.diagnostics.add(Diagnostic::new(
                            self.current_file.clone(),
                            error_location.loc,
                            tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0,
                            vec![name.to_string()],
                        ));
                            return true;
                        }
                    }
                }
            }
            location = parent;
        }
        false
    }

    pub fn check_and_report_error_for_resolving_import_alias_to_type_only_symbol(
        &mut self,
        node: &Arc<Node>,
        _resolved: &Arc<Symbol>,
    ) { ::tsox_core::fntrace::enter("check_and_report_error_for_resolving_import_alias_to_type_only_symbol"); 
        let decl = match &node.data {
            NodeData::ImportEqualsDeclaration(d) => d,
            _ => return,
        };
        let mut name = Some(Arc::clone(&decl.module_reference));
        while let Some(current) = name {
            if let Some(type_only_declaration) = self.get_type_only_declaration_of_entity_name(&current)
            {
                let is_export = matches!(
                    type_only_declaration.kind,
                    SyntaxKind::ExportSpecifier | SyntaxKind::ExportDeclaration
                );
                let message = if is_export {
                    &tsox_core::diagnostics::messages_generated::AN_IMPORT_ALIAS_CANNOT_REFERENCE_A_DECLARATION_THAT_WAS_EXPORTED_USING_EXPORT_TYPE
                } else {
                    &tsox_core::diagnostics::messages_generated::AN_IMPORT_ALIAS_CANNOT_REFERENCE_A_DECLARATION_THAT_WAS_IMPORTED_USING_IMPORT_TYPE
                };
                let related_message = if is_export {
                    &tsox_core::diagnostics::messages_generated::X_0_WAS_EXPORTED_HERE
                } else {
                    &tsox_core::diagnostics::messages_generated::X_0_WAS_IMPORTED_HERE
                };
                let decl_name = if tsox_frontend::ast::is_export_declaration(&type_only_declaration)
                {
                    "*".to_string()
                } else {
                    type_only_declaration
                        .name()
                        .map(|n| n.text().to_string())
                        .unwrap_or_else(|| "*".to_string())
                };
                let related = new_diagnostic_for_node(
                    Some(&type_only_declaration),
                    related_message.clone(),
                    vec![decl_name],
                );
                let diagnostic = Diagnostic::new(
                    self.current_file.clone(),
                    current.loc,
                    message.clone(),
                    vec![],
                );
                let mut diagnostic = diagnostic;
                diagnostic.add_related_info(related);
                self.diagnostics.add(diagnostic);
                break;
            }
            name = match &current.data {
                NodeData::Identifier(_) => None,
                NodeData::QualifiedName(q) => Some(Arc::clone(&q.left)),
                _ => None,
            };
        }
    }

    pub fn check_and_report_error_for_using_namespace_as_type_or_value(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_using_namespace_as_type_or_value"); 
        if meaning.intersects(SymbolFlags::VALUE & !SymbolFlags::TYPE) {
            let resolved =
                self.resolve_name(name, error_location, SymbolFlags::NamespaceModule, false);
            let symbol = resolved.as_ref().map(|r| self.resolve_symbol(r));
            if symbol.is_some() {
                if !is_export_assignment_expression_name(error_location) {
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                        vec![name.to_string()],
                    ));
                }
                return true;
            }
        } else if meaning.intersects(SymbolFlags::TYPE & !SymbolFlags::VALUE) {
            let resolved = self.resolve_name(name, error_location, SymbolFlags::MODULE, false);
            let symbol = resolved.as_ref().map(|r| self.resolve_symbol(r));
            if symbol.is_some() {
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_location.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_TYPE,
                    vec![name.to_string()],
                ));
                return true;
            }
        }
        false
    }

    pub fn check_and_report_error_for_using_type_as_namespace(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_using_type_as_namespace"); 
        if meaning == SymbolFlags::NAMESPACE {
            let resolved = self.resolve_name(
                name,
                error_location,
                SymbolFlags::TYPE & !SymbolFlags::NAMESPACE,
                false,
            );
            let symbol = resolved.map(|r| self.resolve_symbol(&r));
            if let Some(symbol) = symbol {
                if let Some(parent) = error_location.parent() {
                    if let NodeData::QualifiedName(q) = &parent.data {
                        let prop_name = match &q.right.data {
                            NodeData::Identifier(d) => d.text.clone(),
                            _ => String::new(),
                        };
                        let declared = self.get_declared_type_of_symbol(&symbol);
                        if let Some(_prop_type) =
                            self.get_property_type_of_type(&declared, &prop_name)
                        {
                            self.diagnostics.add(Diagnostic::new(
                                self.current_file.clone(),
                                parent.loc,
                                tsox_core::diagnostics::messages_generated::CANNOT_ACCESS_0_1_BECAUSE_0_IS_A_TYPE_BUT_NOT_A_NAMESPACE_DID_YOU_MEAN_TO_RETRIEVE_THE_TYPE_OF_THE_PROPERTY_1_IN_0_WITH_0_1,
                                vec![name.to_string(), prop_name],
                            ));
                            return true;
                        }
                    }
                }
                self.diagnostics.add(Diagnostic::new(
                    self.current_file.clone(),
                    error_location.loc,
                    tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                    vec![name.to_string()],
                ));
                return true;
            }
        }
        false
    }

    pub fn check_and_report_error_for_using_type_as_value(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_using_type_as_value"); 
        if !meaning.contains(SymbolFlags::VALUE) {
            return false;
        }
        if is_primitive_type_name(name) {
            let grandparent = error_location.parent().and_then(|p| p.parent());
            if let Some(grandparent) = grandparent {
                if let Some(great_grandparent) = grandparent.parent() {
                    if tsox_frontend::ast::is_heritage_clause(&grandparent) {
                        let heritage_kind = match &grandparent.data {
                            NodeData::HeritageClause(hc) => hc.token,
                            _ => SyntaxKind::Unknown,
                        };
                        let container_kind = great_grandparent.kind;
                        if container_kind == SyntaxKind::InterfaceDeclaration
                            && heritage_kind == SyntaxKind::ExtendsKeyword
                        {
                            self.diagnostics.add(Diagnostic::new(
                                self.current_file.clone(),
                                error_location.loc,
                                tsox_core::diagnostics::messages_generated::AN_INTERFACE_CANNOT_EXTEND_A_PRIMITIVE_TYPE_LIKE_0_IT_CAN_ONLY_EXTEND_OTHER_NAMED_OBJECT_TYPES,
                                vec![name.to_string()],
                            ));
                            return true;
                        } else if tsox_frontend::ast::is_class_like(&great_grandparent)
                            && heritage_kind == SyntaxKind::ExtendsKeyword
                        {
                            self.diagnostics.add(Diagnostic::new(
                                self.current_file.clone(),
                                error_location.loc,
                                tsox_core::diagnostics::messages_generated::A_CLASS_CANNOT_EXTEND_A_PRIMITIVE_TYPE_LIKE_0_CLASSES_CAN_ONLY_EXTEND_CONSTRUCTABLE_VALUES,
                                vec![name.to_string()],
                            ));
                            return true;
                        } else if tsox_frontend::ast::is_class_like(&great_grandparent)
                            && heritage_kind == SyntaxKind::ImplementsKeyword
                        {
                            self.diagnostics.add(Diagnostic::new(
                                self.current_file.clone(),
                                error_location.loc,
                                tsox_core::diagnostics::messages_generated::A_CLASS_CANNOT_IMPLEMENT_A_PRIMITIVE_TYPE_LIKE_0_IT_CAN_ONLY_IMPLEMENT_OTHER_NAMED_OBJECT_TYPES,
                                vec![name.to_string()],
                            ));
                            return true;
                        }
                    } else {
                        self.diagnostics.add(Diagnostic::new(
                            self.current_file.clone(),
                            error_location.loc,
                            tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                            vec![name.to_string()],
                        ));
                    }
                    return true;
                }
            }
            self.diagnostics.add(Diagnostic::new(
                self.current_file.clone(),
                error_location.loc,
                tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                vec![name.to_string()],
            ));
            return true;
        }
        let resolved = self.resolve_name(
            name,
            error_location,
            SymbolFlags::TYPE & !SymbolFlags::VALUE,
            false,
        );
        let symbol = resolved.map(|r| self.resolve_symbol(&r));
        if let Some(symbol) = symbol {
            let all_flags = self.get_symbol_flags(&symbol);
            if !all_flags.contains(SymbolFlags::VALUE) {
                if is_export_assignment_expression_name(error_location) {
                    return true;
                }
                if Checker::is_es2015_or_later_constructor_name(name) {
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_ES2015_OR_LATER,
                        vec![name.to_string()],
                    ));
                } else if self.maybe_mapped_type(error_location, &symbol) {
                    let other = if name == "K" { "P" } else { "K" };
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE_DID_YOU_MEAN_TO_USE_1_IN_0,
                        vec![name.to_string(), other.to_string()],
                    ));
                } else {
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                        vec![name.to_string()],
                    ));
                }
                return true;
            }
        }
        false
    }

    pub fn check_and_report_error_for_using_value_as_type(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_and_report_error_for_using_value_as_type"); 
        if meaning.intersects(SymbolFlags::TYPE & !SymbolFlags::NAMESPACE) {
            let resolved = self.resolve_name(
                name,
                error_location,
                !SymbolFlags::TYPE & SymbolFlags::VALUE,
                false,
            );
            let symbol = resolved.map(|r| self.resolve_symbol(&r));
            if let Some(symbol) = symbol {
                if !symbol.flags.contains(SymbolFlags::NAMESPACE) {
                    self.diagnostics.add(Diagnostic::new(
                        self.current_file.clone(),
                        error_location.loc,
                        tsox_core::diagnostics::messages_generated::X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                        vec![name.to_string()],
                    ));
                    return true;
                }
            }
        }
        false
    }
}
