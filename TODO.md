# typescript-go -> typescript-rust 迁移目标与任务

更新时间：2026-07-29

本文档是迁移工作的**唯一规划文档**。流程审计、行为差异细节记录在
`MIGRATION.md`，按阶段查阅。

## 项目背景

- Go 源 worktree（oracle）：`/home/cqh/workspace/typescript-rust`，分支 `main`
- Rust 迁移 worktree：`/home/cqh/workspace/typescript-rust-rust`，分支 `rust`
- Rust crate：`tsox`，入口 `src/main.rs`，库入口 `src/lib.rs`
- `edition = "2024"`，`rust-version = "1.96"`（Cargo 1.96 不支持 `edition = "2026"`）

## 验收命令

```sh
# 全量 Rust 测试（需 rustc 1.96+）
cargo test

# parity 集成测试，需 Go oracle 二进制
TSGO_ORACLE=/home/cqh/workspace/typescript-rust/built/local/tsgo cargo test --test parity

# 未设 TSGO_ORACLE 时按以下顺序自动查找 oracle，找不到则跳过并打印原因：
# 1. /home/cqh/workspace/typescript-rust/built/local/tsgo
# 2. /home/cqh/workspace/typescript-rust/_packages/native-preview/bin/tsgo
```

构建 Go oracle：

```sh
cd /home/cqh/workspace/typescript-rust
npm install && npm run build
# 产物：built/local/tsgo
```

## 迁移总目标

1. **CLI/配置/解析/绑定/检查/emit 全栈对齐 Go oracle 行为**，以 exit
   code、stdout、stderr、输出文件集合为验收口径。
2. **真实项目可对齐**：`tsox -p tsconfig.json` 在真实项目（如 `ai-Color-toner`）
   上输出与 Go oracle 一致的诊断集合。
3. **LSP/API/发布独立可用**：`--lsp`、`--api`、npm 包、VS Code 扩展可脱离
   Go 构建产物运行。
4. **Go oracle 退役**：当 Rust parity 足够广时停止依赖 Go worktree。

## 迁移原则

- 按能力边界迁移，不做逐文件机械翻译：CLI → 配置解析 → VFS → program →
  parser → binder → checker → emit → LS/LSP → API → 发布。
- Go 实现作为 oracle，每补一个能力都加对应 parity case。
- 保持 Rust API idiomatic，但错误码、诊断文本、输出文件、CLI 行为优先对齐
  Go/TypeScript。
- 每个阶段都要有可运行验收命令，不能只算“代码写完”。
- 每个迁移行为都应新增或扩展一个 parity case，优先对比 exit code、stdout、
  stderr、输出文件内容。

## 当前进度快照（2026-07-30）

测试基线：`cargo test` 通过（609 个 lib 单测 + 2 个 emit parity + 335 个
checker parity，checker parity 自 2026-07-13 的 106 增长 229 个）。

| 模块 | Rust 行数 | Go 行数 | 完成度 | 备注 |
|------|-----------|---------|--------|------|
| Scanner | 1558 | 4277 | 36% | 转义/JSX/正则/CommentDirectives/ASI 完成；缺 trivia 节点、完整 regex 校验 |
| Parser | 7115 | 9251 | 77% | TS6/7 语法、类型语法、JSX、装饰器、import attributes 完成；缺 reparser/jsdoc |
| Binder | 1639 | ~4000 | ~41% | 容器递归绑定 + FlowNode + NameResolver 基础 + alias + 全局符号 完成；缺完整 flow graph、ARRAY_MUTATION、try/catch/finally、labeled statement、完整 declaration merge |
| Checker | 9050 | ~50K+ | ~18% | 类型结构完整；check_source_file + 标识符解析 + TS2304；relater 含 union/intersection/对象/数组/tuple/signature/index signature/generic/条件/映射类型关系 + 缓存与循环检测；inference 含泛型推断 + contextual typing + infer R；324 parity fixtures 通过；缺 nodebuilder、emitresolver、JSX/JSDoc/grammar checks、mapped type 节点解析 |
| Compiler | 759 | — | 基础 | Program 创建/解析/绑定/emit pipeline 通；checker 已接入 |
| Emitter | 774 | — | 基础 | JS emit 基础；缺 transformer 体系 |
| Printer | 1578 | — | 基础 | 节点→文本基础 |
| AST | ~5500 | — | 基础 | generated 节点 + symbol/flow 类型 |

`--lsp` 和 `--api` 当前为 stub。`cargo fmt --check` 仍未对齐全仓；触碰文件时
顺手格式化，整仓格式化单独排期。剩余 warning 约 31 个，归类为迁移期可接受
（Go/TS 命名对齐、未接入的占位 API、一处 checker re-export 冲突）。

## 下阶段优先级

按依赖与价值排序，逐项推进：

1. **P3.7 Checker 类型关系补齐**：数组、tuple、函数、泛型、条件类型、映射
   类型关系 + signature/index signature 比较。这是 inference/narrowing 真正
   可用的前置。
2. **P3.8 Checker 推断收尾**：变量声明初始化器类型写入 symbol、函数返回值
   推断、条件类型 `infer R` 解析、`type_node_links` 缓存。contextual typing
   入口已具备，需把推断结果落回符号/节点。
3. **P3.1 Binder flow graph 收尾**：`ARRAY_MUTATION`、`ReduceLabel`/`Shared`/
   `Referenced` 后处理、labeled statement、try/catch/finally 异常流。这是
   P3.9 narrowing 的硬依赖。
4. **P3.9 Checker 控制流 narrowing**：`narrowType`、`getNarrowedTypeOfSymbol`、
   discriminated union、`typeof`/`instanceof`/`in` narrowing。
5. **P3.10 Checker nodebuilder**：`type_to_string`、`symbol_to_type_node`、
   hover 信息。诊断文本对齐 oracle 的关键缺口。
6. **P3.2 Binder NameResolver 收尾**：箭头函数参数作用域、enum/namespace
   成员查找、export default 别名、类型参数作用域限制、`infer T`、装饰器
   位置调整。
7. **P1 CLI/tsconfig 收尾**：declaration-driven option parser（NameMap/
   did-you-mean/alternate-mode）、watch options 独立建模、`extends` package
   resolution、typed project references、no-input diagnostics、`vfsmatch`。

## P0：建立迁移工作基线

目标：Rust worktree 可独立运行、可验证、可追溯。

已完成：

- [x] Go/Rust worktree 位置确认；Rust crate 可编译并通过现有测试。
- [x] `MIGRATION.md` 记录 Rust 运行方式与 Go oracle 设置。
- [x] 固定验收命令文档化（`cargo test`、`TSGO_ORACLE=... parity`）。
- [x] warning 清理一轮：清理无争议的 unused imports/variables；剩余 warning
  分类为迁移期可接受。
- [x] scanner 非 ASCII 未知字符 panic 修复（按完整 UTF-8 rune 前进）。
- [x] crate 升级到 `edition = "2024"`，`rust-version = "1.96"`。

剩余任务：

- [ ] 给 CI 设计最小 Rust job：fmt、clippy、test、parity smoke。
- [ ] 新人只看 Rust worktree 文档即可跑通测试（验收）。
- [ ] parity 测试能自动发现可用 Go oracle，或给出明确跳过原因（验收）。

## P1：CLI 和 tsconfig 行为对齐

目标：Rust CLI 行为与 Go oracle 在 exit code / stdout / stderr / 输出文件
集合上一致。

Go 参考：`cmd/tsgo/main.go`、`internal/execute`、`internal/execute/tsc`、
`internal/tsoptions`、`internal/vfs`。
Rust 现状：`src/main.rs`、`src/execute/mod.rs`、`src/tsoptions/mod.rs`、
`src/vfs/mod.rs`。
流程审计见 `MIGRATION.md` 的 “Command Line Argument Flow Audit” 与
“TSConfig Flow Audit”。

已完成：

- [x] Go/Rust 全量 CLI 参数处理流程审计与差异记录。
- [x] Go/Rust `tsconfig.json` 解析/处理流程审计与差异记录。
- [x] `--init` 执行层控制流对齐（先于 `--version`/`--help`，存在 tsconfig 时报错）。
- [x] `--version`/`--help`/`--all`/`--watch --listFilesOnly`/`--project`/ancestor
  tsconfig 查找/`--showConfig` 控制流 bridge。
- [x] tsconfig 顶层 `references`/`compileOnSave` 解析并进入简化版 `--showConfig`。
- [x] `files: []` 不触发默认 include；未显式 `exclude` 时默认排除 `outDir`/
  `declarationDir`；wildcard include 跳过 `node_modules`/`bower_components`/
  `jspm_packages`/`.git`；literal directory include 递归展开。

剩余任务：

- [ ] 对齐 `--help`、`--version`、无输入、未知选项、响应文件等 CLI 行为。
- [ ] 对齐 `--init` 生成的完整 `tsconfig.json` 模板。
- [ ] 对齐退出码：`Success`、`DiagnosticsPresent_*`、`InvalidProject_*`、
  `ProjectReferenceCycle_*`。
- [ ] 迁移 Go declaration-driven option parser：NameMap、did-you-mean、
  alternate-mode diagnostics、TSConfigOnly 规则、enum/list/min-value 校验。
- [ ] 独立建模 watch options，在普通/build parser 中与 compiler/build options 分离。
- [ ] 对齐 tsconfig 查找、`extends`、`files/include/exclude`、`compilerOptions`
  覆盖规则。
- [ ] 将 raw `references` 升级为 typed project references：normalized path、
  original path、circular。
- [ ] 对齐 `extends` 的 package/Node-style resolution、cycle diagnostics 和
  extended config cache。
- [ ] 对齐 no-input diagnostics、config source span diagnostics 和 `vfsmatch`
  root-file expansion。
- [ ] 扩充 parity fixtures：无 tsconfig 且无文件 / 单文件输入 / `-p` 指向目录 /
  `-p` 指向文件 / `--showConfig` / response file / invalid JSON / JSONC。
- [ ] 修复当前 parity 注释中提到的 `rootDir/outDir` 差异。

验收：

- [ ] Rust 与 Go oracle 的 stdout、stderr、exit code、输出文件集合一致。
- [ ] CLI parity 覆盖至少 20 个常见 tsc 场景。

## P2：Scanner / Parser / AST parity

目标：Rust parser 对 `.ts/.tsx/.js/.jsx` 与 bundled libs 的解析结果和诊断
对齐 Go oracle。

Go 参考：`internal/scanner`（scanner.go 2918 + regexp.go 1076 +
unicodeproperties.go 162 + utilities.go 100）、`internal/parser`（parser.go
6827 + jsdoc.go 1355 + reparser.go 748）、`internal/ast`、`_scripts/ast.json`。
Rust 现状：`src/scanner/mod.rs`、`src/parser/mod.rs`、`src/ast/*`、`build.rs`。

### P2.0 AST 生成链路

- [ ] 明确 Rust AST 生成链路是否继续读 Go 侧 `_scripts/ast.json`，还是维护
  Rust 自有 schema。
- [ ] 对齐 generated enum/node 数据的生成命令和检查方式。
- [ ] 生成文件可重复生成，`git diff` 干净。

### P2.1 Scanner 基础能力补齐

已完成：`scanEscapeSequence`/`scanUnicodeEscape`、`reScanGreaterThanTokenInner`、
`scanInvalidCharacter` 完整诊断接入 parser/compiler、`unicodeproperties.go` 用
`unicode-ident` 替换、`CommentDirectives` 收集、`PrecedingLineBreak` 在 ASI
路径完整接入。

- [ ] 保留 trivia 节点（`WhitespaceTrivia`/`NewLineTrivia`/`CommentTrivia`），
  对齐 Go 的 `trivia` 输出。

### P2.2 Scanner 正则字面量

已完成：`reScanSlashToken` 基础（pattern body + 字符类 + 转义 + flags + 未终止诊断）。

- [ ] 迁移 `internal/scanner/regexp.go` 完整 regex body 校验（`regExpParser`：
  命名捕获组、`u`/`v` flag 模式、invalid flag 诊断）。
- [ ] 支持 `lastIndex`、命名捕获组、`d` flag 等现代正则特性。

### P2.3 Scanner JSX / JSDoc

已完成：`ScanJsxToken`/`ScanJsxIdentifier`/`ScanJsxAttributeValue` 全套迁移
与 JSX parser 重写（8 个测试通过）。

- [ ] 迁移 `ScanJSDocToken` + `scanJSDocCommentForTags`（依赖 P2.7 JSDoc parser）。

### P2.4 Parser 类型语法补齐（已全部完成）

- [x] type alias declaration（含 exported/ambient）、call/construct signature、
  generic type parameters、type arguments/references、union/intersection
  precedence、primitive keyword types、array/tuple/rest/readonly、indexed
  access/index signatures/mapped types、conditional types、`keyof`/`infer`/
  `typeof`/`import("x").T`、`as const`/`satisfies`/non-null、literal types 与
  discriminated union、template literal types。

### P2.5 Parser 声明/语句补齐（已全部完成）

- [x] 完整 `declare` 声明调度、装饰器 detailed parsing、import attributes、
  named imports/exports、object/array binding patterns、TS 6/7 新语法
  （`using`/`await using`/`accessor`）、修饰符关键字路由、`yield` 表达式、
  `for await...of`、可选链 `?.`、类型参数方差注解。

### P2.6 Parser reparser

- [ ] 迁移 `internal/parser/reparser.go`（748 行）到 `src/parser/reparser.rs`。
- [ ] `@typedef` JSDoc → type alias 节点追加到 statements。
- [ ] `reparseTopLevelAwait`：外部模块 + `possibleAwaitSpans` 重解析。
- [ ] `collectExternalModuleReferences`。

### P2.7 Parser JSDoc

- [ ] 迁移 `internal/parser/jsdoc.go`（1355 行）到 `src/parser/jsdoc.rs`。
- [ ] `parseJSDocComment`：tag 类型、`@param`、`@returns`、`@typedef`、
  `@callback`、`@template`。
- [ ] JSDoc type expression：`@type {string}`、`@param {string} name`。
- [ ] 节点附加 `jsDoc` 字段。

### P2.8 Parser diagnostic parity（已全部完成）

- [x] diagnostic code 对齐（TS1003 等）、错误消息文本对齐 Go（含参数插值）、
  级联错误恢复点优化（`parseErrorAtRange` 去重 + 24 个 ParsingContext 错误
  映射 + `abortParsingListOrMoveToNextToken`）、`is_list_element` 完整对齐
  Go、`is_start_of_type` 补齐缺失 token、语法错误位置 UTF-16 offset 对齐。

### P2.9 Parser parity fixtures

已完成：bundled libs 全部 100+ 零错误解析（基线 3347 → 0）。

- [ ] 从 Go parser 测试或 TypeScript baselines 挑选 smoke 集合，转成 Rust parity。
- [ ] 典型 `.ts` 解析结果和诊断对齐 oracle。
- [ ] 典型 `.tsx` JSX 解析对齐。
- [ ] 典型 `.js`/`.jsx` 对齐。
- [ ] bundled lib smoke：`lib.es2015.iterable.d.ts`、`lib.dom.d.ts` 错误数验证。

### P2.10 位置信息一致性（已全部完成）

- [x] `LineMap` 对齐 `ComputeECMALineStarts`（LF/CR/CRLF/LS/PS）；UTF-16
  column 计算（`utf16_len` + `utf16_column_at`）。

验收：

- [x] `lib.es5.d.ts` 可零错误解析。
- [ ] 典型 `.ts/.tsx/.js/.jsx` 解析结果和诊断可对齐 oracle。

## P3：Binder / Checker / Diagnostics parity

目标：Rust checker 能在真实项目输出与 Go oracle 一致的诊断集合。

Go 参考：`internal/binder`（binder.go 2795 + nameresolver.go 498 +
referenceresolver.go 262）、`internal/checker`（checker.go 31926 + relater.go
5006 + flow.go 2734 + grammarchecks.go 2202 + inference.go 1651 +
nodebuilderimpl.go 3585 + emitresolver.go 1322 + jsx.go 1482 等）、
`internal/diagnostics`、`internal/nodebuilder`、`internal/pseudochecker`。
Rust 现状：`src/binder/mod.rs`、`src/checker/*`、`src/diagnostics/*`。

### P3.1 Binder 控制流图（checker narrowing 前置依赖）

已完成：FlowNode 数据结构、`START`/`UNREACHABLE` 初始化、变量声明/表达式
flow node、FlowLabel 合并点、`ASSIGNMENT`/`TRUE_CONDITION`/`FALSE_CONDITION`/
`SWITCH_CLAUSE`/`LOOP_LABEL`/`BRANCH_LABEL`/`CALL` flow node、if/while/
do-while/for/for-in/for-of/switch 控制流、return/throw/break/continue、
10 个 flow graph 单元测试。

- [ ] 迁移 `internal/binder/binder.go` 完整 flow 构建逻辑（~1500 行）。
- [ ] `ARRAY_MUTATION`：方法调用副作用。
- [ ] `ReduceLabel`/`Shared`/`Referenced` 后处理。
- [ ] labeled statement 标签支持。
- [ ] try/catch/finally 异常流。

### P3.1a Binder 容器递归绑定（已完成）

- [x] `bind_container` 设置 `parent_symbol`；容器递归绑定；checker
  `resolve_identifier` 改用 scope_stack 遍历；`is_unique_local_name` 同时
  检查 locals + symbol members。

### P3.2 Binder NameResolver

已完成：基础作用域链查找、符号意义过滤、for/for-in/for-of 循环作用域、
`resolveName` 入口、`argumentsSymbol`（区分普通函数/箭头函数）、
`undefinedSymbol`/`globalThisSymbol`、`populate_globals`（lib.d.ts 全局符号）、
`follow_alias`（import/export alias 链）。

- [ ] 迁移 `internal/binder/nameresolver.go` 完整逻辑到
  `src/binder/nameresolver.rs`（剩余：箭头函数参数作用域、enum/namespace
  成员查找、export default 别名、类型参数作用域限制、`infer T` 类型参数、
  装饰器位置调整、alias 符号解析）。

### P3.3 Binder ReferenceResolver

- [ ] 迁移 `internal/binder/referenceresolver.go`（262 行）。
- [ ] 标识符引用记录（用于 find references / rename）。

### P3.4 Binder 声明合并与 export/import binding

- [ ] declaration merge：namespace + function + interface + class 合并规则。
- [ ] export binding：`export { A }` 的 `exportSymbol` → local symbol 链。
- [ ] import binding：`import { A }` 的 `aliasSymbol` → resolved symbol。
- [ ] `delayedSymbol`/`aliasSymbol` 特殊符号处理。
- [ ] 完整 scope 链（当前只有 `container`/`block_scope_container` 两个字段）。

### P3.5 Checker 接入 compiler（已完成）

- [x] `Program::new` 后调用 `Checker::new` + `check_source_file`；
  `get_semantic_diagnostics` 返回 `Vec<Diagnostic>` 并接入 execute 输出管线。
- [ ] `Program` trait 补全 checker 所需方法（`getCommonSourceDirectory`、
  `getCanonicalFileName` 等）。

### P3.6 Checker 核心入口（已完成）

- [x] `check_source_file` 遍历 statements + TS2304；`check_statement` 覆盖
  全部语句；`check_expression` 覆盖全部表达式；`check_class_member`/
  `check_enum_member`/`check_heritage_clause` 辅助方法；`resolve_identifier`
  遍历作用域链；节点→类型缓存 `get_type_of_node`（`type_node_links`）。

### P3.7 Checker 类型关系（relater 完整规则）

已完成：`relater.go` 基础骨架；`is_type_assignable_to` 基础规则
（any/unknown/never/基本类型/字面量）；`is_type_subtype_of`/
`is_type_comparable_to`/`is_type_strict_subtype_of`；union/intersection
类型关系；对象类型结构检查 + 属性类型深度检查 `is_object_type_related_to`；
数组类型协变关系（`is_array_type_related_to`）；tuple 元素逐项比较
（`is_tuple_type_related_to`）；call/construct signature 比较
（`signatures_related_to` 含 pairwise/single-signature/N×M fallback、
`compare_signatures_related` 含 bivariant/contravariant 参数比较、
rest/optional/min-argument-count 处理、return type 比较、void/any wildcard）；
index signature 比较（`is_index_signatures_related_to`）；generic type
reference 协变/逆变推断（`generic_type_reference_related_to`）。

- [x] 条件类型、映射类型关系（`conditional_type_related_to` 含
  permissive/restrictive 短路、`mapped_type_related_to` 含 constraint
  逆变 + template 协变；mapped type 节点解析仍为 stub，待 P3.8
  `get_type_from_mapped_type_node` 落地后才能真正触发 mapped 比较）。
- [x] `relation_comparison_result` 缓存与递归保护：`RelationCacheKey`
  以 `Arc::as_ptr` 指针身份 + `RelationKind` 为键（因 `Type::id` 尚未
  全量赋值），`relation_cache` 按 top-level call 清空，
  `relation_in_progress` 做循环检测，`relater_depth` 做深度兜底。

### P3.8 Checker 类型推断

已完成：`get_type_of_node` 框架；字面量/二元/括号类型推断；`get_type_of_symbol`
骨架；`inference.go` 迁移（`inferTypeArguments` 协变/逆变/约束/默认回退）；
contextual typing（`getContextualType` 完整分发）；函数返回值推断（含 literal
widening，`return 42` → `number`）；表达式类型推断补齐（`as`/`satisfies`/
type assertion `<T>x`/`!x`/conditional `? :`/template/delete/void/await/
property access `x.prop`/element access `x[i]`/unary `!`/`+`/`-`/`++`/`--`）；
数组字面量类型推断（`[1,2,3]` → `number[]`）；`NewExpression` 构造签名返回类型；
conditional 表达式 union 类型推断（`cond ? 1 : 'hi'` → `number | string`）；
对象字面量类型推断（`{ a: 1, b: 'hi' }` → `{ a: 1, b: 'hi' }`，保留 literal
类型以支持 discriminated union 赋值）；`TypeLiteral` 节点解析为结构化对象类型
（`{ a: number; b: string }` 注解生成带 property signature 的 anonymous object
type，含循环保护）；`FunctionType`/`ConstructorType` 节点解析为带 call/construct
signature 的对象类型（`get_type_from_function_type_node`/
`get_type_from_constructor_type_node` + `build_signature_from_function_like_type_node`
含参数/rest/optional 处理）；`get_type_of_symbol` 扩展支持 `Property` flag 符号；
函数表达式/箭头函数参数类型推断（`get_type_of_function_like` 改用
`build_signature_from_function_like_type_node` 构建带参数符号的签名，relater
可检测参数类型不匹配 TS2322，如 `(x: string) => 1` 不可赋值给 `(x: number) => number`）；
contextual typing 注入箭头函数/函数表达式参数类型（`get_type_of_function_like` 获取
contextual function type 的 call signature，无注解参数继承对应位置的上下文参数类型；
两遍构建签名让返回值推断能看到 contextual 参数类型；推断时 push/pop 函数作用域使 body
可解析参数引用）；`set_parent_pointers` 后处理填充 AST `parent` 指针（激活 contextual typing
与 grammar checks 中所有 `node.parent` 访问，单线程 + 树形 AST 安全）。

- [x] 条件类型 `infer R` 解析。已完成：binder 将 infer 类型参数声明为
  ConditionalType 的 locals（`set_parent_pointers` 预处理 + `bind_type_parameter`
  检测 InferType 父节点 + `get_infer_type_container`/`declare_local_symbol`）；
  `get_type_from_infer_type_node` 解析为 TypeParameter 类型；`build_conditional_type`
  构建 ConditionalRoot 含 check/extends 类型和 infer 参数；`resolve_type_reference`
  支持 generic type alias 实例化（`type_argument_stack` 替换映射）；
  `resolve_conditional_type` 运行 `infer_types` 推断 infer 参数、替换 extends 与
  branch、push ConditionalType 到 scope stack 使 branch 可解析 infer 符号；
  `substitute_infer_type_parameters` 扩展支持 Object（数组）与 Tuple 类型（之前
  仅处理 union/intersection，导致 `(infer R)[]` extends 不被替换、条件恒走 false 分支）。
  18 个 conditional type parity 测试通过。
- [ ] 类型推断缓存（`node_links.resolved_type`）。（已通过 `type_node_links` 完成）
- [x] Fresh literal type widening（对象字面量在无 contextual type 时应 widening
  literal 属性）。已实现 `widen_initializer_type` +
  `widen_object_literal_type`：变量声明无注解时对初始化器类型做 widening，
  对象字面量递归 widen 每个属性（`{ a: 1 }` → `{ a: number }`），嵌套对象
  字面量也递归处理。有类型注解时不 widen（contextual typing 保留 literal）。
  7 条 widening parity 测试通过。
- [x] contextual typing 扩展到 CallExpression 参数（`get_contextual_type_for_argument`
  已具备，需把推断结果落回；当前仅变量声明初始化器走通）。已修复
  `check_function_like_body`：在 body 检查前先调用 `get_type_of_node(node)`
  触发 `get_type_of_function_like` → `get_contextual_type` →
  `get_contextual_type_for_argument`，使 call argument 位置的箭头函数/函数表达式
  参数能继承 callee 签名对应位置的参数类型。4 条 contextual typing parity 测试
  通过（含 TS2339 属性不存在检测、valid 属性访问、对象字面量 contextual mismatch）。
- [x] 成员访问类型检查（`x.toUpperCase()` 在 `x: number` 上应报 TS2339）。已实现
  `check_property_access`/`has_property_of_type`，覆盖对象字面量、原始类型、联合/
  交叉类型、类型参数约束、数组/元组 `length`、索引签名；binder 增补 `TypeParameter`
  符号声明；`get_type_of_function_like` 在 prime 参数前 push 作用域使类型参数注解
  可解析；`FunctionDeclaration` 处理改为先 `get_type_of_function_like` 再检查 body；
  relater `is_index_signatures_related_to` 在源无索引签名时回退到
  `members_related_to_index_info`（使 `{ a: 1 }` 可赋值给 `{ [key: string]: number }`）；
  flow 修复 `types_overlap`/`narrow_by_switch_on_discriminant_property` 支持 literal
  比较。16 条 TS2339 parity 测试全部通过。
- [x] 调用表达式参数类型检查（TS2345）。已实现 `check_call_arguments` 解析 callee
  类型并比对各参数类型与签名参数类型；覆盖 CallExpression/NewExpression、对象字面量
  参数、联合类型参数、arrow function callee；`get_type_of_class_declaration` 从
  constructor 构建构造签名并缓存到 class symbol；parser 修复 `new Foo('hi')` 被解析
  为 `new (Foo('hi'))` 的问题（unwrap trailing CallExpression）；parser 增补
  `Constructor` 声明解析（`constructor(...)` 现在生成 `ConstructorDeclaration` 而非
  `MethodDeclaration`）；binder 将 `Constructor` 加入 `is_block_scoped_container` 使
  构造函数参数存入 locals。16 条 TS2345 parity 测试全部通过。

### P3.9 Checker 控制流 narrowing

已完成：`narrowType`/`getNarrowedTypeOfSymbol`、null/undefined 排除、`typeof`
narrowing、truthiness narrowing、`instanceof`/`in` narrowing、discriminated union
narrowing（`obj.kind === "value"`）、assignment-driven type 更新、switch 语句
narrowing（`switch (x)`/`switch (obj.kind)`，含 default 子句）、type predicate
（user-defined type guard）narrowing（`if (isString(x))`/`if (!isString(x))`）、
optional chain containment narrowing（`if (x?.a)`/`x?.a === value`）、
equality narrowing for literal types（`==` loose 比较、enum 成员、literal
replacement、strict null/undefined 区分、false-branch unit removal）、
function return type widening（`return 42` → `number`）、function-like container
flow isolation（`bind_container` 为函数体保存/恢复 `current_flow`）、
typeof switch narrowing（`switch (typeof x)`，含 default 子句排除）、
parser fix（`typeof x` 现在解析为 `TypeOfExpression` 而非 `PrefixUnaryExpression`）、
switch (true) narrowing（`switch (true) { case cond: ... }`，含 default 子句
排除全部 case 条件、前序 case 条件取反）、
asserts x is T narrowing（assertion 函数，`asserts x` truthy 收窄、
`asserts x is T` 类型收窄、多参数支持、CALL flow 节点收窄）、
7+4+3+3+7+6+5+5 个 parity fixtures。

### P3.10 Checker nodebuilder

已完成：`nodebuilder.rs` 模块；`type_to_string`/`type_to_string_ex` 直接序列化
（intrinsic/literal/union/intersection/type parameter/indexed access/template
literal/tuple/array/reference/function/object literal/enum/symbol 类型）；
parenthesization for function types in unions/arrays；serialization level 递归保护；
5 个 type display parity fixtures（通过 TS2322 message_args 验证）。

- [ ] `symbol_to_type_node`/`symbol_to_display_parts`。
- [ ] hover 信息生成。

### P3.11 Checker emitresolver

已完成：`emitresolver.rs` 模块；`is_declaration_visible`、`get_enum_member_value`
（string/number/negative numeric）、`is_optional_parameter`、
`is_literal_const_declaration`、`get_constant_value`、
`is_referenced_alias_declaration`、`is_value_alias_declaration`、
`get_effective_declaration_flags`、`get_symbol_of_declaration`、
`is_const_enum_member`。

- [ ] 完整的 visibility tracking（alias marking visitor）。
- [ ] `isEntityNameVisible`/`isSymbolAccessible`（declaration emit 需要）。

### P3.12 Checker JSX / JSDoc / Grammar checks（已完成）

已完成：
- [x] 迁移 `internal/checker/grammarchecks.go`（P3.12a）：modifier 校验、
  parameter list 校验、break/continue target 校验、JSX grammar 校验
  （tag name、type arguments、duplicate attributes、comma operator）。
- [x] 迁移 `internal/checker/jsx.go`（P3.12b）：JSX 元素类型检查、component
  signature 检查（TS2604）、intrinsic element 校验、attribute 校验。
- [x] 迁移 `internal/checker/jsdoc.go`（P3.12c）：`check_unmatched_jsdoc_parameters`
  + `contains_arguments_reference`。JSDoc 检查在 P2.7（JSDoc parser）落地前为
  no-op，但 `contains_arguments_reference` 独立可用。

### P3.13 Diagnostics message 表

已完成：Go diagnostic message 表迁移（2154 条，`messages_generated.rs`）；
主要 code/category/message 对齐；`key_to_message`/`format_message` 占位符插值。

- [ ] `format_message` 行为完全对齐 Go（UTF-8 校验、无效索引 panic）。
- [ ] 本地化支持（locale/loc_generated）。

### P3.14 Checker parity fixtures（已完成）

- [x] type-check parity fixtures 覆盖最小闭环；`.js` + JSDoc 行为测试；JSX
  type-check smoke；累计至少 50 个 checker parity fixtures 通过（当前 126 个）。

验收：

- [x] `cargo test` 中出现 `check_source_file` 调用路径单测。
- [ ] Rust 能在 `ai-Color-toner` 项目上输出与 Go oracle 一致的诊断集合
  （数量级一致）。
- [x] 至少 50 个 checker parity fixtures 通过（当前 126 个）。
- [ ] `tsox -p tsconfig.json` 在真实项目上输出非空类型错误诊断。

## P4：Emit / Transformer / SourceMap / Declaration emit

目标：输出文件路径和内容与 Go oracle 一致，或差异被记录为有意差异。

Go 参考：`internal/compiler`、`internal/printer`、`internal/transformers`、
`internal/sourcemap`、`internal/outputpaths`。
Rust 现状：`src/compiler/mod.rs`、`src/printer/mod.rs`、`src/emitter/mod.rs`、
`src/sourcemap/mod.rs`。

- [ ] 对齐 JS emit：target、module、jsx、imports/exports、helpers。
- [ ] 对齐 declaration emit：`.d.ts`、`.d.ts.map`、strip internal、declaration maps。
- [ ] 对齐 sourcemap：路径、sources、sourcesContent、VLQ mappings。
- [ ] 对齐 output path：`rootDir`、`outDir`、`declarationDir`、mixed JS/TS。
- [ ] 补齐 transformer 体系或明确替代设计。
- [ ] 扩充 parity fixtures：CommonJS / ES modules / JSX preserve/react/react-jsx /
  decorators / enum/namespace / source maps / declaration emit。

验收：

- [ ] 输出文件路径和内容与 Go oracle 一致，或差异被记录为有意差异。
- [ ] emit parity 覆盖至少 30 个 fixtures。

## P5：Module Resolution / Package JSON / Bundled Libs

目标：常见 npm 包解析结果与 Go oracle 一致；bundled lib 相关诊断和 emit 不
依赖外部 TypeScript checkout。

Go 参考：`internal/module`、`internal/packagejson`、`internal/bundled`、
`internal/tspath`、`internal/nativepath`。
Rust 现状：`src/module/mod.rs`、`src/packagejson/mod.rs`、`src/bundled/mod.rs`、
`src/tspath/mod.rs`。

- [ ] 对齐 node/module resolution：classic、node10、node16、nodenext、bundler。
- [ ] 对齐 `paths`、`baseUrl`、`rootDirs`、`typeRoots`、`types`。
- [ ] 对齐 package `exports`、`imports`、`typesVersions`、`type`。
- [ ] 对齐 bundled libs 的加载方式和版本。
- [ ] 对齐大小写敏感文件系统行为。
- [ ] 增加 node_modules fixture parity。

## P6：Build / Watch / Incremental

目标：小型 project references fixture 与 Go oracle 行为一致；incremental
第二次构建能跳过未变更项目。

Go 参考：`internal/execute/build`、`internal/execute/incremental`、
`internal/execute/watchmanager`、`internal/fswatch`、`internal/project`。
流程审计见 `MIGRATION.md` 的 “Build Mode Flow Audit”。

已完成：`--build`/`-b` 外层 dispatch 对齐；`parse_build_command_line` +
`ParsedBuildCommandLine` + `BuildOptions`；build mode 中 `-v` 解析为 `verbose`；
空 project 默认 `"."`；非法组合（`clean+force`/`clean+verbose`/`clean+watch`/
`watch+dry`）拒绝；raw `references` DFS 桥接。

- [ ] 补齐 build parser 的 build-specific did-you-mean 和完整 watch options。
- [ ] 支持 Go 等价的 typed project reference graph。
- [ ] 支持 `.tsbuildinfo` 读写。
- [ ] 支持 incremental rebuild。
- [ ] 支持 watch mode，明确文件监听库选择。
- [ ] 对齐 project reference cycle、up-to-date 判断、输出跳过逻辑。
- [ ] 设计 watch 测试，避免 flaky。

## P7：Language Service / LSP

目标：VS Code extension 能指向 Rust binary 并完成 initialize；至少
hover/completion/diagnostics 三项通过 parity smoke。

Go 参考：`cmd/tsgo/lsp.go`、`internal/ls`、`internal/lsp`、`internal/project`、
`internal/fourslash`。
Rust 现状：`src/main.rs` 中 `--lsp` 仍为 not implemented。

- [ ] 选择 Rust LSP 栈：直接 JSON-RPC、`tower-lsp`，或自研最小协议层。
- [ ] 实现 `--lsp` 启动、stdio transport、initialize/shutdown。
- [ ] 迁移 project service 基础：open/close/change watched files。
- [ ] 逐步迁移 LS features：diagnostics / hover / completion / definition /
  references / rename / document symbols / formatting。
- [ ] 迁移 fourslash 测试策略，先只保留关键 smoke。

## P8：API / npm package / VS Code extension

目标：`npm run build` 或新 Rust build task 能产出可运行 binary；native-preview
包内 binary 可执行 `--version`；extension 能启动 Rust LSP smoke。

Go 参考：`cmd/tsgo/api.go`、`internal/api`、`_packages/native-preview`、
`_extension`、`_extension-nightly`、`Herebyfile.mjs`。
Rust 现状：`src/main.rs` 中 `--api` 仍为 not implemented；根 `package.json`、
extension、native-preview 仍是 Go 命名和构建链。

- [ ] 决定 Rust binary 名称：继续兼容 `tsgo`，还是使用 `tsox` 并提供 shim。
- [ ] 为 npm package 增加 Rust binary 构建/拷贝流程。
- [ ] 实现或替代 `--api` transport。
- [ ] 更新 native-preview package 的 bin、postinstall、README。
- [ ] 更新 VS Code extension 查找 binary 的逻辑。
- [ ] 保留 Go oracle 构建路径，直到 Rust parity 足够。

## P9：工具链、代码质量和发布

目标：CI 能在干净环境跑通 Rust checks；发布包不依赖本地 Go 构建产物。

- [ ] 增加 `rustfmt.toml` 或确认默认 rustfmt。
- [ ] 增加 clippy 策略：先允许迁移期 warning，逐步收紧。
- [ ] 建立 Rust codegen 命令，覆盖 AST、diagnostics、bundled libs。
- [ ] 更新 `.gitignore`，纳入 `target/`、生成产物、临时 baseline。
- [ ] 更新 CI workflow。
- [ ] 设计 benchmark：Go vs Rust CLI cold run、incremental、checker、emit。
- [ ] 发布前安全检查：license、NOTICE、third-party deps。

## 已知风险

- TypeScript 语义极大，必须依赖 oracle parity 和 baselines，不能靠局部单测
  判断完成度。
- Rust 所有权模型可能要求重构 AST/checker 数据结构，过早追求 idiomatic 可能
  导致行为漂移。
- LSP/project service 涉及并发和缓存，建议在 CLI/checker/emit 稳定后再大规模
  迁移。
- Go 仓库的生成脚本、baselines、native-preview 包装较多，迁移期间要避免同时
  改太多构建路径。

## record.warn 基线

2026-07-11 对 `/home/cqh/workspace/typescript-rust/record.warn` 聚合分析：

- 文件规模：3347 行，约 346 KiB；全是 `TS1003` parser syntax errors；96 文件。
- 来源分布：bundled libs 2895 条（86.5%）、项目源码 365 条（10.9%）、项目
  dist declaration 87 条（2.6%）。
- 最集中 bundled lib：`lib.es2015.iterable.d.ts` 1024、`lib.dom.d.ts` 947、
  `lib.es2015.collection.d.ts` 206、`lib.es5.d.ts` 113、`lib.es2015.core.d.ts`
  112、`lib.decorators.d.ts` 94。
- 最集中项目文件：`src/types.ts` 108、`src/OverlayComponents.tsx` 44、
  `src/ComposerPage.tsx` 34、`src/AiTestControls.tsx` 27、
  `src/AiTestControls.test.tsx` 23、`src/AiModelField.test.tsx` 22。
- 高频 token：`<`/`>` 泛型/JSX、`|`/`&` union/intersection、`[`/`]` 数组/
  tuple/indexed/mapped、`=>`/`(`/`)` arrow/function、`declare` ambient、
  `import { } from`、keyword type nodes。

结论：根因是 parser 对 TypeScript declaration/type 语法支持不足，非单点 panic。
P2.4/P2.5/P2.9 完成后 bundled libs 已零错误解析（3347 → 0）。`record.warn-1`
流程逻辑追踪：已修复 `files: []` 不触发默认 include、未显式 exclude 时默认排除
`outDir`/`declarationDir`、wildcard include 跳过常见 package dirs、literal
directory include 递归展开、`declare` modifier 前缀正确分派。

## Warning 状态

2026-07-11 已完成一轮低风险 warning 清理：清理明显未使用 import、未使用
变量、重复 match arm、无意义 iterator 赋值。剩余 lib warning 约 31 个，归类
为迁移期可接受：

- Go/TypeScript 命名对齐：`DiagnosticsPresent_OutputsSkipped`、
  `BlockScoped`、`parse_bracketedList` 等。
- 暂未接入的迁移占位 API：`expected_json_type`、`next_auto_generate_id`、
  `compiler_diagnostic` 等。
- 需要单独设计的公开 re-export 冲突：`checker::RelationComparisonResult`。

`cargo fmt --check` 仍会因仓库既有未格式化文件失败；只格式化触碰的 Rust 文件，
整仓格式化单独排期。
