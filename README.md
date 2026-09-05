# tsox — TypeScript compiler (Go → Rust port)

`tsox` is the Rust port of the Go native TypeScript compiler
([typescript-go](https://github.com/microsoft/typescript-go), TS 7.1.0-dev
snapshot). The Go implementation stays as the behavior oracle.

**Status**: the full upstream regression corpus — 12,466 cases
(compiler 6,537 + conformance 5,907 + transpile 22) — passes with **0 FAIL**
across double full sweeps; known accepted differences are ledgered in
[`tests/baselines/reference/triaged-go.txt`](./tests/baselines/reference/triaged-go.txt).

## Quick start

Requirements: Rust toolchain **1.96+** (the crate uses `edition = "2024"`).

```sh
git submodule update --init   # fetch the official TypeScript test corpus
cargo build
```

## Running a batch of tests（一次跑一批）

The main regression target is `submodule_compiler`. It replays the official
TypeScript test cases against committed baselines. **A "page" is 100 cases**;
select any window with `TSOX_SUBMODULE_START` / `TSOX_SUBMODULE_END`
(1-based, inclusive, either bound optional).

```sh
# 例：一次跑 10 页（1000 个用例，compiler 套件，6 并发）
TSOX_SUBMODULE_START=1 TSOX_SUBMODULE_END=1000 TSOX_SUBMODULE_JOBS=6 \
  cargo test --test submodule_compiler

# 例：第 30–34 页（用例 2901–3400）
TSOX_SUBMODULE_START=2901 TSOX_SUBMODULE_END=3400 TSOX_SUBMODULE_JOBS=6 \
  cargo test --test submodule_compiler

# 换套件（compiler 是默认；conformance / transpile 需显式指定）
TSOX_SUBMODULE_SUITE=conformance TSOX_SUBMODULE_START=1 TSOX_SUBMODULE_END=1000 \
  TSOX_SUBMODULE_JOBS=6 cargo test --test submodule_compiler
```

Progress logs stream to stderr and `tests/baselines/local/`; each case ends
with a `PASS` / `DIFF` / `SKIP` / `FAIL` line. Runtime reference: 1000 cases
at 6 workers ≈ 15 min (compiler), full 12,444-case sweep ≈ 2.5 h.

## Running a single test case（一次跑一个用例做对比）

知道某个用例刚修好（或想复核某个差异）时，只运行它：

```sh
# 例：单独重跑 arrayFind.ts（过滤词不区分大小写、是子串匹配）
TSOX_SUBMODULE_FILTER=arrayFind TSOX_SUBMODULE_JOBS=1 \
  cargo test --test submodule_compiler -- --nocapture
```

- `TSOX_SUBMODULE_FILTER=<子串>` 按文件名子串过滤（case-insensitive），
  可与 `TSOX_SUBMODULE_START/END` 窗口叠加
- `-- --nocapture` 把每例 `PASS/DIFF/SKIP` 明细直接打到终端
- `TSOX_SUBMODULE_JOBS=1` 串行跑，规避多配置矩阵在并发下的 30s 超时假象
  （历轮 sweep 的超时 SKIP 均用此法单跑核实）

**对比实际输出与基线**：不一致时，实际输出写在
`tests/baselines/local/<suite>/<用例名>.errors.txt`，与参考基线
`tests/baselines/reference-go/<suite>/<用例名>.errors.txt` 直接 diff：

```sh
diff tests/baselines/reference-go/compiler/arrayFind.errors.txt \
     tests/baselines/local/compiler/arrayFind.errors.txt
```

确认新输出正确后，用 `TSOX_BASELINE_ACCEPT=1` 重跑同一命令写入新基线；
已知差异登记在 `tests/baselines/reference/triaged-go.txt`（go 口径）/
`triaged.txt`（upstream 口径），sweep 将其记为 accepted-diff 而非 FAIL。

## Baseline 口径

默认 oracle 是 **tsgo 自有基线**（`tests/baselines/reference-go/`，与
typescript-go 行为对齐）。旧的 upstream JS-tsc 基线保留为交叉口径：

```sh
TSOX_BASELINE_FLAVOR=upstream cargo test --test submodule_compiler
```

## Test gates

Five test targets make up the gates (details and history in
[`TESTING.md`](./TESTING.md)):

```sh
cargo test --lib                     # 1362 library unit tests
cargo test --test checker_parity     # 1010 checker parity tests
cargo test --test lsp_integration    # 15 LSP integration tests
cargo test --test parity             # 2 emit-parity tests (Go oracle)
cargo test --test submodule_compiler # official TypeScript cases (see above)
```

## Performance

Benchmark suite against the Go compiler:
[idealjs/ts-go-rust-bench](https://github.com/idealjs/ts-go-rust-bench) —
9 categories / 34 cases designed from a 12,444-case paired timing study.
Headline (2026-09-04): single-file compile median **31× slower than Go**
(fixed pipeline cost, dominated by default-lib parsing); the checker kernel
itself is at parity or faster. Root-cause analysis and prioritized fixes are
in the bench repo's `results/`.

## Parity tests against the Go oracle

Build the oracle in the adjacent Go worktree and point `TSGO_ORACLE` at it
(auto-discovery: `../typescript-go/built/local/tsgo`):

```sh
TSGO_ORACLE=../typescript-go/built/local/tsgo cargo test --test parity
```

## Documentation

| doc | contents |
|---|---|
| [`TESTING.md`](./TESTING.md) | test methods, gates, current status — **start here for testing** |
| [`roadmap/TODO.md`](./roadmap/TODO.md) | current TODO list |
| [`docs/MIGRATION.md`](./docs/MIGRATION.md) | migration overview, commands, flow audits (see audit-*.md) |
| [`docs/ANALYSIS.md`](./docs/ANALYSIS.md) | Go vs Rust structure comparison |
| [`docs/RUST_ADAPTATIONS.md`](./docs/RUST_ADAPTATIONS.md) | deliberate divergences from Go |
| [`docs/completed-inventory.md`](./docs/completed-inventory.md) | completed-work inventory & gaps |
| [`docs/structure-debt.md`](./docs/structure-debt.md) | AGENTS.md rule 2/3 (file size / test layout) debt & plan |
| [`docs/FIXING.md`](./docs/FIXING.md) | fixing workflow conventions |
| [`docs/INTEGRATION_TEST.md`](./docs/INTEGRATION_TEST.md) | real-project integration results |
| [`docs/PACKAGING.md`](./docs/PACKAGING.md) | packaging/distribution (see packaging-build-matrix.md) |
| [`docs/test-history.md`](./docs/test-history.md) | append-only sweep/run history archive |
| [`issues/known-issues.md`](./issues/known-issues.md) | known issues index (correctness / perf / risks) |
| [`roadmap/`](./roadmap/) | future plans (TODO, LSP migration) |

## Worktree layout

- Rust port (this repo, `rust-2` branch)
- Go oracle: `../typescript-go` (same-clone worktree of typescript-go)

## CI

`.github/workflows/rust.yml` runs `cargo fmt --check`, `cargo clippy`,
`cargo test --lib`, and `cargo test --test parity` on push/PR.
