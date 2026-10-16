//! 常驻 worker 池化逐例重置:清空跨例残留的全局缓存与 parser thread_local 状态。
//!
//! 不触碰 SOURCE_FILE_DATA_KEY_COUNTER 等全局 id 计数器。

pub fn reset_per_case_state() {
    crate::ast::mig::m3d_4::reset_source_file_data_cells();
    crate::ast::node_source_file::reset_bind_diagnostics_store();
    crate::parser::mig::m4d::reset_parser_reparsed_clones();
}
