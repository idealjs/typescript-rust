# tsox — TypeScript compiler (Go → Rust port)

`tsox` is the Rust port of the Go native TypeScript compiler
([typescript-go](https://github.com/microsoft/typescript-go)). The Go
implementation is kept as the behavior oracle until Rust parity is broad
enough to stand alone.

## Quick start

Requirements: Rust toolchain **1.96+** (the crate uses `edition = "2024"`).

```sh
# build
cargo build

# run the compiler
cargo run -- --help
cargo run -- --version

# run all library tests (gating)
cargo test --lib

# run the parity integration test (gating)
# without a Go oracle this still exercises the Rust smoke cases and
# skips the oracle comparison gracefully
cargo test --test parity
```

## Parity tests against the Go oracle

To compare Rust output against the Go compiler, build the oracle in the
adjacent Go worktree and point `TSGO_ORACLE` at it:

```sh
# in /Users/cqh/workspace/typescript-go
npm install && npm run build
# produces built/local/tsgo

# back in the Rust worktree
TSGO_ORACLE=/Users/cqh/workspace/typescript-go/built/local/tsgo cargo test --test parity
```

If `TSGO_ORACLE` is unset, the test auto-discovers the oracle at
`../typescript-go/built/local/tsgo` or `_packages/native-preview/bin/tsgo`,
and prints a clear skip reason when none is found.

## Documentation

- [`TODO.md`](./TODO.md) — migration goals, staged tasks, progress snapshot,
  and next priorities. **Start here.**
- [`MIGRATION.md`](./MIGRATION.md) — flow audits and behavior-diff details by
  phase, plus run/build instructions.
- [`ANALYSIS.md`](./ANALYSIS.md) — Go vs Rust structure comparison, module
  mapping, and completed-work inventory.

## Worktree layout

- Rust migration (primary, `rust` branch): `/Users/cqh/workspace/typescript-rust`
- Go oracle (`main` branch): `/Users/cqh/workspace/typescript-go`

## CI

`.github/workflows/rust.yml` runs `cargo fmt --check`, `cargo clippy`,
`cargo test --lib`, and `cargo test --test parity` on push/PR to `rust`/`main`.
During migration fmt and clippy are informational; test and parity smoke gate.
