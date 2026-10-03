# corpus 修复 subagent 派发模板

> 供主 agent 编排语料修复循环时生成派发 prompt。占位符 `{{...}}` 由主 agent 填充。

## 模板正文

```
# 任务：{{目标名}}（修复入口，目标：使其 PASS）

## 目标
用例：compiler/{{用例名}}.ts
定位 Go/Rust 语义分歧，按 Go 修根因。验收 = 复跑 PASS（主 agent 执行，你禁测）。

## 失败信息（三层锚点，按序使用）
分片：/tmp/flywheel_shards/ 下 grep 用例名（.types 首分歧锚点——已知偏差形态跳过——→ 错误 diff → 执行栈对照段）。
完整序列：主仓 corpus_go_trace.csv / corpus_rust_trace.csv 按行首 case 名 grep。

## 环境
- Go oracle（只读）：/home/cqh/workspace/typescript-go
- 你的 worktree：{{worktree}}（分支 {{branch}}）。开工 pwd 验证；git 一律 git -C {{worktree}} ...
- 只改 crates/ 下生产代码；主仓绝对只读。

## 参考数据（主仓根，只读）
- corpus_results.csv——FAIL 全量（水位）
- corpus_rust_trace.csv / corpus_go_trace.csv——两侧函数调用序列
- corpus_rust_types.csv / corpus_go_types.csv——两侧 .types 全量输出
- corpus_types_anchor.csv——.types 首分歧锚点（按行首 case 名 grep）
- corpus_stack_diff.csv——执行栈差集（go_only / rust_only）
- 分片：/tmp/flywheel_shards/<用例名>.md（本例汇总视图）

## 收尾
- 无法继续推进时：提交已有可信修改，写 progress_notes.md（worktree 根，不入库），按汇报结构收尾。

## 禁止事项
- 禁止执行任何测试、构建命令（cargo/rustc/跑用例）。验证由主 agent 统一执行。
- 禁止探针式调试（插桩+跑用例）。
- 禁改：corpus_*.csv / func_alignment.csv（仓库根全部数据 CSV）/ AGENTS.md / tools/ / .traces/ / crates/tsox/tests/corpus/。
- 禁止 skip、改断言、改基线、空壳实现（恒返 None/空函数/删真实逻辑换占位）。
- 符号不存在时三选一：grep 等价符号改接线 / 按 Go 最小真实实现 / 保留错误记交接。

## 提交
- 每个独立修复立即 git commit，只 add crates/ 下生产路径。
- progress_notes.md 不入库。

## 故障退出
- Bash 连续 3 次失败：写 progress_notes.md 后立即结束并汇报。

## 汇报
- rootCause 引用双侧源码 文件:行号；functionTable 必交：
  | commit | 文件 | Rust 函数(增/改/删) | 对齐的 Go 函数 | 用例效果 |
```
