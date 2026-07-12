# typescript-go -> typescript-rust TODO

更新时间：2026-07-12

## 当前结论

- Go 源 worktree：`/home/cqh/workspace/typescript-rust`，当前分支 `main`，仍是完整的 `typescript-go` 代码布局。
- Rust 迁移 worktree：`/home/cqh/workspace/typescript-rust-rust`，当前分支 `rust`，已经有初步 Rust crate。
- Rust crate 当前名为 `tsox`，入口在 `src/main.rs`，库入口在 `src/lib.rs`。
- Rust 侧已经有模块骨架：`ast`、`binder`、`checker`、`compiler`、`execute`、`parser`、`scanner`、`printer`、`tsoptions`、`vfs` 等。
- `cargo test` 当前通过：589 个 lib 单测 + 2 个 parity 测试通过。
- 关键缺口：Checker 仍是 stub（`check_source_file` 未实现真实逻辑），Binder 缺 flow graph 与 name resolver；module resolution、watch/build/incremental、fourslash/baseline、npm/vscode 包装尚未迁移到 Rust 方案。

## 2026-07-12 迁移进度快照

| 模块 | Rust 行数 | Go 行数 | 完成度 | 备注 |
|------|-----------|---------|--------|------|
| Scanner | 1558 | 4277 | 36% | 转义/JSX/正则/CommentDirectives/ASI 已完成；缺 trivia 节点、完整 regex 校验 |
| Parser | 7115 | 9251 | 77% | TS6/7 语法、类型语法、JSX、装饰器、import attributes 已完成；缺 reparser/jsdoc |
| Binder | 607 | ~4000 | ~15% | 仅符号声明骨架；缺 flow graph、name resolver、reference resolver |
| Checker | 5545 | ~50K+ | ~11% | 类型数据结构完整；`check_source_file` 为 stub；缺 relater/inference/flow/jsx/jsdoc |
| Compiler | 743 | — | 基础 | Program 创建/解析/绑定/emit pipeline 已通；checker 已接入但无诊断输出 |
| Emitter | 774 | — | 基础 | JS emit 基础；缺 transformer 体系 |
| Printer | 1568 | — | 基础 | 节点→文本基础 |
| AST | ~5500 | — | 基础 | generated 节点 + symbol/flow 类型 |

## 下一阶段重点（2026-07-12 起）

P3 Binder/Checker 是当前最大瓶颈：
1. P3.6 实现 `check_source_file` 真实逻辑（先做 identifier resolution + TS2304 "Cannot find name"）
2. P3.1 Binder flow graph 骨架（为后续 narrowing 打基础）
3. P3.7 Relater 基础规则（`is_type_assignable_to` 基本类型/字面量/union）
4. P3.13 Diagnostics message 表对齐

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
- 2026-07-11 已在 `MIGRATION.md` 的 "Command Line Argument Flow Audit"
  记录 Go/Rust 全量 CLI 参数处理流程和差异。
- 2026-07-11 已在 `MIGRATION.md` 的 "TSConfig Flow Audit" 记录
  Go/Rust `tsconfig.json` 查找、解析、root file 展开和 showConfig
  处理差异。
- 已对齐的普通 CLI 外层行为：
  - command-line parse errors 先报错并退出。
  - `--init` 先于 `--version` / `--help` 处理；存在 `tsconfig.json` 时
    报错，否则写入 `tsconfig.json` 并成功退出。
  - `--version`、`--help` / `--all`、`--watch --listFilesOnly`、
    `--project/-p`、ancestor `tsconfig.json` 查找、`--showConfig` 的
    控制流已有 Rust bridge。
  - `tsconfig.json` 顶层 `references` 和 `compileOnSave` 已解析并进入
    简化版 `--showConfig` 输出。
- 剩余关键差异：Rust option declarations 仍是子集；缺
  NameMap/did-you-mean/alternate-mode diagnostics、完整 watch options、
  TSConfig-only command-line 校验、Go 等价 response-file diagnostics、
  declaration-driven help/showConfig、watch/incremental 执行分支。Rust
  `tsconfig.json` 解析仍缺完整 declaration-driven validation、Node/package
  extends resolution、typed project references、no-input diagnostics 和 Go
  等价 vfsmatch。

任务：

- [x] 记录 Go/Rust 全量 CLI 参数处理流程审计和差异。
- [x] 记录 Go/Rust `tsconfig.json` 解析和处理流程审计及差异。
- [ ] 对齐 `--help`、`--version`、无输入、未知选项、响应文件等 CLI 行为。
- [x] 对齐 `--init` 的执行层控制流，避免继续进入 config 查找/编译。
- [ ] 对齐 `--init` 生成的完整 `tsconfig.json` 模板。
- [ ] 对齐退出码：`Success`、`DiagnosticsPresent_*`、`InvalidProject_*`、`ProjectReferenceCycle_*`。
- [ ] 迁移 Go 的 declaration-driven option parser：NameMap、did-you-mean、alternate-mode diagnostics、TSConfigOnly 规则、enum/list/min-value 校验。
- [ ] 独立建模 watch options，并在普通/build parser 中与 compiler/build options 分离。
- [ ] 对齐 `tsconfig.json` 查找、`extends`、`files/include/exclude`、`compilerOptions` 覆盖规则。
- [x] 解析 `tsconfig.json` 顶层 `references` / `compileOnSave` 并在 `--showConfig` 输出。
- [ ] 将 raw `references` 升级为 typed project references：normalized path、original path、circular。
- [ ] 对齐 `extends` 的 package/Node-style resolution、cycle diagnostics 和 extended config cache。
- [ ] 对齐 no-input diagnostics、config source span diagnostics 和 `vfsmatch` root-file expansion。
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

> 2026-07-11 重新审计：Scanner ~21%（897/4277 行），Parser ~49%（4508/9251 行）。
> 2026-07-12 更新：Scanner ~36%（1558/4277 行），Parser ~77%（7115/9251 行）。P2.5 声明/语句补齐完成（async/abstract/const enum/yield/for await/optional chaining/variance annotations）。P2.8 diagnostic parity 完成。
> record.warn 基线：`ai-Color-toner` 项目 3347 条 TS1003，86.5% 来自 bundled libs 类型语法缺失。

Go 参考：

- `internal/scanner`（scanner.go 2918 + regexp.go 1076 + unicodeproperties.go 162 + utilities.go 100）
- `internal/parser`（parser.go 6827 + jsdoc.go 1355 + reparser.go 748）
- `internal/ast`
- `_scripts/ast.json` / `_scripts/generate-*`

Rust 现状：

- `src/scanner/mod.rs`（1379 行）
- `src/parser/mod.rs`（6584 行）
- `src/ast/*`
- `build.rs`

### P2.0 AST 生成链路

- [ ] 明确 Rust AST 生成链路是否继续读取 Go 侧 `_scripts/ast.json`，还是维护 Rust 自有 schema。
- [ ] 对齐 generated enum/node 数据的生成命令和检查方式。
- [ ] 生成文件可重复生成，`git diff` 干净。

### P2.1 Scanner 基础能力补齐

- [x] 迁移 `scanEscapeSequence` / `scanUnicodeEscape`，替换 `scan_string` 中 `pos += 2` 的简化跳过。
      2026-07-12: `Scanner::scan_escape_sequence` 已实现，正确处理 `\xHH`、`\uHHHH`、`\u{...}`、
      行连接（`\`+`\r`/`\r\n`/`\n`）、单字符转义。`scan_string` 和 `scan_template` 均已更新调用。
      修复了 `\x22` 在 `"` 字符串中导致字符串提前终止的 bug。`unescape_string`（token_value 路径）
      此前已支持完整转义。已添加 5 个转义序列测试。
- [x] 迁移 `reScanGreaterThanTokenInner`：`>` → `>=` / `>>` / `>>>` / `>>=` / `>>>=` 多字符运算符回溯。
      2026-07-12: `Scanner::re_scan_greater_than` 已实现，处理 `>>`/`>>>`/`>=`/`>>=`/`>>>=` → `>`。
      `Parser::re_scan_greater_than` 包装调用。`parse_optional_type_arguments` 和 `parse_optional_type_parameters`
      在 `expect(GreaterThanToken)` 前调用。关键修复：`is_list_terminator` 的 `TypeArguments` 分支
      改为 `token != CommaToken`（对齐 Go），使 `>>` 能正确终止类型参数列表。
      已验证 `Map<string, Array<number>>`、`<T extends A<B>>` 等嵌套泛型场景。
- [x] 迁移 `scanInvalidCharacter` 完整诊断（byte span + message code），接入 parser/compiler diagnostics 路径。
      2026-07-12: Scanner 新增 `errors: Vec<ScannerError>` 内部收集 + `report_error` 方法 + `take_errors` 方法。
      `parse_source_file_text_with_diagnostics` 在解析后调用 `scanner.take_errors()` 将扫描器错误
      转换为 `ParserDiagnostic` 并入诊断列表。同时修复 `scan_string` 未检测 EOF 导致未终止字符串
      无诊断的 bug（现统一在 `!terminated` 时报告）。已有测试验证 `·` 无效字符和 `"unterminated`
      未终止字符串均正确产生诊断，CLI 退出码正确变为 `DiagnosticsPresent_OutputsSkipped`。
- [x] 迁移 `unicodeproperties.go`：用 `unicode-ident` crate 或自建表替换 `is_alphabetic` 简化判断。
      2026-07-12: 已添加 `unicode-ident = "1"` 依赖，`is_unicode_identifier_start`/`is_unicode_identifier_part`
      改用 `unicode_ident::is_xid_start`/`is_xid_continue`，对齐 Go 的 `unicodeESNextIdentifierStart/Part`。
      ZWNJ (U+200C) 和 ZWJ (U+200D) 额外保留。已有非 ASCII 测试（`·`/`中`/`🦀`）通过。
- [ ] 保留 trivia 节点（`WhitespaceTrivia` / `NewLineTrivia` / `CommentTrivia`），对齐 Go 的 `trivia` 输出。
- [x] 收集 `CommentDirectives`（`@ts-expect-error` / `@ts-ignore`）。
      2026-07-12: Scanner 新增 `CommentDirectiveKind`/`CommentDirective` 类型和 `comment_directives` 字段。
      `process_comment_directive` 方法在单行/多行注释扫描后调用，检测 `@ts-expect-error` 和 `@ts-ignore`。
      `SourceFile` 新增 `comment_directives` 字段，parser 在构建 SourceFile 时从 scanner 收集。
      已通过 6 个 scanner 测试 + 1 个 parser 传播测试。
      注意：`/// <reference>` 和 `// @ts-check`/`// @ts-nocheck` 是 parser-side pragmas，属不同机制，待后续实现。
- [x] `PrecedingLineBreak` 在 ASI 路径中完整接入 parser。
      2026-07-12: Scanner 在 `scan_whitespace`/`scan_multi_line_comment` 中正确设置 `preceding_line_break`，
      `scan()` 将其复制到 `has_preceding_line_break`。Parser 的 ASI 路径已完整接入：
      `can_parse_semicolon`/`try_parse_semicolon`/`parse_semicolon`、postfix `++`/`--`、
      non-null assertion `!`、type arguments after expression、`throw` expression、`this is T` type predicate。
      已添加 3 个 ASI 测试（基本 ASI、postfix 跨行禁止、throw 跨行）。

### P2.2 Scanner 正则字面量

- [x] 实现 `reScanSlashToken` 基础：pattern body 扫描（`[...]` 字符类 + `\` 转义）+ flags 消费 + 未终止诊断。
      2026-07-12: `Scanner::re_scan_slash_token` 已实现，处理 `/` 和 `/=` → `RegularExpressionLiteral`。
      正确处理字符类内的 `/`、转义 `/`、flags 消费、未终止（EOF/换行）诊断。
      `Parser::re_scan_slash_token` 包装调用，`parse_primary_expression` 的 `SlashToken`/`SlashEqualsToken` 分支
      调用后产生 `RegularExpressionLiteral` 节点。已通过 7 个 scanner 测试 + 2 个 parser 测试。
      Go 的 `ReScanSlashToken` 由 parser 在 primary expression 位置触发（不是 scanner 自动检测上下文）。
- [ ] 迁移 `internal/scanner/regexp.go` 完整 regex body 校验（`regExpParser`：命名捕获组、`u`/`v` flag 模式、invalid flag 诊断）。
- [ ] 支持 `lastIndex`、命名捕获组、`d` flag 等现代正则特性。

### P2.3 Scanner JSX / JSDoc

- [x] 迁移 `ScanJsxToken` / `ScanJsxIdentifier` / `ScanJsxAttributeValue`。
      2026-07-12: Scanner 新增 `scan_jsx_token` / `scan_jsx_token_ex`（处理 `<`、`</`、`{`、JsxText/JsxTextAllWhiteSpaces）、`scan_jsx_identifier`（扩展带 dash 的标识符如 `my-component`）、`scan_jsx_attribute_value`（跳过空白后扫描引号字符串或回退到 `scan()`）。Parser 新增 `scan_jsx_text` / `scan_jsx_identifier` / `scan_jsx_attribute_value` 辅助方法。全面重写 JSX parser 函数：
      - `parse_jsx_element_or_fragment(in_expression_context)`: 使用 `expect_without_advancing(GreaterThanToken)` + `scan_jsx_text()` 替代 `expect(GreaterThanToken)` + `scan_jsx_text()`（避免跳过 token）。
      - `parse_jsx_name`: 在 `parse_identifier_name_or_keyword` **之前**调用 `scan_jsx_identifier()`（与 Go `parseJsxTagName` 一致）。
      - `parse_jsx_attribute`: 简化 spread 判断为 `OpenBraceToken` 即 spread（与 Go 一致），属性名前调用 `scan_jsx_identifier`。
      - `parse_jsx_expression(in_expression_context)`: 非 expression context 下 `}` 后用 `scan_jsx_text`；expression context 下用 `expect(CloseBraceToken)`。
      - `parse_jsx_children`: 改用 match + `LessThanSlashToken` 判断（替代 `LessThanToken` + `SlashToken` 前瞻），显式处理 `JsxText`/`JsxTextAllWhiteSpaces`。
      - `parse_jsx_text`: 使用 `scan_jsx_text()` 替代 `next_token()`，根据 `JsxTextAllWhiteSpaces` 设置 `contains_only_trivia_white_spaces`。
      - `parse_jsx_closing_element(in_expression_context)`: 使用 `LessThanSlashToken`（替代 `LessThanToken` + `SlashToken`），`>` 后根据 context 选择 `next_token()` 或 `scan_jsx_text()`。
      新增 `expect_without_advancing` 辅助方法。新增 8 个 JSX 测试（simple element、fragment、self-closing、dashed tag name、expression children、nested elements、spread attribute、member expression tag）。578 tests pass, 108 bundled libs 0 diagnostics。
- [ ] 迁移 `ScanJSDocToken` + `scanJSDocCommentForTags`（依赖 P2.7 JSDoc parser）。

### P2.4 Parser 类型语法补齐（record.warn P1，最高优先）

- [x] type alias declaration 完整：`type A = ...`，含 exported/ambient 场景。
      2026-07-12: `parse_type_alias_declaration_with_modifiers` 已支持 modifiers（export/declare）。已验证 `export type`、`declare type` 正常调度。
- [x] call signature / construct signature：`() => T`、`new (...) => T`、`abstract new (...) => T`。
      2026-07-12: 修复 `parse_parenthesized_or_function_type` 错误地总是尝试解析参数列表。添加 `is_start_of_function_type_with_open_paren` 前瞻检测，正确区分 `(type)` 与 `(params) => T`。`new () => T` 和 `abstract new () => T` 由 `parse_constructor_type` 处理。
- [x] generic type parameters：`<T>`、`<T extends U>`、默认类型参数、约束组合 `extends A & B`。
      2026-07-12: `parse_optional_type_parameters` / `parse_type_parameter` 已实现。修复 `is_list_element` 未包含 `TypeParameters` 导致 `<T extends U>` 解析失败的问题。
- [x] type arguments / type references：`Foo<T>`、qualified names `A.B.C`、nested type refs。
      2026-07-12: `parse_type_reference` / `parse_optional_type_arguments` 已实现。已验证 `A.B.C<T>` 正常解析。
- [x] union/intersection precedence：`A | B & C` 正确分组。
      2026-07-12: `parse_union_type_or_higher` → `parse_intersection_type_or_higher` → `parse_type_operator_or_higher` 链已实现，intersection 优先级高于 union。
- [x] primitive keyword type nodes：`string`/`number`/`boolean`/`symbol`/`bigint`/`object`/`unknown`/`any`/`void`/`undefined`/`null`/`never`/`this`。
      2026-07-11: `parse_non_array_type` 中 keyword type 现在返回 `NodeData::KeywordTypeNode`（kind 仍为 keyword kind），
      而非错误的 `TypeReference`。含 dotted type reference 回退（`String.fromCharCode`）。
- [x] array/tuple/rest/readonly：`T[]`、`readonly T[]`、`[A, B]`、`readonly [A]`、`...T[]`。
      2026-07-12: `parse_postfix_type_or_higher` 支持 `T[]` 和 `T[K]`。`parse_tuple_element_type` 支持命名元组 `name: T`、可选元素 `T?`、rest `...T`。`readonly` 由 `parse_type_operator_or_higher` 处理。
- [x] indexed access / index signatures：`T[K]`、`[K in keyof T]`、`[key: string]: T`。
      2026-07-12: Indexed access `T[K]` 已由 `parse_postfix_type_or_higher` 支持。Index signatures `{ [key: string]: T }` 和 `{ readonly [key: string]: T }` 由 `parse_type_member` 处理。Mapped type `{ [K in keyof T]: V }` 由 `parse_mapped_type` 处理。
- [x] mapped types：`{ [K in keyof T]: V }`、`-?`、`+readonly`。
      2026-07-12: `parse_mapped_type` / `parse_mapped_type_parameter` / `next_is_start_of_mapped_type` 已实现。支持 readonly/±readonly、optional/±?、`as` key remapping。
- [x] conditional types：`T extends U ? X : Y`、distributive。
      2026-07-12: `parse_type` 中 `extends` 检测已实现 `ConditionalTypeNode`。`infer R` 由新增的 `parse_infer_type` 方法支持。已验证嵌套条件类型和 `(infer U)[]` 模式。
- [x] `keyof T` / `infer R` / `typeof x` / `import("x").T`。
      2026-07-11: `typeof X` → `TypeQuery`，`import("x").T` → `ImportType`，`typeof import("x").T` → `ImportType(is_type_of=true)`。
      2026-07-12: `infer R` 由新增的 `parse_infer_type` 方法支持。
      `keyof T` 已由 `parse_type_operator_or_higher` 支持。
- [x] `as const` / `satisfies T` / non-null assertion `expr!`。
      2026-07-12: Non-null assertion `expr!` 由 `parse_call_and_member_chain` 中新增的 `ExclamationToken` 分支支持。`as const` 和 `satisfies T` 此前已支持。
- [x] string/numeric literal types 与 discriminated union members。
      2026-07-11: 负数字面量类型 `-1` → `LiteralType` 已支持。`this is T` 和 `asserts x` type predicate 已支持。
      2026-07-12: `identifier is T` type predicate 在返回类型位置已支持（`parse_type_or_type_predicate`）。
      修复了 `lib.es5.d.ts` 中 `(value: T) => value is S` 等函数类型返回类型的解析错误。
      `lib.es5.d.ts` 现可零错误解析（从基线 3347 TS1003 降至 0）。
- [x] template literal types：`` `prefix${T}suffix` ``。
      2026-07-11: `parse_template_type` / `parse_template_type_spans` / `parse_template_type_span` 已实现。

### P2.5 Parser 声明/语句补齐

- [x] 完整 `declare` 声明调度：`declare module`、`declare namespace`、`declare global`、`declare class` 完整体。
      2026-07-12: 新增 `parse_ambient_external_module_declaration` 处理 `declare module "name"` 和 `declare global`。
      `parse_declaration_with_modifiers` 中添加 `GlobalKeyword` 调度。修复 `parse_enum_member` 错误调用 `parse_semicolon`
      和 `parse_enum_declaration` 使用 `parse_list` 而非 `parse_delimited_list` 的问题。
      已验证：declare module "foo"、declare namespace A.B.C、declare global、declare class/enum/interface/type/var/function 全部通过。
- [x] 装饰器 detailed parsing（`@Decorator` + 参数装饰器 + 元数据）。
      2026-07-12: `parse_decorator` 解析 `@` + left-hand-side expression。`make_modifier_list_with_decorators`
      合并 token modifiers 与 decorator 节点（设置 `ModifierFlags::Decorator`）。`parse_declaration_with_modifiers`
      的 modifier 循环里收集 `@` decorators。`parse_statement` 的 `AtToken` 分支处理顶层 decorators。
      `parse_class_member` 在解析成员名前收集 decorators/modifiers。已通过 6 个测试：
      `@decorator class`、`@decorator` on method/property、`@Dec({ option: true })` 带参数、
      `@A @B` 多 decorator、`@Namespace.Dec` 成员表达式。
- [x] import attributes：`import x from "y" with { type: "json" }`。
      2026-07-12: `try_parse_import_attributes` / `parse_import_attributes` / `parse_import_attribute` 已实现。
      支持 `with { key: value }` 和已废弃的 `assert { ... }`。`is_list_element` 新增 `ImportAttributes` 分支
      （`is_identifier_or_keyword || StringLiteral`）。`parse_import_declaration` 和 `parse_export_declaration`
      在 module specifier 后调用 `try_parse_import_attributes`。已添加 3 个测试。
- [x] named imports/exports 完整：`import { A, B as C } from "x"`、`export { A } from "x"`、`export * as N from "x"`。
      2026-07-12: 经审计，`parse_named_imports` / `parse_import_specifier`（含 `type` 修饰符和 `as` 重命名）、
      `parse_named_exports` / `parse_export_specifier`、`parse_namespace_import`、`export * as N from "x"`
      （NamespaceExport）均已实现。现有测试已覆盖。
- [x] object/array binding patterns 与 destructuring 默认值。
      2026-07-12: 经审计，`parse_identifier_or_pattern` → `parse_array_binding_pattern` / `parse_object_binding_pattern`
      已实现。支持 rest 元素 `...rest`、默认值 `= expr`、嵌套 pattern、属性重命名 `{ a: b }`。现有测试已覆盖。
- [x] TS 6/7 新语法（`using`/`accessor` 语义位置补齐）。
      2026-07-12: `using x = ...` 声明：`is_using_declaration` 前瞻检测 `using` 后跟绑定标识符或 `{` 且无换行，
      `parse_statement` 的 `UsingKeyword` 分支调用 `parse_variable_statement`，`NodeFlags::Using` 已设置。
      `await using x = ...` 声明：`is_await_using_declaration` 前瞻检测，`parse_statement` 的 `AwaitKeyword` 分支，
      `parse_variable_declaration_list` 处理 `AwaitKeyword` → `NodeFlags::AwaitUsing`。
      `accessor` 修饰符：`modifier_flag` 新增 `AccessorKeyword => ModifierFlags::Accessor`。
      已通过 3 个测试：using 声明（含 `using = 1` 标识符回退和换行回退验证）、await using 声明、accessor 属性。
      `defer` 关键字已在 scanner 中，但 `defer {}` 语句语法尚未标准化（Stage 3），暂不实现。
- [x] 修饰符关键字路由：`async`/`abstract`/`const enum`/`static`/`readonly`/`public`/`private`/`protected`/`accessor`/`global` 声明调度。
      2026-07-12: `parse_statement` 新增修饰符关键字分支，通过 `is_start_of_declaration` → `parse_declaration_with_modifiers` 路由。
      `is_start_of_declaration` + `scan_start_of_declaration` 完整移植 Go 的前瞰逻辑（含 `clone_state` 方法）。
      `parse_declaration_with_modifiers` 的 modifier 循环新增 `ConstKeyword`/`AccessorKeyword`/`OverrideKeyword`，
      其中 `const` 仅在后跟 `enum` 时作为修饰符（对齐 Go `nextTokenCanFollowModifier`）。
      `is_let_declaration` 前瞻检测 `let` 后跟绑定标识符或解构模式。
- [x] `yield` 表达式（生成器上下文）。
      2026-07-12: Parser 新增 `yield_context`/`await_context` 字段，`parse_function_block` 在生成器/async 函数体中设置上下文。
      `is_yield_expression` + `parse_yield_expression` 实现完整 yield 语义（`yield expr`、`yield* expr`、无值 `yield`）。
      `parse_assignment_expression` 优先检查 yield。
- [x] `for await...of` 循环。
      2026-07-12: `parse_for_statement` 在 `for` 后检测 `await` 关键字，消费为 `await_modifier` 并传入 `ForOfStatement`。
- [x] 可选链 `?.`（QuestionDotToken）。
      2026-07-12: Scanner `text_to_token` 映射新增 `?.` → `QuestionDotToken`（此前缺失）。Parser `parse_member_expression_rest` 已有 `QuestionDotToken` 分支。
- [x] 类型参数方差注解（`in`/`out`/`const`）。
      2026-07-12: `parse_type_parameter_modifiers` 新增，收集 `InKeyword`/`OutKeyword`/`ConstKeyword` 作为类型参数修饰符。
      `parse_type_parameter` 调用该方法并在 `TypeParameterDeclarationData.modifiers` 中存储。

### P2.6 Parser reparser

- [ ] 迁移 `internal/parser/reparser.go`（748 行）到 `src/parser/reparser.rs`。
- [ ] `@typedef` JSDoc → type alias 节点追加到 statements。
- [ ] `reparseTopLevelAwait`：外部模块 + `possibleAwaitSpans` 重解析。
- [ ] `collectExternalModuleReferences`。

### P2.7 Parser JSDoc

- [ ] 迁移 `internal/parser/jsdoc.go`（1355 行）到 `src/parser/jsdoc.rs`。
- [ ] `parseJSDocComment`：tag 类型、`@param`、`@returns`、`@typedef`、`@callback`、`@template`。
- [ ] JSDoc type expression：`@type {string}`、`@param {string} name`。
- [ ] 节点附加 `jsDoc` 字段。

### P2.8 Parser diagnostic parity

- [x] diagnostic code 对齐（TS1003 等）。
      2026-07-12: `ParserDiagnostic` 从 `{ message: String }` 重构为 `{ message: Message, message_args: Vec<String>, range: TextRange }`，
      携带正确的 code/category/key/text。`parser_diagnostic_to_diagnostic` 不再硬编码 TS1003，改用 `Diagnostic::new(Some(file), pd.range, pd.message, pd.message_args)`。
      `expect()` 使用 `X_0_EXPECTED` + `token_to_string(expected)`；`parse_identifier()` 使用 `IDENTIFIER_EXPECTED`；
      unexpected token 使用 `UNEXPECTED_TOKEN`；scanner errors 使用 `INVALID_CHARACTER`/`UNTERMINATED_STRING_LITERAL`/`UNTERMINATED_TEMPLATE_LITERAL`/`UNTERMINATED_REGULAR_EXPRESSION_LITERAL`。
- [x] 错误消息文本对齐 Go（含参数插值）。
      2026-07-12: `Message::format` 使用 `{0}`/`{1}` 占位符插值，对齐 Go `diagnostics.Message.Format`。
      `token_to_string` 函数（scanner/mod.rs）反转 keywords/punctuation 映射，为 `X_0_EXPECTED` 等消息提供 token 文本。
      添加了 `identifier`/`end of file`/`numeric literal`/`string literal`/`template literal` 等非 keyword/punctuation token 的文本映射。
- [x] 级联错误恢复点优化，减少同一行的 TS1003 风暴。
      2026-07-12: 完整移植 Go 的 `parseErrorAtRange`/`parseErrorAt`/`parseErrorAtCurrentToken` 去重逻辑：
      如果上一个错误的位置与当前位置相同，则不再报告新错误。
      移植 `parsingContextErrors`：24 个 ParsingContext 变体各自映射到特定诊断消息（如 `SourceElements` → `DECLARATION_OR_STATEMENT_EXPECTED`，
      `TypeMembers` → `PROPERTY_OR_SIGNATURE_EXPECTED`，`ClassMembers` → `UNEXPECTED_TOKEN_A_CONSTRUCTOR_METHOD_ACCESSOR_OR_PROPERTY_WAS_EXPECTED`）。
      移植 `abortParsingListOrMoveToNextToken`：报告上下文错误后，如果处于某个外层列表的终止符位置则中断，否则跳过当前 token 继续。
      此前 `parse_list`/`parse_delimited_list` 遇到无效 token 时静默跳过，现在会正确报告错误。
      Scanner errors 绕过去重逻辑（直接 push 到 diagnostics 向量），避免被同位置的 parser 错误抑制。
- [x] `is_list_element` 完整对齐 Go 的 `isListElement`。
      2026-07-12: 补齐此前缺失的 `ImportOrExportSpecifiers`/`ObjectBindingElements`/`ArrayBindingElements`/`RestProperties`/
      `HeritageClauses`/`JsxAttributes`/`JsxChildren`/`JSDocParameters`/`JSDocComment` 分支。
      `ImportOrExportSpecifiers` 支持 `from "mod"` 前瞻检测（bail out 以提供更好错误消息）和 StringLiteral/arbitrary module namespace identifiers。
      `TypeArguments`/`TupleElementTypes` 补齐 `CommaToken` 检查；`ArrayLiteralMembers` 补齐 `CommaToken`/`DotToken`/`DotDotDotToken` 检查；
      `ArgumentExpressions` 补齐 `DotDotDotToken` 检查。
      此前 `ImportOrExportSpecifiers` 缺失导致 `import { foo } from "y"` 中 `foo` 被误判为非列表元素，触发错误恢复和虚假 "Identifier expected" 诊断。
- [x] `is_start_of_type` 补齐 Go 的 `isStartOfType` 缺失 token。
      2026-07-12: 添加 `DotDotDotToken`（rest 类型）、`AmpersandToken`（intersection 类型）、`AsteriskToken`、`QuestionToken`、
      `ExclamationToken`、`MinusToken`、`TemplateHead`。此前 `type T = [string, ...number[]]` 等会报 "Type expected" 错误。
- [x] 语法错误位置：UTF-16 offset 对齐 Go（见 P2.10）。
      2026-07-12: 已在 P2.10 中完成。`line_and_character` 返回 UTF-16 列号，对齐 Go `GetECMALineAndUTF16CharacterOfPosition`。

### P2.9 Parser parity fixtures

- [ ] 从 Go parser 测试或 TypeScript baselines 中挑选 smoke 集合，转成 Rust parity。
- [ ] 典型 `.ts` 解析结果和诊断对齐 oracle。
- [ ] 典型 `.tsx` JSX 解析对齐。
- [ ] 典型 `.js` / `.jsx` 对齐。
- [x] bundled lib smoke：`lib.es5.d.ts` 零错误解析（2026-07-12 从基线 3347 降至 0）。`lib.dom.d.ts` 等待验证。
      2026-07-12: **全部 bundled libs（100+）均零错误解析**（从 191 diagnostics 降至 0）。
      修复三个 parser 缺口：
      (1) 计算属性名 `[Symbol.iterator]()` 在 type member 位置被误判为 index signature —
      `is_index_signature_start` 改用 Go 的 `nextIsUnambiguouslyIndexSignature` 2-token 前瞻。
      (2) 上下文关键字（`static`/`private`/`public`）在 type member/class member 位置被贪婪消费为 modifier —
      `parse_type_member_modifiers`/`parse_class_member`/`parse_declaration_with_modifiers` 新增
      `canFollowModifier` 检查（对齐 Go `tryParseModifier`）。
      (3) heritage clause 类型参数中的 tuple 类型 `extends Foo<[number, number]>` 解析失败 —
      `is_list_element` 的 `HeritageClauseElement` 分支缺失（默认返回 false），导致 token 被跳过。
- [ ] bundled lib smoke：`lib.es2015.iterable.d.ts`、`lib.dom.d.ts` 错误数验证。

### P2.10 位置信息一致性

- [x] `LineMap` 对齐 Go `ComputeECMALineStarts`。
      2026-07-12: `LineMap::from_text` 重写为逐字节扫描，处理 ECMAScript 行终止符：
      LF (`\n`)、CR (`\r`)、CRLF (`\r\n`)、LS (`\u2028`)、PS (`\u2029`)。
      此前只处理 `\n`，CRLF 文件会产生错误行号。
- [x] UTF-16 column 计算。
      2026-07-12: 新增 `utf16_len` 函数（对齐 Go `core.UTF16Len`），计算字符串的 UTF-16 码元数。
      `LineMap::utf16_column_at` 使用 `utf16_len(&text[line_start..offset])` 计算 UTF-16 列号。
      `line_and_character` 改为返回 UTF-16 列号（对齐 Go `GetECMALineAndUTF16CharacterOfPosition`），
      此前返回 byte 列号。diagnostic writer 的 `format_diagnostic_compact`/`format_diagnostic_pretty`/`code_snippet`
      全部更新传入 `&file.text`。已添加 7 个测试：CRLF、CR-only、LS/PS、ASCII/非 ASCII/emoji UTF-16 列号。

验收：

- [x] `lib.es5.d.ts` 可零错误解析。（2026-07-12: 已达成，0 diagnostics）
- [ ] 典型 `.ts/.tsx/.js/.jsx` 解析结果和诊断可对齐 oracle。

## P3：Binder / Checker / Diagnostics parity

> 2026-07-11 重新审计：Binder ~17%（607/3555 行），Checker ~6%（4908/77157 行）。
> 关键问题：Checker 从未被 compiler 调用；binder 控制流图只初始化了 START/UNREACHABLE。

Go 参考：

- `internal/binder`（binder.go 2795 + nameresolver.go 498 + referenceresolver.go 262）
- `internal/checker`（checker.go 31926 + relater.go 5006 + flow.go 2734 + grammarchecks.go 2202 + inference.go 1651 + nodebuilderimpl.go 3585 + emitresolver.go 1322 + jsx.go 1482 等）
- `internal/diagnostics`
- `internal/nodebuilder`
- `internal/pseudochecker`

Rust 现状：

- `src/binder/mod.rs`（607 行）
- `src/checker/*`（checker.rs 999 + types.rs 1791 + typenode.rs 550 + relater.rs 423 + utilities.rs 701 + mapper.rs 232 + tracer.rs 186）
- `src/diagnostics/*`

### P3.1 Binder 控制流图（checker narrowing 前置依赖）

- [ ] 迁移 `internal/binder/binder.go` 中的 flow 构建逻辑（~1500 行）。
- [ ] `ASSIGNMENT` flow node：变量赋值时创建。
- [ ] `TRUE_CONDITION` / `FALSE_CONDITION`：`if (x)` / `while (x)` 分支条件。
- [ ] `SWITCH_CLAUSE`：switch case 分支。
- [ ] `LOOP_LABEL` / `BRANCH_LABEL`：break/continue 标签。
- [ ] `ARRAY_MUTATION`：方法调用副作用。
- [ ] `CALL` flow node：函数调用对 narrow 类型的影响。
- [ ] `REduceLabel` / `Shared` / `Referenced` 后处理。

### P3.2 Binder NameResolver

- [ ] 迁移 `internal/binder/nameresolver.go`（498 行）到 `src/binder/nameresolver.rs`。
- [ ] 作用域链查找：`lookupName` / `lookupSymbol`。
- [ ] 特殊符号：`undefinedSymbol`、`argumentsSymbol`、`globalThisSymbol`。
- [ ] `resolveName` 入口供 checker 调用。

### P3.3 Binder ReferenceResolver

- [ ] 迁移 `internal/binder/referenceresolver.go`（262 行）。
- [ ] 标识符引用记录（用于 find references / rename）。

### P3.4 Binder 声明合并与 export/import binding

- [ ] declaration merge：namespace + function + interface + class 合并规则。
- [ ] export binding：`export { A }` 的 `exportSymbol` → local symbol 链。
- [ ] import binding：`import { A }` 的 `aliasSymbol` → resolved symbol。
- [ ] `delayedSymbol` / `aliasSymbol` 特殊符号处理。
- [ ] 完整 scope 链（当前只有 `container`/`block_scope_container` 两个字段）。

### P3.5 Checker 接入 compiler

- [x] 在 `src/compiler/mod.rs` 的 `Program::new` 后调用 `Checker::new` + `check_source_file`。
      2026-07-12: `Program::get_semantic_diagnostics(self: &Arc<Self>)` 创建 `Checker`，对每个 source file 调用 `check_source_file`，返回 `get_semantic_diagnostics`。
      execute 模块中 `perform_compilation` 改用 `Arc<Program>`，在 parse diagnostics 后调用 `get_semantic_diagnostics` 并合并 error_count。
      Checker `check_source_file` 当前为 stub（P3.6），返回空 diagnostics，但管线已完整接入。
- [ ] `Program` trait 补全 checker 所需方法（`getCommonSourceDirectory`、`getCanonicalFileName` 等）。
- [x] checker diagnostics 接入 program diagnostics 输出。
      2026-07-12: `Checker::get_semantic_diagnostics` 返回 `Vec<Diagnostic>`，execute 模块将其包装为 `Arc<Diagnostic>` 后调用 `report_diagnostics` 输出。

### P3.6 Checker 核心入口

- [ ] 实现 `check_source_file`：遍历 statements。
- [ ] 实现 `check_statement`：variable / function / class / interface / type alias / enum / import / export。
- [ ] 实现 `check_expression`：identifier / literal / binary / call / member access / arrow / object literal / array literal。
- [ ] 节点 → 类型缓存（`get_cached_type` / `cache_type` 已有骨架，需在 check 路径中填充）。

### P3.7 Checker 类型关系（relater 完整规则）

- [ ] 迁移 `internal/checker/relater.go`（5006 行）到 `src/checker/relater.rs`（当前 423 行）。
- [ ] `is_type_assignable_to` 完整规则：基本类型、字面量、union/intersection、对象、数组、tuple、函数、泛型、条件类型、映射类型。
- [ ] `is_type_subtype_of` / `is_type_strict_subtype_of`。
- [ ] `is_type_comparable_to`。
- [ ] `relation_comparison_result` 缓存与递归保护。

### P3.8 Checker 类型推断

- [ ] 迁移 `internal/checker/inference.go`（1651 行）到 `src/checker/inference.rs`。
- [ ] 泛型推断：`inferTypeArguments`。
- [ ] contextual typing：`getContextualType`。
- [ ] 二元运算符类型推断。
- [ ] 条件类型 `infer R` 解析。

### P3.9 Checker 控制流 narrowing

- [ ] 依赖 P3.1 binder flow graph。
- [ ] `narrowType`：根据 flow node 收窄类型（`if (x !== null)` → 排除 null）。
- [ ] `getNarrowedTypeOfSymbol`。
- [ ] discriminated union narrowing。
- [ ] `typeof` / `instanceof` / `in` narrowing。

### P3.10 Checker nodebuilder

- [ ] 迁移 `internal/checker/nodebuilderimpl.go`（3585 行）+ `nodebuilder.go` + `nodebuilder_hover.go` + `nodebuilderscopes.go` + `pseudotypenodebuilder.go` + `nodecopy.go`。
- [ ] `type_to_string`：类型 → 可读字符串（当前 `type_to_string` 只是 stub）。
- [ ] `symbol_to_type_node` / `symbol_to_display_parts`。
- [ ] hover 信息生成。

### P3.11 Checker emitresolver

- [ ] 迁移 `internal/checker/emitresolver.go`（1322 行）。
- [ ] `getEmitResolver`：emit 阶段所需的符号信息（`isDeclarationVisible`、`isOptional` 等）。

### P3.12 Checker JSX / JSDoc / Grammar checks

- [ ] 迁移 `internal/checker/jsx.go`（1482 行）：JSX 元素类型检查、属性检查。
- [ ] 迁移 `internal/checker/jsdoc.go`：JSDoc 类型检查。
- [ ] 迁移 `internal/checker/grammarchecks.go`（2202 行）：语法层面规则（`override`、`abstract`、`accessor` 等）。

### P3.13 Diagnostics message 表

- [ ] 迁移/生成 Go 的 diagnostic message 表，避免手写漂移。
- [ ] 对齐主要 diagnostic code、category、message、span。
- [ ] 错误消息插值参数对齐。

### P3.14 Checker parity fixtures

- [ ] 建立 type-check parity fixtures，先覆盖最小闭环：变量、函数、类、接口、泛型、union/intersection。
- [ ] `.js` + JSDoc 行为测试。
- [ ] JSX type-check smoke。
- [ ] 累计至少 50 个 checker parity fixtures 通过。

验收：

- [ ] `cargo test` 中出现 `check_source_file` 调用路径的单测。
- [ ] Rust 能在 `ai-Color-toner` 项目上输出与 Go oracle 一致的诊断集合（数量级一致）。
- [ ] 至少 50 个 checker parity fixtures 通过。
- [ ] `tsox -p tsconfig.json` 在真实项目上输出非空类型错误诊断（当前为空，因为 checker 未接入）。

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
  - 已增加临时 raw `references` DFS：solution config 会先访问被引用项目；
    自身 `files: []` 时不会再尝试编译空根项目。
- 剩余关键差异：Rust build parser 仍缺 build-specific did-you-mean 和完整
  watch-options 模型；仍缺 typed project reference graph、`.tsbuildinfo`、
  up-to-date 判定、clean/dry/force/verbose 的真实 orchestrator 行为、watch
  build mode。

任务：

- [x] 记录 Go/Rust `-b` 流程审计和差异。
- [x] 修正 `-b` 外层 dispatch，不再把 build mode 伪装成普通 source-file 编译。
- [x] 迁移 `ParseBuildCommandLine` 的核心等价物，区分 build options / compiler options / projects，并校验非法组合。
- [ ] 补齐 build parser 的 build-specific did-you-mean 和完整 watch options。
- [x] 支持 `--build` / raw project references 基本 DFS 桥接。
- [ ] 支持 Go 等价的 typed project reference graph。
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
