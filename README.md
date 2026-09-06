# tsox — TypeScript compiler (Go → Rust port)

`tsox` 是 Go 原生 TypeScript 编译器
（[typescript-go](https://github.com/microsoft/typescript-go)，TS 7.1.0-dev
快照）的 Rust 移植。Go 实现保留作为行为 oracle。

**状态**：官方回归语料全量 12,466 用例（compiler 6,537 + conformance
5,907 + transpile 22）双轮全量 sweep **0 FAIL**；已知接受的差异登记在
[`tests/baselines/reference/triaged-go.txt`](./tests/baselines/reference/triaged-go.txt)。

## Quick start

环境要求：Rust 工具链 **1.96+**（crate 使用 `edition = "2024"`）。

```sh
git submodule update --init   # 拉取官方 TypeScript 测试语料
cargo build
```

## Running a batch of tests（一次跑一批）

主回归目标是 `submodule_compiler`：把官方 TypeScript 用例对着已提交
基线重放。**一"页" = 100 个用例**；用 `TSOX_SUBMODULE_START` /
`TSOX_SUBMODULE_END` 选择任意窗口（1-based、闭区间、两端均可省略）。

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

进度日志流式输出到 stderr 和 `tests/baselines/local/`；每个用例以一行
`PASS` / `DIFF` / `SKIP` / `FAIL` 收尾。耗时参考：6 并发跑 1000 用例
≈ 15 分钟（compiler），全量 12,444 用例 sweep ≈ 2.5 小时。

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

## Running a test case on BOTH implementations（go/rust 双跑单用例）

对拍定位差异用。`test.sh` 接受**用例路径**（裸文件名 / suite 相对 /
完整相对路径均可）或**差异清单行号**（1-based，对应
`scripts/gostd/divergence_worklist.csv`，即 736 条 Go✓Rust✗ 靶子），
对同一用例分别运行 tsgo（Go oracle）与 tsox（Rust），各配置的输出
与 tsgo 自有基线做统一 diff：

```sh
# 按路径（三种写法等价）
./test.sh compiler/arrayFind.ts
./test.sh arrayFind
./test.sh _submodules/TypeScript/tests/cases/compiler/arrayFind.ts

# 按差异清单行号（行号 42 = divergence_worklist.csv 第 42 行，
# 非 default 配置自动限定到该配置）
./test.sh 42

# 只跑一侧；一次传多个目标
./test.sh --side rust 42
./test.sh 1 arrayFind compiler/2dArrays.ts
```

每个配置打印 `go : 一致/不一致` 与 `rust : 一致/不一致` 加统一 diff
（`-` 为基线行、`+` 为该侧实际输出）；`skip-*`/`timeout` 带原因。
退出码：0 = 双侧全部一致，1 = 任一不一致，2 = 用法错误（路径找不到
或有歧义时列出候选）。

- Rust 侧要求已构建 release 测试二进制：`cargo test --release
  --test submodule_compiler --no-run`（或 `RUST_EXE=<路径>` 指定）
- Go 侧用 `~/workspace/typescript-go/built/local/tsgo`（可用
  `TSGO_BIN` 覆盖）；配置展开 / skippedTests / 基线口径与两侧行内
  runner 完全同源
- **探针边界**：tsgo CLI 与 harness 有少量已知语义差（如 p-mode
  overlay、部分选项不可经 CLI 传递）。当某用例 go 与 rust 输出
  **逐字节相同**却都判「不一致」时，属探针限制而非真实分歧，
  直接对照 `scripts/gostd/gaps/` 下的基线语料即可
- 差异清单（`divergence_worklist.csv`）由
  `scripts/gostd/rust_fresh_run.py` + `build_diff_csv.py` 重生成；
  分诊结论回填其「rust 当前处理方式」列

## Baseline 口径

默认 oracle 是 **tsgo 自有基线**（`tests/baselines/reference-go/`，与
typescript-go 行为对齐）。旧的 upstream JS-tsc 基线保留为交叉口径：

```sh
TSOX_BASELINE_FLAVOR=upstream cargo test --test submodule_compiler
```

## Test gates（测试门禁）

五个测试目标构成门禁（细节与历史见 [`TESTING.md`](./TESTING.md)）：

```sh
cargo test --lib                     # 1362 个库单元测试
cargo test --test checker_parity     # 1010 个 checker parity 测试
cargo test --test lsp_integration    # 15 个 LSP 集成测试
cargo test --test parity             # 2 个 emit-parity 测试（Go oracle）
cargo test --test submodule_compiler # 官方 TypeScript 用例（见上文）
```

## Performance（性能）

对照 Go 编译器的基准测试套件：
[idealjs/ts-go-rust-bench](https://github.com/idealjs/ts-go-rust-bench) —
9 类 34 例，基于 12,444 用例配对计时研究设计。头条结论（2026-09-04）：
单文件编译中位数**比 Go 慢 31×**（固定管线成本，由默认库解析主导）；
checker 内核本身已达到持平或更快。根因分析与修复优先级在 bench 仓的
`results/`。

## Parity tests against the Go oracle（Go oracle 对拍测试）

在相邻的 Go worktree 构建 oracle 并把 `TSGO_ORACLE` 指向它
（自动发现路径：`../typescript-go/built/local/tsgo`）：

```sh
TSGO_ORACLE=../typescript-go/built/local/tsgo cargo test --test parity
```

## Documentation（文档）

| 文档 | 内容 |
|---|---|
| [`TESTING.md`](./TESTING.md) | 测试方法、门禁、当前状态 — **测试从这里开始** |
| [`roadmap/TODO.md`](./roadmap/TODO.md) | 当前 TODO 列表 |
| [`docs/MIGRATION.md`](./docs/MIGRATION.md) | 迁移总览、命令、流程审计（见 audit-*.md） |
| [`docs/ANALYSIS.md`](./docs/ANALYSIS.md) | Go vs Rust 结构对比 |
| [`docs/RUST_ADAPTATIONS.md`](./docs/RUST_ADAPTATIONS.md) | 对 Go 的有意偏离 |
| [`docs/completed-inventory.md`](./docs/completed-inventory.md) | 已完成工作清单与缺口 |
| [`docs/structure-debt.md`](./docs/structure-debt.md) | AGENTS.md 规则 2/3（文件行数/测试布局）债务与计划 |
| [`docs/FIXING.md`](./docs/FIXING.md) | 修复流程约定 |
| [`docs/INTEGRATION_TEST.md`](./docs/INTEGRATION_TEST.md) | 真实项目集成结果 |
| [`docs/PACKAGING.md`](./docs/PACKAGING.md) | 打包/分发（见 packaging-build-matrix.md） |
| [`docs/test-history.md`](./docs/test-history.md) | sweep/运行历史归档（只增不改） |
| [`issues/known-issues.md`](./issues/known-issues.md) | 已知问题索引（正确性/性能/风险） |
| [`roadmap/`](./roadmap/) | 未来规划（TODO、LSP 迁移） |

## Worktree layout（worktree 布局）

- Rust 移植（本仓库，`rust-2` 分支）
- Go oracle：`../typescript-go`（同一克隆的 typescript-go worktree）

## CI

`.github/workflows/rust.yml` 在 push/PR 时运行 `cargo fmt --check`、
`cargo clippy`、`cargo test --lib` 和 `cargo test --test parity`。
