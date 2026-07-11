# TypeScript Go to Rust Migration

This worktree contains the Rust migration of the Go native TypeScript compiler.
The Go implementation is kept as the behavior oracle until Rust parity is broad
enough to stand alone.

## Worktrees

- Go oracle worktree: `/home/cqh/workspace/typescript-rust`
- Rust migration worktree: `/home/cqh/workspace/typescript-rust-rust`

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
TSGO_ORACLE=/home/cqh/workspace/typescript-rust/built/local/tsgo cargo test --test parity
```

If `TSGO_ORACLE` is not set, the parity test searches for a runnable Go binary
in the adjacent Go worktree:

1. `/home/cqh/workspace/typescript-rust/built/local/tsgo`
2. `/home/cqh/workspace/typescript-rust/_packages/native-preview/bin/tsgo`

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
/home/cqh/workspace/typescript-rust/built/local/tsgo
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

## Build Mode Flow Audit

Recorded on 2026-07-11 while comparing Go `tsgo -b` with Rust `tsox -b`.

### Go `tsgo -b`

Source path:

- `cmd/tsgo/main.go`
- `internal/execute/tsc.go`
- `internal/tsoptions/commandlineparser.go`
- `internal/execute/build/orchestrator.go`
- `internal/execute/build/buildtask.go`

Flow:

1. `cmd/tsgo/main.go` calls `execute.CommandLine(ctx, sys, args, testing)`.
2. `execute.CommandLine` only enters build mode when the first argument is
   `-b`, `--b`, `-build`, or `--build`.
3. Build mode calls `tsoptions.ParseBuildCommandLine(commandLineArgs, sys)`.
   This is separate from ordinary `ParseCommandLine`.
4. `ParseBuildCommandLine` splits parsed options into build options, compiler
   options, watch options, project paths, and parse errors.
5. If no project path is supplied, Go treats `tsgo -b` as `tsgo -b .`.
6. `ParseBuildCommandLine` rejects invalid build option combinations such as
   `--clean --force`, `--clean --verbose`, `--clean --watch`, and
   `--watch --dry`.
7. `tscBuildCompilation` reports parse errors, handles profiling/help, creates
   `build.NewOrchestrator`, and calls `orchestrator.Start`.
8. The orchestrator resolves project paths/configs, recursively parses project
   references, generates the build graph, detects project-reference cycles, and
   then calls `buildOrClean`.
9. Each `BuildTask` waits for upstream tasks, computes up-to-date status using
   `.tsbuildinfo`, inputs, outputs, package.json lookup state, and upstream
   timestamps, then either skips, cleans, pseudo-builds, or compiles.
10. Real builds use `compiler.NewProgram`, wrap it in `incremental.NewProgram`,
    call `tsc.EmitAndReportStatistics`, write outputs through the build task,
    update `.tsbuildinfo`/timestamps, and propagate downstream invalidation.
11. Watch build mode keeps the graph and caches alive, reconciles watches, and
    runs incremental cycles from changed paths.

Important behavior points:

- `--build` is a mode selector, not an ordinary compiler option.
- `--build` must be the first command-line argument.
- Project arguments are project/config paths, not source files.
- Empty project list defaults to `"."`.
- Solution configs with `files: []` are valid build graph roots; they do not
  cause default libs to be parsed by themselves.

### Rust `tsox -b`

Source path:

- `src/main.rs`
- `src/execute/mod.rs`
- `src/tsoptions/mod.rs`
- `src/compiler/mod.rs`

Current flow after this audit:

1. `src/main.rs` calls `execute::command_line(&sys, &args)`.
2. `command_line` enters build mode only when the first argument is `-b`,
   `--b`, `-build`, or `--build`.
3. If a build-mode argument appears later, Rust now reports the Go-compatible
   "must be the first command line argument" error instead of silently treating
   it as ordinary compilation.
4. Rust now has a separate `parse_build_command_line` path with
   `ParsedBuildCommandLine` and `BuildOptions`, mirroring the first layer of
   Go `ParseBuildCommandLine`.
5. The build parser splits build options, compiler options, project paths, and
   parse errors. In build mode, `-v` maps to `verbose`, not `version`.
6. Empty project list defaults to `"."`, then the temporary Rust build bridge
   resolves project paths against the current directory.
7. The parser rejects the Go-invalid build option combinations:
   `--clean --force`, `--clean --verbose`, `--clean --watch`, and
   `--watch --dry`.
8. Bare build arguments are treated as project/config paths. A directory maps
   to `tsconfig.json`; an existing file is treated as a config file; a missing
   path reports an error.
9. Each resolved config is compiled by the ordinary `perform_compilation`
   pipeline.
10. `src/compiler/mod.rs` now matches Go's root-file gate for default libs:
   default libs are loaded only when the parsed config has at least one root
   file and `noLib` is not true.

Remaining known differences:

- Rust build command parsing is still narrower than Go's full
  `ParseBuildCommandLine`: it does not yet implement build-specific
  did-you-mean diagnostics or the full watch-options model.
- Rust does not yet parse or build project references as a graph.
- Rust does not yet implement `.tsbuildinfo` read/write, up-to-date checks,
  pseudo builds, clean/dry/force/verbose, or downstream invalidation.
- Rust build mode currently compiles each supplied project independently through
  the normal compile path.
- Rust watch build mode is not implemented.

Migration rule for `-b`: Rust must continue moving toward the Go build
orchestrator behavior. Avoid adding new behavior that treats `-b` as a source
file compilation shortcut.

## Migration Rule

Every migrated behavior should add or extend a parity case. Prefer comparing
exit code, stdout, stderr, and emitted file contents against the Go oracle before
considering a task done.
