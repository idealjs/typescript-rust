//! 常驻 worker 池化逐例重置:清空跨例残留的全局缓存。
//!
//! 不清 cached_parse 的 lib AST 缓存(池化收益来源)。

pub fn reset_per_case_state() {
    crate::mig::m4w::reset_package_json_info_cache();
    crate::mig::m3l_cm_2::reset_content_mapper_source_file_info_registry();
}
