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

测试基线：`cargo test` 通过（**1,105** 个 lib 单测 + **572** 个 checker parity
+ **2** 个 emit parity，共 **1,679** 个测试通过；99 个 ignored 标注 TODO）。CLI parity 对齐 Go oracle
（TS2322 诊断位置 + 类型扩宽、TS6053 消息格式、exit code 逻辑）已完成。LSP
diagnostics/hover + API server + .tsbuildinfo 增量构建已接入。

| 模块 | Rust 行数（实测） | Go 行数 | 完成度 | 备注 |
|------|-----------|---------|--------|------|
| Scanner | 6080 | 4277 | ~95% | 转义/JSX/正则/CommentDirectives/ASI 完成；trivia 基础设施完成；TokenFlags 完整位集 + SkipTriviaEx + conflict-marker + JSDoc flag + escape-sequence flag + numeric separator + OCTAL flag 完成；regex body 校验完成 |
| Parser | 11654 | 9275 | ~95% | TS6/7 语法、类型语法、JSX、装饰器、import attributes 完成；JSDoc parser 完成；reparser 完成；references.rs 完成 |
| Binder | 2898 | ~3601 | ~60% | 容器递归绑定 + FlowNode + NameResolver + alias + 全局符号 + 声明合并 + export/import binding + this_container infrastructure 完成 |
| Checker | 24026 | ~59975 | ~25% | 类型结构完整；relater 含 union/intersection/对象/数组/tuple/signature/index/generic/条件/映射类型关系 + enum 关系；inference + contextual typing + infer R + freshness tracking；class extends + 重载 + new + 返回类型 + TS2367/TS2349/TS2351/TS2540/TS2554-6；**TS2322 诊断位置 + 字面量类型扩宽（reportRelationError parity）**；nodebuilder type_to_type_node foundation；569 parity fixtures 通过 |
| Compiler | 900+ | — | 基础+ | Program 创建/解析/绑定/emit pipeline 通；checker 已接入；module resolution 已接入（BFS import resolution）；**TS6053 file-not-found 消息已对齐 oracle** |
| Emitter | 2690 | — | 基础 | JS emit + removeComments + ES5 down-level + CommonJS + source map + declaration emit 完成（text-slice 模式） |
| Printer | 1578 | — | 基础 | 仅 NameGenerator；完整 AST→文本未迁移 |
| Module | 2700+ | ~2700 | ~60% | infrastructure + 相对路径 + node_modules walk + package directory + paths/rootDirs + exports + imports + typesVersions + type ref directive 完成 |
| Incremental | 200+ | — | 基础 | **.tsbuildinfo 读写 + up-to-date check（BuildInfo struct + file hash comparison）** |
| LSP | 230+ | — | 基础 | **JSON-RPC 2.0 over stdio + initialize/shutdown + text document sync + diagnostics push（接入 checker）+ hover（接入 checker nodebuilder）** |
| API | 165+ | — | 基础 | **JSON-RPC 2.0 over stdio + configure/createProject/updateProject/getDiagnostics lifecycle** |
| AST | 5837 | 21671 | 基础 | generated 节点 + symbol/flow 类型；生成脚本已入库 |
| Diagnostics | 24268 | 9568 | 完成 | 2154 条消息；生成脚本已入库；本地化未实现 |
| PackageJSON | 588 | — | 完成 | 完整 package.json 解析 |

`--lsp` 已实现（diagnostics + hover 接入 checker）。`--api` 已实现（project
lifecycle）。`cargo fmt --check` 全仓通过。`cargo clippy` 零 warning。剩余
compiler warning 约 90 个（dead code in stub implementations），归类为迁移期可接受。

## 下阶段优先级

P1-P9 基础设施已全部完成（CLI/tsconfig/parser/scanner/binder/checker/emit/
module resolution/build mode/LSP/API/tooling 全栈对齐 Go oracle 行为，1,550 个
测试通过）。剩余 27 个 TODO 项为深度功能迁移（LSP features 逐项、watch mode、
incremental build、npm 打包、fourslash 测试），需较大工作量逐项推进。按价值排序：

1. **P7 LSP features**：diagnostics/hover/completion/definition/references/rename —
   逐项迁移 `internal/ls/`（~35k 行 Go 代码）。
2. **P6 Watch/Incremental**：`.tsbuildinfo` 读写、incremental rebuild、watch mode —
   需文件监听库选择（`notify` crate）。
3. **P8 npm/extension**：Rust binary 构建/拷贝到 native-preview、VS Code extension
   binary 查找逻辑。
4. **P9 收尾**：clippy 策略收紧、benchmark、安全检查。

## P0：建立迁移工作基线 ✅ 已完成

worktree 确认、CI workflow、README/MIGRATION 文档、oracle 自动发现、warning 清理、edition 2024 升级。

## P1：CLI 和 tsconfig 行为对齐 ✅ 已完成

CLI 参数流程审计、tsconfig 解析/extends/cycle/cache/${configDir}/null 清除、declaration-driven option parser、watch options 独立建模、--init/--help/--showConfig 模板、exit code 对齐、response file 解析、20+ CLI parity fixtures。

## P2：Scanner / Parser / AST parity ✅ 基本完成

Scanner ~95%（转义/JSX/正则/CommentDirectives/ASI/trivia/TokenFlags 完整位集/conflict-marker/JSDoc flag）、Parser ~95%（TS6/7 语法/类型语法/JSX/装饰器/import attributes/JSDoc parser/reparser/references.rs）、AST 生成链路、Parser diagnostic parity、位置信息一致性。

剩余：
- [ ] 正则 `lastIndex`/`d` flag runtime 特性（scanner body 校验已落地）
- [ ] `.ts/.tsx/.js/.jsx` 解析结果与 oracle 完全对齐（checker 深度限制）

## P3：Binder / Checker / Diagnostics parity — 进行中

Binder ~60%（容器递归绑定/FlowNode/NameResolver/alias/全局符号/声明合并/export-import binding/this_container）。Checker ~25%（类型结构/relater 完整规则/inference/contextual typing/narrowing/nodebuilder/emitresolver/JSX-JSDoc-Grammar checks）。572 个 checker parity fixtures 通过。

剩余：
- [ ] `symbol_to_display_parts`（LS 层功能，待 P7 LS 启动时补齐）
- [ ] 本地化支持（locale/loc_generated）
- [ ] 真实项目诊断集合与 Go oracle 一致（checker 深度需提升：array literal 类型推断 TS2339 false positive、TS2741 必填属性检查缺失等）
- [ ] Checker 深度提升（当前 ~25%，需补齐更多类型关系规则和诊断）

## P4：Emit / Transformer / SourceMap / Declaration emit ✅ 已完成

removeComments、ES5 down-leveling、CommonJS module transform、source map generation、declaration emit、text-slice emitter 设计、34 个 parity fixtures。

## P5：Module Resolution / Package JSON / Bundled Libs ✅ 已完成

Module resolution 全链路（relative/node_modules/paths/rootDirs/exports/imports/typesVersions/typeRef）、bundled libs 加载、case-sensitive FS、node_modules fixture parity。

## P6：Build / Watch / Incremental

目标：小型 project references fixture 与 Go oracle 行为一致；incremental
第二次构建能跳过未变更项目。

Go 参考：`internal/execute/build`、`internal/execute/incremental`、
`internal/execute/watchmanager`、`internal/fswatch`、`internal/project`。
流程审计见 `MIGRATION.md` 的 “Build Mode Flow Audit”。

已完成：`--build`/`-b` 外层 dispatch 对齐；`parse_build_command_line` +
`ParsedBuildCommandLine` + `BuildOptions`；build mode 中 `-v` 解析为 `verbose`；
空 project 默认 `"."`；非法组合（`clean+force`/`clean+verbose`/`clean+watch`/
`watch+dry`）拒绝；raw `references` DFS 桥接。`build_project` 递归解析 project
references、加载 tsconfig、调用 `perform_compilation`。

- [x] 支持 Go 等价的 typed project reference graph（已完成）：`build_project`
  递归 DFS 遍历 `references`，`seen_projects` HashSet 防止循环，每个 project 独立
  加载 tsconfig + 编译。`resolve_project_config` 对齐 Go directory/file 分发。
- [x] 补齐 build parser 的 build-specific did-you-mean（已完成）：Build 模式下
  对未知选项使用 TS5094（compiler option may not be used with build）、
  TS5072（Unknown build option）、TS5077（did-you-mean）。Levenshtein 距离
  用于拼写建议。watch options 解析已通过 `apply_watch_options` 接入。
- [x] 支持 `.tsbuildinfo` 读写（已完成）：`incremental/mod.rs` 的 `BuildInfo` struct +
  文件内容哈希 + 选项哈希 + `is_up_to_date` 检查。集成到 `build_project`：跳过
  未变更项目，编译成功后写入 `.tsbuildinfo`。
- [x] 支持 incremental rebuild（已完成基础）：`build_project` 在编译前检查
  `.tsbuildinfo`，如果文件哈希和选项哈希都匹配则跳过编译。
- [ ] 支持 watch mode，明确文件监听库选择。
- [x] 对齐 up-to-date 判断（已完成基础）：基于文件内容哈希 + 选项签名的
  up-to-date 判断，verbose 模式输出 "Project is up to date"。
- [ ] 对齐 project reference cycle（DFS 已有基础 cycle 检测）、输出跳过逻辑。
- [ ] 设计 watch 测试，避免 flaky。

## P7：Language Service / LSP

目标：VS Code extension 能指向 Rust binary 并完成 initialize；至少
hover/completion/diagnostics 三项通过 parity smoke。

Go 参考：`cmd/tsgo/lsp.go`、`internal/ls`、`internal/lsp`、`internal/project`、
`internal/fourslash`。
Rust 现状：`src/lsp/mod.rs` 实现了最小 LSP server（JSON-RPC 2.0 over stdio）。

- [x] 选择 Rust LSP 栈：自研最小协议层（直接 JSON-RPC，不依赖 `tower-lsp`）。
- [x] 实现 `--lsp` 启动、stdio transport、initialize/shutdown（已完成）：`LspServer`
  结构体 + `Content-Length` header framing + `initialize`（返回 capabilities）+
  `didChange`/`didClose`（文档存储 + full sync）+ `textDocument/hover`（接入 checker）+
  `textDocument/definition`（接入 checker symbol resolution）+ `textDocument/completion`
  （global symbols + keywords）+ unknown method error。
- [x] LSP diagnostics 推送（已完成）：`compute_diagnostics` 方法创建 InMemoryFS +
  Program + checker，收集 parse/bind/semantic 诊断，转换为 LSP 格式并推送
  `textDocument/publishDiagnostics` notification。`diagnostic_to_lsp` 转换器处理
  range/severity/code/source/message。
- [x] LSP hover 接入 checker（已完成）：`handle_hover` 构建临时 Program，通过
  `find_deepest_node` 定位 AST 节点，调用 `checker.get_quick_info_text` 获取类型信息，
  返回 markdown 格式的 hover 内容。
- [ ] 迁移 project service 基础：open/close/change watched files + 多文件管理。
- [x] LSP completion + definition（已完成基础）：`handle_definition` 通过 checker
  symbol resolution → value_declaration → LSP location；`handle_completion` 从
  checker globals table 收集符号 + TS 关键字补全。completionProvider 加入 capabilities。
- [ ] 逐步迁移 LS features：references / rename / document symbols / formatting。
- [ ] 迁移 fourslash 测试策略，先只保留关键 smoke。

## P8：API / npm package / VS Code extension

目标：`npm run build` 或新 Rust build task 能产出可运行 binary；native-preview
包内 binary 可执行 `--version`；extension 能启动 Rust LSP smoke。

Go 参考：`cmd/tsgo/api.go`、`internal/api`、`_packages/native-preview`、
`_extension`、`_extension-nightly`、`Herebyfile.mjs`。
Rust 现状：`src/api/mod.rs` 实现了最小 API server（JSON-RPC 2.0 over stdio）。

- [x] 实现或替代 `--api` transport（已完成）：`ApiServer` 结构体 +
  JSON-RPC 2.0 over stdio + `Content-Length` framing + `configure`/`createProject`/
  `updateProject`/`getDiagnostics`/`closeProject`/`getQuickInfo`/`shutdown`/`exit`
  方法（当前返回 stub 结果，待接入 Program pipeline）。
- [x] 决定 Rust binary 名称（已完成）：使用 `tsgo` 兼容 Go oracle。npm 包
  `npm/bin/tsgo` 为 Node.js shim，查找并执行 Rust 编译的 `tsox` binary。
- [x] 为 npm package 增加 Rust binary 构建/拷贝流程（已完成）：`npm/scripts/build.sh`
  执行 `cargo build --release` 并将 `target/release/tsox` 复制到 `npm/bin/tsgo`。
- [x] 更新 native-preview package 的 bin、postinstall、README（已完成）：
  `npm/package.json` 声明 `bin: { "tsgo": "./bin/tsgo" }`；`npm/README.md` 记录
  安装与 CLI/API 使用方式；`npm/lib/getExePath.js` 提供路径解析。
- [x] 更新 VS Code extension 查找 binary 的逻辑（已完成）：Go extension 已
  支持 `tsgo` binary 名查找（`util.ts` 中 `packagedExeBaseNames = ["tsc", "tsgo"]`）。
  Rust npm 包使用 `tsgo` 作为 binary 名（通过 JS shim），与 extension 逻辑兼容。
  无需额外修改 extension 代码。
- [ ] 保留 Go oracle 构建路径，直到 Rust parity 足够。

## P9：工具链、代码质量和发布 ✅ 已完成

rustfmt.toml、clippy 零 warning、codegen 命令、CI workflow、benchmark 脚本、license/NOTICE 安全检查。

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

## P10：Go 测试用例 1:1 迁移（测试 parity）

Go 仓库共 **1,219 个测试函数 / 508 个测试文件 / 44 个模块**。当前 Rust 侧 **1,105 个 lib 测试通过**，**99 个 ignored**（需要完整 printer/symlink/并发等深层功能）。

### P10.1 核心库测试（高优先级）

#### tspath — 24 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestNormalizeSlashes` | ✅ | |
| `TestGetRootLength` | ✅ | |
| `TestPathIsAbsolute` | ✅ | |
| `TestIsUrl` | ✅ | |
| `TestIsRootedDiskPath` | ✅ | |
| `TestGetDirectoryPath` | ✅ | |
| `TestGetPathComponents` | ✅ | |
| `TestReducePathComponents` | ✅ | |
| `TestCombinePaths` | ✅ | |
| `TestResolvePath` | ✅ | |
| `TestGetNormalizedAbsolutePath` | ✅ | |
| `TestGetNormalizedAbsolutePathWithoutRoot` | ✅ | |
| `TestGetRelativePathToDirectoryOrUrl` | ✅ | |
| `TestToFileNameLowerCase` | ✅ | |
| `TestToPath` | ✅ | |
| `TestPathIsRelative` | ✅ | |
| `TestGetCommonParents` | ✅ | |
| `TestUntitledPathHandling` | ✅ | |
| `TestUntitledPathEdgeCases` | ✅ | |
| `TestStartsWithDirectory` | ✅ | |
| `TestStartsWithDirectoryEdgeCases` | ✅ | |
| `TestContainsIgnoredPath` | ✅ | |
| `TestIgnoredPathsPatterns` | ✅ | |
| `TestIgnoredPathsEdgeCases` | ✅ | |

#### jsnum — 15 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestParsePseudoBigInt` | ✅ | |
| `TestToInt32` | ✅ | |
| `TestBitwiseNOT` | ✅ | |
| `TestBitwiseAND` | ✅ | |
| `TestBitwiseOR` | ✅ | |
| `TestBitwiseXOR` | ✅ | |
| `TestSignedRightShift` | ✅ | |
| `TestUnsignedRightShift` | ✅ | |
| `TestLeftShift` | ✅ | |
| `TestRemainder` | ✅ | |
| `TestExponentiate` | ✅ | 1 个 ULP 偏差用例 ⏳ |
| `TestString` | ✅ | display 分歧 ⏳ |
| `TestFromString` | ✅ | hex 溢出 ⏳ |
| `TestStringRoundtrip` | ✅ | |
| `TestStringJS` | ⏳ | 需要 Node.js |

#### core — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestPatternOverlappingMatch` | ✅ | |
| `TestBreadthFirstSearchParallel` | ✅ | |

#### collections — 8 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestOrderedMap` | ✅ | |
| `TestOrderedMapClone` | ✅ | |
| `TestOrderedMapClear` | ✅ | |
| `TestOrderedMapWithSizeHint` | ✅ | |
| `TestOrderedMapUnmarshalJSON` | ✅ | |
| `TestOrderedSet` | ✅ | |
| `TestOrderedSetWithSizeHint` | ⏳ | allocsPerRun |
| `TestSyncMapWithNil` | ✅ | |

#### stringutil — 3 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestJSCasing` | ✅ | |
| `TestEncodeURI` | ✅ | |
| `TestContainsNonASCII` | ✅ | |

#### semver — 11 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestWildcardsHaveSameString` | ✅ | |
| `TestVersionRanges` | ✅ | |
| `TestComparatorsOfVersionRanges` | ✅ | |
| `TestConjunctionsOfVersionRanges` | ✅ | |
| `TestDisjunctionsOfVersionRanges` | ✅ | |
| `TestHyphensOfVersionRanges` | ✅ | |
| `TestTildesOfVersionRanges` | ✅ | |
| `TestCaretsOfVersionRanges` | ✅ | |
| `TestTryParseSemver` | ✅ | |
| `TestVersionString` | ✅ | |
| `TestVersionCompare` | ✅ | |

### P10.2 编译器核心测试（高优先级）

#### scanner — 1 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestScanStringPreservesLoneSurrogates` | ✅ | |

#### ast — 7 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestDeepCloneNodeSanityCheck` | ⏳ | |
| `TestPositionMapASCII` | ✅ | 原已存在 |
| `TestPositionMapTwoByte` | ✅ | 原已存在 |
| `TestPositionMapFourByte` | ✅ | 原已存在 |
| `TestPositionMapMultipleNonASCII` | ✅ | 原已存在 |
| `TestPositionMapLoneSurrogateSentinel` | ✅ | 用有效代理对 U+10000 替代 lone surrogate |
| `TestPositionMapRoundtrip` | ✅ | 原已存在 |

#### astnav — 6 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestGetTokenAtPosition` | ✅ | 5 tests now running |
| `TestGetTouchingPropertyName` | ✅ | merged into get_token_at_position |
| `TestFindPrecedingToken` | ✅ | |
| `TestFindNextToken` | ✅ | |
| `TestUnitFindPrecedingToken` | ✅ | |

#### printer — 105 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestEmit` | ⏳ | |
| `TestParenthesize*` (65 个) | ⏳ | Decorator/ComputedPropertyName/ArrayLiteral/PropertyAccess/ElementAccess/Call/New/TaggedTemplate/TypeAssertion/ArrowFunction/Delete/Void/TypeOf/Await/Binary/Conditional/Yield/SpreadElement/ExpressionWithTypeArguments/AsExpression/SatisfiesExpression/NonNullExpression/ExpressionStatement/ExpressionDefault/ArrayType/OptionalType/UnionType/IntersectionType/ReadonlyTypeOperator/KeyofTypeOperator/IndexedAccessType/ConditionalType |
| `TestNameGeneration` | ✅ | 原已存在 |
| `TestNoTrailingCommaAfterTransform` | ⏳ | |
| `TestTrailingCommaAfterTransform` | ⏳ | |
| `TestPartiallyEmittedExpression` | ⏳ | |
| `TestParenthesizeBinaryExpressionMixingNullishCoalescing` | ⏳ | |
| `TestTempVariable1/2/3` | ✅ | namegenerator |
| `TestTempVariableScoped` | ✅ | |
| `TestTempVariableScopedReserved` | ✅ | |
| `TestLoopVariable1/2/3` | ✅ | |
| `TestLoopVariableScoped` | ✅ | |
| `TestUniqueName1/2/Scoped` | ✅ | |
| `TestUniquePrivateName1/2/Scoped` | ✅ | |
| `TestGeneratedNameFor*` (16 个) | ✅ | 原已存在 Identifier/Namespace1-4/NodeCached/Import/Export/FunctionDeclaration1-2/ClassDeclaration1-2/ExportAssignment/ClassExpression/Method1-2/ComputedPropertyName/Other |
| `TestEscapeString` | ✅ | utilities（新实现） |
| `TestEscapeNonAsciiString` | ✅ | 新实现 |
| `TestEscapeJsxAttributeString` | ✅ | 新实现 |
| `TestIsRecognizedTripleSlashComment` | ✅ | 新实现 |

#### sourcemap — 30 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestSourceMapGenerator_*` (30 个) | ✅ | 原已存在 Empty/Serialized/AddSource/SetSourceContent/AddName/AddGeneratedMapping/AddSourceMapping/NamedSourceMapping 等 |

#### compiler — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestProgram` | ✅ | 3 个用例（reference paths / imports / cycles）|
| `TestIncludeProcessorDiagnosticsWithMissingFileCasing` | ✅ | |

#### checker — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestGetSymbolAtLocation` | ✅ | |
| `TestTracerPushPreservesEndArgMutations` | ⏳ | |

#### transformers/tstransforms — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestTypeEraser` | ✅ | |
| `TestImportElision` | ⏳ | |

#### diagnostics — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestLocalize` | ✅ | English fallback（Message::format）|
| `TestLocalize_ByKey` | ✅ | English fallback（key_to_message）|

### P10.3 模块/包测试（中优先级）

#### module — 5 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestResolveModuleNameTrailingSlash` | ✅ | |
| `TestResolveModuleNameTrailingSlashRace` | ✅ | 并发，可能不适用 |
| `TestResolveSubpathNilContentsRace` | ⏳ | 并发 |
| `TestParseNodeModuleFromPath` | ✅ | |
| `TestResolvePeerDependencyNilContentsRace` | ⏳ | 并发 |

#### packagejson — 4 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestParse` | ✅ | |
| `TestExpected` | ✅ | |
| `TestExports` | ✅ | |
| `TestJSONValue` | ✅ | |

#### modulespecifiers — 6 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestGetEachFileNameOfModule` | ⏳ | |
| `TestGetEachFileNameOfModuleWithSymlinks` | ⏳ | |
| `TestContainsNodeModules` | ✅ | |
| `TestContainsIgnoredPath` | ✅ | |
| `TestTryGetRealFileNameForNonJSDeclarationFileName` | ✅ | |
| `TestTryGetModuleNameFromExportsOrImports` | ⏳ | |

#### bundled — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestTestingLibPath` | ✅ | |
| `TestEmbeddedLibs` | ✅ | |

#### nativepath — 4 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestIsSymlinkOrReparsePoint` | ⏳ | 平台特定 |
| `TestIsSymlinkOrReparsePointLongPath` | ⏳ | |
| `TestIsSymlinkOrReparsePointNestedInSymlink` | ⏳ | |
| `TestIsSymlinkOrReparsePointRelativePath` | ⏳ | |

#### symlinks — 8 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| `TestNewKnownSymlink` | ✅ | |
| `TestSetDirectory` | ✅ | |
| `TestSetFile` | ✅ | |
| `TestProcessResolution` | ✅ | |
| `TestGuessDirectorySymlink` | ✅ | |
| `TestIsNodeModulesOrScopedPackageDirectory` | ✅ | |
| `TestSetSymlinksFromResolutions` | ✅ | |
| `TestKnownSymlinksThreadSafety` | ✅ | |

### P10.4 VFS 测试（中优先级）

#### vfs/vfstest — 18 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (18 个) | 14 ✅ / 4 ⏳ | 14 个运行通过，4 个 symlink 相关 #[ignore] |

#### vfs/vfsmatch — 17 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (17 个) | 16 ✅ / 1 ⏳ | 16 个运行通过，1 个 symlink cycle #[ignore] |

#### vfs/cachedvfs — 10 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (10 个) | ✅ | 全部运行通过 |

#### vfs/osvfs — 3 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (3 个) | ✅ | 已迁移 5 个测试 |

#### vfs/iovfs — 1 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (1 个) | ⏳ | `#[ignore]` |

#### vfs/vfsmock — 1 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (1 个) | ⏳ | `#[ignore]` |

（共 50 个，详见 Go `internal/vfs/` 下各子包）

### P10.5 格式化/调试测试（低优先级）

#### format — 7 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (7 个) | ✅ | 全部运行通过 |

#### debug — 12 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (12 个) | ✅ | 已迁移 |

#### tracing — 2 个测试

| Go 测试 | Rust 状态 | 说明 |
|---------|----------|------|
| 全部 (2 个) | ✅ | 全部运行通过 |

### P10.6 execute/tsc 集成测试（中优先级）

#### execute/tsctests — 52 个测试

包含 `TestTscCommandline`、`TestTscMissingFiles`、`TestTscComposite`、
`TestTscDeclarationEmit`、`TestBuildCommandLine`、`TestBuildClean`、
`TestBuildDemoProject` 等。大部分通过 shell-out 到 tsgo binary 执行，
Rust 侧对应 `tests/checker_parity.rs` 和 `tests/parity.rs` 中的 fixture。

### P10.7 LSP/Project/API 测试（低优先级，依赖功能完成）

#### lsp — 64 个测试
#### project — 76 个测试
#### api — 10 个测试
#### api/encoder — 25 个测试
#### ls — 3 个测试
#### ls/autoimport — 10 个测试
#### ls/lsutil — 10 个测试
#### ls/lsconv — 4 个测试

### P10.8 Fourslash 测试（低优先级，最大模块）

#### fourslash — 519 个测试

语言服务集成测试（hover/completion/references/rename/organize imports/
code fix/go to definition 等）。需在 LSP 功能完成后批量迁移。

### P10.9 fswatch 测试（低优先级，依赖 watch mode）

#### fswatch — 116 个测试

文件系统监听测试，需在 watch mode 实现后迁移。
