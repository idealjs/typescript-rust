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
9. Each resolved config is parsed, then raw `references` are followed with a
   temporary depth-first bridge before the current config is compiled. Configs
   with no root files, such as solution configs with `files: []`, are skipped
   after their references are visited.
10. Referenced projects still compile through the ordinary
   `perform_compilation` pipeline.
11. `src/compiler/mod.rs` now matches Go's root-file gate for default libs:
   default libs are loaded only when the parsed config has at least one root
   file and `noLib` is not true.

Remaining known differences:

- Rust build command parsing is still narrower than Go's full
  `ParseBuildCommandLine`: it does not yet implement build-specific
  did-you-mean diagnostics or the full watch-options model.
- Rust follows raw project references with a DFS bridge, but does not yet build
  Go's typed project-reference graph.
- Rust does not yet implement `.tsbuildinfo` read/write, up-to-date checks,
  pseudo builds, clean/dry/force/verbose, or downstream invalidation.
- Rust build mode currently compiles each reached project through the normal
  compile path.
- Rust watch build mode is not implemented.

Migration rule for `-b`: Rust must continue moving toward the Go build
orchestrator behavior. Avoid adding new behavior that treats `-b` as a source
file compilation shortcut.

## Command Line Argument Flow Audit

Recorded on 2026-07-11 while comparing all-argument handling in Go `tsgo`
with Rust `tsox`.

### Go `tsgo`

Source path:

- `cmd/tsgo/main.go`
- `internal/execute/tsc.go`
- `internal/tsoptions/commandlineparser.go`
- `internal/tsoptions/declscompiler.go`
- `internal/tsoptions/declsbuild.go`
- `internal/tsoptions/declsWatch.go`
- `internal/execute/tsc/init.go`
- `internal/execute/tsc/help.go`

Top-level dispatch:

1. `cmd/tsgo/main.go` passes CLI args to `execute.CommandLine`.
2. `execute.CommandLine` checks only the first argument for build mode:
   `-b`, `--b`, `-build`, or `--build`.
3. Build mode calls `tsoptions.ParseBuildCommandLine` with the full original
   argument list, including the build selector.
4. All other invocations call `tsoptions.ParseCommandLine`.

Ordinary command parser:

1. `ParseCommandLine` calls a shared `parseCommandLineWorker` with compiler
   option declarations and compiler-mode did-you-mean diagnostics.
2. The parser scans args left to right.
3. Empty args are ignored.
4. `@file` reads a response file, tokenizes whitespace and quoted strings,
   reports unterminated quotes, and recursively parses the resulting args in
   place.
5. `-` or `--` removes at most two leading dashes to form the input option
   name.
6. The parser first looks up compiler options, then watch options. Unknown
   options produce diagnostics with did-you-mean and alternate-mode guidance.
7. Non-option args become `fileNames`.
8. Option values are parsed according to option metadata:
   boolean options accept optional `true`/`false`; string/number/enum/list
   options require values; list values are comma-separated; enum values are
   validated; `null` is accepted and propagated where allowed.
9. TSConfig-only options are rejected on the command line except for the
   TypeScript-compatible `false`/`null` escape cases.
10. Parsed raw options are converted into absolute-path-aware compiler
    options, watch options, raw option maps, and `ParsedCommandLine`.

Ordinary execution flow:

1. Report command-line parse errors and exit with
   `DiagnosticsPresent_OutputsSkipped`.
2. If `--pprofDir` is present, start profiling.
3. If `--init`, call `tsc.WriteConfigFile` and exit successfully unless the
   config write reports diagnostics.
4. If `--version`, print version and exit success.
5. If `--help` or `--all`, print help and exit success.
6. Reject `--watch --listFilesOnly`.
7. If `--project/-p` is present, source files on the command line are illegal.
   The project value is interpreted as either a directory containing
   `tsconfig.json` or as a config file path.
8. Without `--project`, Go searches ancestors from the current directory for
   `tsconfig.json` unless `--ignoreConfig` is true and files were supplied.
9. If source files are supplied and a config was found, Go errors unless
   `--ignoreConfig` was supplied.
10. If no files and no config are found, Go reports missing config for
    `--showConfig`; otherwise it prints version/help and exits with a
    diagnostic status.
11. If a config is selected, Go parses `tsconfig.json`, applies command-line
    compiler option overrides, tracks extended config cache state, and returns
    config parse errors as `DiagnosticsPresent_OutputsGenerated`.
12. If `--showConfig`, print the effective config and exit success.
13. If `--watch`, create and start the watch program.
14. Else if options are incremental, run incremental compilation.
15. Otherwise run normal compilation.

Build parser:

1. `ParseBuildCommandLine` uses build option declarations plus common compiler
   and watch options.
2. It separates build options, compiler options, watch options, project paths,
   raw options, and parse errors.
3. `-v` means `verbose` in build mode, not `version`.
4. Empty projects default to `"."`.
5. Invalid combinations are rejected before orchestration:
   `clean+force`, `clean+verbose`, `clean+watch`, and `watch+dry`.
6. Build execution reports parse errors, handles profiling/help, then hands
   the parsed build command to the build orchestrator.

### Rust `tsox`

Source path:

- `src/main.rs`
- `src/execute/mod.rs`
- `src/tsoptions/mod.rs`
- `src/core/compiler_options.rs`
- `src/vfs/mod.rs`

Top-level dispatch:

1. `src/main.rs` passes CLI args to `execute::command_line`.
2. `command_line` checks only the first argument for build mode:
   `-b`, `--b`, `-build`, or `--build`.
3. If a build selector appears later, Rust currently reports a direct
   "must be the first command line argument" diagnostic.
4. Build mode calls `parse_build_command_line`.
5. Ordinary mode calls `parse_command_line`.

Ordinary command parser:

1. `parse_command_line` calls a shared Rust `parse_command_line_worker`.
2. Empty args are ignored.
3. `@file` reads a response file through the configured VFS and recursively
   parses it.
4. `-` or `--` currently uses `trim_start_matches('-')`, which removes all
   leading dashes, not just two.
5. `--name=value` is accepted. This is useful, but it is not modeled in the
   same way as the Go worker.
6. Options are looked up in a Rust `OPTIONS` slice. Unknown options produce a
   simple ad-hoc diagnostic, without did-you-mean or alternate-mode guidance.
7. Non-option args become file names and are normalized to absolute paths in
   ordinary mode.
8. Boolean, string, number, enum, and list options are parsed, but the metadata
   and validation are much narrower than Go. TSConfig-only command-line rules,
   enum-specific diagnostics, min-value checks, and full watch options are not
   yet mirrored.
9. The parser directly fills `CompilerOptions`; watch options and raw command
   option maps are not yet first-class.

Ordinary execution flow:

1. Report command-line parse errors and exit with
   `DiagnosticsPresent_OutputsSkipped`.
2. `--init` now writes `tsconfig.json` before `--version`/`--help`, matching
   Go's control-flow order. The emitted template is currently a smaller Rust
   template, not the full Go `generateTSConfig` output.
3. `--version` prints the Rust port version and exits success.
4. `--help` or `--all` prints simplified help and exits success.
5. Reject `--watch --listFilesOnly`.
6. `--project/-p` rejects mixed source files, then interprets the value as a
   directory with `tsconfig.json` or as a config file.
7. Without `--project`, Rust searches ancestors for `tsconfig.json` unless
   `--ignoreConfig` is true and files were supplied.
8. If source files are supplied and a config was found, Rust errors unless
   `--ignoreConfig` was supplied.
9. If no files and no config are found, Rust reports missing config and prints
   simplified help.
10. If a config is selected, Rust parses `tsconfig.json`, applies command-line
    option overrides, and returns config parse errors as
    `DiagnosticsPresent_OutputsGenerated`.
11. `--showConfig` prints a simplified effective config and exits success.
12. Rust currently always falls through to ordinary `perform_compilation`; it
    does not yet branch to watch or incremental compilation.

Build parser and execution:

1. `parse_build_command_line` now separates build options, compiler options,
   project paths, and errors.
2. `-v` means `verbose` in build mode.
3. Empty projects default to `"."`.
4. Invalid build option combinations are rejected.
5. Rust build execution is still a bridge: it resolves each supplied
   project/config, follows raw `references` depth-first, skips no-root solution
   configs after references, and compiles reached leaf projects through
   `perform_compilation`. It is not yet the Go build orchestrator.

### Differences To Close

- Rust option declarations are still a subset. Many Go compiler, watch, and
  TSConfig-only options are missing or only partially validated.
- Rust does not yet have Go's `NameMap`/did-you-mean/alternate-mode diagnostic
  machinery.
- Rust response-file parsing does not yet report unterminated quoted strings
  like Go.
- Rust strips all leading dashes for option names; Go strips at most two.
- Rust does not yet model watch options separately from compiler options.
- Rust does not yet preserve raw command-line options in the same structure Go
  uses for `--init`, `--showConfig`, and config override wrapping.
- Rust `--init` now follows Go's control-flow order, but the generated
  `tsconfig.json` template is still simplified.
- Rust help/showConfig output is simplified and not yet generated from the
  same option declaration metadata as Go.
- Rust has no watch execution branch.
- Rust has no incremental execution branch.
- Rust build mode has the parser shell and a temporary raw-reference DFS, but
  not the typed project-reference orchestrator, `.tsbuildinfo`, up-to-date
  checks, or real clean/dry/force behavior.

Migration rule for all arguments: parameter handling should be migrated from
the Go declaration-driven parser and execution order. Avoid adding ad-hoc Rust
CLI behavior unless it is a temporary bridge that is documented here.

## TSConfig Flow Audit

Recorded on 2026-07-11 while comparing Go `tsconfig.json` parsing and Rust
`tsconfig.json` parsing.

### Go TSConfig Flow

Source path:

- `internal/execute/tsc.go`
- `internal/tsoptions/tsconfigparsing.go`
- `internal/tsoptions/parsedcommandline.go`
- `internal/tsoptions/showconfig.go`
- `internal/execute/build/host.go`

Flow:

1. Ordinary CLI chooses a config file through `--project` or ancestor
   `tsconfig.json` search before calling `GetParsedCommandLineOfConfigFile`.
2. Build mode resolves project paths to config files through project-reference
   helpers and the build host.
3. Config files are parsed as JSONC through the TypeScript parser, preserving
   source-file locations for diagnostics.
4. Top-level config keys are converted by declaration metadata, including
   `compilerOptions`, `watchOptions`, `typeAcquisition`, `files`, `include`,
   `exclude`, `references`, `extends`, and `compileOnSave`.
5. `extends` is resolved with TypeScript/Node-style config lookup, including
   package configs. The parser tracks a resolution stack and extended config
   cache.
6. Parent compiler options are lower priority than child compiler options;
   command-line options are applied as overrides.
7. Parent `files`/`include`/`exclude` specs are inherited only until the child
   supplies its own corresponding spec.
8. If neither `files` nor `include` is present, Go uses the default include
   spec `**/*`. Explicit `files: []` prevents default include fallback.
9. If no explicit `exclude` is present, Go uses `outDir` and
   `declarationDir` as default excludes when they exist.
10. Include/exclude/file specs are validated, support config-directory template
    substitution, and are matched through `vfsmatch.ReadDirectory`.
11. Wildcard directory walks skip common package folders such as
    `node_modules`, `bower_components`, `jspm_packages`, and dot-git folders.
12. Explicit `files` are retained even when an `exclude` pattern matches them.
13. `references` become `core.ProjectReference` entries with normalized paths,
    original paths, and `circular` flags.
14. `compileOnSave` is retained on the parsed command line.
15. Go reports no-input diagnostics only when the raw config is allowed to
    report them; for example configs with `files` or `references` follow the
    TypeScript rules instead of blindly erroring.
16. `--showConfig` calls `ConvertToTSConfig`, which serializes effective
    compiler options, implied options, resolved file names, include/exclude,
    references, and `compileOnSave`.

### Rust TSConfig Flow

Source path:

- `src/execute/mod.rs`
- `src/tsoptions/mod.rs`
- `src/compiler/mod.rs`

Current flow:

1. Ordinary CLI chooses a config file through `--project` or ancestor
   `tsconfig.json` search before calling
   `get_parsed_command_line_of_config_file`.
2. Build mode currently resolves each supplied project/config path and calls
   the same parser for each config.
3. Rust strips JSONC comments and parses the result through the local JSON
   parser. It does not yet keep source-file positions for config diagnostics.
4. `extends` is supported for a single string value. Parent compiler options
   and root-file specs are merged before child values.
5. `compilerOptions`, `files`, `include`, `exclude`, `references`, and
   `compileOnSave` are parsed. `references` and `compileOnSave` are now carried
   into the simplified `--showConfig` output.
6. Explicit `files: []` prevents default include fallback.
7. Default include is `**/*` when neither `files` nor `include` is present.
8. If no explicit `exclude` exists, Rust adds `outDir` and `declarationDir` to
   the effective exclude list.
9. Literal directory includes recurse.
10. Wildcard include walks skip common package folders and `.git`.
11. Explicit `files` are deduplicated and are not removed by `exclude`.
12. Program construction now matches Go's root-file gate for default libs:
    default libs are loaded only when there is at least one root file and
    `noLib` is not true.

### TSConfig Differences To Close

- Rust config parsing is not declaration-driven, so option type diagnostics,
  TSConfig-only option rules, enum diagnostics, and path normalization are
  still incomplete.
- Rust JSONC parsing strips comments with a simple preprocessor; Go parses
  JSONC through the TypeScript parser and can report source-positioned config
  diagnostics.
- Rust `extends` does not yet fully implement Go's package/Node-style config
  resolution, resolution-stack cycle diagnostics, or extended config cache.
- Rust does not yet model full `watchOptions` or `typeAcquisition`.
- Rust stores `references` as raw JSON values for showConfig; Go stores typed
  `ProjectReference` values with normalized path/original path/circular data.
- Rust `--showConfig` remains simplified: it does not yet serialize full
  effective compiler options, implied options, resolved file names relative to
  config, or command-line-only option stripping like Go.
- Rust no-input diagnostics are still incomplete compared with Go's
  `shouldReportNoInputFiles` and raw-config checks.
- Rust file matching is a pragmatic glob implementation, not yet fully
  equivalent to Go/TypeScript `vfsmatch`.

Migration rule for tsconfig: config parsing defines the program root set.
When lib/parser errors differ, verify `tsconfig.json` root-file expansion,
default include/exclude, references, and compiler-option overrides before
classifying the issue as a parser or lib bug.

## Migration Rule

Every migrated behavior should add or extend a parity case. Prefer comparing
exit code, stdout, stderr, and emitted file contents against the Go oracle before
considering a task done.
