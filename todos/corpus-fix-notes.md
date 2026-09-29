# corpus 修复记录（飞轮单发流水）

飞轮单发串行纪律下的逐次修复记录：一次一个文件（编译轮）或一个用例（测试轮），修一片记一条，最新在上。每条含：文件/用例、根因、对照的 Go 源、改动点、结果与 CSV 变化。系统记忆已清空，本文件是修复连续性的唯一留存。

## 基线（2026-09-29，自第 21 轮记忆迁移）

- 终态：CSV 4042 败（generatorTypeCheck 族 6 例 + yield 上下文定型 3 例转绿后）。
- 已知剩余深水（待单发消化）：
  - yield* 二级上下文链（26/64）：数组元素从合成 Generator 的 yield 型取、IIFE callee 从 call 上下文签名回传，两跳都缺。
  - 生成器体自然返回推断（25/62/63）：头行已对齐，差异在 elaboration 链折叠——relater 的 TYPES_RETURNED_BY 需支持「属性+签名返回+属性」折叠出 `a().b()` 形态，迭代协议比较走 next 专道。
  - 本地接口 extends Iterator+Iterable 的合成成员迭代型提取（8）。
  - 注解型 `{[Symbol.iterator](): void}` 不报 TS2488 的通道（28）。
- 运行口径：全量 `ulimit -v 4194304 + TSOX_SUBMODULE_LIMIT=0 + --release --no-fail-fast`，cd crates/tsox 跑 corpus；CSV 只记 FAIL，写操作一律绝对路径 `/home/cqh/workspace/ts2rust-port/corpus_results.csv`。
