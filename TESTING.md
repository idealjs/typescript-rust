# TESTING — 测试状态与历史

## 当前状态（2026-09-04；各轮记录按时间正序追加在下方各段）

- **全量：compiler 6,537 + conformance 5,907 + transpile 22 = 12,466 条，双轮全量 sweep 全 0 FAIL**（第二轮为修复后验证轮；6 并发约 2.5h/轮）。
- 四套门禁：`cargo test --lib` 1353 / `--test parity` 2 / `--test checker_parity` 1010 / `--test lsp_integration` 15。
- 遗留已知差异全部在 `tests/baselines/reference/triaged.txt`（日期根因组）；性能工作在 idealjs/ts-go-rust-bench（根因报告在其 results/）。
- 历史规则与命令见下方「## 整理轮回归（2026-09-04，整理+单测后全量验证）

- **整理**：sweep 脚本归档 `scripts/sweeps/`（34）；次级文档归拢 `docs/`（9）；README/TESTING.md 头部/TODO.md 重写为当前状态；全部编译警告清零（cargo fix 清 unused imports；port-parity 占位 dead_code 逐项 allow；`convergence_tests`/`node_format_tests` 补 `#[cfg(test)]` 门——此前泄漏进生产 lib 编译；serde 字段名 struct 级 allow）。
- **单元测试**：lib 1353→**1362**（+9），新增修复族回归单测：TS2349 apparent-type 链、never-交集 callee、联合目标/联合源 elaboration、optional 的 strictNullChecks 门控、IndexedAccess TP 注记、node10 TS5107+6280 链、每文件 @jsx pragma 覆盖。
- **四门禁**：1362 / 2 / 1010 / 15 全绿；三笔提交 1d515ab / 179b776 / 1b1ad10。
- **12,466 条回归（6 并发，~2.5h）：0 FAIL**（5,665P / 2,305S / 4,474ad + transpile 22ad）；4 例超时 SKIP（jsxRuntimePragma / arbitraryModuleNamespaceIdentifiers_module / callChainWithSuper / tsxReactEmitSpreadAttribute）单跑核实为已知慢多配置族（真实输出或台账 DIFF，非挂死）。与上一验证轮结论一致，整理无回归。

## Go 标准对齐轮（2026-09-04，按 tsgo 源码标准重建测试口径）

- **oracle 切换**：默认基线改为 **tsgo 自有基线**（`typescript-go/tsc/testdata/baselines/reference` 镜像至 `tests/baselines/reference-go/`，7,200 份 errors.txt；比对时裁扁平段 + CRLF 归一）。旧 upstream 基线树保留，`TSOX_BASELINE_FLAVOR=upstream` 可切回。台账按口径分家：go 口径用 `triaged-go.txt`（初始为空），upstream 口径沿用 `triaged.txt`。
- **用例集对齐**：应用 tsgo `compiler_runner.go` 的 `skippedTests`（47 项，removed-options/API samples）；配置级跳过"tsgo 树无该 (suffix) 基线"的配置（其运行器不产出即不判定）。
- **逐用例双跑对拍**（每个用例每个配置两侧各真实运行一遍，字节比对同一 tsgo 基线）。

  原始计数（各自全量）：

  | | Rust (tsox) | Go (tsgo) |
  |---|---|---|
  | 运行配置数 | **14,911** | **14,816** |
  | 与 tsgo 基线一致 | **5,978** | **6,792** |
  | 与 tsgo 基线不一致 | **4,763** | **6,978** |
  | 跳过 | **4,170** | **1,046** |

  逐配置配对视图（并集 15,021 个配置）：

  | 配对结果 | 配置数 | 谁对 |
  |---|---|---|
  | Rust ✓ 且 Go ✓ | 3,709 | 双方一致 |
  | Rust ✗ 且 Go ✗ | 3,986 | 双方一致（都与其基线不符） |
  | Rust ✓ / Go ✗ | 2,201 | Rust 对（Go 当前二进制漂移出其基线树） |
  | Rust ✗ / Go ✓ | **736** | **Go 对——剩余差距，全部工作靶子** |
  | 一侧或双侧跳过 | 4,057 | 不判定 |
  | 覆盖缺口：Go 跑了 / Rust 没跑 | 222 | 需补齐我们的配置展开 |
  | 覆盖缺口：Rust 跑了 / Go 没跑 | 110 | 我方多出的配置展开 |

  用例级：**9,667 / 12,399（78.0%）完全对齐**；含「Go✓Rust✗」配置的用例 **715 个**（652 为默认配置、40 为 target=es2015）；worklist：`scripts/gostd/mismatch_worklist.csv`。
- **715 例差距聚类**（`scripts/gostd/gap_clusters.json` + `gaps/*.diff` 逐例语料）：
  - **664 = tsgo 基线空而我们多报**（长尾检查器缺口族：上下文类型回调漏推 → TS7006/7019、find+谓词收窄 → TS2322、TS7010 类）——tsgo 默认 `strict: true`（declscompiler.go DefaultValueDescription）下这些程序本应干净，是我们的收窄/推断缺口，逐例修复
  - **44 = tsgo 报而我们漏**（TS2318 全局类型未找到 ×9、TS5069/TS5053/TS5055/TS2209/TS5009/5056/5074/5091/18035/5067/6379 等选项与声明文件诊断族）
  - **11 = 我们多报 TS5107**（其基线为 node10 移除前时代）+ **8 = 我们多报 TS6053**（@types 自动发现缺失）+ **9 同码文本/位置差**
- 过程记录：曾假设"strict 族 Unknown 应默认关"并翻转 `get_strict_option_value`——tsgo CLI 实测（TS2454/TS7010 空旗标即报）与 decls DefaultValueDescription:true 证明翻转错误，已完整回滚（src 仅余注释措辞差异）；判定以 tsgo CLI+自有基线双重实证为准。
- 测量工具入库 `scripts/gostd/`（godecls 解析 Go 选项声明得 72 项 vary-by / diffrun 逐配置双跑 / rust_diff / pair_analyze 配对）；全程修正三个测量伪影：指令行剥离的行号语义、`<no content>` 哨兵、CRLF 书写器编码（用例源码 9,247 个 CRLF 文件原样不动作被测内容）。
- 四门禁 1362/2/1010/15 绿；本次仅动 tests/ 与 scripts/，src/ 无改动。

# 测试流程

# 测试流程

# 测试流程」「# conformance 轮」与各页记录；旧 sweep 脚本在 `scripts/sweeps/`，过程性文档在 `docs/`。

# 测试流程

1. 每次测试 1000 个测试用例,不需要考虑回归。指令如下

```
TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=999 TSOX_SUBMODULE_JOBS=4 cargo test --test submodule_compiler
```

2. 测试完成后，检查测试日志，对比 go 版，是否符合测试预期。
  - 如果不符合预期，检查原逻辑，尝试修复，重复测试步骤
  - 如果符合测试预期，记录到`当前批次`后，执行后100个测试用例，重复测试步骤

3. 严格 1000 个一批、只向前、不做回归
4. 不允许原逻辑未跳过的情况下，进行跳过测试用例

# 分诊（triage）规则

## 什么是分诊

某失败用例的背后是**一整个未移植的子系统**（如 .d.ts 声明产生、泛型实例化、
contextual signature 推理），无法在一批内逐例修复时，把该用例的基线路径登记到
`tests/baselines/reference/triaged.txt`。harness 再跑它时：

- 用例**照常运行、照常对比输出**（不是 skip；skip 是因不支持的编译选项直接不运行）
- 输出与官方基线不一致时，因路径在 triaged.txt 中，记为 `accepted-diff` 而非 fail

批次结果三类：`passed`（与官方基线完全一致）/ `accepted-diff`（分诊的已知差异）/
`skipped`（选项不支持未运行）。0 fail = passed + accepted-diff 覆盖全部。

## 判断标准

- 能在合理范围内对齐 Go 行为的**必须修**，不许用分诊掩盖实现错误
- 分诊的必须是"缺子系统"，不是"实现错了但懒得改"
- 每个分诊组必须带注明根因的日期头，说明缺什么

## triaged.txt 条目格式

- 路径相对 `tests/baselines/reference/`，即 `compiler/<stem>.errors.txt`
- `<stem>` 是**去掉 .ts/.tsx 扩展名**的用例名；多配置基线（如 `// @strict: true, false`
  产生的两份基线）写成 `compiler/<stem>(key=val).errors.txt`，
  例：`compiler/deleteExpressionMustBeOptional(strict=true).errors.txt`
- 组头两行 `##` 包裹：`## 日期 批次范围: 根因描述 ##`
- 同一根因的用例聚成一组，修复子系统后整组删除

## 人工查看分诊条目

```bash
# 总览：按组头列出所有分诊组及其根因
grep '^##' tests/baselines/reference/triaged.txt

# 条目总数
grep -c '^compiler/' tests/baselines/reference/triaged.txt

# 查看某个根因组下的全部条目（例如 const-enum 组；组头是多行 ##，
# 终止条件用下一组的日期头 /^## 2026-/，并过滤掉 ## 行）
sed -n '/const-enum checker family/,/^## 2026-/{/^##/!p}' tests/baselines/reference/triaged.txt | grep '^compiler/'

# 复核单个用例的真实差异：先跑它，再对比参考与实际输出
TSOX_SUBMODULE_START=6100 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_FILTER=<用例名> \
  TSOX_SUBMODULE_JOBS=1 cargo test --test submodule_compiler
diff "tests/baselines/reference/compiler/<stem>.errors.txt" \
     "tests/baselines/local/compiler/<stem>.errors.txt"
# （无差异时 local 下是对应的 .errors.txt.delete 标记：官方有基线、我们无错误输出）

# 修复某子系统后，从台账删除整组条目，并重跑受影响批次验证转绿
```

## 历史记录

逐轮过程记录（Page-N 分页明细、各 sweep 收官数据）已移至 [`docs/test-history.md`](./docs/test-history.md)（按时间正序追加的数据存档）。
