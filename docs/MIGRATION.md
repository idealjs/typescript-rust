# TypeScript Go to Rust Migration

This worktree contains the Rust migration of the Go native TypeScript compiler.
The Go implementation is kept as the behavior oracle until Rust parity is broad
enough to stand alone.

> **规划文档**：迁移目标、阶段任务、进度快照、下阶段优先级统一记录在
> [`TODO.md`](../roadmap/TODO.md)。本文档只保留流程审计与行为差异细节，按阶段查阅。
>
> 2026-08-03 状态摘要：`cargo test` 通过（1,282 lib + 2 emit parity + 920
> checker parity）。集成测试里程碑：ai-Color-toner 项目 JS/d.ts 产物与 Go
> oracle **字节一致**，诊断 **0 错误**。Parser ~95%、Binder ~60%、Checker ~30%。
> Emitter 基础完成 + JS/d.ts 字节对齐。详见 TODO.md 与 INTEGRATION_TEST.md。

## Worktrees

- Rust migration worktree (primary): `/Users/cqh/workspace/typescript-rust`, branch `rust`
- Go oracle worktree: `/Users/cqh/workspace/typescript-go`, branch `main`

## Rust Commands

Run all Rust tests:

```sh
cargo test
```

Run the parity integration test:

```sh
cargo test --test parity
```

Run parity against an explicit Go oracle binary:

```sh
TSGO_ORACLE=/Users/cqh/workspace/typescript-go/built/local/tsgo cargo test --test parity
```

If `TSGO_ORACLE` is not set, the parity test searches for a runnable Go binary
in the adjacent Go worktree:

1. `/Users/cqh/workspace/typescript-go/built/local/tsgo`
2. `/Users/cqh/workspace/typescript-rust/_packages/native-preview/bin/tsgo`

If no oracle is found, the oracle comparison test is skipped and prints the
reason to stderr.

## Building the Go Oracle

From the Go worktree:

```sh
npm install
npm run build
```

The expected local binary is:

```sh
/Users/cqh/workspace/typescript-go/built/local/tsgo
```

## Current Status

- `cargo test` passes.
- The Rust binary is named `tsox`.
- The crate uses `edition = "2024"` with `rust-version = "1.96"`. Cargo 1.96
  does not support an `edition = "2026"` manifest value.
- `--lsp` and `--api` are currently stubs.
- CLI/emit parity is intentionally narrow and should be expanded fixture by
  fixture.
- `cargo fmt --check` is not clean for the whole worktree yet. Format touched
  files during migration, and schedule a separate whole-worktree formatting pass.
- Remaining `cargo test` warnings are currently migration-period warnings:
  Go/TypeScript-compatible names, not-yet-wired placeholder APIs, and one
  checker re-export collision.
- Scanner non-ASCII unknown characters such as `·` no longer panic. The scanner
  reports byte spans correctly, but command-line invalid-character diagnostics
  still need to be wired through the parser/compiler diagnostics path.


## Flow Audits
三个编译器流程审计已拆分：
- [`docs/audit-build-mode.md`](./audit-build-mode.md) — Build Mode Flow Audit
- [`docs/audit-command-line-flow.md`](./audit-command-line-flow.md) — Command Line Argument Flow Audit
- [`docs/audit-tsconfig-flow.md`](./audit-tsconfig-flow.md) — TSConfig Flow Audit
## Migration Rule

Every migrated behavior should add or extend a parity case. Prefer comparing
exit code, stdout, stderr, and emitted file contents against the Go oracle before
considering a task done.
