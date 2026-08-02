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

剩余：
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
