# 格式化引擎移植（Go tsc/internal/format → crates/tsox-frontend/src/format）

> 版本：v0.1 · 2026-09-11

## 现状

- `crates/tsox-frontend/src/format/mod.rs` 是纯桩（format_document/format_selection 返回空）。
- 语料中 157 个格式化用例（format*/formatting*/format_selection*/as_operator_formatting 等）全部失败，是 323 个失败中最大的单一簇。
- 上游参照：`TypeScript/tsc/internal/format/`（约 2600 行核心）与 `tsc/internal/ls/format.go`（LS 适配层）。

## 上游结构（已勘察）

| Go 文件 | 行数 | 内容 | Rust 落点（建议） |
|---|---|---|---|
| scanner.go | 374 | 格式化扫描器：逐 token 产出 leading trivia / token / trailing trivia 三段；`advance()`/`readTokenInfo(node)`/`isOnToken`；`shouldRescanGreaterThanToken`（泛型箭头 `>>` 拆分）、JSX 标识符重扫 | format/scanner.rs |
| rules.go | 450 | 135 条 `rule(...)` 规格：按 (上一个 token 范围, 当前 token 范围) 键注册，含 `tokenRange` 组合子（anyTokenExcept/keywords/binaryOperators/unary* 等）；`getRulesForTokenContextPair` 用 context mask 索引 | format/rules.rs |
| rule.go | 109 | Rule 结构（action: Ignore/Space/NewLine/Delete + flags） | format/rule.rs |
| rulecontext.go | 629 | RuleContext 构造：token 对的 parent 链、RangeToTokenRange 重扫（大于号、JSX）、`NodeWillIndentChild` 等谓词 | format/rule_context.rs |
| span.go | 1262 | formatSpanWorker：execute 主循环（processNode 递归 + processPair 逐 token 对产出 TextChange）、trailing-range 特例、缩进 trivia | format/span_worker.rs |
| indent.go | 821 | 动态缩进器：getIndentationForNode/absolute 首行缩进/delta | format/indent.rs |
| ls/format.go | — | LS 适配：ProvideFormatDocument/Range/OnType → TextEdits | tsox-lsp ls/format.rs（新） |

## 移植顺序（每步可独立验证）

1. **扫描器** scanner.rs：移植 formattingScanner（注意 `SetSkipTrivia(false)`、startPos 前 `wasNewLine=true` 初始化）。验证：单元测试对比 Go 单测的 token 三段切分。
2. **规则骨架** rule.rs + rules.rs：先移植 tokenRange 组合子 + 全部 135 条规格（机械翻译，规格表是数据不是逻辑）。
3. **上下文** rule_context.rs：RuleContext 字段与谓词（`IsArgument`、`NodeWillIndentChild` 等）。
4. **worker** span_worker.rs：execute 主循环 + processPair + processNode/processChildNodes 递归 + 结尾 trailing-pair 特例（span.go:300-330 的连续性检查）。
5. **缩进** indent.rs：dynamicIndenter + getIndentationForNode。
6. **LS 适配**：LanguageService::provide_format_document/range（默认 FormatCodeSettings：fourslash 默认 tab=4、convertTabsToSpaces、trimTrailing=false——以 Go lsutil.GetDefaultFormatCodeSettings 为准）+ fourslash api 的 format_document/format_selection/format_on_enter 接线。
7. **OnEnter/OnType**：getFormattingEditsAfterKeystroke（format.go:227）。

## 验收

- 语料 format* 簇 157 例逐步转绿（format01 最简：`namespace Default{var x= ( { } ) ;}` → `namespace Default { var x = ({}); }`）。
- 每步跑 `tools/fourslash_shard.py`（内存护栏）确认无回归、无 oom。
- 前端 crate 单测（format/tests.rs 现为空壳，补 Go format_test.go 的对应断言）。

## 勘察补充（2026-09-11）

- 我们扫描器已具备：`set_range`（=ResetTokenState）、`re_scan_greater_than`、`re_scan_slash_token`、`scan_jsx_identifier`、`set_language_variant`，且默认产出 trivia（无需 SetSkipTrivia 开关）。
- 待补两个小件：`re_scan_template_token`、`re_scan_jsx_token`/`re_scan_jsx_attribute_value`。
- 规则表为 135 条 `rule(name, left, right, context, action, flags)` 规格，`tokenRange` 组合子全部机械可译；规则动作位掩码（StopProcessing/InsertSpace/InsertNewLine/DeleteSpace/DeleteToken/InsertTrailingSemicolon）见 rule.go。
- worker 的 AST 遍历用 NodeVisitor 回调实现 processChildNode/processChildNodes——移植时用等价递归即可，不需要 visitor 抽象。

## 风险与注意

- 规则表是数据：整体机械翻译，不要按直觉增删——否则修 A 破 B。
- `>>` 重扫（shouldRescanGreaterThanToken）与 JSX 标识符重扫是最易错的两个特例。
- 逐用例失败原因可批量提取：`cargo test -- <names>` + 解析 `---- cases::X stdout ----` 块（多过滤器要放 `--` 之后）。
- 全量验证一律走 `tools/fourslash_shard.py`（内存护栏 + 断点续跑），脚本自带重建二进制。
