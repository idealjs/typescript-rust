#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;

impl Checker {
    pub fn new(program: Arc<dyn Program>, tracer: Arc<Tracer>) -> Self { ::tsox_core::fntrace::enter("new"); 
        let compiler_options = Arc::new(program.options().clone());
        let files = program.source_files().to_vec();

        let language_version = compiler_options.get_emit_script_target();
        let module_kind = compiler_options.get_emit_module_kind();
        let module_resolution_kind = compiler_options.get_module_resolution_kind();

        let legacy_decorators = compiler_options.experimental_decorators.is_true();
        let emit_standard_class_fields = compiler_options.get_emit_standard_class_fields();
        let allow_unreachable_code = compiler_options.allow_unreachable_code;
        let allow_unused_labels = compiler_options.allow_unused_labels;
        let strict_null_checks =
            compiler_options.get_strict_option_value(compiler_options.strict_null_checks);
        let strict_function_types =
            compiler_options.get_strict_option_value(compiler_options.strict_function_types);
        let strict_bind_call_apply =
            compiler_options.get_strict_option_value(compiler_options.strict_bind_call_apply);
        let strict_property_initialization = compiler_options
            .get_strict_option_value(compiler_options.strict_property_initialization);
        let strict_builtin_iterator_return = compiler_options
            .get_strict_option_value(compiler_options.strict_builtin_iterator_return);
        let no_implicit_any =
            compiler_options.get_strict_option_value(compiler_options.no_implicit_any);
        let no_implicit_this =
            compiler_options.get_strict_option_value(compiler_options.no_implicit_this);
        let use_unknown_in_catch_variables = compiler_options
            .get_strict_option_value(compiler_options.use_unknown_in_catch_variables);
        let exact_optional_property_types =
            compiler_options.exact_optional_property_types.is_true();
        let no_unchecked_indexed_access = compiler_options.no_unchecked_indexed_access.is_true();
        let can_collect_symbol_alias_accessibility_data = compiler_options
            .verbatim_module_syntax
            .is_false_or_unknown();

        let mut file_index_map = HashMap::new();
        for (i, file) in files.iter().enumerate() {
            file_index_map.insert(file.id(), i);
        }
        crate::checker::mig::m3a_2::r31k1_defs::register_diagnostic_source_files(&files);

        let intrinsic = |flags: crate::checker::types::TypeFlags, name: &str| {
            Arc::new(crate::checker::types::Type::new(
                flags,
                crate::checker::types::TypeData::Intrinsic(crate::checker::types::IntrinsicTypeData {
                    intrinsic_name: name.to_string(),
                }),
            ))
        };
        let regular_literal = |value: bool| {
            Arc::new(crate::checker::types::Type::new(
                crate::checker::types::TypeFlags::BooleanLiteral,
                crate::checker::types::TypeData::Literal(crate::checker::types::LiteralTypeData {
                    value: crate::checker::types::LiteralValue::Boolean(value),
                    fresh_type: OnceLock::new(),
                    regular_type: OnceLock::new(),
                }),
            ))
        };

        let mut checker = Self {
            id: NEXT_CHECKER_ID.fetch_add(1, Ordering::Relaxed),
            program,
            compiler_options,
            files,
            file_index_map,

            type_count: 0,
            symbol_count: 0,
            signature_count: 0,
            total_instantiation_count: 0,
            instantiation_count: 0,
            instantiation_depth: 0,
            imported_type_resolution: Vec::new(),
            alias_type_resolution_stack: Vec::new(),
            cs_echo_inference: false,
            active_inferential_contextual: None,
            jsx_attr_ctx_guard: None,
            discriminated_contextual_types: std::collections::HashMap::new(),

            language_version,
            module_kind,
            module_resolution_kind,
            legacy_decorators,
            emit_standard_class_fields,
            strict_null_checks,
            allow_unreachable_code,
            strict_function_types,
            strict_bind_call_apply,
            strict_property_initialization,
            strict_builtin_iterator_return,
            no_implicit_any,
            no_implicit_this,
            use_unknown_in_catch_variables,
            exact_optional_property_types,
            no_unchecked_indexed_access,
            can_collect_symbol_alias_accessibility_data,

            globals: SymbolTable::default(),
            undefined_symbol: Some(Arc::new(Symbol::new(SymbolFlags::Property, "undefined"))),
            arguments_symbol: Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "arguments",
            ))),
            require_symbol: Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "require",
            ))),
            unknown_symbol: None,
            global_this_symbol: None,

            string_literal_types: HashMap::new(),
            number_literal_types: HashMap::new(),
            bigint_literal_types: HashMap::new(),
            unique_es_symbol_types: HashMap::new(),
            nan_type: None,

            indexed_access_types: HashMap::new(),
            template_literal_types: HashMap::new(),
            string_mapping_types: HashMap::new(),
            intrinsic_marker_type: OnceLock::new(),
            marker_super_type: OnceLock::new(),
            marker_sub_type: OnceLock::new(),
            marker_other_type: OnceLock::new(),
            marker_types: std::collections::HashSet::new(),
            variance_stack: Vec::new(),
            reliability_flags: 0,
            cached_types: HashMap::new(),
            union_types: HashMap::new(),
            intersection_types: HashMap::new(),
            tuple_types: HashMap::new(),
            error_types: HashMap::new(),

            global_interface_members: HashMap::new(),
            boxed_global_types: HashMap::new(),

            diagnostics: DiagnosticsCollection::default(),
            suggestion_diagnostics: DiagnosticsCollection::default(),

            node_links: LinkStore::new(),
            signature_links: LinkStore::new(),
            symbol_node_links: LinkStore::new(),
            type_node_links: LinkStore::new(),
            enum_member_links: LinkStore::new(),
            assertion_links: LinkStore::new(),
            array_literal_links: LinkStore::new(),
            switch_statement_links: LinkStore::new(),
            jsx_element_links: LinkStore::new(),

            symbol_reference_links: LinkStore::new(),
            value_symbol_links: LinkStore::new(),
            mapped_symbol_links: LinkStore::new(),
            deferred_symbol_links: LinkStore::new(),
            alias_symbol_links: LinkStore::new(),
            module_symbol_links: LinkStore::new(),
            late_bound_links: LinkStore::new(),
            export_type_links: LinkStore::new(),
            members_and_exports_links: LinkStore::new(),
            type_alias_links: LinkStore::new(),
            declared_type_links: LinkStore::new(),
            class_instance_type_cache: HashMap::new(),
            base_ctor_type_cache: HashMap::new(),
            this_type_cache: HashMap::new(),
            type_resolution_stack: Vec::new(),
            variable_type_frame_depth: 0,
            signature_return_resolutions: Vec::new(),
            partial_fn_type_builds: std::collections::HashSet::new(),
            rt_infer_boundary_marks: Vec::new(),
            call_return_query_depth: 0,
            callee_resolution_depth: 0,
            in_flight_object_literal_types: HashMap::new(),
            type_argument_stack: Vec::new(),
            erase_signature_strict: false,
            type_argument_name_frames: Vec::new(),
            type_node_subst_cache: HashMap::new(),
            type_node_resolving: HashSet::new(),
            type_resolution_depth: 0,
            heritage_degraded_events: 0,
            type_node_query_epochs: Vec::new(),
            heritage_retry_counts: HashMap::new(),
            type_node_subst_cache_limit: 300_000,
            speculation_depth: 0,
            type_parameter_resolving: HashSet::new(),
            ts2313_reported: HashSet::new(),
            ts2354_checked_files: HashSet::new(),
            requested_external_emit_helpers: HashMap::new(),
            degraded_type_ptrs: std::collections::HashSet::new(),
            jsx_implicit_namespace: HashMap::new(),
            pending_jsx_2875: None,
            relater_error_chain: Vec::new(),
            relater_excess_error_node: None,
            relater_error_node: None,
            relater_chain_active: false,
            relater_no_common_self_reports: 0,
            relater_pending_primitive_source: false,
            property_lookup_skips_index_synthesis: false,
            relater_depth: 0,
            deferred_constraint_depth: 0,
            deferred_conditional_root_stack: Vec::new(),
            relation_count: 0,
            relater_overflow: false,
            new_call_fallback_signature: false,
            relater_intersection_target_depth: 0,
            relation_reference_canon: HashMap::new(),
            subst_object_in_progress: std::collections::HashMap::new(),
            subst_reference_shell_cache: std::collections::HashMap::new(),
            in_return_substitution: false,
            relater_source_stack: Vec::new(),
            relater_target_stack: Vec::new(),
            relation_cache: HashMap::new(),
            probe_cache_permissive: HashMap::new(),
            probe_cache_restrictive: HashMap::new(),
            enum_relation: HashMap::new(),
            relation_in_progress: std::collections::HashSet::new(),
            relation_maybe_keys: Vec::new(),
            relation_maybe_key_set: std::collections::HashSet::new(),
            relater_bail_maybe: false,
            interface_extends_reported: std::collections::HashSet::new(),
            interface_simultaneous_reported: std::collections::HashSet::new(),
            indexed_access_2538_reported: std::collections::HashSet::new(),
            arith_operand_error_nodes: std::collections::HashSet::new(),
            computed_property_name_checked: std::collections::HashSet::new(),
            symbol_reference_kinds: dashmap::DashMap::new(),
            spread_links: LinkStore::new(),
            variance_links: LinkStore::new(),
            reverse_mapped_symbol_links: LinkStore::new(),
            reverse_mapped_cache: HashMap::new(),
            pending_annotated_param_inferences: Vec::new(),
            inference_loop_depth: 0,
            generic_index_type_cache: HashMap::new(),
            could_contain_type_variables_cache: HashMap::new(),
            primitive_apparent_types: HashMap::new(),
            template_resolving_ids: std::collections::HashSet::new(),
            mapped_shell_resolving: std::collections::HashSet::new(),
            template_resolution_letway: false,
            deferred_indexed_access_cache: HashMap::new(),
            index_type_cache: HashMap::new(),
            interface_shell_reify_cache: HashMap::new(),
            reverse_mapped_print_stack: Vec::new(),
            alias_args_resolution_stack: Vec::new(),
            alias_type_instantiation_stack: Vec::new(),
            infer_subst_ancestor_stack: Vec::new(),
            infer_subst_memo: Vec::new(),
            alias_instantiation_cache: HashMap::new(),
            current_alias_frame: None,
            reverse_mapped_depth: Vec::new(),
            reverse_mapped_target_depth: Vec::new(),
            marked_assignment_symbol_links: LinkStore::new(),
            symbol_container_links: LinkStore::new(),
            symbol_table_alias_cache: HashMap::new(),
            class_expression_name_tables: HashMap::new(),
            source_file_links: LinkStore::new(),
            declaration_links: LinkStore::new(),
            declaration_file_links: LinkStore::new(),
            last_combined_modifier_flags_node: None,
            last_combined_modifier_flags_result: ModifierFlags::empty(),

            any_type: OnceLock::new(),
            unknown_type: OnceLock::new(),
            undefined_type: OnceLock::new(),
            null_type: OnceLock::new(),
            string_type: OnceLock::new(),
            number_type: OnceLock::new(),
            bigint_type: OnceLock::new(),
            boolean_type: OnceLock::new(),
            es_symbol_type: OnceLock::new(),
            void_type: OnceLock::new(),
            never_type: OnceLock::new(),
            silent_never_type: OnceLock::new(),
            non_primitive_type: OnceLock::new(),
            true_type: OnceLock::new(),
            false_type: OnceLock::new(),
            error_type: OnceLock::new(),
            unresolved_type: OnceLock::new(),

            auto_type: OnceLock::new(),

            empty_object_type: OnceLock::new(),
            empty_generic_type: OnceLock::new(),
            any_function_type: OnceLock::new(),
            no_constraint_type: OnceLock::new(),
            circular_constraint_type: OnceLock::new(),

            any_array_type: OnceLock::new(),
            auto_array_type: OnceLock::new(),
            any_readonly_array_type: OnceLock::new(),

            global_object_type: OnceLock::new(),
            global_function_type: OnceLock::new(),
            global_array_type: OnceLock::new(),
            global_readonly_array_type: OnceLock::new(),
            global_string_type: OnceLock::new(),
            global_number_type: OnceLock::new(),
            global_boolean_type: OnceLock::new(),
            global_bigint_type: OnceLock::new(),
            global_reg_exp_type: OnceLock::new(),
            typeof_type: OnceLock::new(),
            global_this_type: OnceLock::new(),
            global_promise_type: OnceLock::new(),
            array_type_cache: std::collections::HashMap::new(),
            interface_instantiation_cache: std::collections::HashMap::new(),
            merged_ns_instance_type_cache: std::collections::HashMap::new(),
            pending_interface_shells: std::collections::HashMap::new(),
            pending_arg_shells: std::collections::HashMap::new(),
            arg_shell_seq: 0,
            arg_shell_resolves: 0,
            allow_unused_labels,
            within_unreachable_code: false,
            interface_build_depth: 0,
            reported_unreachable_nodes: std::collections::HashSet::new(),
            this_location_errors_reported: std::collections::HashSet::new(),
            namespace_value_suppressed_nodes: std::collections::HashSet::new(),
            typequery_instantiation_cache: std::collections::HashMap::new(),
            fn_typequery_shells: std::collections::HashMap::new(),
            fn_typequery_shell_ptrs: std::collections::HashSet::new(),
            attached_type_args_cache: std::collections::HashMap::new(),
            filling_class_members: std::collections::HashSet::new(),
            class_build_in_progress: std::collections::HashSet::new(),
            pending_base_merges: Vec::new(),
            array_type_parameter_symbols: None,
            array_member_type_cache: std::collections::HashMap::new(),
            array_type_intern_cache: std::collections::HashMap::new(),
            intersection_intern_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache_limit: 300_000,
            instantiated_member_owner: std::collections::HashMap::new(),

            any_signature: OnceLock::new(),
            unknown_signature: OnceLock::new(),
            resolving_signature: OnceLock::new(),

            current_node: None,
            inline_level: 0,
            serialization_level: 0,
            type_print_stack: Vec::new(),
            display_approximate_length: 0,
            display_truncating: false,
            current_file: None,
            current_file_id: 0,
            display_enclosing_file: None,
            module_display_specifiers: std::collections::HashMap::new(),
            display_enclosing_node: None,
            current_file_symbol: None,
            scope_stack: Vec::new(),
            function_scope_count: 0,
            arrow_function_scope_count: 0,
            globals_populated: false,
            break_continue_context_stack: Vec::new(),
            this_container_stack: Vec::new(),
            ambient_context_depth: 0,
            ambient_statement_reported: std::collections::HashSet::new(),
            namespace_value_depth: 0,
            accessor_pair_return_hint: None,
            this_type_stack: Vec::new(),
            display_target_override: None,
            enclosing_class_stack: Vec::new(),
            call_arg_arrow_context: Vec::new(),
            resolving_type_aliases: std::collections::HashSet::new(),
            alias_resolution_stack: Vec::new(),
            alias_circular_frames: std::collections::HashSet::new(),
            alias_circular_reported: std::collections::HashSet::new(),
            resolving_function_like: std::collections::HashSet::new(),
            class_statics_resolution_stack: Vec::new(),
            class_type_resolution_stack: Vec::new(),
            inference_constraint_in_flight: Vec::new(),
            resolving_contextual_calls: std::collections::HashSet::new(),
            logical_rhs_narrowing_frames: Vec::new(),
            in_ctor_body_stack: Vec::new(),
            return_type_stack: Vec::new(),

            flow_analysis_disabled: false,
            definite_assignment_check_depth: 0,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            union_or_intersection_property_cache: HashMap::new(),
            type_instantiation_count: 0,
            type_instantiation_limit_reported: false,
            flow_node_reachable: HashMap::new(),
            switch_exhaustive_state: HashMap::new(),
            flow_inline_level: 0,
            in_static_member_type: false,
            suppress_cannot_find_name_in_type_nodes: 0,
            suppress_source_file: None,

            narrowable_reference_query_stack: Vec::new(),
            binding_pattern_narrowing_stack: Vec::new(),
            merged_symbols: HashMap::new(),
            merged_symbol_targets: HashMap::new(),

            awaited_type_stack: Vec::new(),
            flow_loop_stack: Vec::new(),
            flow_loop_cache: HashMap::new(),
            cached_signatures: HashMap::new(),
            substitution_types: HashMap::new(),
            undefined_properties: HashMap::new(),
            this_expando_kinds: HashMap::new(),
            this_expando_locations: HashMap::new(),
            conditional_constraint_depth: 0,
            contextual_infos: Vec::new(),
            inference_context_infos: Vec::new(),
            contextual_arg_types: HashMap::new(),
            active_mappers: Vec::new(),
            active_type_mappers_caches: Vec::new(),
            deferred_diagnostic_callbacks: Vec::new(),
            type_resolutions: Vec::new(),
            resolution_start: 0,
            factory: tsox_frontend::ast::mig::m3c::NodeFactory {
                hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
                text_count: 0,
                node_count: 0,
            },
            wildcard_type: intrinsic(crate::checker::types::TypeFlags::Any, "any"),
            blocked_string_type: intrinsic(crate::checker::types::TypeFlags::Any, "any"),
            missing_type: intrinsic(crate::checker::types::TypeFlags::Undefined, "undefined"),
            unique_literal_type: intrinsic(crate::checker::types::TypeFlags::Never, "never"),
            unreachable_never_type: intrinsic(crate::checker::types::TypeFlags::Never, "never"),
            regular_false_type: regular_literal(false),
            regular_true_type: regular_literal(true),
            undefined_widening_type: intrinsic(crate::checker::types::TypeFlags::Undefined, "undefined"),
            null_widening_type: intrinsic(crate::checker::types::TypeFlags::Null, "null"),
            number_or_big_int_type: intrinsic(crate::checker::types::TypeFlags::Number, "number"),
            string_number_symbol_type: intrinsic(crate::checker::types::TypeFlags::String, "string"),

            marker_sub_type_for_check: None,
            marker_super_type_for_check: None,
            variance_type_parameter: None,

            tracer,
            packages_map: None,
            mu: Mutex::new(()),
        };

        {
            let mut global_this = Symbol::new(SymbolFlags::ValueModule, "globalThis");
            global_this.check_flags = CheckFlags::Readonly;
            let global_this = Arc::new(global_this);
            checker
                .globals
                .insert("globalThis".to_string(), Arc::clone(&global_this));
            checker.global_this_symbol = Some(global_this);

        }

        checker.undefined_widening_type = checker.nullish_widening_type(checker.undefined_type());
        checker.null_widening_type = checker.nullish_widening_type(checker.null_type());
        checker.number_or_big_int_type =
            checker.get_union_type(vec![checker.number_type(), checker.bigint_type()]);
        checker.string_number_symbol_type = checker.get_union_type(vec![
            checker.string_type(),
            checker.number_type(),
            checker.es_symbol_type(),
        ]);

        checker.unknown_symbol = Some(Arc::new(Symbol::new(
            SymbolFlags::Property,
            "unknown",
        )));
        let any_type = checker.any_type();
        let error_type = checker.error_type();
        let any_signature = checker.new_signature(
            SignatureFlags::None,
            None,
            &[],
            None,
            &[],
            &any_type,
            None,
            0,
        );
        let _ = checker.any_signature.set(any_signature);
        let unknown_signature = checker.new_signature(
            SignatureFlags::None,
            None,
            &[],
            None,
            &[],
            &error_type,
            None,
            0,
        );
        let _ = checker.unknown_signature.set(unknown_signature);
        let resolving_signature = checker.new_signature(
            SignatureFlags::None,
            None,
            &[],
            None,
            &[],
            &any_type,
            None,
            0,
        );
        let _ = checker.resolving_signature.set(resolving_signature);
        checker.initialize_closures();
        checker.initialize_iteration_resolvers();
        checker.populate_globals();
        checker.globals_populated = true;
        checker.initialize_checker();

        checker
    }
}
