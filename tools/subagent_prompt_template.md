# corpus 修复 subagent 派发模板(v2)

> 供主 agent 编排语料修复循环时生成派发 prompt。占位符 `{{...}}` 由主 agent 填充。
> v2 相对 v1 的改动动机:上一波 8 agent 并行 3.4 小时后 ticket 过期,4/8 零提交被中断,
> 根因是 prompt 缺时间预算、缺退出契约、单一末端提交点、无 CPU 上限、未禁全量验证。
> 记录见 decisions/ 下对应 ADR。

## 模板正文

```
# 任务:Rust 版 TypeScript 编译器语料修复(簇:{{cluster-id}})

## 时限与退出契约(最高优先级,先读)
- 总预算 {{budget-min}} 分钟。开始时记下时刻,每完成一步对照剩余时间。
- 剩 {{budget-rest-min}} 分钟时无论进行到哪:停止新根因,把已验证的修改提交,写 progress_notes.md 交接,输出报告,结束。
- 单个根因投入 {{budget-input-min}} 分钟仍未让任何清单内用例转绿:停,分析写进 progress_notes.md,换下一个根因或收尾。
- 验收线({{n}} 代表例全绿)达成且已提交:立即报告退出,不开新根因。簇内其余用例只在预算富余时顺带,不作为交付条件。
- 汇报后任务即结束。你的交付物 = 分支上的 commit + progress_notes.md + 最终报告,不是"把簇修完"。

## 提交纪律(防中断丢失)
- 每完成一个独立根因修复并通过单例验证,立即 git commit(不等整簇批量)。
- commit 只 add 你改的源码路径;assigned_cases.txt / progress_notes.md / 简报不入库。
- 中断随时可能发生:工作区长时间只有未提交修改 = 交付为零。

## 环境故障协议
- Bash 工具连续 3 次失败(适配器 shutting down、超时等):把现场写进 progress_notes.md,立即结束退出。等待重试累计不超过 5 分钟。
- 疑似死循环/OOM(进程被杀、输出不完整):受控探针测斜率、变体二分,不盲目重跑;定位超 15 分钟即按超时处理。

## 资源上限(4 agent 并行约定,违者打回)
- 构建:`CARGO_BUILD_JOBS=3`;批量测试:`TSOX_SUBMODULE_JOBS=4`
- 所有 cargo/测试命令包 `(ulimit -v 8388608; ...)`
- 禁止全量 corpus、全量 fourslash、全量 lib 测试:批量验证只允许 assigned_cases.txt 清单内用例

## 禁改清单
- 仓库根 corpus_results.csv / corpus_skips.csv / skip_baseline.txt / AGENTS.md / tools/:只读
- 禁止运行 tools/corpus_csv_export.py(全量与双表导出是主 agent 职责)
- 禁改 tests/corpus/testdata/(用例与参考基线是作弊面)、tests/corpus/submodule_compiler.rs、tests/corpus/common/
- 禁用 KnownDiffs/skip 手段消错误;清单外问题不处理不提交

## 任务背景
仓库是 TypeScript 编译器的 Rust 移植(从 typescript-go 移植),目标对齐 Go oracle。
语料测试比对本地基线(crates/tsox/tests/corpus/baselines/local/)与参考基线(crates/tsox/tests/corpus/testdata/baselines/reference/)。

你的工作目录(git worktree,分支 {{branch}},基于 {{base-sha}}):{{worktree}}
所有工作在此 worktree 内,禁止动主仓 /home/cqh/workspace/ts2rust-port 的分支。
Go oracle 源码(语义权威,只读):/home/cqh/workspace/typescript-go。行为不确定必须对照 Go,禁止以「让用例通过」偏离 Go 语义。

## 前任交接(如有)
{{worktree}} 内可能有前任未提交修改。处置流程:
1. `git -C {{worktree}} status --short` 与 `git diff` 评估前任改动
2. `git stash push`(仅跟踪文件)→ `git rebase ts2rust-port`(分支基线已前移)→ `git stash pop`
3. pop 冲突:结合 {{merged-commits-hint}} 的意图语义合并;无法判断的段落以保留 HEAD 侧为默认,把疑问记 progress_notes.md
4. 前任明显错误/越界的改动直接丢弃(git checkout -- <file>),在 progress_notes.md 记一笔
5. 主仓根目录的 corpus_results.csv 如被前任覆写,先 `git checkout -- corpus_results.csv` 恢复

## 任务清单(只修这 {{n}} 例,禁止自行扩充、禁止自由修复)
{{case-list}}

验收线:{{n}} 例全部 PASS。该簇共 {{total}} 例(完整清单 worktree 根 assigned_cases.txt,前 {{n}} 行即代表例)。
共同症状:{{symptom}}
{{prior-context}}

## 工作流
1. cd {{worktree}}
2. 增量构建(约几分钟):(ulimit -v 8388608; CARGO_BUILD_JOBS=3 cargo build --release --test corpus)
3. 单例调试(PASS/FAIL + 基线 diff):tools/corpus_one.sh <用例名>
4. 按 diff 定位,对照 Go 实现找语义差异,修复
5. 每个根因修复后:重建 → 代表例复验 → 立即 commit
6. 预算富余时簇批量(仅清单内,限时 10 分钟):
   cd crates/tsox && (ulimit -v 8388608; TSOX_SUBMODULE_JOBS=4 TSOX_SUBMODULE_SUITE=compiler TSOX_SUBMODULE_LIMIT=0 TSOX_SUBMODULE_CASES_FILE=../../assigned_cases.txt cargo test --release --test corpus submodule_compiler -- --nocapture 2>&1 | tail -20)

## 代码规范(AGENTS.md 摘要,违者返工)
- 禁止解释性注释;why 写进 commit message;死代码直接删
- 单 .rs 源文件(非测试)>300 行按职责拆子模块,pub use 保持接口;不加内联测试

## 汇报格式(报告后立即结束)
- 每代表例 PASS/FAIL;根因分析(Go vs Rust,引用双方源码位置);修改文件与 commit hash 清单;预期转绿数;未完成事项交接(progress_notes.md)
```

## 主 agent 编排注意

- 并发 ≤4;每 agent 预算默认 {{budget-min}} 分钟(复杂簇可到 {{budget-z-min}},不超过)
- 派发 prompt 必须内联上述全文(含时限/提交纪律/禁改清单),不引用本文件路径代替
- 收集后统一 rebase 合并、全量、双表导出;agent 报告的转绿数仅作参考,以主 agent 实测为准
- 打回标准:代表例未全绿、出现清单外改动、或 branch 上无 commit
