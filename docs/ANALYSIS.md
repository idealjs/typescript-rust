# Go vs Rust 结构对比与已完成项盘点

更新时间：2026-07-31

本文档是 typescript-go → typescript-rust 迁移的**结构对比与已完成项盘点**。
迁移目标、阶段任务、进度快照、下阶段优先级见 [`TODO.md`](../roadmap/TODO.md)；
流程审计与行为差异细节见 [`MIGRATION.md`](./MIGRATION.md)。

## 1. Worktree 与基线

| 角色 | 路径 | 分支 |
|------|------|------|
| Rust 迁移主工作目录 | `/Users/cqh/workspace/typescript-rust` | `rust` |
| Go oracle | `/Users/cqh/workspace/typescript-go` | `main` |

测试基线（2026-07-31）：`cargo test --lib` 通过 609 个；`tests/` 集成测试 503 个
（501 checker parity + 2 emit parity）。总计 **1112 个测试**通过。

## 2. Go 代码库整体框架

typescript-go（Corsa）采用严格分层架构：

```
cmd/tsgo            CLI 入口（32 行 main.go，三模式分发：CLI / --lsp / --api）
  ↓
execute             命令编排：tsc / build / incremental / watchmanager
  ↓
compiler            Program 编排：解析 → 绑定 → 检查 → emit
  ↓
scanner → parser → binder → checker → printer   核心管线
  ↓
ast (数据模型) / diagnostics (消息+多语言) / tsoptions / module / vfs
  ↓
ls / lsp / project / api   语言服务与进程间通信
```

`internal/` 下共 44 个一级子包，非测试 Go 代码约 **21–23 万行**。三大代码集中点：

1. `internal/checker/` — 24 文件，**≈ 59,822 行**（占 27%），其中 `checker.go` 单文件
   31,926 行
2. `internal/ls/` — ~55 文件，≈ 35,000 行（语言服务）
3. `internal/lsp/` + `internal/ast/` + `internal/diagnostics/` — 各约 2 万行
   （多为生成代码）

**生成链路**：`_scripts/ast.json`（schema） + `_scripts/generate-go-ast.ts`/
`generate-ts-ast.ts`/`generate-encoder.ts`（TypeScript 写就）→ 生成
`internal/ast/ast_generated.go`(10047) / `kind_generated.go`(463) /
`kind_stringer_generated.go`(375) / `internal/api/encoder/*_generated.go`。
diagnostics 生成器是 Go 内联的 `internal/diagnostics/generate.go`(460) → 生成
`diagnostics_generated.go`(8626) + 13 种语言的 `loc/*.json.gz`。

## 3. Rust 代码库整体框架

typescript-rust（crate `tsox`，edition 2024，rust-version 1.96）的 `src/lib.rs`
声明 27 个顶级模块，**一一镜像 Go `internal/` 包结构**，命名与错误码优先对齐
Go/TypeScript 而非 idiomatic Rust。`src/` 共 69 个 .rs 文件，**84,117 行**。

### 3.1 模块对照表

| Rust 模块 | 行数 | Go 对应 | 完成度 | 备注 |
|-----------|------|---------|--------|------|
| `ast/` | 7476 | `internal/ast`(20847) | ~34% | generated 节点已对齐；缺生成脚本 |
| `binder/` | 2115 | `internal/binder`(3555) | ~41% | 单文件 mod.rs；缺 nameresolver/referenceresolver 拆分 |
| `checker/` | **21844** | `internal/checker`(59822) | ~20% | 15 文件已拆分；主战场 |
| `collections/` | 1090 | `internal/collections` | 基础 | MultiMap/OrderedMap/OrderedSet/Set/SyncMap/Cow |
| `compiler/` | 768 | `internal/compiler`(5658) | 基础 | Program pipeline 通；缺 incremental/build orchestrator |
| `core/` | 1825 | `internal/core`(2691) | 基础 | arena/bfs/text/compiler_options/tristate/stack 等 |
| `diagnostics/` | 24260 | `internal/diagnostics`(9423) | 完成 | 2154 条消息；**生成脚本未入库**；本地化未实现 |
| `diagnosticwriter/` | 210 | `internal/diagnosticwriter` | 基础 | — |
| `emitter/` | 774 | `internal/compiler` emit | 基础 | 源文本切片式 JS emit；缺 transformer/declaration emit |
| `evaluator/` | 420 | `internal/evaluator` | 基础 | — |
| `execute/` | 2152 | `internal/execute`(7831) | 基础 | CLI + build mode bridge；缺 incremental/watch |
| `glob/` | 404 | `internal/glob` | 基础 | — |
| `jsnum/` | 1012 | `internal/jsnum` | 基础 | — |
| `json/` | 115 | `internal/json` | 基础 | — |
| `locale/` | 74 | `internal/locale` | 占位 | — |
| `module/` | 377 | `internal/module`(2700) | 基础 | 缺 resolver 主路径 |
| `packagejson/` | 588 | `internal/packagejson` | 基础 | — |
| `parser/` | 7282 | `internal/parser`(9071) | 77% | 缺 reparser.rs/jsdoc.rs |
| `printer/` | 1578 | `internal/printer`(10313) | 基础 | 仅 NameGenerator；完整 AST→文本未迁移 |
| `scanner/` | 1570 | `internal/scanner`(4256) | 36% | 缺完整 regex 校验/trivia 节点 |
| `semver/` | 820 | `internal/semver` | 基础 | — |
| `sourcemap/` | 1237 | `internal/sourcemap` | 基础 | — |
| `stringutil/` | 231 | `internal/stringutil` | 基础 | — |
| `tsoptions/` | 3181 | `internal/tsoptions`(6046) | 基础 | 缺 declaration-driven parser |
| `tspath/` | 1797 | `internal/tspath` | 基础 | — |
| `vfs/` | 449 | `internal/vfs`(3131) | 基础 | 缺 vfsmatch/cachedvfs/trackingvfs |
| **未迁移** | — | `internal/ls`(35k) | 0% | 语言服务（completions/hover/rename/...） |
| **未迁移** | — | `internal/lsp`(21k) | 0% | LSP 服务器（`--lsp` 为 stub） |
| **未迁移** | — | `internal/project`(11k) | 0% | LSP 项目管理 |
| **未迁移** | — | `internal/api`(9.7k) | 0% | 进程间 API（`--api` 为 stub） |
| **未迁移** | — | `internal/fswatch`(5k) | 0% | 原生文件监控（CGO/汇编） |
| **未迁移** | — | `internal/format`(4.2k) | 0% | 代码格式化 |
| **未迁移** | — | `internal/fourslash` | 0% | FourSlash 测试框架 |

### 3.2 关键文件行数（src/ 前 20）

| 行数 | 文件 |
|------|------|
| 24068 | `src/diagnostics/messages_generated.rs`（生成） |
| 7282 | `src/parser/mod.rs` |
| 5915 | `src/checker/checker.rs` |
| 5478 | `src/ast/node_data_generated.rs`（生成） |
| 3326 | `src/checker/relater.rs` |
| 3181 | `src/tsoptions/mod.rs` |
| 2225 | `src/checker/typenode.rs` |
| 2262 | `src/checker/flow.rs` |
| 2152 | `src/execute/mod.rs` |
| 2115 | `src/binder/mod.rs` |
| 1839 | `src/checker/types.rs` |
| 1797 | `src/tspath/mod.rs` |
| 1578 | `src/printer/mod.rs` |
| 1570 | `src/scanner/mod.rs` |
| 1472 | `src/checker/inference.rs` |
| 1327 | `src/checker/grammarchecks.rs` |
| 1237 | `src/sourcemap/mod.rs` |
| 1012 | `src/jsnum/mod.rs` |
| 888 | `src/checker/nodebuilder.rs` |
| 877 | `src/checker/emitresolver.rs` |

### 3.3 checker/ 模块拆分对照

| Rust 文件 | 行数 | Go 对应 | 状态 |
|-----------|------|---------|------|
| `checker.rs` | 5915 | `checker.go`(31926) | 核心：check_source_file / statement / expression |
| `relater.rs` | 3326 | `relater.go`(5006) | union/intersection/object/array/tuple/signature/index/generic/条件/映射 |
| `flow.rs` | 2262 | `flow.go`(2734) | narrowing：typeof/instanceof/in/truthiness/discriminated union |
| `typenode.rs` | 2225 | `typenode.go` | TypeLiteral/FunctionType/Conditional/Mapped/Infer |
| `types.rs` | 1839 | `types.go`(1459) | ObjectType/Signature/TypeParameter/Union/Intersection/Tuple |
| `inference.rs` | 1472 | `inference.go`(1651) | inferTypeArguments / contextual typing / infer R |
| `grammarchecks.rs` | 1327 | `grammarchecks.go`(2202) | modifier/parameter list/break-continue/JSX grammar |
| `nodebuilder.rs` | 888 | `nodebuilderimpl.go`(3585) | type_to_string / symbol_to_string；**symbol_to_type_node 仍为 stub** |
| `emitresolver.rs` | 877 | `emitresolver.go`(1322) | visibility tracking 完成 |
| `utilities.rs` | 701 | `utilities.go`(1844) | — |
| `jsx.rs` | 329 | `jsx.go`(1482) | TS2604 |
| `jsdoc.rs` | 225 | `jsdoc.go`(100) | JSDoc parser 落地前为 no-op |
| `mapper.rs` | 232 | `mapper.go`(315) | **3 处 placeholder 闭包回退** |
| `tracer.rs` | 186 | `tracer.go`(366) | — |

## 4. 生成文件占比

src/ 下 3 个生成文件共 **29,905 行**，占 84,117 总行数的 **35.55%**：

| 生成文件 | 行数 | 声明的生成器 | 生成器是否入库 |
|---------|------|-------------|---------------|
| `src/ast/node_data_generated.rs` | 5478 | `_scripts/generate-rust-ast.ts` | **否** |
| `src/ast/syntax_kind_generated.rs` | 359 | 同上 | **否** |
| `src/diagnostics/messages_generated.rs` | 24068 | `_scripts/generate-rust-diagnostics.ts` | **否** |

**风险**：当前这些文件不可重复生成、无法验证 `git diff` 干净。Go 侧生成器
（`_scripts/generate-go-ast.ts` + `internal/diagnostics/generate.go`）已就绪并
作为输入源。Rust 侧生成脚本缺失是 P2.0 的核心待办。
