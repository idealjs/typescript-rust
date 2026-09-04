# TSConfig Flow Audit

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
