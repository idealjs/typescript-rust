use tsox_frontend::ast::mig::m3g_2::{is_outermost_optional_chain, is_type_only_import_declaration};
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2a::r20k6_defs::*;
use crate::checker::mig::wc3::symbol_ptr_key;
use crate::checker::types_type_flags_instantiable_non_primitive::ELEMENT_FLAGS_VARIABLE;
use tsox_frontend::ast::mig::m3g_3::{is_part_of_type_node, is_valid_type_only_alias_use_site};
use crate::checker::mig::m3a_2::{has_dot_dot_dot_token, is_const_type_reference_name, new_diagnostic_for_node};
use crate::checker::mig::wc1b::is_tuple_type;
use crate::checker::checker::*;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

#[derive(Default)]
pub struct ContextualInfo {
    pub node: Option<Arc<Node>>,
    pub t: Option<Arc<Type>>,
    pub is_cache: bool,
}

#[derive(Default)]
pub struct InferenceContextInfo {
    pub node: Option<Arc<Node>>,
    pub context: Option<Arc<InferenceContext>>,
}

impl Checker {
    pub fn new_template_literal_type(
        &mut self,
        texts: &[String],
        types: &[Arc<Type>],
    ) -> Arc<Type> {
        let mut data = empty_template_literal_type_data();
        data.texts = texts.to_vec();
        data.types = types.to_vec();
        self.new_type(
            TypeFlags::TemplateLiteral,
            ObjectFlags::empty(),
            TypeData::TemplateLiteral(data),
        )
    }

    pub fn new_type(
        &mut self,
        flags: TypeFlags,
        object_flags: ObjectFlags,
        data: TypeData,
    ) -> Arc<Type> {
        self.type_count += 1;
        let object_flags = object_flags
            & !(ObjectFlags::CouldContainTypeVariablesComputed
                | ObjectFlags::CouldContainTypeVariables
                | ObjectFlags::MembersResolved);
        Arc::new(Type {
            flags,
            object_flags,
            id: self.type_count,
            symbol: None,
            alias: None,
            data,
        })
    }

    pub fn new_typed_property_descriptor_type(
        &mut self,
        property_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_typed_property_descriptor_type();
        self.create_type_from_generic_global_type(&global, &[property_type.clone()])
    }

    pub fn new_union_type(&mut self, object_flags: ObjectFlags, types: &[Arc<Type>]) -> Arc<Type> {
        let mut data = UnionTypeData::default();
        data.union_or_intersection.types = types.to_vec();
        self.new_type(TypeFlags::Union, object_flags, TypeData::Union(data))
    }

    pub fn on_failed_to_resolve_symbol(
        &mut self,
        error_location: &Arc<Node>,
        name: &str,
        meaning: SymbolFlags,
        name_not_found_message: &'static tsox_core::diagnostics::Message,
    ) {
        if is_const_type_reference_name(error_location) {
            return;
        }
        if error_location.parent().is_some_and(|p| p.kind == SyntaxKind::JSDocLink)
            || self.check_and_report_error_for_missing_prefix(error_location, name)
            || self.check_and_report_error_for_extending_interface(error_location)
            || self.check_and_report_error_for_using_type_as_namespace(error_location, name, meaning)
            || self.check_and_report_error_for_exporting_primitive_type(error_location, name)
            || self.check_and_report_error_for_using_namespace_as_type_or_value(
                error_location, name, meaning,
            )
            || self.check_and_report_error_for_using_type_as_value(error_location, name, meaning)
            || self.check_and_report_error_for_using_value_as_type(error_location, name, meaning)
        {
            return;
        }
        let declaration_name = if tsox_frontend::ast::is_identifier(error_location)
            && error_location.text() == name
        {
            declaration_name_to_string(Some(error_location))
        } else {
            name.to_string()
        };
        let suggested_lib = self.get_suggested_lib_for_non_existent_name(name);
        if !suggested_lib.is_empty() {
            self.error_message(error_location, *name_not_found_message, &[declaration_name, suggested_lib]);
            return;
        }
        let suggestion = self.get_suggested_symbol_for_nonexistent_symbol(error_location, name, meaning);
        if let Some(suggestion) = suggestion {
            let is_ambient_global_augmentation = suggestion
                .value_declaration
                .as_ref()
                .is_some_and(|d| {
                    tsox_frontend::ast::is_ambient_module(d)
                        && tsox_frontend::ast::is_global_scope_augmentation(d)
                });
            if !is_ambient_global_augmentation {
                let suggestion_name = self.symbol_to_string(&suggestion);
                let is_unchecked_js =
                    self.is_unchecked_js_suggestion(Some(error_location), Some(&suggestion), false);
                let message = if meaning == SymbolFlags::NAMESPACE {
                    tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1
                } else if is_unchecked_js {
                    tsox_core::diagnostics::messages_generated::COULD_NOT_FIND_NAME_0_DID_YOU_MEAN_1
                } else {
                    tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1
                };
                let mut diagnostic = new_diagnostic_for_node(
                    Some(error_location),
                    message,
                    vec![declaration_name.clone(), suggestion_name.clone()],
                );
                if let Some(value_declaration) = &suggestion.value_declaration {
                    diagnostic.add_related_info(new_diagnostic_for_node(
                        Some(value_declaration),
                        tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                        vec![suggestion_name.clone()],
                    ));
                }
                self.add_error_or_suggestion(!is_unchecked_js, diagnostic);
                return;
            }
        }
        self.error_message(error_location, *name_not_found_message, &[declaration_name]);
    }

    pub fn on_successfully_resolved_symbol(
        &mut self,
        error_location: &Arc<Node>,
        result: &Arc<Symbol>,
        meaning: SymbolFlags,
        last_location: Option<&Arc<Node>>,
        associated_declaration_for_containing_initializer_or_binding_name: Option<&Arc<Node>>,
        within_deferred_context: bool,
    ) {
        let name = result.name.clone();
        let is_in_external_module = last_location.is_some_and(|location| {
            tsox_frontend::ast::is_source_file(location)
                && self
                    .get_source_file_of_node(location)
                    .is_some_and(|f| tsox_frontend::ast::is_external_or_common_js_module(&f))
        });
        if meaning.intersects(SymbolFlags::BlockScopedVariable)
            || meaning.intersects(SymbolFlags::Class | SymbolFlags::ENUM)
                && meaning == SymbolFlags::VALUE
        {
            let export_or_local_symbol =
                self.get_export_symbol_of_value_symbol_if_exported(result);
            if export_or_local_symbol.flags.intersects(
                SymbolFlags::BlockScopedVariable | SymbolFlags::Class | SymbolFlags::ENUM,
            ) {
                self.check_resolved_block_scoped_variable(&export_or_local_symbol, error_location);
            }
        }
        if is_in_external_module
            && meaning == SymbolFlags::VALUE
            && !error_location
                .flags
                .contains(tsox_frontend::ast::NodeFlags::JSDoc)
        {
            let merged = self.get_merged_symbol(result);
            if !merged.declarations.is_empty()
                && merged.declarations.iter().all(|d| {
                    tsox_frontend::ast::is_namespace_export_declaration(d)
                        || tsox_frontend::ast::is_source_file(d)
                            && d.as_source_file().global_exports.is_some()
                })
            {
                self.error_or_suggestion_message(
                    !self.compiler_options.allow_umd_global_access.is_true(),
                    error_location,tsox_core::diagnostics::messages_generated::X_0_REFERS_TO_A_UMD_GLOBAL_BUT_THE_CURRENT_FILE_IS_A_MODULE_CONSIDER_ADDING_AN_IMPORT_INSTEAD,
                    &[name.clone()],
                );
            }
        }
        if let Some(associated) = associated_declaration_for_containing_initializer_or_binding_name {
            if !within_deferred_context && meaning == SymbolFlags::VALUE {
                let late_bound = self.get_late_bound_symbol(result);
                let candidate = self.get_merged_symbol(&late_bound);
                let root = tsox_frontend::ast::get_root_declaration(associated);
                let associated_symbol = self.get_symbol_of_declaration(associated);
                if associated_symbol
                    .as_ref()
                    .is_some_and(|s| symbol_ptr_key(s) == symbol_ptr_key(&candidate))
                {
                    self.error_message(
                        error_location,tsox_core::diagnostics::messages_generated::PARAMETER_0_CANNOT_REFERENCE_ITSELF,
                        &[declaration_name_to_string(Some(&associated.name().unwrap()))],
                    );
                } else if let Some(candidate_value_declaration) = &candidate.value_declaration {
                    let root_parent_locals = root
                        .parent()
                        .and_then(|p| self.program.symbol_map().locals_of(&p).cloned());
                    if candidate_value_declaration.pos() > associated.pos()
                        && root_parent_locals.is_some_and(|locals| {
                            self.get_symbol(&locals, &candidate.name, meaning)
                                .as_ref()
                                .is_some_and(|s| symbol_ptr_key(s) == symbol_ptr_key(&candidate))
                        })
                    {
                        self.error_message(
                            error_location,tsox_core::diagnostics::messages_generated::PARAMETER_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_AFTER_IT,
                            &[
                                declaration_name_to_string(Some(&associated.name().unwrap())),
                                declaration_name_to_string(Some(error_location)),
                            ],
                        );
                    }
                }
            }
        }
        if meaning.intersects(SymbolFlags::VALUE)
            && result.flags.intersects(SymbolFlags::Alias)
            && !result.flags.intersects(SymbolFlags::VALUE)
            && !is_valid_type_only_alias_use_site(error_location)
        {
            let type_only_declaration =
                self.get_type_only_alias_declaration_ex(result, SymbolFlags::VALUE);
            if let Some(type_only_declaration) = type_only_declaration {
                let message =
                    if tsox_frontend::ast::node_kind_is(
                        &type_only_declaration,
                        &[
                            SyntaxKind::ExportSpecifier,
                            SyntaxKind::ExportDeclaration,
                            SyntaxKind::NamespaceExport,
                        ],
                    ) {
                        tsox_core::diagnostics::messages_generated::X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_EXPORTED_USING_EXPORT_TYPE
                    } else {
                        tsox_core::diagnostics::messages_generated::X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_IMPORTED_USING_IMPORT_TYPE
                    };
                let diagnostic = self.error_message(error_location, message, &[name.clone()]);
                if let Some(diagnostic) = diagnostic {
                    self.add_type_only_declaration_related_info(
                        Arc::try_unwrap(diagnostic).unwrap_or_else(|d| (*d).clone()),
                        Some(&type_only_declaration),
                        &name,
                    );
                }
            }
        }
        if self.compiler_options.isolated_modules.is_true()
            && is_in_external_module
            && meaning == SymbolFlags::VALUE
        {
            let global_symbol = self.get_symbol(&self.globals.clone(), &name, meaning);
            if global_symbol
                .as_ref()
                .is_some_and(|s| symbol_ptr_key(s) == symbol_ptr_key(result))
            {
                if let Some(last_location) = last_location {
                    if tsox_frontend::ast::is_source_file(last_location) {
                        let source_file_locals = self
                            .program
                            .symbol_map()
                            .locals_of(&last_location)
                            .cloned();
                        let non_value_symbol = source_file_locals
                            .and_then(|locals| self.get_symbol(&locals, &name, !SymbolFlags::VALUE));
                        if let Some(non_value_symbol) = non_value_symbol {
                            let import_decl = non_value_symbol.declarations.iter().find(|d| {
                                tsox_frontend::ast::node_kind_is(
                                    d,
                                    &[
                                        SyntaxKind::ImportSpecifier,
                                        SyntaxKind::ImportClause,
                                        SyntaxKind::NamespaceImport,
                                        SyntaxKind::ImportEqualsDeclaration,
                                    ],
                                )
                            });
                            if let Some(import_decl) = import_decl {
                                if !is_type_only_import_declaration(import_decl)
                                {
                                    self.error_message(
                                        import_decl,tsox_core::diagnostics::messages_generated::IMPORT_0_CONFLICTS_WITH_GLOBAL_VALUE_USED_IN_THIS_FILE_SO_MUST_BE_DECLARED_WITH_A_TYPE_ONLY_IMPORT_WHEN_ISOLATEDMODULES_IS_ENABLED,
                                        &[name.clone()],
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn pad_tuple_type(&mut self, t: &Arc<Type>, pattern: &Arc<Node>) -> Arc<Type> {
        let pattern_elements = &pattern.elements().unwrap().nodes;
        let target = t.target_tuple_type().unwrap();
        if target
            .combined_flags
            .intersects(ELEMENT_FLAGS_VARIABLE)
            || self.get_type_reference_arity(t) as usize >= pattern_elements.len()
        {
            return Arc::clone(t);
        }
        let mut element_types = self.get_element_types(t);
        let mut element_infos = target.element_infos.clone();
        let arity = self.get_type_reference_arity(t) as usize;
        for i in arity..pattern_elements.len() {
            let e = &pattern_elements[i];
            let is_last = i == pattern_elements.len() - 1;
            if !is_last || !(tsox_frontend::ast::is_binding_element(e) && has_dot_dot_dot_token(e)) {
                let any_type = self.any_type();
                let mut element_type = any_type;
                if !tsox_frontend::ast::is_omitted_expression(e) && self.has_default_value(e) {
                    element_type =
                        self.get_type_from_binding_element(e, false, false);
                }
                element_types.push(element_type);
                element_infos.push(TupleElementInfo {
                    label: None,
                    flags: ElementFlags::Optional,
                    labeled_declaration: None,
                    type_: None,
                });
                if !tsox_frontend::ast::is_omitted_expression(e) && !self.has_default_value(e) {
                    let any_type = self.any_type();
                    self.report_implicit_any(e, &any_type, WideningKind::Normal);
                }
            }
        }
        self.create_tuple_type_ex(element_types, element_infos, target.readonly)
    }

    pub fn parameter_initializer_contains_undefined(
        &mut self,
        declaration: &Arc<Node>,
    ) -> bool {
        let mut links_flags;
        {
            links_flags = self
                .node_links
                .get(declaration)
                .map(|l| l.flags)
                .unwrap_or(NodeCheckFlags::empty());
            if !links_flags
                .intersects(NodeCheckFlags::InitializerIsUndefinedComputed)
            {
                let declaration_key = Arc::as_ptr(declaration) as *const Symbol;
                if !self.push_type_resolution(
                    declaration_key,
                    TypeResolutionProperty::InitializerIsUndefined,
                ) {
                    if let Some(s) = self.get_symbol_of_declaration_opt(declaration) {
                        self.report_circularity_error(&s);
                    }
                    return true;
                }
                let initializer_type =
                    self.check_declaration_initializer(declaration, CheckMode::Normal, None);
                let contains_undefined =
                    self.has_type_facts(&initializer_type, TypeFacts::IS_UNDEFINED);
                if !self.pop_type_resolution() {
                    if let Some(s) = self.get_symbol_of_declaration_opt(declaration) {
                        self.report_circularity_error(&s);
                    }
                    return true;
                }
                if let Some(links) = self.node_links.get_mut(declaration) {
                    if !links
                        .flags
                        .intersects(NodeCheckFlags::InitializerIsUndefinedComputed)
                    {
                        links.flags |= NodeCheckFlags::InitializerIsUndefinedComputed;
                        if contains_undefined {
                            links.flags |= NodeCheckFlags::InitializerIsUndefined;
                        }
                    }
                    links_flags = links.flags;
                }
            }
        }
        links_flags.intersects(NodeCheckFlags::InitializerIsUndefined)
    }

    pub fn parse_big_int_literal_type(&mut self, text: &str) -> Arc<Type> {
        let value = tsox_core::jsnum::PseudoBigInt::parse(text);
        self.get_big_int_literal_type(value)
    }

    pub fn permissive_mapper_worker(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TypeFlags::TypeParameter) {
            return self.wildcard_type();
        }
        Arc::clone(t)
    }

    pub fn pick_longest_candidate_signature(
        &mut self,
        node: &Arc<Node>,
        candidates: &mut Vec<Arc<Signature>>,
        args: &[Arc<Node>],
        check_mode: CheckMode,
    ) -> Arc<Signature> {
        let mut arg_count = args.len();
        if let Some(apparent_argument_count) =
            crate::checker::mig::m2f_4::r28k4_defs::apparent_argument_count()
        {
            arg_count = apparent_argument_count;
        }
        let best_index = self.get_longest_candidate_index(candidates, arg_count as i32);
        let candidate = Arc::clone(&candidates[best_index as usize]);
        let type_parameters = candidate.type_parameters.clone();
        if type_parameters.is_empty() {
            return candidate;
        }
        let type_argument_nodes: Vec<Arc<Node>>;
        if self.call_like_expression_may_have_type_arguments(node) {
            type_argument_nodes = node
                .type_arguments()
                .map(|l| l.nodes.clone())
                .unwrap_or_default();
        } else {
            type_argument_nodes = Vec::new();
        }
        let instantiated = if !type_argument_nodes.is_empty() {
            let type_arguments =
                self.get_type_arguments_from_nodes(&type_argument_nodes, &type_parameters);
            self.create_signature_instantiation(&candidate, &type_arguments)
        } else {
            self.infer_signature_instantiation_for_overload_failure(
                node,
                &type_parameters,
                &candidate,
                args,
                check_mode,
            )
        };
        candidates[best_index as usize] = Arc::clone(&instantiated);
        instantiated
    }

    pub fn pop_active_mapper(&mut self) {
        self.active_mappers.pop();

        self.active_type_mappers_caches.pop();
    }

    pub fn pop_contextual_type(&mut self) {
        self.contextual_infos.pop();
    }

    pub fn pop_inference_context(&mut self) {
        self.inference_context_infos.pop();
    }

    pub fn produce_deferred_diagnostics(&mut self) {
        let callbacks = std::mem::take(&mut self.deferred_diagnostic_callbacks);
        for cb in callbacks {
            cb();
        }
    }

    pub fn propagate_optional_type_marker(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<Node>,
        was_optional: bool,
    ) -> Arc<Type> {
        if was_optional {
            if is_outermost_optional_chain(node) {
                return self.get_optional_type(Arc::clone(t));
            }
            return self.add_optional_type_marker(t);
        }
        Arc::clone(t)
    }

    pub fn push_active_mapper(&mut self, mapper: Arc<TypeMapper>) {
        self.active_mappers.push(mapper);

        self.active_type_mappers_caches
            .push(HashMap::new());
    }

    pub fn push_cached_contextual_type(&mut self, node: &Arc<Node>) {
        let t = self.get_contextual_type(node, ContextFlags::empty());
        self.contextual_infos.push(ContextualInfo {
            node: Some(Arc::clone(node)),
            t,
            is_cache: true,
        });
    }

    pub fn push_contextual_type(&mut self, node: &Arc<Node>, t: &Arc<Type>, is_cache: bool) {
        self.contextual_infos.push(ContextualInfo {
            node: Some(Arc::clone(node)),
            t: Some(Arc::clone(t)),
            is_cache,
        });
    }

    pub fn push_inference_context(&mut self, node: &Arc<Node>, context: &Arc<InferenceContext>) {
        self.inference_context_infos.push(InferenceContextInfo {
            node: Some(Arc::clone(node)),
            context: Some(Arc::clone(context)),
        });
    }

    pub fn recombine_unknown_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if Arc::ptr_eq(t, &self.unknown_union_type()) {
            return self.unknown_type();
        }
        Arc::clone(t)
    }


    pub fn record_potential_collision_with_weak_map_set_in_generated_code(
        &mut self,
        node: &Arc<Node>,
        name: Option<&Arc<Node>>,
    ) {
        if self.language_version <= ScriptTarget::ES2021
            && (self.need_collision_check_for_identifier(node, name, "WeakMap")
                || self.need_collision_check_for_identifier(node, name, "WeakSet"))
        {
            let node = Arc::clone(node);
            let checker_ptr =
                crate::checker::mig::m2f_4::r28k4_defs::SendCheckerPtr::from_checker(self);
            self.add_deferred_diagnostic(Box::new(move || {
                let c = unsafe { checker_ptr.get() };
                c.check_weak_map_set_collision(&node);
            }));
        }
    }

    pub fn register_for_unused_identifiers_check(&mut self, node: &Arc<Node>) {
        if let Some(source_file) = self.get_source_file_of_node(node) {
            let links = self.source_file_links.get_or_default(&source_file);
            links.identifier_check_nodes.push(Arc::clone(node));
        }
    }
}

pub struct TupleNormalizer {
    pub types: Vec<Arc<Type>>,
    pub infos: Vec<TupleElementInfo>,
    pub last_required_index: isize,
    pub first_rest_index: isize,
    pub last_optional_or_rest_index: isize,
}

impl TupleNormalizer {
    pub fn normalize(
        &mut self,
        c: &mut Checker,
        element_types: &[Arc<Type>],
        element_infos: &[TupleElementInfo],
    ) -> bool {
        self.last_required_index = -1;
        self.first_rest_index = -1;
        self.last_optional_or_rest_index = -1;
        for (i, t) in element_types.iter().enumerate() {
            let info = &element_infos[i];
            if info.flags.intersects(ElementFlags::Variadic) {
                if t.flags.intersects(TypeFlags::Any) {
                    self.add(
                        Arc::clone(t),
                        TupleElementInfo {
                            label: None,
                            flags: ElementFlags::Rest,
                            labeled_declaration: info.labeled_declaration.clone(),
                            type_: None,
                        },
                    );
                } else if t.flags.intersects(TypeFlags::INSTANTIABLE_NON_PRIMITIVE)
                    || c.is_generic_mapped_type(t)
                {
                    self.add(Arc::clone(t), info.clone());
                } else if is_tuple_type(t) {
                    let spread_types = c.get_element_types(t);
                    if spread_types.len() + self.types.len() >= 10_000 {
                        let message =
                            if is_part_of_type_node(&c.current_node.clone().unwrap()) {
                                tsox_core::diagnostics::messages_generated::TYPE_PRODUCES_A_TUPLE_TYPE_THAT_IS_TOO_LARGE_TO_REPRESENT
                            } else {
                                tsox_core::diagnostics::messages_generated::EXPRESSION_PRODUCES_A_TUPLE_TYPE_THAT_IS_TOO_LARGE_TO_REPRESENT
                            };
                        c.error_message(&c.current_node.clone().unwrap(), message, &[]);
                        return false;
                    }
                    let spread_infos = t.target_tuple_type().unwrap().element_infos.clone();
                    for (j, s) in spread_types.iter().enumerate() {
                        self.add(Arc::clone(s), spread_infos[j].clone());
                    }
                } else {
                    let mut s: Option<Arc<Type>> = None;
                    if c.is_array_like_type(t) {
                        s = c.get_index_type_of_type(
                        t,
                        crate::checker::services_checker_7::IndexKind::Number,
                    );
                    }
                    let s = s.unwrap_or_else(|| c.error_type());
                    self.add(
                        s,
                        TupleElementInfo {
                            label: None,
                            flags: ElementFlags::Rest,
                            labeled_declaration: info.labeled_declaration.clone(),
                            type_: None,
                        },
                    );
                }
            } else {
                self.add(Arc::clone(t), info.clone());
            }
        }
        for i in 0..self.last_required_index.max(0) {
            if self.infos[i as usize].flags.intersects(ElementFlags::Optional) {
                self.infos[i as usize].flags = ElementFlags::Required;
            }
        }
        if self.first_rest_index >= 0 && self.first_rest_index < self.last_optional_or_rest_index {
            let mut types: Vec<Arc<Type>> = Vec::new();
            for i in self.first_rest_index..=self.last_optional_or_rest_index {
                let mut t = Arc::clone(&self.types[i as usize]);
                if self.infos[i as usize].flags.intersects(ElementFlags::Variadic) {
                    let number_type = c.number_type();
                    t = c.get_indexed_access_type(&t, &number_type);
                }
                types.push(t);
            }
            let union = c.get_union_type(types);
            self.types[self.first_rest_index as usize] = union;
            let rest_start = (self.first_rest_index + 1) as usize;
            let rest_end = (self.last_optional_or_rest_index + 1) as usize;
            self.types.drain(rest_start..rest_end);
            self.infos.drain(rest_start..rest_end);
        }
        true
    }

    fn add(&mut self, t: Arc<Type>, info: TupleElementInfo) {
        if info.flags.intersects(ElementFlags::Required) {
            self.last_required_index = self.types.len() as isize;
        }
        if info.flags.intersects(ElementFlags::Rest) && self.first_rest_index < 0 {
            self.first_rest_index = self.types.len() as isize;
        }
        if info.flags.intersects(ElementFlags::Optional | ElementFlags::Rest) {
            self.last_optional_or_rest_index = self.types.len() as isize;
        }
        self.types.push(t);
        self.infos.push(info);
    }
}

pub struct ObjectLiteralDiscriminator<'a> {
    pub c: &'a mut Checker,
    pub props: Vec<Arc<Node>>,
    pub members: Vec<Arc<Symbol>>,
}

impl<'a> ObjectLiteralDiscriminator<'a> {
    pub fn len(&self) -> usize {
        self.props.len() + self.members.len()
    }

    pub fn name(&mut self, index: usize) -> String {
        if index < self.props.len() {
            let prop_symbol = self
                .c
                .get_symbol_of_node(&self.props[index])
                .expect("property symbol");
            return prop_symbol.name.clone();
        }
        self.members[index - self.props.len()].name.clone()
    }
}
