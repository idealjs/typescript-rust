# tsox — TypeScript compiler (Go → Rust port)

`tsox` is the Rust port of the Go native TypeScript compiler
([typescript-go](https://github.com/microsoft/typescript-go), TS 7.1.0-dev
snapshot). The Go implementation stays as the behavior oracle.

**Status**: the full upstream regression corpus — 12,466 cases
(compiler 6,537 + conformance 5,907 + transpile 22) — passes with **0 FAIL**
across double full sweeps; known accepted differences are ledgered in
[`tests/baselines/reference/triaged.txt`](./tests/baselines/reference/triaged.txt).

## Quick start

Requirements: Rust toolchain **1.96+** (the crate uses `edition = "2024"`).

```sh
cargo build
cargo run -- --help
```

## Test gates

Five test targets make up the gates (details and history in
[`TESTING.md`](./TESTING.md)):

```sh
cargo test --lib                     # 1353 library unit tests
cargo test --test checker_parity     # 1010 checker parity tests
cargo test --test lsp_integration    # 15 LSP integration tests
cargo test --test parity             # 2 emit-parity tests (Go oracle)
cargo test --test submodule_compiler # official TypeScript cases (see below)
```

### Official-case regression harness

Fetch the corpus once, then slice however you need:

```sh
git submodule update --init   # official test corpus

TSOX_SUBMODULE_LIMIT=0 cargo test --test submodule_compiler          # all
TSOX_SUBMODULE_START=1000 TSOX_SUBMODULE_END=2000 \                  # window
  cargo test --test submodule_compiler
TSOX_SUBMODULE_FILTER=classExpression \                              # substring
  cargo test --test submodule_compiler
TSOX_SUBMODULE_JOBS=6 cargo test --test submodule_compiler           # pin workers
TSOX_SUBMODULE_SUITE=conformance …                                   # other suite
```

Every case runs as a one-shot worker subprocess (checker stack overflows kill
only the worker); progress logs stream to stderr and
`tests/baselines/local/`. On mismatch the actual output lands next to the
reference for diffing; accept with `TSOX_BASELINE_ACCEPT=1`. Known gaps are
triaged into `tests/baselines/reference/triaged.txt` (dated root-cause
groups — the sweep treats those as accepted-diff, never silent skips).

A full-corpus sweep at 6 workers takes ~2.5h; historical sweep scripts live in
[`scripts/sweeps/`](./scripts/sweeps/).

## Performance

Benchmark suite against the Go compiler:
[idealjs/ts-go-rust-bench](https://github.com/idealjs/ts-go-rust-bench) —
9 categories / 34 cases designed from a 12,444-case paired timing study.
Headline (2026-09-04): single-file compile median **31× slower than Go**
(fixed pipeline cost, dominated by default-lib parsing); the checker kernel
itself is at parity or faster. Root-cause analysis and prioritized fixes are
in the bench repo's `results/`.

## Documentation

| doc | contents |
|---|---|
| [`TESTING.md`](./TESTING.md) | test methods, sweep history, current baseline — **start here for testing** |
| [`TODO.md`](./TODO.md) | current TODO list |
| [`docs/MIGRATION.md`](./docs/MIGRATION.md) | flow audits, behavior-diff by phase, build notes |
| [`docs/ANALYSIS.md`](./docs/ANALYSIS.md) | Go vs Rust structure comparison, module mapping |
| [`docs/RUST_ADAPTATIONS.md`](./docs/RUST_ADAPTATIONS.md) | where the Rust port deliberately diverges from Go |
| [`docs/FIXING.md`](./docs/FIXING.md) | fixing workflow conventions |
| [`docs/INTEGRATION_TEST.md`](./docs/INTEGRATION_TEST.md) | real-project integration results |
| [`docs/LSP_MIGRATION_PLAN.md`](./docs/LSP_MIGRATION_PLAN.md) | LSP porting plan |
| [`docs/PACKAGING.md`](./docs/PACKAGING.md) | packaging/distribution notes |
| [`docs/ISSUES_RISK_ANALYSIS.md`](./docs/ISSUES_RISK_ANALYSIS.md) | known risks |
| [`docs/triage-CLASSIFICATION.md`](./docs/triage-CLASSIFICATION.md) | triage taxonomy |

## Parity tests against the Go oracle

Build the oracle in the adjacent Go worktree and point `TSGO_ORACLE` at it
(auto-discovery: `../typescript-go/built/local/tsgo`):

```sh
TSGO_ORACLE=../typescript-go/built/local/tsgo cargo test --test parity
```

## Worktree layout

- Rust port (this repo, `rust-2` branch)
- Go oracle: `../typescript-go` (same-clone worktree of typescript-go)

## CI

`.github/workflows/rust.yml` runs `cargo fmt --check`, `cargo clippy`,
`cargo test --lib`, and `cargo test --test parity` on push/PR.
