# Command Line Argument Flow Audit

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
