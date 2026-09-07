#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn new(program: Arc<dyn Program>, tracer: Arc<Tracer>) -> Self {
        let compiler_options = Arc::new(program.options().clone());
        let files = program.source_files().to_vec();

        let language_version = compiler_options.get_emit_script_target();
        let module_kind = compiler_options.get_emit_module_kind();
        let module_resolution_kind = compiler_options.get_module_resolution_kind();

        let legacy_decorators = compiler_options.experimental_decorators.is_true();
        let emit_standard_class_fields = compiler_options.get_emit_standard_class_fields();
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
        let can_collect_symbol_alias_accessibility_data = compiler_options
            .verbatim_module_syntax
            .is_false_or_unknown();

        let mut file_index_map = HashMap::new();
        for (i, file) in files.iter().enumerate() {
            file_index_map.insert(file.id(), i);
        }

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

            language_version,
            module_kind,
            module_resolution_kind,
            legacy_decorators,
            emit_standard_class_fields,
            strict_null_checks,
            strict_function_types,
            strict_bind_call_apply,
            strict_property_initialization,
            strict_builtin_iterator_return,
            no_implicit_any,
            no_implicit_this,
            use_unknown_in_catch_variables,
            exact_optional_property_types,
            can_collect_symbol_alias_accessibility_data,

            globals: SymbolTable::default(),
            undefined_symbol: Some(Arc::new(Symbol::new(SymbolFlags::Property, "undefined"))),
            arguments_symbol: Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "arguments",
            ))),
            require_symbol: None,
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
            type_resolution_stack: Vec::new(),
            type_argument_stack: Vec::new(),
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
            relater_chain_active: false,
            relater_depth: 0,
            deferred_constraint_depth: 0,
            relation_count: 0,
            relater_overflow: false,
            relater_intersection_target_depth: 0,
            subst_object_in_progress: std::collections::HashMap::new(),
            in_return_substitution: false,
            relater_source_stack: Vec::new(),
            relater_target_stack: Vec::new(),
            relation_cache: HashMap::new(),
            probe_cache_permissive: HashMap::new(),
            probe_cache_restrictive: HashMap::new(),
            enum_relation: HashMap::new(),
            relation_in_progress: std::collections::HashSet::new(),
            interface_extends_reported: std::collections::HashSet::new(),
            indexed_access_2538_reported: std::collections::HashSet::new(),
            arith_operand_error_nodes: std::collections::HashSet::new(),
            computed_property_name_checked: std::collections::HashSet::new(),
            symbol_reference_kinds: dashmap::DashMap::new(),
            spread_links: LinkStore::new(),
            variance_links: LinkStore::new(),
            reverse_mapped_symbol_links: LinkStore::new(),
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
            global_reg_exp_type: OnceLock::new(),
            global_this_type: OnceLock::new(),
            global_promise_type: OnceLock::new(),
            array_type_cache: std::collections::HashMap::new(),
            interface_instantiation_cache: std::collections::HashMap::new(),
            typequery_instantiation_cache: std::collections::HashMap::new(),
            attached_type_args_cache: std::collections::HashMap::new(),
            array_type_parameter_symbols: None,
            array_member_type_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache_limit: 300_000,

            any_signature: OnceLock::new(),
            unknown_signature: OnceLock::new(),
            resolving_signature: OnceLock::new(),

            current_node: None,
            inline_level: 0,
            serialization_level: 0,
            type_print_stack: Vec::new(),
            current_file: None,
            current_file_id: 0,
            current_file_symbol: None,
            scope_stack: Vec::new(),
            function_scope_count: 0,
            arrow_function_scope_count: 0,
            globals_populated: false,
            break_continue_context_stack: Vec::new(),
            this_container_stack: Vec::new(),
            ambient_context_depth: 0,
            ambient_ts1036_reported_blocks: std::collections::HashSet::new(),
            namespace_value_depth: 0,
            accessor_pair_return_hint: None,
            this_type_stack: Vec::new(),
            display_target_override: None,
            enclosing_class_stack: Vec::new(),
            call_arg_arrow_context: Vec::new(),
            resolving_type_aliases: std::collections::HashSet::new(),
            resolving_function_like: std::collections::HashSet::new(),
            class_statics_resolution_stack: Vec::new(),
            class_type_resolution_stack: Vec::new(),
            resolving_contextual_calls: std::collections::HashSet::new(),
            logical_rhs_narrowing_frames: Vec::new(),
            in_ctor_body_stack: Vec::new(),
            return_type_stack: Vec::new(),

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            type_instantiation_count: 0,
            type_instantiation_limit_reported: false,
            flow_node_reachable: HashMap::new(),
            flow_inline_level: 0,
            in_static_member_type: false,
            suppress_cannot_find_name_in_type_nodes: 0,
            suppress_source_file: None,

            merged_symbols: HashMap::new(),

            tracer,
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

            if let Some(ref undef) = checker.undefined_symbol {
                checker
                    .globals
                    .insert("undefined".to_string(), Arc::clone(undef));
            }
        }

        checker
    }
}
