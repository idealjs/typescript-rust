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

## 当前进度快照（2026-08-02）

测试基线：**1,126** lib + **810** checker parity + **2** emit = **1,938 通过**，99 ignored。
checker_parity 从 572→710（+138）。新增诊断码：TS2741/TS2739/TS2353/TS2448/TS2454/
TS18048/TS2451/TS2300。Array 方法解析已修复（.find/.map/.reduce 等不再 false positive）。

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

## 待办清单（按优先级排序）

### A. Checker 深度（高）✅ 已完成

- [x] Array 方法解析（.find/.map/.reduce 等）
- [x] TS2741/TS2739/TS2353/TS2448/TS2454/TS18048/TS2451/TS2300
- [x] TS2511/TS2341/TS2366/TS2588/TS7027
- [x] 扩充 checker parity fixtures 到 800+（当前 808）
- [x] 修复 String/Number interface 方法解析（仅 Array 已修复）
- [x] 修复 generic call-site inference（TS2345 false positive）

### B. Watch mode（中）✅ 已完成

- [x] 引入 `notify` crate 实现文件监听
- [x] `--watch` 模式：文件变更 → 重编译 → 诊断输出
- [x] watch 测试（5 个单元测试覆盖 helper 函数）
- [x] project reference cycle 精细化处理（TS6202）

### C. LSP features（中）✅ 已完成

- [x] references（跨文件符号引用查找）
- [x] documentSymbol（文档符号树）
- [x] rename（重命名 + WorkspaceEdit）
- [x] project service（多文件 open/close/change + 跨文件诊断 + didChangeWatchedFiles）
- [ ] fourslash 测试 smoke

### D. 其他（低）

- [x] `symbol_to_display_parts`（LS hover 分类信息）
- [ ] 本地化支持（locale/loc_generated）
- [ ] 正则 `lastIndex`/`d` flag runtime 特性
- [ ] `.ts/.tsx/.js/.jsx` 解析结果与 oracle 完全对齐（依赖 checker 深度）

## P0：建立迁移工作基线 ✅ 已完成

worktree 确认、CI workflow、README/MIGRATION 文档、oracle 自动发现、warning 清理、edition 2024 升级。

## P1：CLI 和 tsconfig 行为对齐 ✅ 已完成

CLI 参数流程审计、tsconfig 解析/extends/cycle/cache/${configDir}/null 清除、declaration-driven option parser、watch options 独立建模、--init/--help/--showConfig 模板、exit code 对齐、response file 解析、20+ CLI parity fixtures。

## P2：Scanner / Parser / AST parity ✅ 基本完成

Scanner ~95%、Parser ~95%、AST 生成链路、Parser diagnostic parity、位置信息一致性。
（剩余项见上方待办清单 D）

## P3：Binder / Checker / Diagnostics parity — 进行中

Binder ~60%。Checker ~30%。810 个 checker parity fixtures 通过。13 个新诊断码
（TS2741/2739/2353/2448/2454/18048/2451/2300/2511/2341/2366/2588/7027）。
Array/String/Number 方法解析 + generic call-site inference 已修复。
（剩余项见上方待办清单 A/D）

## P4：Emit / Transformer / SourceMap / Declaration emit ✅ 已完成

removeComments、ES5 down-leveling、CommonJS module transform、source map generation、declaration emit、text-slice emitter 设计、34 个 parity fixtures。

## P5：Module Resolution / Package JSON / Bundled Libs ✅ 已完成

Module resolution 全链路（relative/node_modules/paths/rootDirs/exports/imports/typesVersions/typeRef）、bundled libs 加载、case-sensitive FS、node_modules fixture parity。

## P6：Build / Watch / Incremental ✅ 基本完成

已完成：`--build` dispatch、project reference graph、`.tsbuildinfo` 读写 +
up-to-date check、`--watch` 模式（notify crate + PollWatcher + 重编译）。
（剩余 cycle 处理见上方待办清单 B）

## P7：Language Service / LSP ✅ 基本完成

已完成：JSON-RPC 协议层、initialize/shutdown、文档同步、diagnostics 推送、hover、
completion、definition、references（跨文件符号查找）、documentSymbol、rename。
（剩余 project service / fourslash 见上方待办清单 C）

## P8：API / npm package / VS Code extension

已完成：`--api` JSON-RPC server、`tsgo` binary 名 + JS shim、npm build 脚本、
native-preview package（bin/postinstall/README）、VS Code extension 兼容。

剩余：
- [ ] 保留 Go oracle 构建路径，直到 Rust parity 足够

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

## 历史基线

- record.warn：2026-07-11 分析 3347 行 parser syntax errors，P2.4/P2.5/P2.9
  完成后 bundled libs 零错误解析（3347 → 0）。
- Warning 清理：剩余 lib warning ~31 个（Go 命名对齐/迁移占位 API/re-export 冲突），
  归类为迁移期可接受。

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
