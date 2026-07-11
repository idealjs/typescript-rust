# typescript-go -> typescript-rust TODO

更新时间：2026-07-11

## 当前结论

- Go 源 worktree：`/home/cqh/workspace/typescript-rust`，当前分支 `main`，仍是完整的 `typescript-go` 代码布局。
- Rust 迁移 worktree：`/home/cqh/workspace/typescript-rust-rust`，当前分支 `rust`，已经有初步 Rust crate。
- Rust crate 当前名为 `tsox`，入口在 `src/main.rs`，库入口在 `src/lib.rs`。
- Rust 侧已经有模块骨架：`ast`、`binder`、`checker`、`compiler`、`execute`、`parser`、`scanner`、`printer`、`tsoptions`、`vfs` 等。
- `cargo test` 当前通过：483 个 lib 单测 + 2 个 parity 测试通过。
- 关键缺口：CLI 只覆盖部分编译链路；`--lsp` 和 `--api` 仍是 stub；module resolution、watch/build/incremental、fourslash/baseline、npm/vscode 包装尚未迁移到 Rust 方案。

## 迁移原则

- 先建立可验证的端到端 parity，再扩大覆盖面。
- 不做逐文件机械翻译，按能力边界迁移：CLI、配置解析、VFS、program、parser、binder、checker、emit、LS/LSP、API、发布。
- Go 实现暂时作为 oracle，Rust 每补一个能力都加对应 parity case。
- 保持 Rust API idiomatic，但错误码、诊断文本、输出文件、命令行行为要优先对齐 Go/TypeScript。
- 每个阶段都要有可运行验收命令，不能只算“代码写完”。

## P0：建立迁移工作基线

- [x] 确认 Go worktree 和 Rust worktree 位置。
- [x] 确认 Rust crate 能编译并通过现有测试。
- [x] 创建迁移 TODO。
- [x] 在 Rust worktree 增加 `README.md` 或 `MIGRATION.md`，说明如何运行 Rust 迁移版、如何设置 Go oracle。
- [x] 增加固定验收命令文档：
  - `cargo test`
  - `TSGO_ORACLE=/path/to/tsgo cargo test --test parity`
  - 后续补充 `cargo clippy`、`cargo fmt --check`
- [x] 清理或标注当前 warnings，决定哪些保留为对齐 Go 命名，哪些必须修复。
- [ ] 给 CI 设计最小 Rust job：fmt、clippy、test、parity smoke。
- [x] 修复 scanner 处理非 ASCII 未知字符时按 byte 切片导致的 panic。
- [x] 将 Rust crate 升级到 Cargo 支持的最新 edition：`edition = "2024"`，并设置 `rust-version = "1.96"`。

验收：

- [ ] 新人只看 Rust worktree 文档即可跑通测试。
- [ ] parity 测试能自动发现可用 Go oracle，或给出明确跳过原因。

## P1：CLI 和 tsconfig 行为对齐

Go 参考：

- `cmd/tsgo/main.go`
- `internal/execute`
- `internal/execute/tsc`
- `internal/tsoptions`
- `internal/vfs`

Rust 现状：

- `src/main.rs`
- `src/execute/mod.rs`
- `src/tsoptions/mod.rs`
- `src/vfs/mod.rs`

任务：

- [ ] 对齐 `--help`、`--version`、无输入、未知选项、响应文件等 CLI 行为。
- [ ] 对齐退出码：`Success`、`DiagnosticsPresent_*`、`InvalidProject_*`、`ProjectReferenceCycle_*`。
- [ ] 对齐 `tsconfig.json` 查找、`extends`、`files/include/exclude`、`compilerOptions` 覆盖规则。
- [ ] 扩充 parity fixtures：
  - [ ] 无 `tsconfig` 且无文件。
  - [ ] 单文件输入。
  - [ ] `-p` 指向目录。
  - [ ] `-p` 指向文件。
  - [ ] `--showConfig`。
  - [ ] response file。
  - [ ] invalid JSON / JSONC。
- [ ] 修复当前 parity 注释中提到的 `rootDir/outDir` 差异。

验收：

- [ ] Rust 和 Go oracle 的 stdout、stderr、exit code、输出文件集合一致。
- [ ] CLI parity 覆盖至少 20 个常见 tsc 场景。

## P2：Scanner / Parser / AST parity

Go 参考：

- `internal/scanner`
- `internal/parser`
- `internal/ast`
- `_scripts/ast.json`
- `_scripts/generate-*`

Rust 现状：

- `src/scanner/mod.rs`
- `src/parser/mod.rs`
- `src/ast/*`
- `build.rs`

任务：

- [ ] 明确 Rust AST 生成链路是否继续读取 Go 侧 `_scripts/ast.json`，还是维护 Rust 自有 schema。
- [ ] 对齐 generated enum/node 数据的生成命令和检查方式。
- [ ] 增加 parser diagnostic parity：
  - [ ] 语法错误位置。
  - [ ] JSX。
  - [ ] JSDoc。
  - [ ] decorators。
  - [ ] import attributes。
  - [ ] TS 6/7 新语法。
- [ ] 从 Go parser 测试或 TypeScript baselines 中挑选 smoke 集合，转成 Rust parity。
- [ ] 检查 UTF-16 position、line map、source span 和 diagnostic span 是否全链路一致。

验收：

- [ ] 典型 `.ts/.tsx/.js/.jsx` 解析结果和诊断可对齐 oracle。
- [ ] 生成文件可重复生成，`git diff` 干净。

## P3：Binder / Checker / Diagnostics parity

Go 参考：

- `internal/binder`
- `internal/checker`
- `internal/diagnostics`
- `internal/nodebuilder`
- `internal/pseudochecker`

Rust 现状：

- `src/binder/mod.rs`
- `src/checker/*`
- `src/diagnostics/*`

任务：

- [ ] 建立 type-check parity fixtures，先覆盖最小闭环：变量、函数、类、接口、泛型、union/intersection。
- [ ] 对齐 symbol table、scope、declaration merge、export/import binding。
- [ ] 对齐主要 diagnostic code、category、message、span。
- [ ] 迁移/生成 diagnostics message，避免手写漂移。
- [ ] 对齐 checker 的核心类型关系：
  - [ ] assignability。
  - [ ] subtype。
  - [ ] inference。
  - [ ] contextual typing。
  - [ ] control flow narrowing。
- [ ] 增加 `.js` + JSDoc 行为测试。
- [ ] 增加 JSX type-check smoke。

验收：

- [ ] Rust 能在常见类型错误项目上输出与 Go oracle 一致的诊断集合。
- [ ] 至少 50 个 checker parity fixtures 通过。

## P4：Emit / Transformer / SourceMap / Declaration emit

Go 参考：

- `internal/compiler`
- `internal/printer`
- `internal/transformers`
- `internal/sourcemap`
- `internal/outputpaths`

Rust 现状：

- `src/compiler/mod.rs`
- `src/printer/mod.rs`
- `src/emitter/mod.rs`
- `src/sourcemap/mod.rs`

任务：

- [ ] 对齐 JS emit：target、module、jsx、imports/exports、helpers。
- [ ] 对齐 declaration emit：`.d.ts`、`.d.ts.map`、strip internal、declaration maps。
- [ ] 对齐 sourcemap：路径、sources、sourcesContent、VLQ mappings。
- [ ] 对齐 output path：`rootDir`、`outDir`、`declarationDir`、mixed JS/TS。
- [ ] 补齐 transformer 体系或明确替代设计。
- [ ] 扩充 parity fixtures：
  - [ ] CommonJS。
  - [ ] ES modules。
  - [ ] JSX preserve/react/react-jsx。
  - [ ] decorators。
  - [ ] enum/namespace。
  - [ ] source maps。
  - [ ] declaration emit。

验收：

- [ ] 输出文件路径和内容与 Go oracle 一致，或差异被记录为有意差异。
- [ ] emit parity 覆盖至少 30 个 fixtures。

## P5：Module Resolution / Package JSON / Bundled Libs

Go 参考：

- `internal/module`
- `internal/packagejson`
- `internal/bundled`
- `internal/tspath`
- `internal/nativepath`

Rust 现状：

- `src/module/mod.rs`
- `src/packagejson/mod.rs`
- `src/bundled/mod.rs`
- `src/tspath/mod.rs`

任务：

- [ ] 对齐 node/module resolution：classic、node10、node16、nodenext、bundler。
- [ ] 对齐 `paths`、`baseUrl`、`rootDirs`、`typeRoots`、`types`。
- [ ] 对齐 package `exports`、`imports`、`typesVersions`、`type`。
- [ ] 对齐 bundled libs 的加载方式和版本。
- [ ] 对齐大小写敏感文件系统行为。
- [ ] 增加 node_modules fixture parity。

验收：

- [ ] 常见 npm 包解析结果与 Go oracle 一致。
- [ ] bundled lib 相关诊断和 emit 不依赖外部 TypeScript checkout。

## P6：Build / Watch / Incremental

Go 参考：

- `internal/execute/build`
- `internal/execute/incremental`
- `internal/execute/watchmanager`
- `internal/fswatch`
- `internal/project`

Rust 现状：

- 目前没有完整 build orchestrator；`src/execute/mod.rs` 只有有限 `-b`
  外层对齐。2026-07-11 已在 `MIGRATION.md` 的 "Build Mode Flow Audit"
  记录 Go `tsgo -b` 和 Rust `tsox -b` 的流程对比。
- 已对齐的外层行为：
  - `--build` / `-b` 只能作为首参数触发 build mode。
  - 非首位 `--build` 报错，不再被当作普通 compiler option。
  - Rust 已有 `parse_build_command_line`、`ParsedBuildCommandLine` 和
    `BuildOptions`，不再用普通 `parse_command_line` 伪装 build mode。
  - build mode 中 `-v` 解析为 `verbose`，而不是普通编译模式的
    `version`。
  - `tsox -b` 空 project 列表默认按 `"."` 处理。
  - build mode 的 bare arguments 按 project/config 路径处理，不再当
    source file 列表普通编译。
  - 已按 Go 规则拒绝 `--clean --force`、`--clean --verbose`、
    `--clean --watch`、`--watch --dry`。
- 剩余关键差异：Rust build parser 仍缺 build-specific did-you-mean 和完整
  watch-options 模型；仍缺 project reference graph、`.tsbuildinfo`、
  up-to-date 判定、clean/dry/force/verbose 的真实 orchestrator 行为、watch
  build mode。

任务：

- [x] 记录 Go/Rust `-b` 流程审计和差异。
- [x] 修正 `-b` 外层 dispatch，不再把 build mode 伪装成普通 source-file 编译。
- [x] 迁移 `ParseBuildCommandLine` 的核心等价物，区分 build options / compiler options / projects，并校验非法组合。
- [ ] 补齐 build parser 的 build-specific did-you-mean 和完整 watch options。
- [ ] 支持 `--build` / project references 基本图构建。
- [ ] 支持 `.tsbuildinfo` 读写。
- [ ] 支持 incremental rebuild。
- [ ] 支持 watch mode，明确文件监听库选择。
- [ ] 对齐 project reference cycle、up-to-date 判断、输出跳过逻辑。
- [ ] 设计 watch 测试，避免 flaky。

验收：

- [ ] 小型 project references fixture 与 Go oracle 行为一致。
- [ ] incremental 第二次构建能跳过未变更项目。

## P7：Language Service / LSP

Go 参考：

- `cmd/tsgo/lsp.go`
- `internal/ls`
- `internal/lsp`
- `internal/project`
- `internal/fourslash`

Rust 现状：

- `src/main.rs` 中 `--lsp` 仍为 not implemented。

任务：

- [ ] 选择 Rust LSP 栈：直接 JSON-RPC、`tower-lsp`，或自研最小协议层。
- [ ] 实现 `--lsp` 启动、stdio transport、initialize/shutdown。
- [ ] 迁移 project service 基础：open/close/change watched files。
- [ ] 逐步迁移 LS features：
  - [ ] diagnostics。
  - [ ] hover。
  - [ ] completion。
  - [ ] definition。
  - [ ] references。
  - [ ] rename。
  - [ ] document symbols。
  - [ ] formatting。
- [ ] 迁移 fourslash 测试策略，先只保留关键 smoke。

验收：

- [ ] VS Code extension 能指向 Rust binary 并完成 initialize。
- [ ] 至少 hover/completion/diagnostics 三项通过 parity smoke。

## P8：API / npm package / VS Code extension

Go 参考：

- `cmd/tsgo/api.go`
- `internal/api`
- `_packages/native-preview`
- `_extension`
- `_extension-nightly`
- `Herebyfile.mjs` 打包任务

Rust 现状：

- `src/main.rs` 中 `--api` 仍为 not implemented。
- 根 `package.json`、extension、native-preview 仍是 Go 命名和构建链。

任务：

- [ ] 决定 Rust binary 名称：继续兼容 `tsgo`，还是使用 `tsox` 并提供 shim。
- [ ] 为 npm package 增加 Rust binary 构建/拷贝流程。
- [ ] 实现或替代 `--api` transport。
- [ ] 更新 native-preview package 的 bin、postinstall、README。
- [ ] 更新 VS Code extension 查找 binary 的逻辑。
- [ ] 保留 Go oracle 构建路径，直到 Rust parity 足够。

验收：

- [ ] `npm run build` 或新 Rust build task 能产出可运行 binary。
- [ ] native-preview 包内 binary 可执行 `--version`。
- [ ] extension 能启动 Rust LSP smoke。

## P9：工具链、代码质量和发布

任务：

- [ ] 增加 `rustfmt.toml` 或确认默认 rustfmt。
- [ ] 增加 clippy 策略：先允许迁移期 warning，逐步收紧。
- [ ] 建立 Rust codegen 命令，覆盖 AST、diagnostics、bundled libs。
- [ ] 更新 `.gitignore`，纳入 `target/`、生成产物、临时 baseline。
- [ ] 更新 CI workflow。
- [ ] 设计 benchmark：Go vs Rust CLI cold run、incremental、checker、emit。
- [ ] 发布前安全检查：license、NOTICE、third-party deps。

验收：

- [ ] CI 能在干净环境跑通 Rust checks。
- [ ] 发布包不依赖本地 Go 构建产物。

## 第一批建议执行任务

1. [x] 在 Rust worktree 添加 `MIGRATION.md`，记录当前命令和 oracle 用法。
2. [x] 修复或标注 `cargo test` 中的 warning，至少清理无争议的 unused imports/variables。
3. 扩充 `tests/parity.rs` 的 CLI/emit fixtures，优先覆盖 `rootDir/outDir`、`--showConfig`、invalid tsconfig。
4. [x] 把 `TSGO_ORACLE` 默认路径改得更稳：优先查环境变量，再查相邻 Go worktree的 `built/local/tsgo` 和 `_packages/native-preview/bin/tsgo`。
5. 为 `--lsp` 和 `--api` 建立 tracker issue/TODO 小节，先不实现大功能，但明确启动验收。

## 已知风险

- TypeScript 语义极大，必须依赖 oracle parity 和 baselines，不能靠局部单测判断完成度。
- Rust 所有权模型可能要求重构 AST/checker 数据结构，过早追求 idiomatic 可能导致行为漂移。
- LSP/project service 涉及并发和缓存，建议在 CLI/checker/emit 稳定后再大规模迁移。
- Go 仓库的生成脚本、baselines、native-preview 包装较多，迁移期间要避免同时改太多构建路径。

## Warning 状态

2026-07-11 已完成一轮低风险 warning 清理：

- 已清理：明显未使用 import、只在测试中使用的 import 作用域、未使用变量、重复 match arm、无意义 iterator 赋值。
- `cargo test` 当前通过：483 个 lib 单测 + 2 个 parity 测试。
- 剩余 lib warning 约 31 个，当前归类为迁移期可接受：
  - Go/TypeScript 命名对齐：`DiagnosticsPresent_OutputsSkipped`、`BlockScoped`、`parse_bracketedList` 等。
  - 暂未接入的迁移占位 API：`expected_json_type`、`next_auto_generate_id`、`compiler_diagnostic` 等。
  - 需要单独设计的公开 re-export 冲突：`checker::RelationComparisonResult`。
- `cargo fmt --check` 仍会因仓库既有未格式化文件失败；本轮只格式化了本次触碰的 Rust 文件，避免制造全仓格式化 diff。

## 已修复问题

### 2026-07-11：scanner 非 ASCII 未知字符 panic

- 现象：打包后的 `tsox` 扫描 `·` 时 panic：`end byte index 1 is not a char boundary`。
- 原因：`src/scanner/mod.rs` 的 punctuation 扫描在 1-char token 分支里使用 `&remaining[..1]`，对 UTF-8 多字节字符会切在字符内部。
- 修复：1-char punctuation 只在首字符 UTF-8 长度为 1 时查询 token 表；未知非 ASCII 字符按完整 UTF-8 字符长度前进并报告 scanner error callback。
- 新增测试：
  - scanner 单测覆盖 `·`、CJK、emoji，断言 token text 和 diagnostic byte span。
  - command-line smoke 覆盖含 `·` 的源文件不 panic。
- Go 版现状：`internal/scanner/scanner_test.go` 目前只有字符串/代理对保留测试，未看到 `·` 或非 ASCII unknown punctuation 的 scanner 单测。Go scanner 本体使用 `charAndSize()` / `utf8.DecodeRuneInString`，`scanInvalidCharacter()` 会按完整 rune size 前进。
- 后续任务：Rust parser/command-line 还没有把 scanner invalid-character diagnostics 接入最终诊断集合；当前 CLI smoke 只保证不 panic，后续应改成与 Go oracle 一致的 invalid character 诊断。

## record.warn 汇总

2026-07-11 对 `/home/cqh/workspace/typescript-rust/record.warn` 做了聚合分析。

- 文件规模：3347 行，约 346 KiB。
- 错误形态：3347 条全是 `TS1003` parser syntax errors。
- 涉及文件：96 个。
- 来源分布：
  - bundled libs：2895 条，约 86.5%。
  - 项目源码：365 条，约 10.9%。
  - 项目 dist declaration：87 条，约 2.6%。
- 最集中的 bundled lib 文件：
  - `lib.es2015.iterable.d.ts`：1024 条。
  - `lib.dom.d.ts`：947 条。
  - `lib.es2015.collection.d.ts`：206 条。
  - `lib.es5.d.ts`：113 条。
  - `lib.es2015.core.d.ts`：112 条。
  - `lib.decorators.d.ts`：94 条。
- 最集中的项目文件：
  - `src/types.ts`：108 条。
  - `src/OverlayComponents.tsx`：44 条。
  - `src/ComposerPage.tsx`：34 条。
  - `src/AiTestControls.tsx`：27 条。
  - `src/AiTestControls.test.tsx`：23 条。
  - `src/AiModelField.test.tsx`：22 条。
- 高频 token/语法症状：
  - `<` / `>`：泛型、类型参数、类型实参、JSX 起始符，合计最高。
  - `|` / `&`：union / intersection types。
  - `[` / `]`：数组类型、tuple、indexed access、mapped types、computed property name。
  - `=>` / `(` / `)`：arrow functions、function type、call signatures。
  - `declare`：ambient declarations / `.d.ts` 支持不足。
  - `import { ... } from`：named import/export parsing 不完整。
  - keyword type nodes：`string`、`number`、`boolean`、`symbol`、`object`、`unknown`、`void`、`undefined`、`null`。

结论：`record.warn` 不是单点 panic，而是 parser 仍处于早期状态。最优先的根因不是修某一个 bundled lib，而是补 TypeScript declaration/type syntax；否则默认加载 bundled libs 时会持续产生海量级联错误。

### 2026-07-11：record.warn-1 流程逻辑追踪

对 `/home/cqh/workspace/typescript-rust/record.warn-1` 重新聚合后，确认不能直接把所有错误归为 parser bug。Go/TS 的普通 `tsc -p tsconfig.json` 会严格按 tsconfig 的 root file/include/exclude/default lib 流程构造 program；Rust 版在这层存在差异，导致额外文件进入 program，放大了 parser 错误：

- `ai-Color-toner/tsconfig.json` 是 solution config：`files: []` + `references`。Go/TS 不会因为 `files` 为空数组而回退到默认 `**/*`；Rust 之前无法区分“没有 files 属性”和“显式 files: []”，会错误扫全项目。
- `tsconfig.app.json` 设置了 `outDir: "dist"`。Go/TS 在没有显式 `exclude` 时会把 `outDir` / `declarationDir` 当作默认 exclude；Rust 之前没有实现，所以 `dist/src/*.d.ts` 被纳入诊断。
- `include: ["src"]` 在 Go/TS 中表示递归包含目录内支持的源文件；Rust 之前只按字面 glob 匹配，修正流程后才能稳定复现项目源码错误集合。
- 默认 lib 是 program 的一部分，这一点和 Go/TS 一致；bundled lib 的剩余错误仍然是 parser 对 `.d.ts` 类型语法支持不足，不是“第三方被错误加载”。

已修复并加测试：

- [x] `files: []` 不触发默认 include。
- [x] 未显式 `exclude` 时默认排除 `outDir` / `declarationDir`。
- [x] wildcard include 跳过 `node_modules`、`bower_components`、`jspm_packages`。
- [x] literal directory include（如 `include: ["src"]`）递归展开支持的 TS/TSX/DTS 源文件。
- [x] parser 先处理 `declare/export declare/...` modifier 前缀，再分派到 declaration parser，避免 `declare` 被误当表达式语句导致级联错误。

验证：

- `cargo test` 通过：494 个 lib 单测 + 2 个 parity 测试 + doc tests。
- `target/debug/tsox --noLib --listFilesOnly -p /home/cqh/workspace/ai-Color-toner/tsconfig.json` 不再列出项目源文件，符合 solution config 的普通编译行为。
- `target/debug/tsox --noLib --listFilesOnly -p /home/cqh/workspace/ai-Color-toner/tsconfig.app.json` 只列出 `src` 下源文件，不列 `dist` / `node_modules`。

## record.warn TODO

P0：减少噪声并建立可验证基线

- [ ] 增加一个 `record.warn` 复现 fixture：最小项目 + 默认 lib 加载，记录当前 3347 条 `TS1003` 的基线摘要。
- [ ] 增加 parser 错误聚合脚本或测试 helper，输出按 file/message/token/category 的统计，避免手工读大日志。
- [x] 在 CLI/手工验证中加入 `--noLib` 与默认 lib 两组场景，明确区分“用户源码解析错误”和“bundled lib 解析错误”。
- [ ] 临时在迁移期文档里推荐 `--noLib` 调试路径，避免 bundled libs 淹没真实项目错误。
- [x] 对齐 Go/TS 的 tsconfig 输入集合逻辑：`files: []`、默认 `outDir`/`declarationDir` exclude、common package dirs、literal directory include。

P1：先让 bundled `.d.ts` 能基本解析

- [x] 支持 top-level `declare` declarations 的入口调度：`declare var`、`declare function`、`declare interface`、`declare type` 基础 smoke 已覆盖；复杂声明体仍按后续 type grammar TODO 处理。
- [ ] 支持 type alias declaration：`type A = ...`，包括 exported/ambient 场景。
- [ ] 支持 interface/class/member property signatures 与 method signatures。
- [ ] 支持 call signatures / construct signatures：`() => T`、`new (...) => T`、`abstract new (...) => T`。
- [ ] 支持 generic type parameters：`<T>`、`<T extends U>`、默认类型参数、约束组合。
- [ ] 支持 type arguments / type references：`Foo<T>`、qualified names、nested type refs。
- [ ] 支持 union/intersection types：`A | B`、`A & B`，并处理 precedence。
- [ ] 支持 primitive/keyword type nodes：`string`、`number`、`boolean`、`symbol`、`bigint`、`object`、`unknown`、`any`、`void`、`undefined`、`null`、`never`、`this`。
- [ ] 支持 array/tuple/rest/readonly types：`T[]`、`readonly T[]`、`[A, B]`、`readonly [A]`、`...T[]`。
- [ ] 支持 indexed access / index signatures：`T[K]`、`[K in keyof T]`、`[key: string]: T`。
- [ ] 支持 mapped/conditional/keyof/infer types：`keyof T`、`T extends U ? X : Y`、`infer R`、`{ [K in keyof T]: ... }`。

P2：补项目源码常见语法

- [ ] 支持 named imports/exports：`import { A, B as C } from "x"`、`export { A } from "x"`。
- [ ] 支持 TSX/JSX 基础解析：JSX element、fragment、attributes、expression containers。
- [ ] 支持 arrow function expression 与 typed parameters。
- [ ] 支持 non-null assertion：`expr!`。
- [ ] 支持 object/array binding patterns 与 destructuring。
- [ ] 支持 string/numeric literal types 和 discriminated union members。
- [ ] 支持 `as` assertions、`satisfies`、`as const`。

P3：诊断和级联控制

- [ ] parser 遇到 unsupported type syntax 时改进恢复点，减少同一行/同一声明的级联 `TS1003`。
- [ ] scanner invalid-character diagnostics 接入 parser/compiler diagnostics 路径。
- [ ] 为 bundled lib parsing 添加 smoke test：至少 `lib.es5.d.ts`、`lib.es2015.iterable.d.ts`、`lib.dom.d.ts` 不产生 parser panic，错误数按阶段下降。
- [ ] 为项目 `ai-Color-toner` 添加外部 fixture 或 snapshot，按错误类别跟踪下降趋势。
