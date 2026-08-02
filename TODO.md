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

**2,038 通过**（1,126 lib + 910 checker_parity + 2 emit），99 ignored。

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

## 待办清单

已完成：A(Checker 深度 13 诊断码 + 方法解析 + generic inference + 910 parity)、
B(Watch mode + cycle)、C(LSP references/rename/documentSymbol/project service)、
D(symbol_to_display_parts)。

剩余：
- [ ] fourslash 测试 smoke
- [ ] 本地化支持（locale/loc_generated）
- [ ] 正则 `lastIndex`/`d` flag runtime 特性
- [ ] `.ts/.tsx/.js/.jsx` 解析结果与 oracle 完全对齐

## P0-P9 状态

P0 基线 ✅ | P1 CLI/tsconfig ✅ | P2 Scanner/Parser ✅ | P3 Binder/Checker 进行中(~30%) |
P4 Emit ✅ | P5 Module Resolution ✅ | P6 Build/Watch ✅ | P7 LSP ✅ | P8 npm/API ✅ | P9 工具链 ✅

## P10：Go 测试用例 1:1 迁移

Go **1,219 测试 / 508 文件 / 44 模块**。Rust **1,126 lib 通过 / 99 ignored**。

全 ✅ 模块：tspath(24), semver(11), core(2), stringutil(3), scanner(1), astnav(5),
sourcemap(30), compiler(2), symlinks(8), packagejson(4), bundled(2), debug(12),
format(7), tracing(2), collections(7/8), ast(6/7), diagnostics(2), vfs/cachedvfs(10),
vfs/osvfs(3).

剩余 ⏳ 项：

| 模块 | ⏳ 数 | 原因 |
|------|-------|------|
| printer | ~70 | 需完整 AST→文本 printer |
| jsnum | 1 | TestStringJS 需 Node.js |
| collections | 1 | allocsPerRun |
| ast | 1 | DeepCloneNode |
| checker | 1 | TracerPushPreservesEndArgMutations |
| transformers | 1 | ImportElision |
| module | 2 | 并发竞态测试 |
| modulespecifiers | 3 | 模块说明符解析 |
| nativepath | 4 | 平台特定 symlink |
| vfs/vfstest | 4 | symlink |
| vfs/vfsmatch | 1 | symlink cycle |
| vfs/iovfs | 1 | IO VFS 包装器 |
| vfs/vfsmock | 1 | VFS mock |
| execute/tsctests | 52 | shell-out 集成测试 |
| lsp/project/api/ls | 202 | LS 功能测试 |
| fourslash | 519 | 语言服务集成 |
| fswatch | 116 | 文件监听测试 |
