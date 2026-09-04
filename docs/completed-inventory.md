# 已完成项盘点与缺口清单

## 5. 已完成项盘点（按模块）

### 5.1 P0 基线（完成）
- [x] Go/Rust worktree 重组（主目录跑 `rust` 分支，Go oracle 在独立 worktree）
- [x] `cargo test` 通过；crate `tsox`，edition 2024，rust-version 1.96
- [x] `MIGRATION.md` 流程审计 + `TODO.md` 规划文档
- [x] warning 清理一轮；scanner 非 ASCII panic 修复

### 5.2 P1 CLI/tsconfig（部分完成）
- [x] CLI 参数处理流程审计；`--init`/`--version`/`--help`/`--all`/
  `--watch --listFilesOnly`/`--project`/ancestor tsconfig/`--showConfig` 控制流
- [x] tsconfig 顶层 `references`/`compileOnSave`；`files: []` 不触发默认 include；
  wildcard include 跳过 `node_modules`/`.git` 等；literal directory 递归展开
- [ ] 剩余：declaration-driven option parser、watch options 独立建模、
  `extends` package resolution、typed project references、no-input diagnostics、
  `vfsmatch`、退出码对齐、parity fixtures 扩充

### 5.3 P2 Scanner/Parser/AST（大部分完成）
- [x] **P2.1 Scanner 基础**：`scanEscapeSequence`/`scanUnicodeEscape`/
  `reScanGreaterThanTokenInner`/`scanInvalidCharacter`/`unicodeproperties` 用
  `unicode-ident` 替代/`CommentDirectives`/`PrecedingLineBreak` ASI
- [x] **P2.2 Scanner regex 基础**：`reScanSlashToken`（pattern body + 字符类 +
  转义 + flags + 未终止诊断）
- [x] **P2.3 Scanner JSX**：`ScanJsxToken`/`ScanJsxIdentifier`/
  `ScanJsxAttributeValue` 全套 + JSX parser 重写
- [x] **P2.4 Parser 类型语法**（全部）：type alias / call signature / generic /
  union/intersection / mapped / conditional / keyof/infer/typeof/import() /
  as const/satisfies/non-null / template literal types
- [x] **P2.5 Parser 声明/语句**（全部）：`declare` 调度、装饰器、import attributes、
  binding patterns、TS6/7 新语法（`using`/`await using`/`accessor`）、
  `yield`、`for await...of`、可选链、类型参数方差注解
- [x] **P2.8 Parser diagnostic parity**（全部）：diagnostic code 对齐、错误消息
  文本对齐、`parseErrorAtRange` 去重、24 个 ParsingContext 错误映射、
  `is_list_element` 对齐、UTF-16 offset 对齐
- [x] **P2.9 Parser bundled libs**：111 个 `lib.*.d.ts` 全部零错误解析（基线
  3347 → 0）
- [x] **P2.10 位置信息**（全部）：`LineMap` 对齐 `ComputeECMALineStarts`，
  UTF-16 column 计算
- [ ] 剩余：P2.0 生成脚本入库；P2.1 trivia 节点；P2.2 完整 regex body 校验
  （`regExpParser`/命名捕获组/`u`/`v`/`d` flag）；P2.6 reparser.rs；
  P2.7 jsdoc.rs；P2.9 真实文件 parser parity fixtures

### 5.4 P3 Binder/Checker/Diagnostics（进行中）

#### P3.1 Binder 控制流图
- [x] FlowNode 数据结构、`START`/`UNREACHABLE`、变量声明/表达式 flow node
- [x] FlowLabel 合并点；`ASSIGNMENT`/`TRUE_CONDITION`/`FALSE_CONDITION`/
  `SWITCH_CLAUSE`/`LOOP_LABEL`/`BRANCH_LABEL`/`CALL` flow node
- [x] if/while/do-while/for/for-in/for-of/switch 控制流
- [x] return/throw/break/continue
- [x] try/catch/finally 异常流（normal/exception/return 路径 + finally 合并）
- [x] `ARRAY_MUTATION`：`arr.push(x)`/`arr.unshift(x)` 检测 + evolving array
  类型演化（6 条 parity 测试）
- [ ] 剩余：`ReduceLabel`/`Shared`/`Referenced` 后处理；labeled statement

#### P3.1a Binder 容器递归绑定（完成）
- [x] `bind_container` 设置 `parent_symbol`；容器递归绑定；checker
  `resolve_identifier` 改用 scope_stack 遍历；`is_unique_local_name` 同时
  检查 locals + symbol members

#### P3.2 Binder NameResolver
- [x] 基础作用域链查找、符号意义过滤、for/for-in/for-of 循环作用域、
  `resolveName` 入口、`argumentsSymbol`（区分普通函数/箭头函数）、
  `undefinedSymbol`/`globalThisSymbol`、`populate_globals`
- [ ] 剩余：箭头函数参数作用域、enum/namespace 成员查找、export default 别名、
  类型参数作用域限制、`infer T`、装饰器位置调整

#### P3.4 Checker 类型结构（完成）
- [x] Type 枚举完整（ObjectType/Signature/TypeParameter/Union/Intersection/
  Tuple/TemplateLiteral/ConditionalRoot 等）

#### P3.5 Checker 入口与标识符解析
- [x] `check_source_file`、标识符解析、TS2304 未定义符号

#### P3.6 Checker relater（大体完成）
- [x] union/intersection/对象/数组/tuple/signature/index signature/generic/
  条件/映射类型关系 + 缓存与循环检测
- [x] class extends 继承 + this 类型解析
- [x] 比较无重叠检查 TS2367
- [x] 不可调用/不可构造检查 TS2349/TS2351
- [x] 只读属性赋值检查 TS2540
- [x] declaration merge checker 侧
- [ ] 剩余：`isEnumTypeRelatedTo`/`isUnknownLikeUnionType`（relater.rs 两处
  TODO）

#### P3.7 Checker 推断（部分完成）
- [x] 泛型推断 + contextual typing + infer R
- [x] 函数重载解析 + `new` 表达式实例类型
- [x] 返回语句类型检查
- [x] **本轮**：参数数量检查 TS2554/TS2555/TS2556（含 spread、rest 元素类型
  检查、overload arity）+ 12 个 parity fixtures
- [ ] 剩余：contextual typing from return type、parameter contextual typing
  + binding patterns、freshness tracking（`checker.rs:1460` freshType，
  影响 literal widening 精度）

#### P3.9 Checker 控制流 narrowing（大体完成）
- [x] typeof/instanceof/in narrowing、truthiness、discriminated union、
  type predicate、switch(true)、asserts
- [ ] 剩余：按 fixture 缺口补齐

#### P3.10 Checker nodebuilder（部分完成）
- [x] `type_to_string`、`symbol_to_string`、`get_quick_info_text`（hover）
- [ ] 剩余：`symbol_to_type_node`/`symbol_to_display_parts`（declaration emit
  前置依赖）

#### P3.11 Checker emitresolver（完成）
- [x] visibility tracking、alias marking visitor、`is_entity_name_visible`

#### P3.13/P3.14 Diagnostics
- [x] 2154 条消息文本对齐 Go
- [ ] 剩余：生成脚本入库；本地化；`format_message` 完全对齐 Go（UTF-8 校验、
  无效索引 panic）

### 5.5 已知 stub / 风险点（汇总）

| 位置 | 问题 | 影响 |
|------|------|------|
| `src/checker/mapper.rs` | 3 处 placeholder 闭包回退（`new_simple_type_mapper`/`new_array_type_mapper`） | 类型替换在 fallback 分支返回原类型，正确性风险 |
| `src/checker/nodebuilder.rs` | `symbol_to_type_node`/`symbol_to_display_parts` 未实现 | declaration emit 前置依赖缺失 |
| `src/checker/checker.rs:1460` | `freshType` freshness tracking 未实现 | literal widening 精度不足 |
| `src/checker/relater.rs` | `isEnumTypeRelatedTo`/`isUnknownLikeUnionType` 两处 TODO | enum/unknown 类型关系不完整 |
| `src/checker/typenode.rs` | mapped type 节点解析仍为 stub | `get_type_from_mapped_type_node` 待落地 |
| `_scripts/generate-rust-ast.ts` | 未入库 | `node_data_generated.rs` 不可重复生成 |
| `_scripts/generate-rust-diagnostics.ts` | 未入库 | `messages_generated.rs` 不可重复生成 |
| `--lsp` / `--api` | stub | LSP/API 服务未迁移 |
| `build.rs` | 优先从 Go worktree 读 `bundled/libs` | 跨 worktree 依赖未解耦 |
| `cargo fmt --check` | 未对齐全仓 | 触碰文件时顺手格式化，整仓格式化单独排期 |
| 剩余 warning ~77 个 | Go/TS 命名对齐、未接入占位 API、checker re-export 冲突 | 迁移期可接受 |

## 6. 测试基线对照

| 类别 | 数量 | 说明 |
|------|------|------|
| `cargo test --lib` | 609 | src/ 内 `#[cfg(test)]`，分布在 51 个文件 |
| `tests/checker_parity.rs` | 501 | 类型检查 parity（自 2026-07-13 的 106 增长 395 个） |
| `tests/parity.rs` | 2 | emit parity（simple_emit / type_only_declarations / nested_out_dir，3 fixture） |
| **总计** | **1112** | 全部通过 |

`checker_parity.rs` 覆盖诊断码：TS2304/TS2322/TS2339/TS2345/TS2349/TS2351/
TS2367/TS2420/TS2554/TS2555/TS2556 等。`parity.rs` 在 `TSGO_ORACLE` 环境变量
存在时与 Go oracle 二进制对照 exit code/stdout/stderr/输出文件，否则跳过。

## 7. 下阶段优先级（与 TODO.md 一致）

按依赖与价值排序：

1. **P3.7 Checker 类型关系补齐**：`isEnumTypeRelatedTo`/`isUnknownLikeUnionType`
   + mapper.rs 三处 placeholder 闭包回退修复
2. **P3.8 Checker 推断收尾**：contextual typing from return type、parameter
   contextual typing + binding patterns、freshness tracking
3. **P3.1 Binder flow graph 收尾**：`ReduceLabel`/`Shared`/`Referenced` 后处理、
   labeled statement
4. **P3.9 Checker 控制流 narrowing**：按 fixture 缺口补齐
5. **P3.10 Checker nodebuilder**：`symbol_to_type_node`/`symbol_to_display_parts`
6. **P3.2 Binder NameResolver 收尾**：箭头函数参数作用域、enum/namespace 成员
   查找、export default 别名、类型参数作用域限制、`infer T`、装饰器位置调整
7. **P2.0 AST/diagnostics 生成链路**：补齐 `_scripts/generate-rust-ast.ts` 与
   `_scripts/generate-rust-diagnostics.ts`，使生成文件可重复生成
8. **P1 CLI/tsconfig 收尾**：declaration-driven option parser、watch options
   独立建模、`extends` package resolution、typed project references、
   no-input diagnostics、`vfsmatch`

## 8. 长期缺口（未启动）

下列 Go 子模块在 Rust 侧尚未启动，按价值排序：

| Go 模块 | 行数 | 价值 | 备注 |
|---------|------|------|------|
| `internal/ls/` | ~35k | 高 | 语言服务：completions/hover/rename/findallrefs/codeactions/autoimport |
| `internal/lsp/` | ~21k | 高 | LSP 服务器（`--lsp`）；含 `lsp_generated.go`(17262) |
| `internal/project/` | ~11k | 高 | LSP 项目管理：session/snapshot/configfileregistry/ata |
| `internal/api/` | ~9.7k | 中 | 进程间 API（`--api`）；msgpack/jsonrpc |
| `internal/fswatch/` | ~5k | 中 | 原生文件监控（fsevents/inotify/fanotify/kqueue，CGO/汇编） |
| `internal/format/` | ~4.2k | 中 | 代码格式化 |
| `internal/printer/` | ~10.3k | 高 | 完整 AST→文本（Rust 侧仅 NameGenerator） |
| `internal/vfs/` 扩展 | ~3.1k | 中 | vfsmatch/cachedvfs/trackingvfs |
| `internal/module/` | ~2.7k | 高 | 模块解析 resolver 主路径 |
| `internal/fourslash` | — | 低 | FourSlash 测试框架（测试基础设施） |

## 9. 本轮（2026-07-31）新增工作

- TS2554/TS2555/TS2556 参数数量检查（含 spread TS2556、rest 元素类型检查、
  overload arity）落地于 `src/checker/checker.rs` 的 `check_call_arity`
- 12 个 parity fixtures 新增至 `tests/checker_parity.rs`，覆盖 too few/ too
  many / required-then-rest / spread TS2556 / overload arity 等场景
- `src/binder/mod.rs`/`src/checker/emitresolver.rs`/`src/checker/nodebuilder.rs`/
  `src/checker/typenode.rs` 触碰性修改
- 本文档（`ANALYSIS.md`）创建，作为 Go vs Rust 结构对比与已完成项盘点的唯一
  参考；`TODO.md` 下阶段优先级章节同步更新
