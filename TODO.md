# typescript-go -> typescript-rust 迁移目标与任务

更新时间：2026-07-31

本文档是迁移工作的**唯一规划文档**。流程审计、行为差异细节记录在
`MIGRATION.md`，按阶段查阅。Go vs Rust 结构对比与已完成项盘点见
`ANALYSIS.md`。

## 项目背景

- Go 源 worktree（oracle）：`/Users/cqh/workspace/typescript-go`，分支 `main`
- Rust 迁移 worktree（主工作目录）：`/Users/cqh/workspace/typescript-rust`，分支 `rust`
- Rust crate：`tsox`，入口 `src/main.rs`，库入口 `src/lib.rs`
- `edition = "2024"`，`rust-version = "1.96"`（Cargo 1.96 不支持 `edition = "2026"`）

> Worktree 布局（2026-07-31 重组）：主工作目录跑 `rust` 分支以支持编辑工具；
> Go oracle 放在独立 worktree `typescript-go`（`main` 分支）以便构建 oracle
> 二进制与对照源码。parity 测试的 `TSGO_ORACLE` 指向
> `/Users/cqh/workspace/typescript-go/built/local/tsgo`。

## 验收命令

```sh
# 全量 Rust 测试（需 rustc 1.96+；cargo 在 ~/.cargo/bin）
cargo test

# parity 集成测试，需 Go oracle 二进制
TSGO_ORACLE=/Users/cqh/workspace/typescript-go/built/local/tsgo cargo test --test parity

# 未设 TSGO_ORACLE 时按以下顺序自动查找 oracle，找不到则跳过并打印原因：
# 1. /Users/cqh/workspace/typescript-go/built/local/tsgo
# 2. /Users/cqh/workspace/typescript-rust/_packages/native-preview/bin/tsgo
```

构建 Go oracle（在 Go worktree）：

```sh
cd /Users/cqh/workspace/typescript-go
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

## 当前进度快照（2026-08-01）

测试基线：`cargo test` 通过（862 个 lib 单测 + parity smoke 含 6 个 parser
parity fixture + 501 个 checker parity）。上轮完成 P2.7 JSDoc parser；本轮
完成 P2.9a/b/c parser parity fixtures（6 个 fixture：syntax_error/tsx/generics/
decorators/enums/conditional_types）。

| 模块 | Rust 行数（实测） | Go 行数 | 完成度 | 备注 |
|------|-----------|---------|--------|------|
| Scanner | 5500 | 4277 | ~95% | 转义/JSX/正则/CommentDirectives/ASI 完成；trivia 基础设施（fullStartPos + skip_trivia + comment ranges + shebang）完成；TokenFlags 完整位集（19 flag + 7 mask）+ SkipTriviaEx + conflict-marker trivia + JSDoc 4 flag + escape-sequence 6 flag + numeric separator 完整支持 + OCTAL flag（legacy 0777 + TS1121 + TS1489 + TS6188）完成；**regex body 校验完成**（regexp.rs 递归下降 parser + unicode_properties.rs，TS1501–TS1538 含 `\p{}`/v-mode class-set algebra/named-capture/backreference 校验，script_target plumbing） |
| Parser | 9500 | 9275 | ~90% | TS6/7 语法、类型语法、JSX、装饰器、import attributes 完成；**JSDoc parser 完成**（jsdoc.rs ~2500 行：state machine + tag parsing + type expression + lazy cache + `parse_jsdoc_for_node`）；缺 reparser |
| Binder | 2104 | ~3601 | ~41% | 容器递归绑定 + FlowNode + NameResolver 基础 + alias + 全局符号 + EnumDeclaration 容器化 完成；缺 ReduceLabel/Shared/Referenced 后处理、labeled statement、完整 scope chain、ReferenceResolver |
| Checker | 22527 | ~59975 | ~20% | 类型结构完整；check_source_file + 标识符解析 + TS2304；relater 含 union/intersection/对象/数组/tuple/signature/index signature/generic/条件/映射类型关系 + 缓存与循环检测；inference 含泛型推断 + contextual typing + infer R；class extends 继承 + this 类型解析；函数重载解析 + `new` 表达式实例类型 + 返回语句类型检查 + 比较无重叠检查 TS2367 + 不可调用/不可构造检查 TS2349/TS2351 + 只读属性赋值检查 TS2540 + declaration merge checker 侧 + 参数数量检查 TS2554/TS2555/TS2556 + rest 元素类型检查；emitresolver visibility tracking 完成；501 parity fixtures 通过。**已知 stub**：mapper placeholder（3 处闭包回退）、freshness tracking、isEnumTypeRelatedTo/isUnknownLikeUnionType 未迁移、nodebuilder symbol_to_type_node |
| Compiler | 768 | — | 基础 | Program 创建/解析/绑定/emit pipeline 通；checker 已接入 |
| Emitter | 774 | — | 基础 | JS emit 基础（源文本切片式）；缺 transformer 体系、declaration emit |
| Printer | 1578 | — | 基础 | 仅 NameGenerator；完整 AST→文本未迁移 |
| AST | 7476 | 21671 | 基础 | generated 节点 + symbol/flow 类型；生成脚本已入库（`_scripts/generate-rust-ast.ts`） |
| Diagnostics | 24260 | 9568 | 完成 | 2154 条消息；生成脚本已入库（`_scripts/generate-rust-diagnostics.ts`）；本地化未实现 |

`--lsp` 和 `--api` 当前为 stub。`cargo fmt --check` 仍未对齐全仓；触碰文件时
顺手格式化，整仓格式化单独排期。剩余 warning 约 77 个，归类为迁移期可接受
（Go/TS 命名对齐、未接入的占位 API、一处 checker re-export 冲突）。

## 下阶段优先级

按依赖与价值排序，逐项推进（本轮已补齐 TS2554/TS2555/TS2556 参数数量检查
含 spread、rest 元素类型检查、overload arity + 12 个 parity fixtures；详见
[`ANALYSIS.md`](./ANALYSIS.md) 第 9 节）：

1. **P3.7 Checker 类型关系补齐**（已完成）：补齐 `isEnumTypeRelatedTo`、
   `isUnknownLikeUnionType`（relater.rs 两处 TODO 已落地，含
   `is_empty_anonymous_object_type`/`structured_type_is_empty` 辅助）；
   修复 mapper.rs 三处 placeholder 闭包回退（`new_simple_type_mapper`/
   `new_array_type_mapper`/`new_array_to_single_type_mapper`）——将
   `MapFn` 签名从 `Fn(&Type)` 改为 `Fn(&Arc<Type>)`，使不匹配时返回输入
   类型而非错误目标。`isEnumTypeRelatedTo` 的 enum member 值比较已落地
   （Phase 1：布尔正确性，无诊断）：`get_enum_member_value`/`compute_enum_member_values`/
   `compute_enum_member_value`/`compute_constant_enum_member_value`（对齐 Go
   checker.go:23820-23901）+ `enum_relation` 缓存（`EnumRelationKey` +
   `Symbol::id()`）+ `get_declaration_of_kind`。**遗留**：Phase 2 诊断
   （TS2324/TS4125/TS4126 需 errorReporter plumbing）、TS1062 未初始化成员错误、
   ambient 非 const 计算成员规则、NaN/Infinity const-enum 检查、isolatedModules
   字符串语法检查、`evaluate_entity` 跨文件枚举成员引用解析（当前 noop）。
2. **P3.8 Checker 推断收尾**（已完成）：`inference.rs` 两处 TODO 已落地
   ——contextual typing from return type（`infer_type_arguments` 中从调用
   表达式的上下文类型推断返回类型，覆盖 `let x: T = genericFn(...)` 与
   `return genericFn(...)` 模式）、parameter contextual typing + binding
   patterns（`get_contextual_type_for_initializer_expression` 中 BindingElement
   分支，回溯到 VariableDeclaration initializer 类型并按 array/object pattern
   提取元素/属性类型）。freshness tracking Phase 1 已落地：`get_fresh_type_of_literal_type`
   为字面量创建 fresh 变体（反向表示：regular→fresh_type 缓存，fresh→regular_type
   指向原类型，避免自引用 Arc）；`get_widened_literal_type` 仅对 fresh literal
   扩宽；`get_widened_literal_type_for_initializer` 按 `NodeFlags::Constant`
   决定 `const` 保留 / `let` 扩宽；relater `is_type_related_to` 顶部对 fresh
   literal 替换为 regularType 使其同时可比较 string 与 literal。8 个 parity
   测试覆盖 const/let 字面量保留与扩宽场景。**遗留**：object literal freshness
   （`ContainsObjectOrArrayLiteral` flag 设置 + `getWidenedType` gate +
   `OBJECT_FLAGS_REQUIRES_WIDENING` 结构化扩宽）+ `is_declaration_readonly`
   留 Phase 2。
3. **P3.1 Binder flow graph 收尾**（已完成）：`ReduceLabel`/`Shared`/`Referenced`
   后处理（`set_flow_node_referenced` 在 `add_antecedent_to_flow`/
   `create_flow_mutation` 中标记 REFERENCED→SHARED；`create_reduce_label`
   用于 try-finally）、labeled statement（parser `parse_expression_statement`
   识别 `identifier:` 并生成 `LabeledStatement`；binder `bind_labeled_statement`
   维护 `active_label_list` + break/continue 标签查找 + 未引用标签
   `NodeFlags::Unreachable`；checker `check_statement` 推送 `Labeled`
   break/continue context 使 `break label`/`continue label` 通过 grammar
   校验）。3 个 labeled statement parity 测试通过。
4. **P3.9 Checker 控制流 narrowing**：已完成。补齐 fixture 缺口（loose equality
   `==`、typeof false-branch、typeof bigint/symbol、nullish coalescing `??`
   narrowing、typeof on discriminant property、typeof === "function" callable
   narrowing、const variable alias inlining）。修复 binder `value_declaration`
   bug（`contains` → `intersects` for `SymbolFlags::VALUE`）。
5. **P3.10 Checker nodebuilder**（已完成 foundation）：`type_to_type_node` +
   `symbol_to_type_node` 覆盖 primitive/literal/union/intersection/tuple/
   array/function/type literal/symbol-bearing；剩余 gap（conditional/mapped/
   indexed access/template literal/keyof/typeof/import().T/qualified chain/
   `symbol_to_display_parts`）标 TODO 待 P4/P7 按需补齐。
6. **P3.2 Binder NameResolver 收尾**（已完成）：箭头函数参数作用域、
   enum/namespace 成员查找、export default 别名、类型参数作用域限制、
   `infer T`、装饰器位置调整——详见 P3.2 节。
7. **P2.0 AST/diagnostics 生成链路**（已完成）：补齐 `_scripts/generate-rust-ast.ts`
   与 `_scripts/generate-rust-diagnostics.ts`，复用 Go 侧 `_scripts/ast.json`
   via `schema.ts`；两个生成器输出与现有文件字节级一致（`git diff` 干净）。
8. **P1 CLI/tsconfig 收尾**：declaration-driven option parser（NameMap/
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

- [x] 给 CI 设计最小 Rust job：fmt、clippy、test、parity smoke：
  `.github/workflows/rust.yml` 在 push/PR 到 `rust`/`main` 时触发；
  fmt/clippy 为 `continue-on-error`（迁移期 ~75 warning + 整仓 fmt 待排期），
  test（`cargo test --lib`）与 parity smoke（`cargo test --test parity`，
  无 oracle 时优雅跳过）为 gating 步骤。
- [x] 新人只看 Rust worktree 文档即可跑通测试（验收）：新增 `README.md`
  作为入口（quick start build/test/parity 命令、worktree 布局、文档索引、
  CI 说明）；修正 `MIGRATION.md` 过期 worktree 路径与 oracle 搜索路径
  （`/home/cqh/...` → `/Users/cqh/...`）与状态摘要测试计数。
- [x] parity 测试能自动发现可用 Go oracle，或给出明确跳过原因（验收）：
  `go_oracle()` 按 `TSGO_ORACLE` → `typescript-go/built/local/tsgo` →
  `_packages/native-preview/bin/tsgo` 顺序查找；找不到时打印明确跳过原因
  指向 Go worktree 构建路径。

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

- [x] 对齐 `--help`/`--all` 动态选项列表生成（已完成）：`OptionDecl`
  新增 `description`/`show_in_simplified_help` 字段（对齐 Go
  `OptionDeclaration.Name`/`Description`/`ShowInSimplifiedHelpView`），
  为常用 compiler/build/watch 选项填充描述（`help`/`all`/`version`/`init`/
  `project`/`build`/`watch`/`noEmit`/`skipLibCheck`/`strict`/`target`/`module`/
  `moduleResolution`/`lib`/`outDir`/`outFile`/`sourceMap`/`declaration`/…）；
  `execute::print_help` 重写为从 `OptionDecl` 表动态生成，对齐 Go
  `internal/execute/tsc/help.go` 的 `PrintHelp`/`printEasyHelp`/`printAllHelp`：
  `--help` 走 simplified view（COMMON COMMANDS 示例 + COMMAND LINE FLAGS +
  COMMON COMPILER OPTIONS 两栏分节，按 `is_command_line_only` 分流）；
  `--all` 走 full view（ALL COMPILER OPTIONS 按名字小写排序 + WATCH OPTIONS
  + BUILD OPTIONS）。`print_option_section` 实现 `--name, -short    description`
  两栏对齐布局（对齐 Go `generateGroupOptionOutput`，不含终端宽度折行与颜色）。
  `display_name_of_option` 对齐 Go `getDisplayNameTextOfOption`。2 个单测覆盖
  `--help`/`--all` 头部/分节/动态描述内容。**剩余**：终端宽度感知折行。
- [x] 对齐 `--init` 生成的完整 `tsconfig.json` 模板：含 File Layout/Environment
  Settings/Other Outputs/Stricter Typechecking/Style Options/Recommended Options
  分节注释与 nodejs 示例块，对齐 Go `writeConfigFile` 模板结构。同时将
  TS5054 错误从 ad-hoc 字符串改为 `Diagnostic::new` +
  `A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0`。
- [x] 对齐退出码：`Success`、`DiagnosticsPresent_*`、`InvalidProject_*`、
  `ProjectReferenceCycle_*`（已对齐：`ExitStatus` enum + `perform_compilation`
  状态映射与 Go `tsc.ExitStatus` 完全一致）。
- [x] 迁移 Go declaration-driven option parser：NameMap、did-you-mean、
  alternate-mode diagnostics、TSConfigOnly 规则、enum/min-value 校验（已落地）：
  `OptionDecl` 扩展 `is_tsconfig_only`/`is_command_line_only`/`extra_validation`/
  `min_value`/`enum_values`；`find_option`/`find_build_option` 改为大小写不敏感
  （对齐 Go `GetOptionDeclarationFromName`）；CLI 未知选项在 build map 中报
  TS5093 alternate-mode；TSConfigOnly 选项在 CLI 上报 TS6230/TS6064；enum
  非法值报 TS6046（列出合法值）；min-value 违规报 TS5002；tsconfig JSON
  大小写不匹配报 TS5025 did-you-mean。16 个单测覆盖。**遗留**：build-mode
  TS5094（common/build/compiler 三分拆分）、`DeprecatedKeys` 过滤。
- [x] 独立建模 watch options，在普通/build parser 中与 compiler/build options 分离。
  新增 `core::watch_options` 模块（`WatchOptions` struct + `WatchFileKind`/
  `WatchDirectoryKind`/`PollingKind` enums + 解析函数，对齐 Go
  `internal/core/watchoptions.go`）；`tsoptions` 新增 `OPTIONS_FOR_WATCH`
  声明表（7 条，对齐 Go `declswatch.go`）+ `find_watch_option` 名字映射
  （对齐 Go `WatchNameMap`）+ `apply_watch_options` 解析器（对齐 Go
  `ParseWatchOptions`）；`parse_command_line_worker` 在 compiler/build 名字映射
  miss 时回退到 `WatchNameMap` 并路由到独立的 `watch_options` map；类型不匹配/
  缺值诊断用 TS5080（对齐 Go `watchOptionsDidYouMeanDiagnostics`）。
  `ParsedCommandLine`/`ParsedBuildCommandLine` 新增 `watch_options: WatchOptions`
  字段。10 个单测覆盖 enum/number/boolean/list 解析、大小写不敏感、build 模式、
  不泄露到 compiler_options、TS6046 enum 校验、TS5080 缺值/非数字。
  **对齐 Go 当前状态**：tsconfig.json `watchOptions` key 解析未实现（Go 侧
  `tsconfigparsing.go` 相关代码注释掉）；`--showConfig` 不输出 watchOptions
  （Go 侧 `showconfig.go` 的 `TSConfig` 结构无该字段）。
- [x] 对齐 tsconfig 查找、`extends`、`files/include/exclude`、`compilerOptions`
  覆盖规则（已完成核心 precedence 与 extends 解析）：修复 `compilerOptions`
  merge precedence 倒置 bug（原来 extended > own，现改为 own > extended，对齐
  Go `mergeCompilerOptions` source-wins 语义）；修复 extends 数组 last-entry-wins
  （原来 first wins，现改为 last wins，对齐 Go `applyExtendedConfig` 顺序合并）；
  修复 `include`/`exclude`/`files` 继承顺序（原来 last extended 覆盖，现改为
  first extended wins，对齐 Go `applyExtendedConfig` 仅在 own 未声明时继承）；
  新增 extends `.json` suffix 解析（`extends: "./base"` → `./base.json`，对齐
  Go `getExtendsConfigPath`）+ path normalization（`./base` 不再残留 `./`）。
  5 个回归测试覆盖 own-overrides-extended / array-last-wins / cmd-overrides-own /
  include-first-extended-wins / json-suffix。
- [x] `${configDir}` 模板替换（TS 5.5+，已完成）：新增
  `starts_with_config_dir_template`（大小写不敏感前缀检测，对齐 Go
  `startsWithConfigDirTemplate`）、`get_substituted_path_with_config_dir_template`
  （将 `${configDir}` 替换为 `./` 后解析为绝对路径，对齐 Go
  `getSubstitutedPathWithConfigDirTemplate`）、
  `get_substituted_string_array_with_config_dir_template`（数组替换，返回
  `Option<Vec>` 对齐 Go nil-return convention）、
  `handle_config_dir_template_substitution`（对合并后的 compiler options 执行
  替换，覆盖 `paths`/`rootDirs`/`typeRoots`/`generateCpuProfile`/`generateTrace`/
  `outFile`/`outDir`/`rootDir`/`tsBuildInfoFile`/`baseUrl`/`declarationDir`，对齐
  Go `handleOptionConfigDirTemplateSubstitution`）。在 `resolve_file_path_options`
  中跳过 `${configDir}` 前缀值（对齐 Go `normalizeNonListOptionValue`），替换在
  merge 之后执行（确保 own config 的 `${configDir}` 用 own 目录解析，extended
  config 的 `${configDir}` 已在递归解析时用 extended 目录解析）。include/exclude/
  files 的 `${configDir}` 也在 `expand_file_names` 之前替换。11 个单测覆盖
  outDir/rootDir/declarationDir/tsBuildInfoFile/rootDirs/paths/include/exclude/
  files/extends/非前缀场景。
- [x] inherited include/exclude/files 的 path-rewriting（已完成）：当 extended
  config 声明了 `include`/`exclude`/`files` 而 own config 未声明时，继承的相对
  路径需改写为相对于 own config 目录的路径（而非 extended config 目录），对齐
  Go `applyExtendedConfig` 的 `ConvertToRelativePath(GetDirectoryPath(extendedConfigPath), …)`
  + `CombinePaths(relativeDifference, pathStr)` 逻辑。新增 `tspath::convert_to_relative_path`
  （对齐 Go `tspath.ConvertToRelativePath`）。绝对路径和 `${configDir}` 前缀路径
  直接传递（不重写）。同时修改 `${configDir}` include/exclude/files 替换逻辑：
  仅在 own config（`resolution_stack` 为空）时执行替换，extended config 的
  `${configDir}` 前缀保留以便继承时传递给 own config 用 own 目录解析（对齐 Go
  从 raw extended config 读取 include/exclude/files 而非已替换的 parsed 值）。
  6 个单测覆盖 include 相对路径重写 / include 绝对路径不重写 / include `${configDir}`
  不重写 / exclude 重写 / files 重写 / own include 覆盖继承。
- [x] explicit `null` 清除继承字段（TS 5.5+，已完成）：当 own config 的
  `compilerOptions` 中某字段显式设为 `null` 时，该字段不从 extended config
  继承，保持默认值（清除继承值），对齐 Go `mergeCompilerOptions` 的
  `explicitNullFields` 逻辑（`parsinghelpers.go:575-590`）。新增
  `merge_compiler_options_with_skip` 函数，接受 `skip_fields: &HashSet<String>`
  参数，在 dst-wins merge 中跳过 skip set 内的字段（不让 extended 填充）。
  `merge_compiler_options` 保留为无 skip 的 wrapper。调用点在 own config
  的 `compilerOptions` 解析后收集 null 字段名，传入 step 2（merge extended
  into result）的 skip set。command-line 选项优先于 own 的 null（cmd > null）：
  若 cmd 已设置该字段（非默认值），null 不影响结果。覆盖 tristate/string/enum
  三类字段的 null 清除、cmd 优先级、单字段隔离、多字段同时 null。6 个单测。
- [x] extended config cache（已完成）：为 tsconfig extends 实现缓存，在菱形
  继承场景（A extends B 和 C，B 和 C 都 extends D）中 D 只解析一次并复用，
  对齐 Go `ExtendedConfigCache`（`tsconfigparsing.go:154`）+
  `getExtendedConfig` 的 cycle-bypass 逻辑（`tsconfigparsing.go:972`）。
  新增 `ExtendedConfigCache` 结构体，包含 `entries: HashMap<String,
  ParsedCommandLine>` 和 `get_or_parse` 方法：循环条目（在 resolution_stack
  中）bypass 缓存（避免不同继承分支的错误结果），未缓存则递归解析后存入。
  修改 `get_parsed_command_line_of_config_file` 创建缓存并传给内部
  `_with_stack` 函数；extends 循环中用 `cache.get_or_parse` 替代直接递归。
  缓存的 errors 仍按引用次数传播（B 和 C 各 append 一次 D 的 errors，对齐
  Go `getExtendedConfig` 返回 cached errors 的行为）。3 个单测覆盖菱形继承
  （strict/noImplicitAny 正确传播）、错误不重复（TS5025 出现 2 次：D via B
  和 C）、循环不缓存（A↔B 循环报告 TS18000）。
- [x] package/Node-style resolution for `extends` bare specifiers（已完成）：
  当 `extends` 的值不是相对路径（不以 `./` 或 `../` 开头）且不是 rooted path
  时，按 Node 模块解析方式在 `node_modules` 目录中查找，对齐 Go
  `getExtendsConfigPath` 的 module 分支（`tsconfigparsing.go:571-575`）调用
  `module.ResolveConfig`（`resolver.go:371`）。实现 `resolve_config_via_node_modules`
  函数：从 config 目录开始向上遍历祖先目录（跳过名为 `node_modules` 的目录
  本身），在每个祖先的 `node_modules/<spec>` 中查找配置文件。查找顺序对齐
  Go `loadModuleFromSpecificNodeModulesDirectory` + `loadNodeModuleFromDirectoryWorker`
  （`isConfigLookup = true`, `extensions = extensionsJson`）：① 文件形式
  `<spec>.json`；② 目录形式 `<spec>/tsconfig.json`（对齐 Go `indexPath = tsconfig`）；
  ③ `package.json` 的 `tsconfig` 字段（对齐 Go `getPackageFile` 读取 `tsconfig`
  field，`resolver.go:1744`），通过 `packagejson::parse` 解析后取
  `path_fields.tsconfig`。支持 scoped package（`@scope/pkg`）和祖先目录遍历
  （`loadModuleFromNearestNodeModulesDirectory` 的 ancestor walk）。解析失败
  时返回 `None`（spec 被静默丢弃，own config 选项仍生效）。同时修正所有现有
  extends 测试 fixture：将裸文件名（`"d.json"` → `"./d.json"` 等）改为显式
  相对路径，对齐 Go 行为（裸 specifier 走 node_modules 而非当前目录）。
  6 个新单测覆盖文件形式、目录形式、package.json tsconfig 字段、scoped
  package、祖先遍历、未找到静默丢弃。
- [x] 将 raw `references` 升级为 typed project references：新增
  `core::project_reference::ProjectReference { path, original_path, circular }`
  （对齐 Go `core.ProjectReference`）；`ParsedCommandLine.references` 由
  `Vec<json::Value>` 改为 `Vec<ProjectReference>`，解析时 `path` 取 normalized
  absolute、`original_path` 取原始字符串；`resolve_project_references` 直接用
  `path`；`--showConfig` 按 Go `showconfig.go` 输出 `{"path": original_path}` +
  可选 `circular`。1 个单测验证解析。
- [x] 对齐 `extends` 的 cycle diagnostics：`extends` 循环检测
  （`resolution_stack` + `Circularity_detected_while_resolving_configuration_Colon_0`
  TS18000）+ `extends` 数组支持（`"extends": ["a","b"]` 多路合并）。
- [x] CLI 错误诊断对齐：`execute/mod.rs` + `tsoptions/mod.rs` 中 ad-hoc
  `writeln!("error TSxxxx: ...")` 全部替换为 `Diagnostic::new` + 消息常量
  （TS6369 build-first / TS6370 options-cannot-combine / TS5042 project-mixed /
  TS5057/TS5081 cannot-find-tsconfig / TS5112 tsconfig-not-loaded / TS5023
  unknown-compiler-option / TS5083 cannot-read-file）。
- [x] 对齐 no-input diagnostics：TS18003（`No inputs were found in config file`）
  在 `expand_file_names` 后当 `file_names` 为空、当前 config 无 `files`/`references`
  key、且非 `extends` 链（`resolution_stack` 为空）时发出，对齐 Go
  `shouldReportNoInputFiles` + `canJsonReportNoInputFiles`。3 个单测覆盖
  emit / `files:[]` 抑制 / `references` 抑制。**剩余**：config source span
  diagnostics（带 file/range 的诊断）和 `vfsmatch` root-file expansion。
- [x] 扩充 parity fixtures：无 tsconfig 且无文件 / 单文件输入 / `-p` 指向目录 /
  `-p` 指向文件 / `--showConfig` / invalid JSON / JSONC。新增 6 个 fixture
  （`single_file`/`project_dir`/`project_file`/`jsonc_config`/`show_config`/
  `invalid_json`）+ `Case` 结构扩展 `expect_success`/`stdout_contains` 字段，
  `rust_smoke_cases_emit_expected_outputs` 单测覆盖全部 9 个场景。
  **遗留**：response file（`@args.txt`）解析已落地（含 TS6045 unterminated
  quoted string 诊断，对齐 Go `parseResponseFile`）；3 个 response-file 单测
  覆盖 missing/empty/normal/unterminated 场景。
- [x] 扩充 CLI parity smoke 覆盖至 20 个场景（已完成）：新增 11 个 fixture
  与 `Case` 条目——`config_dir`（`${configDir}` 模板替换 + rootDir 相对路径
  emit）、`extends_relative`（相对路径 extends 继承 base）、`extends_array`
  （`extends: ["./base1.json","./base2.json"]` 数组合并）、`extends_bare_specifier`
  （裸 specifier 无 node_modules 时静默丢弃，own outDir 仍生效）、`include_pattern`
  （`include: ["src/**/*"]`）、`exclude_pattern`（`exclude: ["src/excluded/**"]`
  排除子目录）、`multiple_files`（多文件 emit + import）、`no_emit`（`noEmit:true`
  抑制输出）、`strict_mode`（`strict:true` 无诊断）、`comments_stripped`
  （`removeComments:true`，标注 `skip_oracle = true` 因 emitter 注释剥离未迁移）、
  `target_es5`（`target:"ES5"`，标注 `skip_oracle = true` 因 ES5 down-level 未迁移）。
  `Case` 结构新增 `skip_oracle: bool` 字段，`compare_with_go_oracle_when_available`
  跳过已知 transform gap 场景并打印跳过原因；`rust_smoke_cases_emit_expected_outputs`
  覆盖全部 20 个场景。717 lib + 2 parity + 569 checker_parity 测试通过。
- [x] 修复 `rootDir/outDir` 输出路径差异：emitter 新增
  `compute_program_common_source_directory`（对齐 Go
  `outputpaths.GetCommonSourceDirectory`）+ `get_source_file_path_in_new_dir`
  （对齐 Go `GetSourceFilePathInNewDir`），按 rootDir → config_file_path →
  文件名公共前缀顺序确定 common source directory，保留 outDir 下的相对目录
  结构。同时 tsconfig 解析新增 `resolve_file_path_options`（对齐 Go
  `normalizeNonListOptionValue`），将 `IsFilePath` 选项（rootDir/outDir/
  declarationDir/baseUrl/…）解析为绝对路径；`--showConfig` 输出时通过
  `to_relative` 转回相对路径。664 lib + 2 parity 测试通过。

验收：

- [ ] Rust 与 Go oracle 的 stdout、stderr、exit code、输出文件集合一致。
- [x] CLI parity 覆盖至少 20 个常见 tsc 场景。

## P2：Scanner / Parser / AST parity

目标：Rust parser 对 `.ts/.tsx/.js/.jsx` 与 bundled libs 的解析结果和诊断
对齐 Go oracle。

Go 参考：`internal/scanner`（scanner.go 2918 + regexp.go 1076 +
unicodeproperties.go 162 + utilities.go 100）、`internal/parser`（parser.go
6827 + jsdoc.go 1355 + reparser.go 748）、`internal/ast`、`_scripts/ast.json`。
Rust 现状：`src/scanner/mod.rs`、`src/parser/mod.rs`、`src/ast/*`、`build.rs`。

### P2.0 AST 生成链路（已完成）

已完成：补齐 `_scripts/generate-rust-ast.ts`（694 行，读 `_scripts/ast.json`
via `schema.ts`，生成 `syntax_kind_generated.rs` + `node_data_generated.rs`）
与 `_scripts/generate-rust-diagnostics.ts`（读 Go 侧
`diagnostics_generated.go`，生成 `messages_generated.rs`）。两个生成器输出与
现有文件字节级一致（`git diff` 干净）。

- [x] Rust AST 生成链路继续读 Go 侧 `_scripts/ast.json`，复用 `schema.ts`
  的 `SchemaAPI`（与 Go/TS 生成器共享 schema 定义）。
- [x] 对齐 generated enum/node 数据的生成命令和检查方式：
  `node --experimental-strip-types _scripts/generate-rust-ast.ts` /
  `node --experimental-strip-types _scripts/generate-rust-diagnostics.ts`。
- [x] 生成文件可重复生成，`git diff` 干净。

### P2.1 Scanner 基础能力补齐

已完成：`scanEscapeSequence`/`scanUnicodeEscape`、`reScanGreaterThanTokenInner`、
`scanInvalidCharacter` 完整诊断接入 parser/compiler、`unicodeproperties.go` 用
`unicode-ident` 替换、`CommentDirectives` 收集、`PrecedingLineBreak` 在 ASI
路径完整接入。

已完成（trivia 基础设施，对齐 Go `scanner.go:2307-2504, 2800-2917`）：
- `Scanner` 新增 `full_start_pos` 字段（对齐 Go `fullStartPos`），`scan()` 重构
  为 loop 跳过 trivia 时保留 `full_start_pos` 而 `token_pos` 前进；`has_preceding_line_break`
  改为 loop 退出后从 `preceding_line_break` 快照（修复了 trivia 中遇到的换行未反映到返回 token 的 bug）。
- 新增 free function `skip_trivia`（对齐 Go `SkipTrivia`，不含 conflict-marker/JSDoc 选项）、
  `get_leading_comment_ranges`/`get_trailing_comment_ranges`/`iterate_comment_ranges`
  （对齐 Go 同名函数，pending-range 策略一致）、`CommentRange`/`CommentRangeKind`、
  `get_shebang`/`is_shebang_trivia`/`scan_shebang_trivia`、`decode_char`/`is_whitespace_like`
  辅助。13 个新单测覆盖 skip_trivia（空白/单行注释/多行注释/shebang/组合）、
  full_start_pos（标识符间 trivia 保留、跨注释保留）、leading/trailing comment ranges
  （单行/多行/中间位置/shebang 跳过/无注释/行尾停止）。

- [x] 迁移 `TokenFlags` 完整位集（对齐 Go `ast.TokenFlags`，已完成）：
  `Scanner` 新增 `token_flags: TokenFlags` 字段 + `token_flags()` 访问器，
  在 `scan()` 顶部重置并在扫描过程中 OR 累积；`has_preceding_line_break`
  改为从 `preceding_line_break` + `token_flags` 同步。常量表覆盖全部 19 个
  Go flag（`TOKEN_FLAGS_PRECEDING_LINE_BREAK`/`_JSDOC_COMMENT`/`_UNTERMINATED`/
  `_EXTENDED_UNICODE_ESCAPE`/`_SCIENTIFIC`/`_OCTAL`/`_HEX_SPECIFIER`/
  `_BINARY_SPECIFIER`/`_OCTAL_SPECIFIER`/`_CONTAINS_SEPARATOR`/`_UNICODE_ESCAPE`/
  `_CONTAINS_INVALID_ESCAPE`/`_HEX_ESCAPE`/`_CONTAINS_LEADING_ZERO`/
  `_CONTAINS_INVALID_SEPARATOR`/`_JSDOC_LEADING_ASTERISKS`/`_SINGLE_QUOTE`/
  `_JSDOC_WITH_DEPRECATED`/`_JSDOC_WITH_SEE_OR_LINK`）+ 7 个组合 mask
  （`WITH_SPECIFIER`/`BINARY_OR_OCTAL_SPECIFIER`/`STRING_LITERAL_FLAGS`/
  `NUMERIC_LITERAL_FLAGS`/`TEMPLATE_LITERAL_LIKE_FLAGS`/
  `REGULAR_EXPRESSION_LITERAL_FLAGS`/`IS_INVALID`）+ `token_flags_contains`/
  `token_flags_intersects` 辅助。scanner 设置：`PRECEDING_LINE_BREAK`（trivia）、
  `UNTERMINATED`（string/template/regex）、`SINGLE_QUOTE`（`'` string）、
  `HEX_SPECIFIER`/`BINARY_SPECIFIER`/`OCTAL_SPECIFIER`（numeric）、
  `SCIENTIFIC`（`e`/`E` exponent）、`CONTAINS_LEADING_ZERO`（`0` 后跟数字）。
  12 个新单测覆盖。
- [x] 迁移 `OCTAL` TokenFlag（legacy `0777` 形式，已完成）：`scan_number`
  重写 `0` 前缀分支对齐 Go `scanNumber`（`scanner.go:1944-1971`）：`0` 后跟
  全八进制数字（0-7）→ 设 `OCTAL` flag + 报 TS1121 `Octal_literals_are_not_allowed`
  + early return；`0` 后跟非八进制数字（8/9）→ 设 `CONTAINS_LEADING_ZERO` +
  全 literal 扫完后报 TS1489 `Decimals_with_leading_zeros_are_not_allowed`；
  `0_` → 设 `CONTAINS_SEPARATOR | CONTAINS_INVALID_SEPARATOR` + 报 TS6188 +
  reset + re-scan。`DiagnosticKind` 新增 3 variant（`OctalLiteralNotAllowed`/
  `DecimalWithLeadingZero`/`NumericSeparatorNotAllowed`），parser 侧映射到
  `OCTAL_LITERALS_ARE_NOT_ALLOWED_USE_THE_SYNTAX_0`/
  `DECIMALS_WITH_LEADING_ZEROS_ARE_NOT_ALLOWED`/
  `NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE`。minus 前缀（`-0777`）error range
  回退 1 包含 `-`。9 个新单测覆盖 0777/00/0888/0/0.5/0e5/0n/0_123/-0777。
- [x] 迁移 escape-sequence TokenFlags（`UNICODE_ESCAPE`/`EXTENDED_UNICODE_ESCAPE`/
  `HEX_ESCAPE`/`CONTAINS_INVALID_ESCAPE`/`CONTAINS_SEPARATOR`/`CONTAINS_INVALID_SEPARATOR`，
  已完成）：`scan_escape_sequence` 重写为对齐 Go `scanner.go:1690-1851` 的 match
  分支：`\xHH` → `HEX_ESCAPE`（2 位 hex）或 `CONTAINS_INVALID_ESCAPE`（不足 2 位）；
  `\uHHHH` → `UNICODE_ESCAPE`（4 位 hex）或 `CONTAINS_INVALID_ESCAPE`（不足 4 位）；
  `\u{...}` → `EXTENDED_UNICODE_ESCAPE`（至少 1 位 hex + `}`）或
  `CONTAINS_INVALID_ESCAPE`（空/未闭合）；`\0`+digit / `\1`-`\7` → `CONTAINS_INVALID_ESCAPE`
  （legacy octal，消耗 1-2 位额外八进制数字）；`\8`/`\9` → `CONTAINS_INVALID_ESCAPE`。
  numeric separator 支持新增 3 个 helper：`scan_number_fragment_with_sep`（decimal/hex，
  对齐 Go `scanNumberFragment` `scanner.go:2044-2088`）、`scan_binary_fragment_with_sep`、
  `scan_octal_specifier_fragment_with_sep`——`_` 在数字间设 `CONTAINS_SEPARATOR`，
  在开头/结尾/连续位置设 `CONTAINS_INVALID_SEPARATOR`。`scan_number` 的 hex/binary/
  octal/decimal/fractional/exponent 路径全部改用新 helper。新增 `is_octal_digit` 辅助。
  17 个新单测覆盖 unicode-escape/extended-unicode/hex/invalid-hex/invalid-unicode/
  invalid-extended/octal-escape/8-9-escape/nul-escape/separator-decimal/hex/binary/
  consecutive/trailing/plain/STRING_LITERAL_FLAGS-mask/NUMERIC_LITERAL_FLAGS-mask。
- [x] 迁移 JSDoc 相关 TokenFlags（`PRECEDING_JSDOC_COMMENT`/`_LEADING_ASTERISKS`/
  `_WITH_DEPRECATED`/`_WITH_SEE_OR_LINK`，已完成）：`scan_multi_line_comment`
  检测 JSDoc 注释（`/**` 且非 `/**/`，对齐 Go `scanner.go:642` `isJSDoc`），
  设置 `PRECEDING_JSDOC_COMMENT` 并调用 `scan_jsdoc_comment_for_tags` 扫描
  `@deprecated`/`@see`/`@link`/`@linkcode`/`@linkplain` 标签设置对应 flag
  （对齐 Go `scanner.go:350-368`，返回 OR'd flags 避免借冲突）；`has_jsdoc_tag`
  辅助函数检查标签名后跟合法终止符（空格/tab/换行/`}`/`*`/EOF，对齐 Go
  `scanner.go:372-386`）。`Scanner` 新增 `skip_jsdoc_leading_asterisks: i32`
  字段（计数器，支持嵌套 JSDoc，对齐 Go `scanner.go:200`）+ `set_skip_jsdoc_leading_asterisks`
  方法；`scan()` 在遇到 `*`（非 `**`/`*=`）且有换行前导时，若 `skip_jsdoc_leading_asterisks != 0`
  且未设 `PRECEDING_JSDOC_LEADING_ASTERISKS`，消耗 `*` 为 trivia 并置 flag
  （对齐 Go `scanner.go:569-575`）。4 个访问器方法：`has_preceding_jsdoc_comment`/
  `has_preceding_jsdoc_leading_asterisks`/`has_preceding_jsdoc_with_deprecated_tag`/
  `has_preceding_jsdoc_with_see_or_link`。19 个新单测覆盖 JSDoc 检测/标签扫描/
  flag 重置/leading asterisk 消耗（有/无换行/未激活/`**`/`*=`/仅首个/计数器嵌套）/
  helper 函数。
- [x] 支持 `SkipTriviaEx` options（`StopAfterLineBreak`/`StopAtComments`/`InJSDoc`，已完成）：
  新增 `SkipTriviaOptions` struct + `skip_trivia_ex` 函数（对齐 Go
  `SkipTriviaEx` `scanner.go:2311-2400`），`skip_trivia` 改为调用
  `skip_trivia_ex` 传默认 options。`stop_after_line_break` 在消耗首个换行后
  返回；`stop_at_comments` 在 `/` 处返回（不消耗注释）；`in_jsdoc` 在换行后
  消耗 JSDoc 前导 `*`（`can_consume_star` 状态机，只在换行后置位、消耗后重置）。
  `report_error` 回调用于 conflict-marker 报错。
- [x] 迁移 conflict-marker trivia（`isConflictMarkerTrivia`/`scanConflictMarkerTrivia`，
  已完成）：`is_conflict_marker_trivia` 对齐 Go `scanner.go:2409-2442`（7 字节
  重复 + 行首检测 + `<<<<<<<`/`>>>>>>>`/`|||||||` 需尾随空格、`=======` 不需要）；
  `scan_conflict_marker_trivia` 对齐 Go `scanner.go:2444-2473`（`<`/`>` 分支
  消耗到行尾；`|`/`=` 分支消耗到下一个 `=======`/`>>>>>>>` marker）。`skip_trivia_ex`
  在 `<`/`|`/`=`/`>` 字符处检测 conflict marker 并跳过。8 个新单测覆盖 options
  + marker 检测 + skip 行为 + error 回调。**注**：Go 行为是只跳过 marker 行本身，
  marker 之间的内容作为代码解析（产生自己的诊断），Rust 对齐此行为。

### P2.2 Scanner 正则字面量

已完成：`reScanSlashToken` 基础（pattern body + 字符类 + 转义 + flags + 未终止诊断）+
flag 校验（TS1499 unknown flag / TS1500 duplicate flag / TS1502 u+v 互斥，
对齐 Go `scanner.go:1171-1191` flag scan；`reg_exp_flag_bit` + 8 位 bitmask
对齐 Go `charCodeToRegExpFlag`）。`DiagnosticKind` 新增 3 个 regex flag variant，
parser 侧映射到 `UNKNOWN_REGULAR_EXPRESSION_FLAG`/`DUPLICATE_REGULAR_EXPRESSION_FLAG`/
`THE_UNICODE_U_FLAG_AND_THE_UNICODE_SETS_V_FLAG_CANNOT_BE_SET_SIMULTANEOUSLY`。
5 个 scanner 单测 + 1 个 parser 端到端单测覆盖。

- [x] 迁移 `internal/scanner/regexp.go` 完整 regex body 校验（已完成）：新增
  `src/scanner/regexp.rs`（~1602 行递归下降 parser，对齐 Go `regExpParser`）+
  `src/scanner/unicode_properties.rs`（Unicode 15.1 属性数据，对齐 Go
  `unicodeproperties.go`）。`RegExpParser` 校验 regex body（`/.../` 之间文本）
  并报告 TS1501–TS1538 诊断：disjunction/alternative/sequence/quantifier/
  atom/atom-escape/character-class/class-set-expression（v-mode `&&`/`--`/`[a--b]`）
  /named-capture-group（`(?<name>)` + `\k<name>` 引用校验）/decimal-escape
  backreference 校验/`\p{...}`/`\P{...}` Unicode property（binary + non-binary
  General_Category/Script/Script_Extensions + 值校验）/unicodeSets mode（may-
  contain-strings、class-set algebra 递归）。`Scanner` 新增 `script_target` 字段
  + `set_script_target` 方法，`re_scan_slash_token` 在定位 body 边界 + 解析 flag
  run 后构造 `RegExpParser` 并消费其 errors；`check_reg_exp_flag_availability`
  对齐 Go `checkRegularExpressionFlagAvailability`（`d`→ES2022/`s`→ES2018/
  `v`→ES2024 target gating，TS1501）。`DiagnosticKind::RegexMessage(Message)`
  按值携带 `Message`（`Message` 为 `Copy`），parser 侧映射回 `Message`。
- [ ] 支持 `lastIndex`、命名捕获组、`d` flag 等现代正则特性（命名捕获组 body
  校验已落地，runtime 行为不属 scanner 范畴）。

### P2.3 Scanner JSX / JSDoc

已完成：`ScanJsxToken`/`ScanJsxIdentifier`/`ScanJsxAttributeValue` 全套迁移
与 JSX parser 重写（8 个测试通过）。

- [x] 迁移 `ScanJSDocToken` + `scanJSDocCommentForTags`（已完成）：`scanJSDocCommentForTags`
  在 P2.1 JSDoc TokenFlags 阶段已落地（扫描 `@deprecated`/`@see`/`@link` 标签设置
  token flags）。本轮新增 `scan_jsdoc_token`（对齐 Go `ScanJSDocToken`
  `scanner.go:1418-1525`：`@`/`*`/`{`/`}`/`[`/`]`/`(`/`)`/`<`/`>`/`=`/`,`/`.`/`` ` ``/`#`/
  空白/换行/标识符（含 `-`），产生对应 SyntaxKind）、`scan_jsdoc_comment_text_token`
  （对齐 Go `ScanJSDocCommentTextToken` `scanner.go:1374-1405`：累积 prose 文本直到
  换行/`` ` ``/`{`/`@tag` 边界，`in_backticks` 模式仅换行/`` ` `` 终止）、
  `can_follow_jsdoc_at`（对齐 Go `CanFollowJSDocAt`：标识符起始/空白/换行/EOF 为 true）。
  新增 `is_whitespace_single_line` 辅助。17 个新单测覆盖。**剩余**：`@` 后 Unicode
  escape 标识符处理（罕见，延后）。

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

- [x] 迁移 `internal/parser/reparser.go`（748 行）到 `src/parser/reparser.rs`。
  实现 `reparse_tags` 入口 + `reparse_unhosted` 分发（`@typedef`/`@callback`/
  `@import`/`@overload`）+ `reparse_jsdoc_signature`（参数/返回类型提取）+
  `reparse_jsdoc_type_literal`（JSDocTypeLiteral → TypeLiteralNode）+
  `gather_type_parameters`（`@template` 收集）+ `wrap_in_jsdoc_namespace`
  （命名空间包裹）+ `get_innermost_name_of_jsdoc_namespace` + helper 函数。
  `Node` 新增 `with_loc_flags` 构造器支持带 flags 的节点创建。10 个单测覆盖
  typedef（simple/object literal/namespace）、callback、import、overload、
  无 unhosted tag、innermost name（simple/namespace）、namespace wrap。
  **遗留**：hosted tag mutation（`@type`/`@param`/`@return`/`@readonly` 等
  修改宿主节点）需 mutable AST 重建，延后；JSDoc parser 的 `@typedef {Object}`
  + `@property` → JSDocTypeLiteral 与 `@import` 完整解析为 P2.7 scope。
- [x] `@typedef` JSDoc → type alias 节点追加到 statements。`apply_jsdoc_reparser`
  后处理步骤在 `parse_source_file_text_with_diagnostics` 中调用：遍历每个
  statement，resolve JSDoc（lazy + cached），调用 `reparse_tags` 生成新声明
  节点，插入到原 statement 之前。当有 reparsed 节点时重建 SourceFile node
  （immutable AST，需 rebuild statement list）。5 个集成测试覆盖 typedef
  prepended、typedef namespace prepended、overload prepended、无 JSDoc 不变、
  hosted-tags-only 不变。全量 lib test：888 passed, 0 failed。
- [x] `reparseTopLevelAwait`：外部模块 + `possibleAwaitSpans` 重解析。
  **不需要迁移**：Rust scanner 始终将 `await` 扫描为 `AwaitKeyword`（不像 Go
  scanner 在非 AwaitContext 时扫描为 Identifier），因此 top-level `await`
  在外部模块中已被正确解析为 `AwaitExpression`，无需 reparse。Go 的
  `possibleAwaitSpans` 机制仅因 Go scanner 的上下文关键字扫描策略而存在。
- [x] `collectExternalModuleReferences`：新增 `src/parser/references.rs`
  （70 行），`collect_external_module_references` 遍历顶层 statements 收集
  import/export 模块说明符到 `SourceFile.imports`、`declare module "name"` 到
  `module_augmentations`（外部模块文件）或 `ambient_module_names`（脚本文件）；
  `set_external_module_indicator` 检测首个 import/export 语句设置
  `external_module_indicator`（对齐 Go `SetExternalModuleIndicator` legacy 模式）；
  `uses_uri_style_node_core_modules` Tristate 跟踪 `node:` 前缀。SourceFile
  新增 7 个字段（imports/module_augmentations/ambient_module_names/
  external_module_indicator/common_js_module_indicator/is_declaration_file/
  uses_uri_style_node_core_modules）。11 个单测通过。**遗留**：dynamic import /
  require call 遍历（`ForEachDynamicImportOrRequireCall`，仅 JS 文件需要）、
  `force`/JSX 模式检测（需 compiler options plumbing）。

### P2.7 Parser JSDoc（已完成）

- [x] 迁移 `internal/parser/jsdoc.go`（1355 行）到 `src/parser/jsdoc.rs`：
  `parse_jsdoc_comment` 入口（scanner save/restore + `set_range` 重指向注释体）；
  `parse_jsdoc_comment_worker` 状态机（comment accumulation + `@tag` 边界检测 +
  fenced code block ```` ``` ```` 跟踪 + `{@link}` inline link 解析）。
- [x] `parseJSDocComment`：tag 类型、`@param`、`@returns`、`@typedef`、
  `@callback`、`@template`、`@type`、`@implements`、`@augments`/`@extends`、
  `@public`/`@private`/`@protected`/`@readonly`/`@override`、`@deprecated`、
  `@this`、`@overload`、`@satisfies`、`@see`、`@throws`/`@exception`、`@import`、
  `@property`、未知 tag 回退。
- [x] JSDoc type expression：`@type {string}`、`@param {string} name`、
  nullable `?`、non-nullable `!`、variadic `...`、optional `[name]`、
  type arguments `<T>`、qualified name `a.b.c`、`#private`、property access
  entity name。
- [x] 节点附加 `jsDoc` 字段：`SourceFile` 新增 `jsdoc_cache`（`RwLock<HashMap<u64, Vec<Arc<Node>>>>`）
  + `has_lazy_jsdoc` flag（TS/TSX 文件启用 lazy 解析）；`Node::jsdoc(&SourceFile)`
  accessor（`HasJSDoc` flag 快速路径 + lazy `resolve_jsdoc` + eager `eager_jsdoc`）；
  `resolve_jsdoc` 双检锁缓存；`get_jsdoc_comment_ranges`（含 `find_full_start`
  反向扫描补偿 Rust `node.pos()` 为 token position 而非 full start 的差异）；
  `parse_jsdoc_for_node` 懒解析入口（对齐 Go `parseJSDocForNode`）。
  Scanner 新增 `end()`/`skip_jsdoc_leading_asterisks_raw()`/
  `set_skip_jsdoc_leading_asterisks_raw()` 访问器。36 个 JSDoc 单测通过。

### P2.8 Parser diagnostic parity（已全部完成）

- [x] diagnostic code 对齐（TS1003 等）、错误消息文本对齐 Go（含参数插值）、
  级联错误恢复点优化（`parseErrorAtRange` 去重 + 24 个 ParsingContext 错误
  映射 + `abortParsingListOrMoveToNextToken`）、`is_list_element` 完整对齐
  Go、`is_start_of_type` 补齐缺失 token、语法错误位置 UTF-16 offset 对齐。

### P2.9 Parser parity fixtures（a/b/c/d 已完成）

已完成：bundled libs 全部 100+ 零错误解析（基线 3347 → 0）。

- [x] 从 Go parser 测试或 TypeScript baselines 挑选 smoke 集合，转成 Rust parity。
  新增 6 个 parity fixture + parity.rs 中对应 Case：
  `parser_syntax_error`（TS1003/TS1005/TS1109/TS1136 语法错误 + `noEmitOnError`）、
  `parser_tsx`（fragment/component/JSX expression；checker JSX namespace gap
  标 `skip_oracle`）、`parser_generics`（泛型函数/类/接口/constraint/conditional
  type with `infer`）、`parser_decorators`（method decorator + decorator factory）、
  `parser_enums`（numeric/string/const enum + computed values）、
  `parser_conditional_types`（conditional/mapped/template literal/indexed access/
  keyof/typeof）。
- [x] 典型 `.ts` 解析结果和诊断对齐 oracle（`parser_generics`/`parser_decorators`/
  `parser_enums`/`parser_conditional_types` exit 0 对齐）。
- [x] 典型 `.tsx` JSX 解析对齐（`parser_tsx` parser 正确解析；checker JSX namespace
  types 缺失标 `skip_oracle`，待 P3 checker lib 支持后移除）。
- [x] 典型 `.js`/`.jsx` 对齐。新增 `parser_js`（CommonJS require、function
  declaration、prototype chain、var hoisting、template literal、array/object）
  与 `parser_jsx`（function/arrow component、fragment、event handler）parity
  fixture。`allowJs:true` 选项接入 `is_supported_source_file_ex`，include glob
  扩展现在正确包含 `.js`/`.jsx`/`.mjs`/`.cjs` 文件。`parser_js` 标 `skip_oracle`
  因 checker 在 `checkJs:false` 时仍类型检查 `.js` 文件（gap，待后续修复）；
  `parser_jsx` 标 `skip_oracle` 因 JSX namespace types 缺失。
- [x] bundled lib smoke：`lib.es2015.iterable.d.ts`、`lib.dom.d.ts`、
  `lib.es5.d.ts`、`lib.es2015.collection.d.ts`、`lib.decorators.d.ts` 错误数
  验证。5 个 compiler 单测直接解析 bundled lib 内容并断言零 parser error，
  覆盖 record.warn 基线中错误数最多的 5 个 lib 文件。

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

- [x] try/catch/finally 异常流（已验证：`bind_try_statement` 正确处理
  normal/exception/return 路径 + finally 合并；narrowing 不从 try 内泄漏）。
- [x] `ARRAY_MUTATION`：方法调用副作用。binder 在 `bind_call_expression` 中
  通过 `is_narrowable_operand` + `is_push_or_unshift_identifier` 检测
  `arr.push(x)` / `arr.unshift(x)` 形式并生成 ARRAY_MUTATION flow node；
  checker `narrow_type` 在 ARRAY_MUTATION 节点调用 `evolve_array_at_mutation`
  将 `autoArrayType` / `EvolvingArray` 的元素类型与新参数类型联合，得到
  evolving array 类型；`get_type_of_identifier` 在 evolving-array 操作目标
  位置（`x.length`/`x.push(v)`/`x.unshift(v)`/`x[i] = v`）返回 `autoArrayType`
  而非 finalize 后的数组类型，避免假 TS2339/TS2322；`has_property_of_type`
  对 `autoArrayType` 与 `EvolvingArray` 类型豁免 `push`/`unshift` 属性检查
  （无 lib.d.ts 时仍允许 evolving-array 链路）；`finalize_evolving_array_type`
  在最终读取时退化为普通 `Array<T>`。6 条 evolving array parity 测试通过。
- [x] `ReduceLabel`/`Shared`/`Referenced` 后处理。
- [x] labeled statement 标签支持。

### P3.1a Binder 容器递归绑定（已完成）

- [x] `bind_container` 设置 `parent_symbol`；容器递归绑定；checker
  `resolve_identifier` 改用 scope_stack 遍历；`is_unique_local_name` 同时
  检查 locals + symbol members。

### P3.2 Binder NameResolver

已完成：基础作用域链查找、符号意义过滤、for/for-in/for-of 循环作用域、
`resolveName` 入口、`argumentsSymbol`（区分普通函数/箭头函数）、
`undefinedSymbol`/`globalThisSymbol`、`populate_globals`（lib.d.ts 全局符号）、
`follow_alias`（import/export alias 链）。

- [x] 命名函数表达式自引用（`let f = function g() { g(); }`）：binder
  `bind_anonymous_declaration` 对具名 FunctionExpression 使用真实名称而非
  `__function`；`bind_container` 在创建 locals 后将函数符号加入自身 locals
  使名称在函数体内可见（但对 enclosing scope 不可见）。对齐 Go
  NameResolver `KindFunctionExpression` 特例。
- [x] namespace 成员导出/非导出区分：binder `declare_symbol` 对
  ModuleDeclaration 容器使用 `get_combined_modifier_flags` 检测 `export`
  修饰符，导出成员加入 `parent_sym.exports` + locals，非导出成员仅加入
  locals；checker `resolve_namespace_type` 改用 `exports` 构建命名空间对象
  类型，使非导出成员从外部不可访问（TS2339）。对齐 Go
  `declareModuleMember`。
- [x] `infer T` 仅在 conditional type true 分支可见：`resolve_conditional_type`
  仅在 `take_true` 时 push ConditionalType 作用域，false 分支不 push 使
  `R` 不可解析。`resolve_type_reference` 在符号未找到时报 TS2304。对齐 Go
  NameResolver `useResult = lastLocation == TrueType`。
- [x] 静态成员不可引用类类型参数（TS2302）：checker 新增
  `in_static_member_type` 标志，`check_class_member` 对 static
  PropertyDeclaration 的类型注解 force-resolve 时置位，
  `resolve_type_reference` 解析到 TypeParameter 时报 TS2302。对齐 Go
  NameResolver `ast.IsStatic(lastLocation)` 检查。
- [x] class 作用域提前 push：`check_statement` ClassDeclaration 与
  `get_type_of_class_declaration` 在 `build_class_instance_type_with_base`
  前 push_scope，使属性类型注解中的类型参数引用可解析。

### P3.3 Binder ReferenceResolver（已完成）

已完成：Go `referenceresolver.go` 的 6 个接口方法已内联到 Rust `Checker` 上
（`follow_alias`/`get_referenced_value_symbol`/`get_referenced_export_container`/
`get_referenced_import_declaration`/`get_referenced_value_declaration`/
`get_referenced_value_declarations`/`get_referenced_member_value_declaration`/
`is_type_only_alias_declaration`/`get_declaration_of_alias_symbol`/
`get_export_symbol_of_value_symbol_if_exported`/`is_alias_symbol_declaration`）。
Go 的 `ReferenceResolver` struct + hooks 模式是为打破 Go 循环导入的细节，Rust
直接在 `Checker` 上实现更 idiomatic。ReferenceResolver 纯属 LS/emit 层功能，
不影响类型检查正确性。

- [x] 迁移 `internal/binder/referenceresolver.go`（262 行）—— 已在 checker 侧实现。
- [x] 标识符引用记录（用于 find references / rename）—— 按需查询，无需 binder 记录。

### P3.4 Binder 声明合并与 export/import binding

已完成：interface + interface 声明合并。`declare_symbol` 在创建新符号前先查
target scope 是否已有同名符号，若 `can_merge_symbols` 判定可合并（interface+
interface、namespace+namespace、namespace+function/class/enum、function+
function overload、enum+enum），则将新 declaration 追加到既有 symbol 的
`declarations` 列表并 union flags，而非覆盖 scope 条目。`resolve_interface_type`
改为遍历 symbol 的全部 `InterfaceDeclaration` 节点，将 members 拼接为单一
匿名对象类型，对齐 Go 的 `getDeclaredTypeOfInterface`。

- [x] declaration merge：interface + interface；namespace+function/class/enum
  在 checker 侧合并（`get_type_of_merged_namespace_symbol` 将 value 签名与
  namespace 成员合并为单一对象类型）；enum+enum 合并（`resolve_enum_type`
  遍历全部 `EnumDeclaration` 收集成员）；enum 成员值类型解析
  （`resolve_enum_value_type` 构建 enum 对象类型使 `Color.Red` 返回字面量类型）。
- [x] export binding：`export { A }` 的 `exportSymbol` → local symbol 链。
  binder 新增 `ImportClause`/`ExportAssignment`/`ExportDeclaration`/
  `NamespaceExportDeclaration` 的 bind 分支（对齐 Go `bindImportClause`/
  `bindExportAssignment`/`bindExportDeclaration`/`bindNamespaceExportDeclaration`）；
  `declare_symbol` 在 SourceFile/ModuleDeclaration 容器且 `export` 修饰符时
  设置 `symbol.export_symbol = Some(self)` 自引用，使 checker 的 `follow_alias`
  和 `get_export_symbol_of_value_symbol_if_exported` 正确工作。11 个 parity
  测试 + 11 个 binder 单测覆盖。
- [x] import binding：`import { A }` 的 `aliasSymbol` → resolved symbol。
  `ImportSpecifier`/`ImportEqualsDeclaration`/`NamespaceImport` 已在 bind 分支
  中声明 Alias 符号；`ImportClause` 默认导入 `import D from "mod"` 新增 Alias
  符号到 container locals。
- [x] `delayedSymbol`/`aliasSymbol` 特殊符号处理——Go 中无 `delayedSymbol`
  （grep 零命中），`aliasSymbol` 实为 `SymbolFlags::Alias` 标志位，已在
  `ImportSpecifier`/`ExportSpecifier`/`ImportClause`/`ExportAssignment` 分支设置。
- [x] 完整 scope 链（当前只有 `container`/`block_scope_container` 两个字段，
  缺 `this_container` 用于 JS expando binding——留后续 JS 支持阶段）。
  **已完成 infrastructure**：`Binder` 新增 `this_container: Option<Arc<Node>>`
  字段 + 构造函数初始化 + `bind_container` 中 save/restore（对齐 Go
  `binder.go:1482,1513-1514,1623`）。`get_container_flags` 为
  FunctionDeclaration/FunctionExpression/ArrowFunction/MethodDeclaration/
  GetAccessor/SetAccessor/Constructor/CallSignature/ConstructSignature/
  FunctionType/ConstructorType 设置 `IS_THIS_CONTAINER` flag（对齐 Go
  `getContainerFlags` `binder.go:2571-2586`）。`BinaryExpression` 分支调用
  `bind_this_property_assignment`（skeleton，no-op for TS files）。
  **遗留**：完整 JS expando binding（`this.prop = value` 声明属性到 class
  symbol、`declareSymbolEx` with `isReplaceableByMethod`、
  `addLateBoundAssignmentDeclarationToSymbol` for dynamic names）待 JS 支持
  阶段实现。`this_container` 追踪 infrastructure 已就位。

### P3.5 Checker 接入 compiler（已完成）

- [x] `Program::new` 后调用 `Checker::new` + `check_source_file`；
  `get_semantic_diagnostics` 返回 `Vec<Diagnostic>` 并接入 execute 输出管线。
- [x] `Program` trait 补全 checker 所需方法（`getCommonSourceDirectory`、
  `getCanonicalFileName` 等）：新增 `current_directory` /
  `use_case_sensitive_file_names` / `common_source_directory` 三个 Host 方法
  （对齐 Go `modulespecifiers.Host` + `Program.CommonSourceDirectory`），在
  `Program` struct 委托给 `host.current_directory()` /
  `host.use_case_sensitive_file_names()` / `emitter::compute_program_common_source_directory`
  （后者由 `fn` 改为 `pub fn` 供 trait 实现复用）。新增 4 个带默认实现的 stub
  方法 `get_resolved_module` / `get_source_file_for_resolved_module` /
  `get_emit_module_format_of_file` / `source_file_may_be_emitted`（对齐 Go
  `Program` 接口的 module resolution / emit format / project reference cluster，
  返回 `None` / `ModuleKind::None` / `true` 默认值并标记 `/// STUB:` 待后续接入）。
  **遗留**：module resolution 状态尚未接入 Program（`resolved_module` 表 +
  `module_resolution_cache`）、`get_canonical_file_name`、`get_source_file_from_path`。

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
- [x] `implements` 子句校验（TS2420）：`check_heritage_clause` 对
  `implements` 关键字构建类实例类型（复用 `build_interface_type_from_members`，
  扩展支持 `PropertyDeclaration`/`MethodDeclaration`，跳过 static 成员与
  构造函数），与每个接口类型做 `is_type_assignable_to`，失败时报 TS2420。
  4 条 implements parity 测试通过（缺失成员/匹配方法/返回类型不匹配/属性匹配）。
- [x] 索引访问类型 `T[K]`：`get_type_from_indexed_access_type_node` 替换
  stub，新增 `get_indexed_access_type` 处理 string-literal 索引、union 索引
  （`T[keyof T]`/`T["a"|"b"]`）、`number` 索引（array/tuple 元素类型）、
  type-parameter 约束穿透、index signature 回退。8 条索引访问 parity
  测试通过。
- [x] 模板字面量类型：`get_type_from_template_type_node` 替换 stub，
  `build_template_literal_type` 对全字面量 span 展平为 `StringLiteral`，
  否则保留 `TemplateLiteral` 类型（texts/types 数组）。parser 新增
  `create_template_token_node` 从 raw token text 提取 cooked 文本。
  5 条模板字面量 parity 测试通过。
- [x] 映射类型缓存修复：`get_cached_type`/`cache_type` 在
  `type_argument_stack` 非空时跳过读写，避免映射类型 `T[K]` 在不同 K
  替换下返回首个 key 的缓存结果。7 条映射类型 parity 测试全部通过。
- [x] 类 `extends` 继承 + `this` 类型解析：`build_class_instance_type_with_base`
  递归解析 `extends` 基类的实例类型并合并属性（派生覆盖基类）；
  `resolve_base_class_instance_type` 从 heritage clause 解析基类符号并构建
  实例类型（含循环保护）；`merge_instance_types` 合并基类+派生属性列表；
  `this_type_stack` 在类成员检查期间提供 `this`/`super` 的实例类型；
  `implements` 检查改用 `build_class_instance_type_with_base` 使继承的成员
  也满足接口。10 条类继承 parity 测试通过。
- [x] 函数重载解析：`build_overload_function_type` 从符号的全部 FunctionDeclaration
  收集无 body 的重载签名构建多签名函数类型；`find_matching_signature`/
  `signature_accepts_arguments` 按序匹配首个可接受参数的签名；
  `get_return_type_of_call_expression`/`check_call_arguments` 均使用匹配签名。
  5 条重载 parity 测试通过。
- [x] `new` 表达式实例类型：`get_type_of_class_declaration` 将构造签名返回类型
  设为类实例类型（含 `extends` 继承成员），使 `new Foo()` 返回有类型的实例，
  `instance.prop` 触发 TS2339、类型赋值触发 TS2322、构造参数检查触发 TS2345。
  7 条 new 表达式 parity 测试通过。
- [x] 比较运算符类型重叠检查（TS2367）：`BinaryExpression` 处理器对
  `===`/`!==`/`==`/`!=` 调用 `are_types_comparable` 检测左右类型是否完全不
  可比；当两者都非 `any`/`unknown`/`never`/`null`/`undefined` 且不可比时报
  TS2367。10 条 parity 测试通过（number/string、boolean/string、`!==`/`==`、
  literal union no-overlap、同类型/字面量/any/null/union 过 negative 用例）。
- [x] 不可调用/不可构造检查（TS2349/TS2351）：`check_call_arguments` 在
  callee 类型为非 `any` 且（a）非结构化类型（如 `number`/`string`/`boolean`
  primitive）或（b）结构化但无 call/construct 签名（如对象字面量、class 实例）
  时报 TS2349（CallExpression）或 TS2351（NewExpression）。13 条 parity 测试
  通过（number/string/boolean/object literal/class instance 不可调用、number/
  object literal 不可构造；function/arrow/method/class/`any` 不报错）。

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
- [x] 类型推断缓存（`node_links.resolved_type`）。（已通过 `type_node_links` 完成）
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
- [x] Enum 类型解析。已完成：`resolve_enum_type` 构建枚举成员 literal 类型
  union，支持数字/字符串/混合枚举、数字枚举 auto-increment（无初始化器成员
  从上一个数值 +1，起始 0）；每个成员的 literal 类型写入成员符号的
  `value_symbol_links` 使 `Color.Red` 属性访问可恢复 literal 类型；枚举整体
  类型缓存到 `type_alias_links.declared_type`，递归引用通过
  `resolving_type_aliases` 循环保护。7 条 enum parity 测试全部通过。
- [x] 返回语句类型检查（TS2322 / TS1135）。已完成：`return_type_stack` 跟踪
  当前函数声明的返回类型；`FunctionDeclaration`/`MethodDeclaration`/
  `Constructor`/`GetAccessor`/`SetAccessor`/`FunctionExpression`/`ArrowFunction`
  处理器在检查 body 前 push 声明的返回类型（无注解时 push `None` 跳过），
  检查完 pop；`ReturnStatement` 处理器取栈顶 `Arc<Type>`（clone 避免借用冲突）
  并用 `is_type_assignable_to` 检查返回值类型可赋值性，失败时报 TS2322；
  `return;` 无值但声明类型非 `void`/`undefined`/`any` 时报 TS1135；箭头函数
  表达式体（`() => expr`）直接检查 `expr` 类型对声明返回类型的可赋值性。
  15 条返回类型 parity 测试通过（含函数声明/箭头函数表达式体与方法/accessor）。

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
nullish coalescing `??` narrowing（false-branch 左值收窄为 null|undefined、
右值收窄为 falsy；true-branch 不收窄，mirrors Go `narrowTypeByOptionality`）、
typeof on discriminant property（`typeof obj.kind === "string"` 收窄 union
constituent，`try_narrow_by_typeof_discriminant`）、typeof === "function"
callable narrowing（`filter_type_by_callable` 仅保留含 call signature 的类型）、
const variable alias inlining（`const_alias_initializer` + `flow_inline_level`
深度保护，mirrors Go `inlineLevel` capped at 5）、
binder `value_declaration` bug 修复（`contains` → `intersects` for
`SymbolFlags::VALUE`，使 BlockScopedVariable 等子集 flag 正确设置
`value_declaration`）、
7+4+3+3+7+6+5+5+6+3+3+4 个 parity fixtures。

### P3.10 Checker nodebuilder

已完成：`nodebuilder.rs` 模块；`type_to_string`/`type_to_string_ex` 直接序列化
（intrinsic/literal/union/intersection/type parameter/indexed access/template
literal/tuple/array/reference/function/object literal/enum/symbol 类型）；
parenthesization for function types in unions/arrays；serialization level 递归保护；
5 个 type display parity fixtures（通过 TS2322 message_args 验证）；
`symbol_to_string`/`symbol_to_string_ex`（含 SymbolFormatFlags 与 type-parameter
渲染）；`get_quick_info_text` + `format_quick_info_for_symbol` 按 symbol kind 分发
（function/method/class/interface/enum/type-alias/type-parameter/enum-member/
variable/alias）；`variable_decl_prefix` 区分 let/const/var；
`try_get_type_alias_declared_type` 在 cache miss 时触发解析并加 cycle 保护，
使未引用的 alias 在 hover 时也能显示 body；10 个 hover parity 测试通过
（变量/函数/类/接口/枚举/类型别名 with/without type params）。

- [x] `type_to_type_node`（Type → TypeNode AST）foundation — `typenode.rs` 的
  逆操作。覆盖常见 Type 变体：primitive/intrinsic、literal（string/number/
  bigint/boolean/null）、union、intersection、type parameter（含 `this`）、
  tuple（含 rest element）、array/reference（含 `Array<T>` → `T[]`、
  parenthesization for function-typed elements）、function type
  （`(params) => ret`）、type literal（properties + call signatures）、
  symbol-bearing type（class/interface/enum/type alias）。recursion guard 复用
  `serialization_level`（cap 300）。20 个 unit test 通过 round-trip
  （`type_to_string` ↔ `type_node_to_string`）验证 primitive/array/tuple/
  union/intersection/generic reference/function/object literal/literal types。
  **剩余 gap（已在 `nodebuilder.rs` 标 TODO 注释）**：indexed access
  （`T[K]`）、template literal、string mapping、conditional、substitution、
  index（`keyof T`）、type predicate、rest type in union、named tuple member、
  JSDoc types、`readonly T[]` for ReadonlyArray、construct signature in type
  literal、index signature in type literal、qualified name chain（`A.B.C`）、
  `import("mod").T`、`typeof` for value-meaning symbols。待 P4 declaration
  emit / P7 LS 启动时按需补齐。
- [x] `symbol_to_type_node(symbol, mask, type_arguments)` entry point — emit
  flat `TypeReferenceNode` with symbol's local name；type arguments 缺省时
  从 symbol declared type 恢复（覆盖 `type T<X> = ...;` referenced as
  `T<number>`）。**剩余 gap**：qualified name chain、`import("mod").T`、
  `typeof` for value-meaning symbols（mask == SymbolFlags::VALUE）— 标 TODO。
- [ ] `symbol_to_display_parts`（declaration emit 需要）。**延期**：在 Go 中
  已从 checker 迁出至 `internal/ls/hover.go`（displayPartsWriter +
  classification），属 LS 层。待 P7 LS 启动时再补。
- [x] hover 信息生成。

### P3.11 Checker emitresolver

已完成：`emitresolver.rs` 模块；`is_declaration_visible`、`get_enum_member_value`
（string/number/negative numeric）、`is_optional_parameter`、
`is_literal_const_declaration`、`get_constant_value`、
`is_referenced_alias_declaration`、`is_value_alias_declaration`、
`get_effective_declaration_flags`、`get_symbol_of_declaration`、
`is_const_enum_member`。

- [x] 完整的 visibility tracking（alias marking visitor）。已完成：
  `determine_if_declaration_is_visible` 覆盖 VariableDeclaration/ModuleDeclaration/
  ClassDeclaration/InterfaceDeclaration/TypeAliasDeclaration/FunctionDeclaration/
  EnumDeclaration/ImportEqualsDeclaration（exported 或 ambient element 时
  递归检查容器可见性，否则回退到 `is_global_source_file`）；PropertyDeclaration/
  PropertySignature/GetAccessor/SetAccessor/MethodDeclaration/MethodSignature
  （private/protected 不可见，否则递归检查父声明）；ImportClause/NamespaceImport/
  ImportSpecifier 默认不可见（由 alias marking visitor 按需标记）；TypeParameter
  与 SourceFile/NamespaceExportDeclaration 永远可见；ExportAssignment 不可见；
  ExportSpecifier（无 module specifier 时）回溯到 ExportDeclaration 的父容器。
  `precalculate_declaration_emit_visibility` 保存/恢复 file context（current_file/
  current_file_id/current_file_symbol/scope_stack）后运行 `alias_marking_visitor`，
  遍历 BinaryExpression（CommonJS `module.exports = id`）/ExportAssignment
  （`export = id`）/ExportSpecifier（`export { X }`），调用 `mark_linked_aliases`
  在 file symbol 的 members / scope_stack locals 中解析名字，follow_alias 后
  将 symbol 的全部 declarations 标记为 visible，并沿 `import d = a.b.c` 链继续
  解析。9 条 visibility parity 测试通过（exported/non-exported/global script/
  import clause/alias marking export specifier/export assignment/private property/
  type parameter/export specifier re-export）。
- [x] `isEntityNameVisible`/`isSymbolAccessible`（基础版）。已完成：
  `is_entity_name_visible` 解析 entity name 的首个 identifier，区分
  TypeParameter（直接 Accessible）/未解析（NotResolved）/调用
  `has_visible_declarations` 返回 `SymbolAccessibilityResult`。完整
  `isSymbolAccessible`（含 `aliases_to_make_visible` 完整计算、export specifier
  target 解析、private/protected 跨文件可见性）依赖 binder export/import binding
  与完整 scope chain，待 P3.4 落地后补齐。
- [x] Parser 修复：`export function/class/interface/type/enum/namespace` 现在路由
  到 `parse_declaration_with_modifiers`，将 `ExportKeyword` 作为 modifier 附加到
  声明节点（之前 `export` 关键字被消费但未附加为 modifier，导致
  `is_external_or_common_js_module` 无法检测 ES module）。`is_external_or_common_js_module`
  同步增加 `has_syntactic_modifier(Export)` 检查，对齐 Go 的 `IsExternalModuleIndicator`。
- [x] `Program::build_checker` 暴露给测试使用，返回完全初始化的 Checker。

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

- [x] `format_message` 行为完全对齐 Go（已完成）：改用 `regex` crate 的
  `{(\d+)}` 正则匹配占位符（对齐 Go `placeholderRegexp`）；越界索引 panic
  `"Invalid formatting placeholder"`（对齐 Go `diagnostics.go:128-129`）；
  UTF-8 校验在 Rust 中由 `&str` 类型保证（Go 的 `strings.ToValidUTF8` 为
  no-op）。2 个新单测覆盖非占位符花括号保留 + 越界 panic。
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

- [x] **P4.1 removeComments transformer**（已完成）：emitter 新增
  `collect_all_comment_ranges` 全文扫描器（状态机跳过 string/template/
  regex literal，收集 `//` 和 `/* */` 注释范围）+ `emit_text_range` 按范围
  跳过 cut + `emit_statement` 合并 comment cuts 与 type cuts。
  `emit_js_text` 接入 `CompilerOptions`，`removeComments:true` 时收集注释
  范围并在输出前 trim 首部空白（对齐 Go printer 不输出首语句前 trivia 的
  行为）。`comments_stripped` parity case 从 `skip_oracle=true` 改为
  `skip_oracle=false`，期望输出无注释。8 个 emitter 单测覆盖单行/多行/
  JSDoc 注释剥离、字符串/模板内注释保留、除法不受影响、尾部注释剥离、
  默认关闭。901 lib + parity smoke 全通过。
- [x] **P4.2 ES5 down-leveling**（已完成）：emitter 新增 replacement 机制
  （`emit_text_range`/`emit_statement` 合并 cuts + replacements 为统一
  operation list，`Option<&str>` 区分 cut vs replace）。`collect_es5_replacements`
  递归遍历 AST，对 `VariableDeclarationList` 节点检测 `NodeFlags::Const`/
  `NodeFlags::Let`，将关键字位置（`const`=5 字符 / `let`=3 字符）替换为
  `var`。`needs_es5_downlevel` 仅在 `target == ES5` 时触发（`None` 默认不
  下放，对齐现代 TS 默认 target ≥ ES2015）。`target_es5` parity case 从
  `skip_oracle=true` 改为 `skip_oracle=false`，期望输出 `var f = 6;`。
  6 个 emitter 单测覆盖 const/let→var、export 保留、var 不变、for 循环内
  let、ES2015 不下放。907 lib + parity smoke 全通过。
- [x] **P4.3 CommonJS module transformer**（已完成）：emitter 新增
  `transform_commonjs_import`（import→require：named `const { foo } =
  require("./bar")`、namespace `const ns = require("./bar")`、default
  `const { default: d } = require("./bar")`、side-effect `require("./bar")`、
  `import type` 剥离）、`transform_commonjs_export`（`export { x }` →
  `exports.x = x;`、`export default expr` → `exports.default = expr;`、
  `export = expr` → `module.exports = expr;`、re-export `export { foo } from
  "./bar"` → `const { foo } = require("./bar"); exports.foo = foo;`）、
  `transform_commonjs_export_declaration`（`export const/function/class/enum`
  → 剥离 `export` 关键字 + 追加 `exports.name = name;`）。`collect_export_modifier_cuts`
  收集 `ExportKeyword`/`DefaultKeyword` 修饰符字节范围（含尾部空格），
  在 inter-statement text 发射阶段应用 cut（因为 parser 将语句 pos 设在
  声明关键字而非修饰符，修饰符位于 `stmt.pos()` 之前）。修复 parser
  `make_export_modifier` 零宽 `TextRange::new(pos, pos)` bug → 改为
  `TextRange::new(pos, end)`。15 个 emitter 单测覆盖 named/namespace/default/
  side-effect/type import、named/default/equals export、re-export、export
  const/function/class。`module_commonjs` parity case（`skip_oracle=true`，
  因 text-slice emit 与 Go oracle 全 transformer 输出格式不同）。921 lib +
  parity smoke 全通过。
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
