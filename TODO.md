# typescript-go -> typescript-rust 迁移目标与任务

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

## 迁移目标

1. CLI/解析/绑定/检查/emit 全栈对齐 Go oracle
2. 真实项目可对齐（`tsox -p tsconfig.json` 诊断集合一致）
3. LSP/API/npm 包独立可用

## 当前进度（2026-08-02）

**2,137 通过**（1,225 lib + 910 checker_parity + 2 emit），**0 ignored**。

| 模块 | 完成度 |
|------|--------|
| Scanner | ~95% |
| Parser | ~95% |
| Binder | ~60% |
| Checker | ~30% |
| Compiler/Module | ~60% |
| Emitter | 基础完成 |
| LSP | 基础完成（hover/completion/definition/references/rename/documentSymbol/diagnostics）|
| API | 基础完成 |
| --watch | 已实现（notify crate）|
| --build | 已实现（project reference + cycle + .tsbuildinfo）|

## 集成测试

真实项目对比文档：[INTEGRATION_TEST.md](INTEGRATION_TEST.md)（ai-Color-toner）
Go→Rust 差异适配：[RUST_ADAPTATIONS.md](RUST_ADAPTATIONS.md)

当前差距：Rust 产出 104 个 false positive（JSX 全局命名空间 101 + DOM 全局 3），
Go oracle 仅 1 个有效错误。修复方向：JSX 全局类型加载 + DOM lib 全局变量。

## 待办清单

已完成：A(Checker 深度 13 诊断码 + 方法解析 + generic inference + 910 parity)、
B(Watch mode + cycle)、C(LSP references/rename/documentSymbol/project service)、
D(symbol_to_display_parts)、E(零 ignore — 全部 Rust 适配)。

### F. Emit 对齐（高优先级）— 基于 INTEGRATION_TEST.md 差异分析

Go 源码分析结论：Go 使用完整 transformer pipeline（AST 变换 + printer），
Rust 使用 text-slice 模式（直接从源码切片）。需要对齐的关键差异：

#### F1. 非空断言 `!` 擦除（简单，text-slice 可做）
- [ ] 在 emitter 的 cut-range 逻辑中加入 `NonNullExpression` 的 `!` 范围
- Go: `tstransforms/typeeraser.go:263` — 替换为 PartiallyEmittedExpression
- Rust: `src/emitter/mod.rs` cut ranges

#### F2. node_modules 文件排除 emit（简单）
- [ ] 实现 `sourceFileMayBeEmitted` 逻辑，排除 external library 文件
- Go: `emitter.go:450` + `program.go:1917` — `IsSourceFileFromExternalLibrary`
- Rust: `src/compiler/mod.rs` — Program 需跟踪 sourceFilesFoundSearchingNodeModules

#### F3. Import 路径重写 `.tsx` → `.js`（简单，text-slice 可做）
- [ ] 在 emit 时将相对路径 import 的 `.ts`/`.tsx` 扩展名替换为输出扩展名
- Go: `moduletransforms/utilities.go:22` — `rewriteModuleSpecifier`
- Go: `outputpaths/outputpaths.go:109` — `GetOutputExtension`
- Rust: `src/emitter/mod.rs` — emit 时处理 import 声明中的字符串字面量

#### F4. JSX Transform（中等复杂度，核心功能）
- [ ] 实现 `react-jsx` automatic runtime transform
- [ ] JSX element → `_jsx(tagName, propsObject)` 调用
- [ ] JSX fragment → `_Fragment` + `_jsxs` 调用
- [ ] 属性 → 对象属性（`{ id: "x", children: [...] }`）
- [ ] 自动注入 `import { jsx as _jsx } from "react/jsx-runtime"`
- Go: `transformers/jsxtransforms/jsx.go`（~1200 行）
- Go: `ast/utilities.go:2752` — `GetJSXImplicitImportBase`/`GetJSXRuntimeImport`
- Rust: 需新建 `src/transformers/jsx.rs`

#### F5. Type Eraser（类型注解擦除）
- [ ] 擦除函数参数/返回值的类型注解（`: type` 部分）
- [ ] 擦除类型别名、interface 声明
- [ ] 擦除 `as`/`satisfies` 表达式
- Go: `tstransforms/typeeraser.go`
- Rust: text-slice 模式可部分覆盖，完整实现需 AST visitor

#### F6. Import Elision（类型导入擦除）
- [ ] `import type { ... }` 整体移除
- [ ] 混合 import 中仅类型使用的绑定移除
- Go: `tstransforms/importelision.go`
- Rust: text-slice 模式可做，需 checker 的 emitResolver 判断绑定是否类型使用

#### F7. Declaration Emit（大工作量，可延后）
- [ ] 从 checker 符号生成 `.d.ts` 类型声明
- [ ] 过滤值 import（只保留类型 import）
- [ ] 生成返回类型（如 `import("react").JSX.Element`）
- Go: `transformers/declarations/transform.go`（~3000 行）
- 依赖 checker 的 node-builder API（`CreateTypeOfDeclaration` 等）

### G. Checker 对齐（高优先级）

#### G1. JSX 全局命名空间加载
- [ ] `jsx: "react-jsx"` 时加载 @types/react 的 `global.d.ts` JSX 声明
- [ ] 或实现内置 JSX 辅助类型注入

#### G2. DOM lib 全局变量
- [ ] 确保 `lib.dom.d.ts` 的 `declare var document` 在全局作用域可用

#### G3. node_modules .js 文件不解析
- [ ] `allowJs: false`（默认）时不解析 node_modules 中的 .js 文件

### 其他长期项

- [ ] fourslash 测试 smoke
- [ ] 本地化支持（locale/loc_generated）
- [ ] 正则 `lastIndex`/`d` flag runtime 特性
- [ ] `.ts/.tsx/.js/.jsx` 解析结果与 oracle 完全对齐

## P0-P9 状态

P0 基线 ✅ | P1 CLI/tsconfig ✅ | P2 Scanner/Parser ✅ | P3 Binder/Checker 进行中(~30%) |
P4 Emit ✅ | P5 Module Resolution ✅ | P6 Build/Watch ✅ | P7 LSP ✅ | P8 npm/API ✅ | P9 工具链 ✅

## P10：Go 测试用例 1:1 迁移

Go **1,219 测试 / 508 文件 / 44 模块**。Rust **1,225 lib 通过 / 0 ignored**。

所有 Go 测试模块已迁移或适配。Rust 无法直接移植的测试（Go 并发竞态、V8 引擎、
Windows reparse point、AST 深拷贝等）已用 Rust 等价实现替代，详见
RUST_ADAPTATIONS.md。

未迁移的大规模模块（需独立工作量）：
- fourslash（519）：语言服务集成测试，需完整 fourslash 框架
- fswatch（116）：文件监听测试，需 watch mode 稳定后迁移
- lsp/project/api/ls（202）：LS 功能测试，需完整 project service
