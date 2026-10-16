//! 常驻 worker 池化逐例重置:清空跨例残留的全局与 thread_local 状态。
//!
//! 不触碰 NEXT_NODE_ID / NEXT_SYMBOL_ID / NEXT_TYPE_ID 等全局 id 计数器,
//! 也不清 lib AST 缓存(池化收益来源)。可在没有存活 Checker 时安全调用。

pub fn reset_per_case_state() {
    // 全局单例(Pattern 1:Mutex<Option<Arc<Type>>>)
    crate::checker::mig::m1a::r28k2_defs::reset_optional_type();
    crate::checker::mig::m1f::r24k9_defs::reset_resolving_default_type();

    // 全局缓存(Pattern 2:OnceLock 外壳保留,清内部容器)
    crate::checker::mig::m3a_2::r31k1_defs::reset_source_files_by_root_id();

    // 裸指针与 JSX 链接
    crate::checker::mig::m2c_5::r26k4_defs::reset_builder_checker();
    crate::checker::mig::m2c_5::r26k4_defs::reset_jsx_links();

    // thread_local:栈 / 标志 / 缓存
    crate::checker::mig::m2h::r21k10_defs::reset_relation_comparison_state();
    crate::checker::mig::m2e::r19k3_defs::reset_resolving_explicit_type_of_symbol();
    crate::checker::mig::w6_relater::reset_variance_stack_entries();
    crate::checker::mig::w6_flow::reset_flow_scratch_state();
    crate::checker::mig::m2f_3::r24k13_defs::reset_nodebuilder_scratch_state();
    crate::checker::mig::m2b::r23k6_defs::reset_call_state_candidates();
    crate::checker::mig::m1b::r23k1_defs::reset_contextual_binding_patterns_r23k1();
    crate::checker::mig::m1g::r24k16_defs::reset_contextual_binding_patterns_r24k16();
    crate::checker::mig::m2f_4::r28k4_defs::reset_apparent_argument_count();
    crate::checker::mig::wc2_3::r29k1_defs::reset_object_type_flow_caches();
    crate::checker::mig::m1f::r25k6_defs::reset_alias_type_instantiations();
    crate::checker::mig::wc1b::r25k3_defs::reset_conditional_instantiations();
    crate::checker::mig::m2b::r22k6_defs::reset_pattern_ambient_state();
    crate::checker::mig::m2g::r22k9_defs::reset_nodebuilder_side_state();
    crate::checker::mig::m2h::r22k10_defs::reset_node_flow_nodes();
    crate::checker::mig::wc3_6::r24k4_defs::reset_non_existent_properties();
    crate::checker::mig::m2c::r24k11_defs::reset_pattern_ambient_augmentations();
    crate::checker::mig::wc1c::r24k17_defs::reset_signature_composite_caches();
    crate::checker::mig::m2e_2::r24k19_defs::reset_intra_expression_inference_sites();
    crate::checker::mig::m1b::r25k1_defs::reset_source_file_deferred_nodes();
    crate::checker::mig::m1g_5::r25k10_defs::reset_module_import_attributes_types();
    crate::checker::mig::m1g_4::r26k6_defs::reset_unresolved_symbols();
    crate::checker::mig::w9a::r28k8_defs::reset_packages_maps();
    crate::checker::mig::m1e::r28k9_defs::reset_iteration_types_cache();

    // 清单外补充:枚举字面量类型缓存(键含裸地址,跨例复用会串值)
    crate::checker::mig::m1e::r26k3_defs::reset_enum_literal_type_caches();

    // 清单外补充:binder 侧栈与标志
    crate::binder::mig::m3h::reset_last_container();
    crate::binder::mig::m3h_2::reset_pre_switch_case_flow();
    crate::binder::mig::m3h_3::reset_binder_emit_flags();
}
