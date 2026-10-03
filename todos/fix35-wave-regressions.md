# fix35 波收尾暴露的 workspace 级回归（待办）

修复波 fix35 整合后最终全量（corpus 门全绿：修好 24 例、零新增 FAIL）在 workspace
测试面暴露 3 例回归。三者在基线（ts2rust-port 9f26794db8）复跑均通过，属波内引入，
corpus 看不到该面。已裁决修掉 tsoptions 两测（092b681405），以下三例**只归因未修**。

## 1. node10 降级诊断丢失

- 测试：`tsox --lib` convergence_tests::checker_regression_fix_tests::node10_program_reports_deprecation_and_alternate_result（断言处 checker_regression_fix_tests.rs:242，实收空 `[]`）
- 嫌疑 commit：fix35/w10 的 `c3af41a3a0`（Program::new 移除手写选项校验块）+ `bed6358c0a`（接线 verify_compiler_options）——旧手写块内含 node10 降级诊断，替换后丢失
- 裁决入口：对照 Go NewProgram/verifyCompilerOptions 路径中 node10 的 deprecation 发射点补进 verify_compiler_options

## 2. ESM 文件误报 1470

- 测试：`tsox --lib` convergence_tests::checker_node_format_tests::import_meta_reports_1470_only_in_cjs_files（ESM 文件实收 [1470]，应干净）
- 嫌疑 commit：fix35/w10 的 `d50392afa9`（Program::new 移除解析结果无条件 realpath）或 `776a6eb87f`（source_files_found_searching_node_modules）——模块格式/包探测的路径规整变化
- 裁决入口：Go 侧 module format 判定链（package.json "type" 查找）与 realpath 语义差异

## 3. 祖先目录 tsconfig 搜索失败 + 内存分配失败

- 测试：`tsox-execute --lib` execute::tests::finds_config_in_ancestor_directory（FAILED，且伴随 "memory allocation of 15859728 bytes failed"——疑似搜索路径未规范化导致循环/膨胀）
- 嫌疑 commit：fix35/w10 的 `d50392afa9`（realpath 移除波及 execute 侧配置搜索）
- 裁决入口：Go findConfigFile 的路径规整 vs Rust 移除 realpath 后的等价性

## 处置建议

三项同源（w10 深水例的路径/选项装载对齐外溢），建议单开一轮专项修复（不进 corpus
选例，以这三个单测为锚点），修完跑 workspace 全量确认。
