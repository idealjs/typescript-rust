# 测试执行方式（typescript-go → typescript-rust 迁移）

本文档汇总迁移过程中使用的**全部测试方式**、执行命令、前置条件，以及当前
测试基线。其他文档（`MIGRATION.md` / `ANALYSIS.md` 等）只记录流程审计与结构
盘点，统一的测试执行说明集中在此。

更新日期：2026-08-12

---

## 0. 工具链前置条件

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust toolchain | **1.96+**（`edition = "2024"`） | 编译 crate、跑 `cargo test` |
| Go toolchain | （可选）从源码构建 Go oracle | 仅当需要从 `typescript-go` 源码构建 oracle 时 |
| Node.js | 18+ | 仅当需要 `npm install && npm run build` 构建 oracle 时 |

> 本仓库的 Rust 测试**不依赖** Go 工具链即可运行。Go oracle 仅用于
> `--test parity` 的 oracle 对照部分；找不到 oracle 时该部分会被优雅跳过
> （详见第 3 节）。

---

## 1. 测试体系总览

迁移过程中存在 4 个测试目标（test target）+ 2 个示例二进制 + 1 个基准脚本：

| 类别 | 位置 | 形式 | 入口命令 |
|------|------|------|---------|
| **库单元测试** | `src/**/*.rs` 内 `#[cfg(test)]` | `#[test]` | `cargo test --lib` |
| **Checker parity** | `tests/checker_parity.rs` | 集成测试（Program + checker） | `cargo test --test checker_parity` |
| **Emit parity**（含 Go oracle 对照） | `tests/parity.rs` | 集成测试（CLI 进程级） | `cargo test --test parity` |
| **LSP 集成测试** | `tests/lsp_integration.rs` | 集成测试（LanguageService providers） | `cargo test --test lsp_integration` |
| 示例：bundled lib smoke | `examples/smoke_bundled.rs` | 独立二进制（`fn main`） | `cargo run --example smoke_bundled` |
| 示例：parse smoke | `examples/test_parse.rs` | 独立二进制 | `cargo run --example test_parse` |
| 基准：Go vs Rust 冷启动 | `benchmarks/benchmark.sh` | shell 脚本 | `./benchmarks/benchmark.sh [file.ts ...]` |

> ⚠️ **Baseline + submodule 测试（2026-08-12 新增）**：上游 tsgo 的 baseline
> 快照 + git submodule 复用官方测试集两大手段，typescript-rust **已接入
> errors baseline 这一类的首期实现**（第 12 节）。emit/types/symbols 等类别
> 仍待补（第 11 节记录缺口）。

一次跑全部：`cargo test`（会执行上述 4 个 test target + doc-tests）。

### 当前基线（2026-08-13 实跑，Linux，rustc 1.96.0）

> 本表为 2026-08-13 实际执行结果。oracle 为
> `../typescript-go/built/local/tsgo`（`Version 7.1.0-dev`），由 parity 测试
> 自动发现。环境：`cargo 1.96.0` / `rustc 1.96.0` / Go oracle 7.1.0-dev。

**A. 测试目标（gating）**

| 目标 | 命令 | 通过 / 失败 / 忽略 | 耗时 |
|------|------|---------------------|------|
| 库单元测试 | `cargo test --lib` | **1301** / 0 / 0 | 2.2s |
| Checker parity | `cargo test --test checker_parity` | **920** / 0 / 0 | ~70s |
| LSP 集成测试 | `cargo test --test lsp_integration` | **15** / 0 / 0 | <0.01s |
| Emit parity（含 Go oracle 字节对照） | `cargo test --test parity` | **2** / 0 / 0 | ~24s |
| Submodule baseline（官方测试集，默认 1000 case） | `cargo test --test submodule_compiler` | 720 passed / ~164 skipped / 0 failed（§12，per-case 子进程隔离） | ~16min |
| **测试目标合计** | `cargo test` | **lib+checker_parity+lsp+parity = 2237 / 0 / 0**；另含 1000 submodule case | — |

**B. 示例二进制（非 gating，`fn main` 非 `#[test]`）**

| 示例 | 命令 | 结果 |
|------|------|------|
| bundled lib smoke | `cargo run --example smoke_bundled` | ✅ 108 个 lib 全部 0 诊断 |
| parser smoke | `cargo run --example test_parse` | ✅ test1.ts / test3.ts 各 0 诊断 |

**C. 代码质量门（CI 检查）**

| 门 | 命令 | 结果 |
|----|------|------|
| 格式化 | `cargo fmt --check` | ✅ 通过（clean） |
| Clippy | `cargo clippy --all-targets` | ⚠️ 1223 行 warning（迁移期 informational，未 `-D`）。Top：`needless borrow/this`(666) / unused import(85) / unused variable(61) / field-reassign-without-Default(44) |

---

## 2. 库单元测试（`cargo test --lib`）

**测什么**：分布在 `src/` 51 个文件里的 `#[cfg(test)]` 模块，覆盖
scanner/parser/binder/checker/collections/tspath/jsnum/vfs/diagnostics 等
模块的单元逻辑。

**执行**：

```sh
cargo test --lib

# 跑单个模块的测试
cargo test --lib -- parser
cargo test --lib -- checker::checker
```

**已知限制**：少量 `#[ignore]` 测试对应尚未迁移的能力，详见
`RUST_ADAPTATIONS.md`（如 `printer/TestParenthesize*` 需要完整 AST→文本、
`jsnum/TestStringJS` 需要 Node.js V8 引擎）。

---

## 3. Emit parity 测试（`cargo test --test parity`）

**测什么**：`tests/parity.rs` 以 CLI 进程级方式运行 `tsox`，校验
exit code / stdout / stderr / 产物文件内容，并在 Go oracle 可用时**逐字节
对照** Go `tsgo` 的输出。

### 3.1 不带 oracle（纯 Rust smoke）

```sh
cargo test --test parity
```

当没有配置 oracle 时，oracle 对照 case 会被**跳过**（不失败），并在 stderr
打印跳过原因。Rust 自身的 smoke case（产物内容断言）始终执行。

### 3.2 带 Go oracle 对照

`parity.rs` 按以下顺序**自动发现** oracle（找到第一个即用）：

1. 环境变量 `TSGO_ORACLE` 指向的二进制；
2. 相邻 worktree `../typescript-go/built/local/tsgo`（从源码 `npm run build` 的产物）；
3. `../typescript-go/_packages/native-preview/bin/tsgo`（npm 包内的 shim / 二进制）。

三种提供 oracle 的方式：

#### 方式 A：从 typescript-go 源码构建（需要 Go + Node）

```sh
cd ../typescript-go
npm install && npm run build
# 产物：built/local/tsgo
cd -
cargo test --test parity     # 自动发现 ../typescript-go/built/local/tsgo
```

> 注意：此机器当前**未安装 Go**（`spawn go ENOENT`），方式 A 不可用。

#### 方式 B：用已发布的 npm 包二进制（无需 Go）

`@typescript/native-preview-linux-x64` 等 npm 包内置了预编译的 Go 二进制。
任何装过该包的工程 `node_modules` 里都有一个可直接运行的 `tsgo`：

```sh
# 例：a2a 工程装过该包
export TSGO_ORACLE=/home/cqh/workspace/a2a/node_modules/@typescript/native-preview-linux-x64/lib/tsgo
$TSGO_ORACLE --version   # Version 7.0.0-dev.20260407.1
cargo test --test parity
```

#### 方式 C：显式指定

```sh
TSGO_ORACLE=/path/to/tsgo cargo test --test parity
```

> 本机验证：方式 B 下，`compare_with_go_oracle_when_available` 与
> `rust_smoke_cases_emit_expected_outputs` 均**通过**（Rust 产物与 Go oracle
> 字节一致 / smoke 断言通过）。

---

## 4. Checker parity 测试（`cargo test --test checker_parity`）

**测什么**：`tests/checker_parity.rs`（920 个 case）在内存 VFS 中构造
`Program`（含 bundled libs），运行 checker，对产生的语义诊断做断言。覆盖
诊断码 TS2304 / TS2322 / TS2339 / TS2345 / TS2349 / TS2351 / TS2367 / TS2420 /
TS2554 / TS2555 / TS2556 等。

```sh
cargo test --test checker_parity

# 跑单个 case
cargo test --test checker_parity -- checker_var_declaration_no_error
```

**约定**：

- 这类测试**不与 Go oracle 对照**，而是断言 Rust checker 自身的预期行为。
- 其中一部分是 **"KNOWN LIMITATION"** 快照测试：当某项能力尚未迁移时，断言
  当前的（不完美的）诊断数量，并在注释里标注限制。迁移推进后这些断言需要
  同步更新（见第 7 节）。

---

## 5. LSP 集成测试（`cargo test --test lsp_integration`）

**测什么**：构造真实 Program（parse + bind + check），对 `LanguageService`
的各 provider（hover / definition / folding / documentSymbols /
selectionRanges 等）做端到端验证。

```sh
cargo test --test lsp_integration
```

---

## 6. 示例与基准（非 gating）

### 6.1 示例二进制

```sh
# 解析所有 bundled lib.*.d.ts 并报告诊断数
cargo run --example smoke_bundled

# parser smoke
cargo run --example test_parse
```

这些是 `fn main` 的独立二进制，**不是** `#[test]`，不进入 `cargo test`。

### 6.2 冷启动基准

```sh
./benchmarks/benchmark.sh [file.ts ...]
```

对照 `tsgo`（Go oracle）与 `tsox`（Rust）的 `--noEmit` 冷启动耗时。
> 注意：`benchmarks/benchmark.sh` 当前硬编码了旧的 macOS 路径
> （`/Users/cqh/...`），跨机器使用前需要用 `TSOX=` / `TSGO=` 环境变量覆盖，
> 或直接编辑脚本里的默认值。

---

## 7. 迁移期测试维护规则

1. **推进迁移后同步更新 KNOWN LIMITATION 快照**：checker 能力补齐后，
   `checker_parity.rs` 里对应的"断言当前不完美行为"的断言会变得过时
   （例：`checker_dynamic_import_expression_no_error` 原期待 `--noLib` 下
   TS2304 ×2，补齐 `Promise` 解析后变为 TS2304 ×1）。每次更新断言时，
   在注释里记录新的当前行为与 Go oracle 的差异。
2. **优先对比 Go oracle**：能对照 oracle 的场景，优先以 exit code / stdout /
   stderr / 产物文件的逐字节一致为完成标准（见 `MIGRATION.md` 的 Migration
   Rule）。
3. **触碰文件顺手 `cargo fmt`**：整仓 `cargo fmt --check` 尚未对齐，迁移期
   fmt/clippy 为 informational，`cargo test` 与 parity smoke 为 gating。

---

## 8. CI 中的测试（`.github/workflows/rust.yml`）

`test` job 在 push/PR 到 `rust`/`main` 时执行：

```yaml
- cargo fmt --check        # informational（迁移期未整仓对齐）
- cargo clippy --all-targets -- -D clippy::all
- cargo test --lib
- cargo test --test lsp_integration
- cargo test --test parity # CI 无 oracle，仅跑 Rust smoke
```

CI 环境不预装 Go，因此 `--test parity` 在 CI 上只跑 Rust smoke case
（oracle 对照被跳过）。如需在 CI 上跑 oracle 对照，需要额外缓存/下载
`@typescript/native-preview-*` npm 包并设置 `TSGO_ORACLE`。

---

## 9. 上游（TS→Go）迁移测试体系参考

> 本节是 `typescript-go`（tsgo，从 TypeScript 重写为 Go 的"上一棒"）
> 所用测试体系的实地考察。typescript-rust 是 Go→Rust 的"下一棒"，可以
> 借鉴同一套思路。考察日期：2026-08-12。

### 9.1 核心机制：基线测试（Baseline Testing）

tsgo 的行为一致性不靠手写断言，而靠**输出快照对比**。

- `testdata/baselines/reference/` —— **标准答案**（提交进 git，约 49319 个文件）
- `testdata/baselines/local/` —— **本次运行产出**（`.gitignore`，每次 `hereby test` 先清空）

流程（`Herebyfile.mjs` 的 `runTests`）：

1. `rimraf(localBaseline)` 清空 local；
2. `go test ./...` 跑所有 case，每个 case 把诊断/emit/sourcemap/types 写到 local；
3. 跑完对比 local vs reference，不一致就失败；
4. **unused-baseline 检测**：用 `TSGO_BASELINE_TRACKING_DIR` 记录本次实际用到的
   baseline，找出 reference 里没人引用的"僵尸 baseline"，要求清理。

接受新基线：

```sh
npx hereby baseline-accept   # local → reference（覆盖）
npx hereby diff              # 用 $DIFF 工具对比 local/reference
```

对应源码：`Herebyfile.mjs` 的 `runTests`(L633)、`baselineAcceptTask`(L964)；
Go 侧 baseline 写出/对比在 `internal/testutil/baseline/` 与
`internal/testutil/tsbaseline/`（`DoErrorBaseline` / `DoJSEmitBaseline` /
`DoTypeAndSymbolBaseline` / `DoSourcemapBaseline` 等）。

### 9.2 Git Submodule：复用 TS 官方测试集

`.gitmodules`：

```
[submodule "_submodules/TypeScript"]
    url = https://github.com/microsoft/TypeScript.git
    branch = tsgo-port
```

- 把 `microsoft/TypeScript` 的 `tsgo-port` 分支作为 submodule 引入，**直接复用
  TS 官方几十年积累的测试集**（`tests/cases/compiler` + `tests/cases/conformance`），
  不重复编写。
- `internal/testrunner/compiler_runner.go` 的 `NewCompilerBaselineRunner(testType, isSubmodule)`
  用 `isSubmodule` 切换 basePath：
  - `false` → `tests/cases/<suite>`（in-repo 回归测试）
  - `true`  → `_submodules/TypeScript/tests/cases/<suite>`（官方测试集）
- 注册入口（`internal/testrunner/compiler_runner_test.go`）：
  - `TestLocal` → `runCompilerTests(t, false)`
  - `TestSubmodule` → `runCompilerTests(t, true)`，开头 `repo.SkipIfNoTypeScriptSubmodule(t)`
    在 submodule 未 clone 时优雅跳过。

> 跑 submodule 测试：`go test ./internal/testrunner -run '^TestSubmodule/...'`

### 9.3 差异分类管理

submodule 测试会产生大量"Go 与 TS 官方的预期差异"，用两个清单分类：

| 文件 | 用途 |
|------|------|
| `testdata/submoduleAccepted.txt` (83KB) | **有意为之**、永久接受的差异（按 `## 分组 ##` 组织，如 `## jsdoc ##`） |
| `testdata/submoduleTriaged.txt` (5KB) | **待修复**的已知差异，每组带 GitHub issue 链接（如 `## https://github.com/microsoft/typescript-go/issues/3481`） |

diff 落到 `testdata/baselines/reference/submoduleAccepted/` 或
`.../submoduleTriaged/`（而不是让测试直接 fail），实现"已知差异不阻塞、
新差异必现"。

### 9.4 FourSlash 测试框架（语言服务）

- `internal/fourslash/` —— 完整的 FourSlash 测试框架（源自 TS 官方），
  289 个测试文件（`internal/fourslash/tests/*_test.go`）。
- 测试体里用 `@/*1*/`、`@/*2*/` 标记位置，然后 `f.VerifyBaselineGoToDefinition(t, true, "1")`
  等断言语言服务行为（completion / hover / goToDefinition / rename / codeAction …）。
- 每个测试自带 `defer testutil.RecoverAndFail(t, ...)` 兜底 panic。

### 9.5 多文件 case 解析（`// @FileName`）

`internal/testrunner/test_case_parser.go` 的 `makeUnitsFromTest`：单个 `.ts`
测试文件用 `// @FileName: a.ts` / `// @FileName: b.d.ts` 分隔成多个虚拟文件，
配 `// @module: commonjs` / `// @strict: true, false` 等指令设置编译选项，
甚至支持 `// @link:` 符号链接。一个文件 = 一个完整多文件工程 fixture。

### 9.6 命令速查（typescript-go 侧）

```sh
cd ../typescript-go
npx hereby test                 # 跑全部测试（local + submodule，若已 clone）
npx hereby baseline-accept      # 接受本次 local 作为新基线
go test ./internal/testrunner -run '^TestSubmodule/...'   # 只跑 submodule
go test ./internal/fourslash/...                           # 只跑 fourslash
```

### 9.7 对 typescript-rust 的可借鉴项

| 上游机制 | 当前 tsox 现状 | 借鉴建议 | 优先级 |
|---------|---------------|---------|--------|
| **Baseline 快照对比** | ❌ checker_parity 是手写 `assert_diagnostic_count`；parity 是 2 个 emit case | 引入 `tests/baselines/{reference,local}/` 快照机制，诊断/emit/types 落 baseline 文件，`cargo test` 自动 diff。推进 checker 时只需 `baseline-accept` 而非逐条改断言 | **P0** |
| **Submodule 复用 TS 官方测试集** | ❌ 无 | 把 `microsoft/TypeScript` 的 `tests/cases/{compiler,conformance}` 作 submodule 引入，写一个 Rust testrunner 跑这些 case 做 parity。这是把"2227 个自写测试"升级到"数万个官方测试"的关键 | **P1** |
| **差异分类（Accepted/Triaged）** | ⚠️ 用 `checker_parity.rs` 里散落的 `// KNOWN LIMITATION` 注释 | 集中成 `tests/baselines/accepted.txt`（永久差异）+ `triaged.txt`（待修差异，带 issue），已知差异不阻塞 CI | P1（随 P0/P1 一起） |
| **`// @FileName` 多文件 case** | ⚠️ `check_sources(&[("a.ts","..."),...])` 手写 | 移植 `makeUnitsFromTest`，支持单文件 `// @FileName` 分隔，fixture 更紧凑 | P2 |
| **FourSlash** | ❌ LSP 集成测试只有 15 个手写 case | 长期目标：移植 FourSlash 框架测语言服务（completion/hover/rename）。短期先用 baseline 机制覆盖 checker/emit | P3 |
| **unused-baseline 检测** | N/A | baseline 机制上线后再加，防止僵尸快照 | P3 |

**结论**：tsgo 用 **baseline + submodule** 这套组合拳，把"行为一致性验证"从
手写断言升级为"跑官方测试集 + 快照对比"，测试规模从几百跃升到数万。
typescript-rust 目前还停留在手写 `assert_diagnostic_count` 阶段（920 个
checker_parity case）。**最高价值的下一步是引入 baseline 机制 + submodule
复用 TS 官方测试集**，这与 `TODO.md` 第 3 节 P0/P1 一致。

---

## 10. 完整执行记录（2026-08-12）

> 本节存档一次完整测试执行的原命令与输出，供日后回溯/对比基线漂移。
> 执行环境：Linux x86_64，rustc/cargo 1.96.0，Go oracle 7.1.0-dev（源码构建，
> 自动发现于 `../typescript-go/built/local/tsgo`）。

### 10.1 测试目标

```
$ cargo test --lib
test result: ok. 1290 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.31s

$ cargo test --test checker_parity
test result: ok. 920 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 71.20s

$ cargo test --test lsp_integration
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test --test parity          # 自动发现 oracle，无 TSGO_ORACLE
test compare_with_go_oracle_when_available ... ok
test rust_smoke_cases_emit_expected_outputs ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.14s
```

**合计：2227 通过 / 0 失败 / 0 忽略**。

### 10.2 示例二进制

```
$ cargo run --example smoke_bundled
Total bundled libs: 108
Failed libs: 0
Total diagnostics: 0
ALL BUNDLED LIBS PARSE WITH 0 DIAGNOSTICS

$ cargo run --example test_parse
test1.ts: 0 diagnostics
test3.ts: 0 diagnostics
```

### 10.3 代码质量门

```
$ cargo fmt --check          # exit 0, clean

$ cargo clippy --all-targets  # 迁移期 informational，未 -D
# 1223 行 warning，Top 分类：
#   666  this/needless borrow 类
#    85  unused import
#    61  unused variable
#    44  field assignment outside of initializer (Default)
#    38  associated constant ...
#    36  match looks like ...
```

### 10.4 复现步骤

```sh
# 1. （可选）构建 Go oracle——首次需要 Go + goproxy.cn（proxy.golang.org 被墙）
cd ../typescript-go
export PATH="$PATH:/usr/local/go/bin"
export GOPROXY="https://goproxy.cn,direct"
npm install && npm run build          # → built/local/tsgo (7.1.0-dev)
cd -

# 2. 跑全部 Rust 测试（oracle 自动发现，无需 TSGO_ORACLE）
cargo test                            # 4 个 test target，2227/0/0

# 3. （可选）质量门
cargo fmt --check                     # clean
cargo clippy --all-targets            # 1223 warning（informational）
```

---

## 11. ⚠️ 漏测：Git submodule 官方测试集（errors baseline 已补齐）

> 首期实现见第 12 节。本节记录**仍待补**的类别。

### 11.1 现状（漏测确认 2026-08-12）

| 项 | typescript-rust | 上游 tsgo |
|----|-----------------|-----------|
| `.gitmodules` 引 TS 官方测试集 | ❌ **无** | ✅ `_submodules/TypeScript`（`microsoft/TypeScript` 的 `tsgo-port` 分支） |
| 跑官方 case 的 testrunner | ❌ **无** | ✅ `internal/testrunner`，`TestLocal`/`TestSubmodule` |
| 覆盖的官方测试 case | **0**（自写 920 checker_parity） | **12114**（compiler 6419 + conformance 5695） |

**覆盖率差距**：typescript-rust 自写 checker_parity 920 个 case，对比官方
12114 个，约覆盖 **7.6%**；且自写 case 是手写 `assert_diagnostic_count`，
不是真正的"跑官方 fixture 对输出"。

### 11.2 已在 oracle 侧实地验证（typescript-go）

为确认这套测试的形态，本次在 `typescript-go` 侧把 submodule 接通了：

```sh
# GitHub 在沙箱里不可达，改用本地已有的 TypeScript 副本填充 submodule
cd ../typescript-go
git config submodule."_submodules/TypeScript".url /home/cqh/workspace/TypeScript
git config --global protocol.file.allow always   # file transport 默认禁用
git submodule update --init                       # checkout 到 4d4f005c
```

验证结果：
- submodule 测试用例规模：**compiler 6419 + conformance 5695 = 12114 个**
- 实跑 `TestSubmodule` 形态：

```
$ go test ./internal/testrunner/ -run '^TestSubmodule/compiler/.*compilerOptionsOut' -v
=== RUN   TestSubmodule/compilerOptionsOutAndNoEmit.ts
=== RUN   TestSubmodule/compilerOptionsOutDirAndNoEmit.ts
--- PASS: TestSubmodule (0.34s)
    --- SKIP: TestSubmodule/compilerOptionsOutAndNoEmit.ts (unsupported outFile)
    --- PASS: TestSubmodule/compilerOptionsDeclarationAndNoEmit.ts
    --- PASS: TestSubmodule/compilerOptionsOutDirAndNoEmit.ts
ok  github.com/microsoft/typescript-go/internal/testrunner  0.386s
```

每个官方 `.ts` case → 一个 `TestSubmodule/<case>.ts` 子测试，runner 解析 case
内的 `// @FileName`/`// @module` 等指令、编译、把诊断/emit/types 与
`testdata/baselines/reference/submodule*/` 快照对比。已知差异由
`submoduleAccepted.txt`(1078 文件) / `submoduleTriaged.txt`(76 文件) 分类。

### 11.3 补齐方案（与 TODO.md P0/P1 对齐）

**P0 — baseline 快照机制**（submodule 测试的前置依赖）：
当前手写 `assert_diagnostic_count` 无法承接上万 case。需先在 typescript-rust
建立 `tests/baselines/{reference,local}/` + diff + `baseline-accept` 工作流
（仿 tsgo 的 `internal/testutil/{baseline,tsbaseline}/`）。

**P1 — submodule + Rust testrunner**：
1. `git submodule add` 引入 TS 官方测试集（沙箱内可指向本地 `/home/cqh/workspace/TypeScript`）；
2. 在 `tests/` 写 Rust testrunner：枚举 `_submodules/TypeScript/tests/cases/{compiler,conformance}/*.ts`，
   用 `makeUnitsFromTest` 等价逻辑解析 `// @FileName` 多文件 + 编译选项指令，
   跑 checker/emit，输出与 reference baseline 对比；
3. 差异分类：`tests/baselines/accepted.txt`（永久差异）+ `triaged.txt`（待修，带 issue），
   已知差异不阻塞 CI。

**预期收益**：测试规模从 2227（含大量手写单元测试）跃升到 **1.4 万 +**（含
官方全集），行为一致性验证从"抽样"升级为"全量快照对比"。

---

## 12. Baseline + submodule 测试（官方测试集）—— 首期已实现

> 2026-08-12 上线。复刻上游 tsgo 的 baseline 快照 + git submodule 复用官方
> 测试集两大核心手段，**首期实现 errors baseline（诊断快照）这一类**。
> emit/types/symbols/sourcemap 等类别见第 11 节后续计划。

### 12.1 是什么

`tests/submodule_compiler.rs` 执行 TypeScript 官方 `tests/cases/compiler/` 下的
测试用例（经 git submodule 引入，`_submodules/TypeScript`），把 Rust checker 产
出的诊断渲染成快照，与 `tests/baselines/reference/compiler/<case>.errors.txt`
对比。这是**真正的 checker parity**——直接跑上游积累的数千个 case，而非手写
`assert_diagnostic_count`。

每个官方 `.ts` case 顶部可带指令（`// @module: commonjs` / `// @strict: true` /
`// @filename: a.ts` 等），runner 会解析这些指令构造对应的 `CompilerOptions` 和
多文件工程，再编译、收诊断、与 baseline 对比。

### 12.2 怎么跑

```sh
# 默认：跑前 1000 个 compiler case（约 16 分钟；每 case 独立子进程隔离）
cargo test --test submodule_compiler

# 跑指定数量
TSOX_SUBMODULE_LIMIT=200 cargo test --test submodule_compiler

# 跑全部 ~6400 个 compiler case（checker 完善后逐步开放）
TSOX_SUBMODULE_LIMIT=0 cargo test --test submodule_compiler
```

### 12.3 Accept 工作流（baseline 漂移时）

当 checker 改进导致诊断输出变化时，baseline 会"漂移"。审查后若新输出正确，
接受为新基线：

```sh
TSOX_BASELINE_ACCEPT=1 cargo test --test submodule_compiler   # actual → reference
cargo test --test submodule_compiler                           # 复跑确认全绿
```

接受模式会把本次实际输出**覆盖**写入 `tests/baselines/reference/compiler/`，
并删掉不再有诊断的 baseline 文件。

### 12.4 已知差异分类（accepted / triaged）

- `tests/baselines/reference/accepted.txt` —— 永久接受的有意差异（不阻塞 CI）
- `tests/baselines/reference/triaged.txt` —— 待修的已知差异（带说明/issue）

格式：每行一个 `<subfolder>/<name><ext>`（如 `compiler/foo.errors.txt`），
`#` 开头与空行忽略，可用 `## 描述 ##` 分组。命中清单的 case 即使 baseline
漂移也**不 fail**（只记数）。

### 12.5 跳过规则

runner 以下情况**跳过**（不 fail）case，对齐上游 `SkipUnsupportedCompilerOptions`
与 `skippedTests`：

- `SKIPPED_CASES`（移植自 tsgo 的 `skippedTests` + `binderBinaryExpressionStress{,Js}.ts`）——
  引用旧 `typescript.d.ts`、已移除的选项，或病态深嵌套二进制表达式（栈溢出，
  `catch_unwind` 无法捕获）
- 未识别的编译选项指令
- `module = amd/umd/system`、`moduleResolution = node10/classic`、`target = es5`、
  `allowJs`、`baseUrl`/`outFile` 非空（Rust 侧尚未支持）
- `circular*` 家族（circular 类型递归——checker 无 Go 的 `instantiationDepth`
  递归保护，会栈溢出），整族 skip
- checker / 渲染 **panic**（`catch_unwind` 兜底，转成 skip 避免中断整个 run）
- checker **栈溢出**（circular 之外的偶发深递归）：每 case 在**独立子进程**里跑，
  子进程被信号杀掉只记 skip，不中断整轮
- 非 UTF-8 文件（如 `bom-utf16{be,le}.ts`，`read_to_string` 失败 → skip）

> **per-case 子进程隔离（2026-08-13）**：runner 把每个 case 放进一个独立子进程
> （worker 模式：`TSOX_SUBMODULE_WORKER`/`TSOX_SUBMODULE_OUT`），父进程收集结果。
> 这样 checker 的栈溢出（`catch_unwind` 抓不住）只杀子进程，不会中断上万 case 的
> 整轮扫描。代价：每 case 多一次进程启动（~50ms），1000 case 约 16 分钟。

### 12.6 首期范围与后续

**首期（2026-08-12）**：
- ✅ git submodule（`_submodules/TypeScript`，`tsgo-port` 分支，commit `4d4f005c`）
- ✅ errors baseline（诊断快照，`format_diagnostic_compact` 单行格式）
- ✅ `// @filename` 多文件 + `// @module`/`@strict`/... 指令解析
- ✅ accept/triage 工作流 + skip 规则
- ✅ 默认 50 case（已 accept 初始 baseline）

**2026-08-13 扩量到 600 case**，期间修复：
- parser：私有标识符 `#name`、对象字面量 `get`/`set` accessor 与方法、
  参数可访问性修饰符（`public v`）、类 index signature `[k:T]:V`、
  `<` 歧义（`try_parse_type_arguments` 试探性回溯，区分 `i < n` 与 `f<T>()`）、
  泛型箭头函数（`try_parse_generic_arrow_function`，`[async] <T>(params) => body`）
- options：`set_bool` 大小写 bug（`@allowJs`/`@strict` 等 bool 指令原先被静默丢弃）
- 健壮性：`line_and_character` 对越界 offset 做 clamp、`catch_unwind` 包住渲染、
  跳过栈溢出 stress case 与非 UTF-8 文件
- 3 个过时 "KNOWN LIMITATION" 快照随之修正

**2026-08-13 续：扩量到 1000 case（720 pass / ~164 skip / 0 fail）**：
- per-case **子进程隔离**（worker 模式），checker 栈溢出只杀子进程不中断整轮
- `circular*` 整族 skip（checker 缺递归保护）
- 新增 116 个 errors baseline（601–1000，多为 checker 差距快照）

**后续阶段（见第 11 节 + TODO.md）**：
- raise `DEFAULT_LIMIT` 直至全量 6419 compiler case（当前 1000）
- emit / types / symbols / sourcemap baseline 类别
- vary-by 配置矩阵（`// @strict: true, false` 笛卡尔积，首期只取首值）
- conformance 目录（5695 case）
- 与 Go oracle 字节对齐 baseline 格式（首期用 Rust 自己的 compact 格式）

### 12.7 协作者如何准备 submodule

GitHub 直连可达时：

```sh
git submodule update --init --recursive
```

GitHub 不可达（如本沙箱）时，用本地 TypeScript 副本填充：

```sh
git config submodule._submodules/TypeScript.url /path/to/local/TypeScript
git -c protocol.file.allow=always submodule update --init
```

---

## 13. Checker parity gap 分析（官方测试集 vs Go oracle）

> 2026-08-12 首次测量。方法：跑前 300 个 compiler case，对每个 case 同时跑
> Rust checker 与 Go oracle（`tsgo --noEmit`），对比两者的诊断码集合。

### 13.1 总体一致性（140 case 抽样对照）

| 类别 | 数量 | 占比 | 含义 |
|------|------|------|------|
| **完全一致**（双方诊断码集合相等） | **29** | 21% | Rust 行为正确 |
| **漏报**（oracle 报、Rust 不报） | **44** | 31% | Rust 缺检查 |
| **诊断码不同**（双方都报但码集合不同） | **58** | 41% | Rust 诊断不完整/错误 |
| **假阳性**（Rust 报、oracle 不报） | **9** | 6% | Rust 多报 |

**结论：约 79% 的抽样 case 存在 checker parity 问题**。这与 `ANALYSIS.md`
记录的"checker 完成度 ~20%"吻合。baseline 快照本身不能区分正确/错误——
**必须与 Go oracle 对照**才能定位真问题。

### 13.2 漏报 Top（Rust 缺的检查，按频次）

| 诊断码 | 频次 | 含义 | 对应缺口 |
|--------|------|------|---------|
| **TS2307** | 43 | Cannot find module | `module/` resolver 主路径未迁移（ANALYSIS.md） |
| **TS1202** | 32 | Import assignment in ESM target | module format 与 target 交互检查缺失 |
| **TS2300** | 18 | Duplicate identifier | declaration merge / 重复标识符检查 |
| **TS1005** | 18 | `X` expected（语法错误） | parser 对部分语法的恢复/报错 |
| **TS2564** | 17 | Property no initializer | strict property initialization（strict 模式） |
| **TS1183** | 12 | Implementation in ambient context | `.d.ts` 上下文检查 |
| TS7032 | 7 | set accessor 隐式 any | implicit any in accessor |
| TS2322 | 6 | Type not assignable | 类型关系（已有部分，覆盖不全） |
| TS2352 | 5 | Conversion may be a mistake | 类型转换重叠检查 |

### 13.3 假阳性 Top（Rust 误报）

| 诊断码 | 频次 | 含义 | 对应缺口 |
|--------|------|------|---------|
| **TS2339** | 7 | Property does not exist | 属性查找 / 类型成员解析缺陷 |
| TS2345 | 3 | Argument not assignable | 类型关系（参数位） |
| TS2300 | 3 | Duplicate identifier | 误报重复（merge 未处理） |
| TS2349/2322 | 4 | — | 类型关系边界 |

### 13.4 典型差异示例

| Case | Go oracle | Rust | 问题 |
|------|-----------|------|------|
| `ClassDeclaration25` | TS2391 ×2（函数声明缺实现） | TS2420（类未实现接口） | 诊断完全不同 |
| `ExportAssignment7` | TS1203+TS2309+TS2304 | 仅 TS2304 | 漏报 2 个（ESM export 检查） |
| `ParameterList5` | TS2369（参数属性仅构造函数） | TS2304（把 `public` 当标识符） | 诊断错误 |

### 13.5 修复优先级建议

按"影响面 × 可行性"排序：

1. **模块解析（TS2307, 43 次）** — 最大单一缺口。迁移 `internal/module`
   resolver 主路径，可一次性消除最多漏报。
2. **declaration merge / 重复标识符（TS2300）** — 同时出现在漏报与假阳性，
   修复可双向改善。
3. **属性查找（TS2339, 假阳性 7 次）** — 影响最大类的误报，修复后假阳性显著下降。
4. **strict 系列检查（TS2564/TS7032/TS7006）** — implicit-any / 属性初始化检查，
   strict 模式 case 的基础。
5. **ESM/module-format 交互（TS1202/TS1203）** — export/import 在不同 target 下的检查。

### 13.5.1 修复进展：TS2307 emission（2026-08-12）

**根因（经 Explore 确认）**：TS2307 常量已定义（`diagnostics::CANNOT_FIND_MODULE_...`）但**从未被任何代码 emit**。`Program::new` 的 import 解析循环（`compiler/mod.rs`）在解析失败时**没有 else 分支**——失败被静默丢弃。

**已修复**：
1. `src/compiler/mod.rs` 解析循环：当 `is_resolved == false` 时 emit TS2307
   （`Diagnostic::new(file, import_node.loc, CANNOT_FIND_MODULE_..., [spec])`）。
   `file.imports` 只含真实 import（ambient `declare module "x"` 名字分开收集），故无需额外过滤。
2. `tests/submodule_compiler.rs` `build_and_check`：改为取**全集诊断**
   （`program.diagnostics()` 构造诊断 + `get_semantic_diagnostics()` 语义诊断）。
   `get_semantic_diagnostics` 只返回 checker+binder 层，TS2307 属构造层，此前被漏掉。

**效果**：TS2307 从 **0 → 7**（emission 修复生效，无回归：lib 1290 + checker_parity 920 全绿）。

### 13.5.2 重新评估 TS2307 parity（重要纠正，2026-08-12）

进一步深入后发现：**§13.2 把 TS2307 列为"漏报 43 次"是基于有偏的对照**。原因：

oracle 是**单文件**跑（`tsgo --noEmit <case>.ts`），看不到官方 case 里 `// @filename:`
声明的其他虚拟文件。而 Rust testrunner 把所有 `@filename` 文件都放进 InMemoryFS。
因此 oracle 对**相对路径 import**（`./aliasAssignments_moduleA`，目标文件在同 case 里）
误报 TS2307，而 Rust 正确解析到了 → 看起来像"漏报"，其实是 **harness 差异**。

按 import 形态拆分 oracle 的 TS2307（前 300 case）：

| 形态 | oracle TS2307 次数 | 性质 |
|------|-------------------|------|
| **相对路径**（`./b`、`./xxx_backbone`） | 56 | 多为 harness 差异（跨文件 import），Rust 正确解析，**非漏报** |
| **bare module**（`foo`、`m2`） | 19 | 公平对照（任何 harness 都该报） |

bare module 的 19 次里，去除被 skip 的（system/amd），**active case 约 11 个**：
- Rust **正确报了 5 个**（如 `amdDependencyComment1`、`ambientExternalModuleInAnotherExternalModule`）
- **漏报 6 个**，多为特殊形态（`///<amd-dependency path="m2"/>` 注释式依赖、ambient module 嵌套）

**resolver 假成功是误判**：之前推断"resolver 对 bare module 误返回 is_resolved=true"，经单测核实**不成立**——
`resolve_bare_specifier_not_found` 单测证明 resolver 对不存在的 bare module 正确返回 `is_resolved=false`。
agent 的深入排查 + 实地复现（`probe_bare_module_no_node_modules_dir`）均确认。

**结论**：TS2307 emission 已正确工作，真实漏报仅 ~6 个（特殊形态）。TS2307 **不再是高杠杆项**。
真正的大噪声源是 **parser 语法 gap（TS1127/1128/1005）**——引入全集诊断后暴露，
300 case 里 564+554+261 次，是 parser 对部分 TS 语法解析失败，应升为 P0。

### 13.5.3 修复进展：二进制文件检测 TS1490（2026-08-12）

深入分析 parser gap 的分布后发现：**TS1127(564)/TS1128(554) 几乎全部来自单个 case
`TransportStream.ts`**（该 case 内容是一堆无效字节，故意测试二进制输入）。

**根因**：Rust scanner 缺少 Go 的"二进制文件检测"（TS1490 `File_appears_to_be_binary`，
`scanner.go:935-940`）。当 scanner 遇到 UTF-8 解码失败的无效字节（Rust 的 `chars()`
将其表现为 `U+FFFD` 替换字符），Go 报一次 TS1490 并跳到文件末尾；Rust **没有这个检测**，
逐字节报 TS1127，导致单个二进制 case 产生 **1122 个诊断**。

**已修复**（`src/scanner/mod.rs`，对齐 Go `scanner.go:937`）：
1. `DiagnosticKind` 加 `FileAppearsToBeBinary` 变体
2. scanner 的 unknown-character 分支：检测到 `c == '\u{fffd}'` 时，报 TS1490（pos=0,len=0）、
   `pos` 跳到文件末尾、返回 `EndOfFile`
3. `src/parser/mod.rs` 的 DiagnosticKind→Message 映射加 `FileAppearsToBeBinary`

**效果（前 300 case）**：

| 诊断码 | 修复前 | 修复后 | 降幅 |
|--------|--------|--------|------|
| TS1127 | 564 | 7 | -557 |
| TS1128 | 554 | 5 | -549 |
| TransportStream 总诊断 | 1122 | 6 | -1116 |

零回归（lib 1290 + checker_parity 920 全绿）。TransportStream 从 1122→6 行，
与 oracle 的 4 行接近（双方都报 TS1490，仅错误恢复细节略异）。

**剩余**：TS1005 仍有 256 次（分布在 ~59 个 case），是真实的 parser 语法恢复长尾
（`;` expected 等），与二进制检测无关，属独立的后续工作。

### 13.5.4 修复进展：class get/set accessor 解析（2026-08-12）

分析 TS1005 长尾分布后，发现**最大的单一语法簇是 class `get`/`set` accessor**
（59 个 TS1005 case 里 25 个含 accessor 语法）。

**根因**：`parse_class_member`（`parser/mod.rs`）不识别 `get`/`set` 关键字为
accessor，把它们当普通属性名消费，于是 `get name() {}` 在 `(` 处报 `;' expected`。
Go 在 `parseClassElement`（parser.go:1862-1867）用 `parseContextualModifier` +
`canFollowGetOrSetKeyword` 消歧（`get`/`set` 后跟属性名才算 accessor）。

**已修复**（`src/parser/mod.rs`，对齐 Go）：
1. `parse_class_member`：在 modifier 收集后、`parse_property_name` 前，加 `get`/`set`
   检测分支——用 scanner clone 前瞻一个 token，若满足 `token_can_follow_get_or_set`
   （属性名或 `[`，排除 `(`/`:`/`;`/`{`/`*`），则走 `parse_accessor_declaration`
2. 新增 `parse_accessor_declaration`：消费 `get`/`set` → `parse_property_name` →
   `parse_optional_type_parameters` → `parse_parameter_list` → `parse_optional_return_type`
   → body（`{}` 或 `;`），构造 `GetAccessorDeclaration`/`SetAccessorDeclaration`
   （AST 节点类型已存在，无需新增）
3. 新增 `token_can_follow_get_or_set` helper（对齐 Go `canFollowGetOrSetKeyword`）

**效果**：

| 指标 | 修复前 | 修复后 | 降幅 |
|------|--------|--------|------|
| TS1005 总次数 | 256 | 187 | -69（-27%）|
| TS1005 case 数 | 59 | 42 | -17 |

清掉了全部 25 个 accessor 相关 case。零回归（lib 1290 + checker_parity 920 全绿）。
注：`checker_getter_setter_no_error` 原是 pass-through（accessor 误解析时 `value`
当普通属性解析了），现 parser 正确解析 accessor 但 **checker 尚未建模 accessor 成员**，
故 `c.value` 报 TS2339——已更新为 KNOWN LIMITATION 快照（checker accessor 语义是后续工作）。

**剩余**：TS1005 仍有 187 次/42 case，属其他语法点（arrow function 泛型默认值、
object literal methods、async generator 等），逐类排查的长尾工作。

### 13.6 如何复现对照分析

```sh
TSGO=/home/cqh/workspace/typescript-go/built/local/tsgo   # 或 npm 包二进制
# 对单个 case 对照
$TSGO --noEmit _submodules/TypeScript/tests/cases/compiler/<case>.ts   # oracle 诊断
cat tests/baselines/reference/compiler/<case>.errors.txt               # Rust 诊断
```

> 注：本次对照用 Rust baseline（`format_diagnostic_compact` 单行格式）与
> oracle 的 `file(line,col): error TScode` 格式比**诊断码集合**，不比位置/文本。
> 位置级对照留待后续（需统一格式或写专用 diff 工具）。
