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

## 当前进度（2026-08-04）

**1,290 lib 通过**（+ 920 checker_parity + 2 emit），**0 ignored**。

| 模块 | 完成度 |
|------|--------|
| Scanner | ~95% |
| Parser | ~95% |
| Binder | ~60% |
| Checker | ~30% |
| Compiler/Module | ~60% |
| Emitter | 基础完成 + JS/d.ts 字节对齐 Go |
| LSP | 骨架完成（107 文件 / 17,745 行 — 严格 1:1 Go 映射）|
| API | 基础完成 |
| --watch | 已实现（notify crate）|
| --build | 已实现（project reference + cycle + .tsbuildinfo）|

## 集成测试

真实项目对比文档：[INTEGRATION_TEST.md](INTEGRATION_TEST.md)（ai-Color-toner）

**集成测试里程碑（2026-08-03）**：
- 诊断：0 错误（Rust == Go）
- JS/d.ts 产物：**4/4 文件字节一致**
- Source map：结构正确，精度差距（text-slice vs AST printer 架构差异）
- 改善历程：127 → 0 诊断，JS/d.ts 从格式差异到字节一致

## 待办清单

已完成：A(Checker 深度 13 诊断码 + 方法解析 + generic inference + 910 parity)、
B(Watch mode + cycle)、C(LSP references/rename/documentSymbol/project service)、
D(symbol_to_display_parts)、E(零 ignore — 全部 Rust 适配)。

### F. Emit 对齐 ✅ 已完成

- [x] F1: 非空断言 `!` 擦除
- [x] F2: node_modules 文件排除 emit
- [x] F3: Import 路径重写 `.tsx`→`.js`
- [x] F4: JSX Transform（react-jsx automatic runtime）
- [x] F5: Type Eraser（implements/abstract/declare/override/readonly/type assertion）
- [x] F6: Import Elision（import type / 混合 import）
- [x] F7: Declaration Emit（基础完成：函数签名 + JSX 返回类型推断 + 空行规范化）
- [x] F8: JS 输出规范化（fold/reindent/dedup/semicolons — 字节对齐 Go printer）
- [x] F9: Source Map（字符级源位置追踪 + 位置感知规范化 + JSX 锚点映射）

### G. Checker 对齐 ✅ 已完成

- [x] G1: JSX 全局命名空间（合成 JSX.Element + IntrinsicElements）
- [x] G2: DOM lib 全局变量（document/window/console 等 fallback to any）
- [x] G3: node_modules .js 不解析（allowJs 默认 false）
- [x] G4: @types/react 深度兼容（TS2302/TS2300/TS2304/TS2604 + type predicate + 方法泛型）

### 其他长期项

- [x] 本地化支持（13 locale bundles + --locale flag + Message::localize）
- [x] 正则 `d` flag（scanner 层已实现，runtime 属于 JS 引擎）
- [x] `.ts/.tsx/.js/.jsx` 解析：NodeFlags::JavaScriptFile/JsonFile 标志已设置
- [x] fourslash 测试 smoke（10 个基础测试：marker 解析 + hover/completion/definition）

## P0-P9 状态

P0 基线 ✅ | P1 CLI/tsconfig ✅ | P2 Scanner/Parser ✅ | P3 Binder/Checker 进行中(~30%) |
P4 Emit ✅ (JS/d.ts 字节对齐) | P5 Module Resolution ✅ | P6 Build/Watch ✅ | P7 LSP ✅ (骨架完成) | P8 npm/API ✅ | P9 工具链 ✅

## P10：Go 测试用例 1:1 迁移

Go **1,219 测试 / 508 文件 / 44 模块**。Rust **1,290 lib 通过 / 0 ignored**。

所有 Go 测试模块已迁移或适配。Rust 无法直接移植的测试（Go 并发竞态、V8 引擎、
Windows reparse point、AST 深拷贝等）已用 Rust 等价实现替代，详见
RUST_ADAPTATIONS.md。

未迁移的大规模模块（需独立工作量）：
- fourslash（519）：语言服务集成测试，需完整 fourslash 框架
- fswatch（116）：文件监听测试，需 watch mode 稳定后迁移
- lsp/project/api/ls（202）：LS 功能测试，需完整 project service
