# typescript-go -> typescript-rust 迁移任务

## 项目背景

- Go oracle：`/Users/cqh/workspace/typescript-go`，分支 `go`
- Rust 迁移：`/Users/cqh/workspace/typescript-rust`，分支 `rust`
- Crate：`tsox`，edition 2024，rust-version 1.96

## 验收命令

```sh
cargo test

# parity 集成测试，需 Go oracle
TSGO_ORACLE=/Users/cqh/workspace/typescript-go/built/local/tsgo cargo test --test parity
```

## 当前进度（2026-08-04）

**1,290 lib 通过**（+ 920 checker_parity + 2 emit），**0 ignored**。

## 依赖关系图

```
scanner/parser (95-100%)
    │
    ├── binder (75%) ← 需要 nameresolver + referenceresolver
    │
    ├── ast/utilities (45%) ← 需要 4,565 行辅助函数
    │
    ├── checker/checker.rs (25%) ← 核心瓶颈，需要表达式/语句检查
    │   ├── checker/relater.rs (70%)
    │   ├── checker/flow.rs (98%)
    │   ├── checker/inference.rs (100%)
    │   ├── checker/nodebuilder.rs (55%)
    │   ├── checker/services.rs (0%) ← 关键缺失！LSP 的入口
    │   ├── checker/exports.rs (0%)
    │   ├── checker/symbolaccessibility.rs (0%)
    │   └── checker/nodecopy.rs (0%)
    │
    ├── compiler/ (25%) ← 需要 emitter 集成 + fileInclude
    │
    ├── module/ (90%) ← 基本完成
    │
    ├── modulespecifiers/ (18%) ← 需要大量补全
    │
    └── LSP 层 (骨架完成，5-10% 功能实现)
        ├── ls/hover.rs ✅ 已接入 checker
        ├── ls/definition.rs ✅ 已接入 checker
        ├── ls/folding.rs ✅ 已实现（纯 AST）
        ├── ls/selection_ranges.rs ✅ 已实现（纯 AST）
        ├── ls/symbols.rs ✅ 已实现（纯 AST）
        ├── ls/completions.rs ❌ 被 services.go 阻塞
        └── 其余 ls/*.rs ❌ 被 services.go + checker 阻塞
```

---

## 任务清单（按依赖顺序，从底层到上层）

每项标注前置依赖。按条逐个完成。

### ── 第 0 层：已完成的基础设施 ──

| # | 任务 | 状态 | 说明 |
|---|------|------|------|
| 0.1 | Scanner | ✅ 100% | 6,528 行，超过 Go |
| 0.2 | Parser | ✅ 95% | 11,846 行，超过 Go |
| 0.3 | Checker/flow | ✅ 98% | 2,676 行 |
| 0.4 | Checker/inference | ✅ 100% | 1,650 行 |
| 0.5 | Checker/mapper | ✅ 96% | 301 行 |
| 0.6 | Checker/types | ✅ 完成 | 1,852 行 |
| 0.7 | Module/resolver | ✅ 90% | 3,329 行 |
| 0.8 | Printer | ✅ 完成 | 3,162 行 |
| 0.9 | Emitter (JS/d.ts) | ✅ 完成 | 字节对齐 Go |
| 0.10 | Watch/Build | ✅ 完成 | notify + .tsbuildinfo |

### ── 第 1 层：Checker 核心（被所有上层依赖）──

#### 1.1 checker/checker.rs — 表达式/语句检查补全 ⬜

- **现状**：7,867 行 vs Go 31,926 行（25%）
- **缺失**：`checkExpressionWorker` 完整调度（Go 57 种表达式 → Rust 覆盖约 15 种）
- **依赖**：无（自底向上）
- **验收**：checker_parity 测试通过数提升

#### 1.2 checker/utilities.rs — checker 工具函数 ⬜

- **现状**：707 行 vs Go 1,844 行（38%）
- **缺失**：类型判断辅助、`getStringType`、`getNumberType` 等
- **依赖**：1.1

#### 1.3 checker/relater.rs — 子类型关系补全 ⬜

- **现状**：3,538 行 vs Go 5,006 行（70%）
- **缺失**：高级 `isTypeRelatedTo` 场景、泛型签名比较
- **依赖**：1.1

#### 1.4 checker/grammarchecks.rs — 语法检查 ⬜

- **现状**：1,327 行 vs Go 2,202 行（60%）
- **缺失**：部分语法诊断规则
- **依赖**：1.1

#### 1.5 checker/jsx.rs — JSX 类型检查 ⬜

- **现状**：340 行 vs Go 1,482 行（23%）
- **缺失**：JSX 元素类型解析、属性检查
- **依赖**：1.1

#### 1.6 checker/emitresolver.rs — Emit 解析器补全 ⬜

- **现状**：877 行 vs Go 1,322 行（66%）
- **缺失**：部分 emit 标记解析
- **依赖**：1.1

#### 1.7 checker/nodebuilder.rs — 类型节点构建器补全 ⬜

- **现状**：2,601 行 vs Go 4,735 行（55%）
- **缺失**：hover 专用 nodebuilder（Go nodebuilder_hover.go 597 行）、scope builder（251 行）
- **依赖**：1.1

### ── 第 2 层：Checker 公共 API（被 LSP 依赖）──

#### 2.1 checker/services.rs — LS 公共 API ⬜ 🔴 关键阻塞项

- **现状**：**完全缺失**（Go services.go = 1,140 行，66 个方法）
- **缺失**：
  - `getSymbolsInScope` — 作用域符号枚举（completions 核心）
  - `getExportsOfModule` — 模块导出
  - `getContextualType` — 上下文类型
  - `getContextualSignature` — 上下文签名
  - `getCallSignatures` / `getPropertiesOfType` / `getPropertyOfType`
  - `getApparentType` / `getTypeArguments`
- **依赖**：1.1（checker.rs）
- **影响**：阻塞 completions、signature help、code actions

#### 2.2 checker/exports.rs — 类型导出包装 ⬜

- **现状**：**完全缺失**（Go exports.go = 359 行）
- **缺失**：`getStringType`、`getNumberType`、`getBooleanType` 等公共类型 getter
- **依赖**：1.1

#### 2.3 checker/symbolaccessibility.rs — 符号可见性 ⬜

- **现状**：**完全缺失**（Go symbolaccessibility.go = 876 行）
- **缺失**：声明 emit 的符号可访问性检查
- **依赖**：1.1, 2.2

#### 2.4 checker/nodecopy.rs — AST 节点克隆 ⬜

- **现状**：**完全缺失**（Go nodecopy.go = 900 行）
- **缺失**：`cloneTypeNode`、`getSynthesizedClone`（用于 declaration emit）
- **依赖**：1.1

#### 2.5 checker/symboltracker.rs — 符号追踪 ⬜

- **现状**：**完全缺失**（Go symboltracker.go = 129 行）
- **依赖**：1.1

### ── 第 3 层：AST/Binder 补全 ──

#### 3.1 ast/utilities — AST 辅助函数 ⬜

- **现状**：8,336 行 vs Go 20,747 行（45%，但节点结构已完整）
- **缺失**：`utilities.go`（4,565 行）中的 `forEachChild` 辅助、`getNodeAtPosition`、`findChildOfKind` 等
- **依赖**：无
- **影响**：被 LSP 层大量调用

#### 3.2 binder/nameresolver — 名称解析器 ⬜

- **现状**：binder/mod.rs 3,048 行，但 nameresolver.go（498 行）+ referenceresolver.go（262 行）基本缺失
- **依赖**：3.1

#### 3.3 astnav — AST 导航 ⬜

- **现状**：310 行 vs Go 783 行（40%）
- **缺失**：`getTokenAtPosition`、`findChildOfKind`、`getStartOfNode`
- **依赖**：3.1

### ── 第 4 层：Module/Specifier 补全 ──

#### 4.1 modulespecifiers — 模块说明符 ⬜

- **现状**：403 行 vs Go 2,280 行（18%）
- **缺失**：路径计算、比较器、偏好设置
- **依赖**：module/ (已完成 90%)
- **影响**：auto-import、organize-imports

### ── 第 5 层：Compiler 补全 ──

#### 5.1 compiler — 程序管理补全 ⬜

- **现状**：1,433 行 vs Go ~5,658 行（25%）
- **缺失**：emitter 集成、fileInclude 解析、project-reference hosts
- **依赖**：1.1（checker）

### ── 第 6 层：LSP 功能实现（依赖 checker services）──

#### 6.1 ls/completions.rs — 代码补全 ⬜ 🔴

- **现状**：105 行骨架，返回空 `CompletionList`
- **依赖**：**2.1**（services.rs 的 `getSymbolsInScope` + `getContextualType`）
- **工作量**：Go completions.go 约 6,000 行

#### 6.2 ls/find_all_references.rs — 查找引用 ⬜

- **现状**：骨架
- **依赖**：2.1（`getSymbolAtLocation` + `getFindAllReferences`）

#### 6.3 ls/rename.rs — 重命名 ⬜

- **现状**：骨架
- **依赖**：6.2

#### 6.4 ls/signature_help.rs — 签名帮助 ⬜

- **现状**：骨架
- **依赖**：2.1（`getContextualSignature`）

#### 6.5 ls/code_actions.rs — 代码操作 ⬜

- **现状**：骨架
- **依赖**：2.1, 2.3, 2.4

#### 6.6 ls/semantic_tokens.rs — 语义高亮 ⬜

- **现状**：骨架
- **依赖**：2.1

#### 6.7 ls/inlay_hints.rs — 内联提示 ⬜

- **现状**：骨架
- **依赖**：2.1

#### 6.8 ls/diagnostics.rs — 拉取式诊断 ⬜

- **现状**：骨架
- **依赖**：1.1（checker 诊断完善后）

#### 6.9 ls/organize_imports.rs — 整理导入 ⬜

- **现状**：骨架
- **依赖**：4.1（modulespecifiers）

#### 6.10 ls/document_highlights.rs — 文档高亮 ⬜

- **现状**：骨架
- **依赖**：2.1

### ── 第 7 层：LSP 服务器集成 ──

#### 7.1 lsp/server.rs — LSP 请求调度补全 ⬜

- **现状**：骨架
- **依赖**：6.x 全部

#### 7.2 project/session.rs — 会话管理补全 ⬜

- **现状**：骨架
- **依赖**：5.1, 6.x

### ── 已完成的 LSP 功能 ──

| 功能 | 文件 | 状态 |
|------|------|------|
| Hover | ls/hover.rs | ✅ 接入 checker.get_quick_info_display_parts |
| Go-to-Definition | ls/definition.rs | ✅ 接入 checker.get_symbol_at_location |
| Folding Range | ls/folding.rs | ✅ AST 遍历 + region 注释 |
| Selection Range | ls/selection_ranges.rs | ✅ AST 父链遍历 |
| Document Symbols | ls/symbols.rs | ✅ AST 声明遍历 |

---

## 集成测试

真实项目对比文档：[INTEGRATION_TEST.md](INTEGRATION_TEST.md)（ai-Color-toner）

**集成测试里程碑（2026-08-03）**：
- 诊断：0 错误（Rust == Go）
- JS/d.ts 产物：**4/4 文件字节一致**
- Source map：结构正确，精度差距（text-slice vs AST printer 架构差异）
- 改善历程：127 → 0 诊断，JS/d.ts 从格式差异到字节一致

## P0-P9 状态

P0 基线 ✅ | P1 CLI/tsconfig ✅ | P2 Scanner/Parser ✅ | P3 Binder/Checker 进行中 |
P4 Emit ✅ | P5 Module Resolution ✅ | P6 Build/Watch ✅ | P7 LSP 骨架完成 | P8 npm/API ✅ | P9 工具链 ✅

## P10：Go 测试用例 1:1 迁移

Go **1,219 测试 / 508 文件 / 44 模块**。Rust **1,290 lib 通过 / 0 ignored**。

所有 Go 测试模块已迁移或适配。Rust 无法直接移植的测试（Go 并发竞态、V8 引擎、
Windows reparse point、AST 深拷贝等）已用 Rust 等价实现替代，详见
RUST_ADAPTATIONS.md。

未迁移的大规模模块（需独立工作量）：
- fourslash（519）：语言服务集成测试，需完整 fourslash 框架
- fswatch（116）：文件监听测试，需 watch mode 稳定后迁移
- lsp/project/api/ls（202）：LS 功能测试，需完整 project service
