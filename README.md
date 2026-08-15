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
```

## Running the tests manually

Five test targets make up the suite (see [`TESTING.md`](./TESTING.md) for
full details). Run each with `cargo test --test <name>`:

```sh
cargo test --lib                    # 1301 library unit tests (~2s)
cargo test --test checker_parity    # 921 checker parity tests (~50s)
cargo test --test lsp_integration   # 15 LSP integration tests (<1s)
cargo test --test parity            # 2 emit-parity tests, incl. Go-oracle
                                     # comparison when the oracle is available
cargo test --test submodule_compiler  # official TypeScript test cases

# everything above in one command
cargo test
```

The `submodule_compiler` target replays TypeScript's official compiler test
cases against committed snapshots in `tests/baselines/reference/`:

```sh
# one-time: fetch the official test corpus (~6500 cases)
git submodule update --init

# run the default slice (first 1000 cases, ~15 min)
cargo test --test submodule_compiler

# run ALL cases (~6500, ~100 min)
TSOX_SUBMODULE_LIMIT=0 cargo test --test submodule_compiler

# run only the first N cases
TSOX_SUBMODULE_LIMIT=200 cargo test --test submodule_compiler
```

On mismatch, the actual output is written under `tests/baselines/local/` for
inspection. To accept new output (after verifying it matches the official
baselines), re-run with `TSOX_BASELINE_ACCEPT=1`; known gaps go into
`tests/baselines/reference/triaged.txt`.

**CPU usage**: the test suite (especially `submodule_compiler`, which spawns
one subprocess per case) can saturate all cores. To keep the machine
responsive, pin the run to a few cores, e.g. on Linux:

```sh
taskset -c 0-3 cargo test --test submodule_compiler   # ≤400% CPU
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
- [`TESTING.md`](./TESTING.md) — all test methods, execution commands, and the
  current test baseline. **For running tests, start here.**
- [`INTEGRATION_TEST.md`](./INTEGRATION_TEST.md) — real-project integration
  test results (ai-Color-toner): JS/d.ts byte-identical to Go, 0 diagnostics.
- [`MIGRATION.md`](./MIGRATION.md) — flow audits and behavior-diff details by
  phase, plus run/build instructions.
- [`ANALYSIS.md`](./ANALYSIS.md) — Go vs Rust structure comparison, module
  mapping, and completed-work inventory.

## Integration test status

The `ai-Color-toner` project (React + Vite + TypeScript) serves as the
integration benchmark:

| Metric | Status |
|--------|--------|
| Diagnostics | 0 errors (matches Go) |
| `App.js` / `main.js` | Byte-identical to Go |
| `App.d.ts` / `main.d.ts` | Byte-identical to Go |
| Source maps | Structurally correct (lower granularity than Go) |
| Library tests | 1,282 passed, 0 failed |

## Worktree layout

- Rust migration (primary, `rust` branch): `/Users/cqh/workspace/typescript-rust`
- Go oracle (`main` branch): `/Users/cqh/workspace/typescript-go`

## CI

`.github/workflows/rust.yml` runs `cargo fmt --check`, `cargo clippy`,
`cargo test --lib`, and `cargo test --test parity` on push/PR to `rust`/`main`.
During migration fmt and clippy are informational; test and parity smoke gate.
