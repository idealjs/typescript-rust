// 路径契约:基线与语料的相对路径(tests/corpus/…)以 tsox crate 根为 CWD,
// libtest 入口由 cargo 保证;其他入口(xtask worker)须自行设置 current_dir。
pub mod alloc;
pub mod baseline;
pub mod case_index;
pub mod case_parser;
pub mod case_run;
pub mod compiler_run;
pub mod config_expand;
pub mod errors_render;
pub mod program_setup;
pub mod probe;
pub mod skip_rules;
pub mod types_baseline;
pub mod worker;
