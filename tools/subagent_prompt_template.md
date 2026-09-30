# corpus 修复 subagent 派发模板(v3)

> 供主 agent 编排语料修复循环时生成派发 prompt。占位符 `{{...}}` 由主 agent 填充。
> v2 相对 v1 的改动动机:上一波 8 agent 并行 3.4 小时后 ticket 过期,4/8 零提交被中断,
> 根因是 prompt 缺时间预算、缺退出契约、单一末端提交点、无 CPU 上限、未禁全量验证。
> v3 相对 v2:w34 压缩轮实证 30 分钟预算可行(9 agent 全部按时交付),总预算上限固化为
> 30 分钟,复杂簇不再上浮(深修拆多轮滚动+交接);构建耗时由主 agent 派发前预构建吸收。
> 记录见 decisions/ 下对应 ADR。

## 模板正文

```
# 任务:Rust 版 TypeScript 编译器语料修复(簇:{{cluster-id}})

## 时限与退出契约(最高优先级,先读)
- 总预算 {{budget-max}} 分钟。开始时记下时刻,每完成一步对照剩余时间。
- 剩 {{budget-rest-min}} 分钟时无论进行到哪:停止新根因,把已验证的修改提交,写 progress_notes.md 交接,输出报告,结束。
- 单个根因投入 {{budget-input-min}} 分钟仍未让任何清单内用例转绿:停,分析写进 progress_notes.md,换下一个根因或收尾。
- 验收线({{n}} 代表例全绿)达成且已提交:立即报告退出,不开新根因。簇内其余用例只在预算富余时顺带,不作为交付条件。
- 汇报后任务即结束。你的交付物 = 分支上的 commit + progress_notes.md + 最终报告,不是"把簇修完"。

## 提交纪律(防中断丢失)
- 每完成一个独立根因修复立即 git commit(不等整簇批量;验证由主 agent 统一执行,勿自行测试)。
- commit 只 add 你改的源码路径;assigned_cases.txt / progress_notes.md / 简报不入库。
- 中断随时可能发生:工作区长时间只有未提交修改 = 交付为零。

## 环境故障协议
- Bash 工具连续 3 次失败(适配器 shutting down、超时等):把现场写进 progress_notes.md,立即结束退出。等待重试累计不超过 5 分钟。
- 疑似死循环/OOM(进程被杀、输出不完整):受控探针测斜率、变体二分,不盲目重跑;定位超 15 分钟即按超时处理。

## 资源上限与绝对禁测(违者打回)
- **禁止执行任何测试、任何构建命令**(cargo test/build/check、跑用例二进制、tools/corpus_one.sh 一律禁止)——语料修复飞轮中 subagent 同样不允许测试、不需要验证结果,**所有测试(含单例验证)一律由主 agent 执行**;subagent 交付物 = 静态修复 commit + 函数变更表 + 交接笔记,验证状态由主 agent 收集时统一跑
- 探针式调试(eprintln 插桩 + 跑用例看输出)同样禁止,属主 agent 排障手段
- **禁碰内存限制设施**:测试入口的内存守卫(ensure_memory_limit / .init_array)、脚本中的 ulimit -v 4194304 / setrlimit、fourslash_shard.py 的 AS_LIMIT,一律不得修改、放宽或绕过;修复 diff 触及这些位置即打回
- 所有测试运行(全量/批量/单例)一律由主 agent 在 `(ulimit -v 4194304; ...)` 内执行,无内存限制的运行禁止存在
- **禁触主仓**:主仓路径(/home/cqh/workspace/ts2rust-port)对 subagent 绝对只读,禁止对其执行任何 git 写操作(checkout/reset/stash/merge/clean)

## 禁改清单
- 仓库根 corpus_results.csv / corpus_skips.csv / skip_baseline.txt / AGENTS.md / tools/:只读
- 禁止运行 tools/corpus_csv_export.py(全量与双表导出是主 agent 职责)
- 禁改 tests/corpus/testdata/(用例与参考基线是作弊面)、tests/corpus/submodule_compiler.rs、tests/corpus/common/
- 禁用 KnownDiffs/skip 手段消错误;清单外问题不处理不提交

## 任务背景
仓库是 TypeScript 编译器的 Rust 移植(从 typescript-go 移植),目标对齐 Go oracle。
语料测试比对本地基线(crates/tsox/tests/corpus/baselines/local/)与参考基线(crates/tsox/tests/corpus/testdata/baselines/reference/)。

你的工作目录(git worktree,分支 {{branch}},基于 {{base-sha}}):{{worktree}}
**开工第一步执行 `pwd`,输出必须以 {{worktree}} 开头**;此后所有 git 命令一律 `git -C {{worktree}} ...`。所有工作在此 worktree 内,禁止对主仓 /home/cqh/workspace/ts2rust-port 执行任何写操作(checkout/reset/stash/merge/clean 或直接改文件),禁止动其分支。
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

## 工作流(纯静态修复,零测试)
1. cd {{worktree}}
2. 按 assigned_cases.txt 与既定根因/修法配方,对照 Go oracle 源码直接修
3. 每完成一个独立根因修复立即 commit(不等验证——验证由主 agent 统一执行)
4. 自查纪律:修复逻辑以 Go 为准绳;语法严格自查(转义用 \u{..}、括号配平、matches! 守卫用 if);不引入探针/调试插桩

## 代码规范(AGENTS.md 摘要,违者返工)
- 禁止解释性注释;why 写进 commit message;死代码直接删
- 单 .rs 源文件(非测试)>300 行按职责拆子模块,pub use 保持接口;不加内联测试

## 汇报格式(报告后立即结束)
- 每代表例 PASS/FAIL;根因分析(Go vs Rust,引用双方源码位置);修改文件与 commit hash 清单;预期转绿数;未完成事项交接(progress_notes.md)
- **函数变更表**(必交,单独一节):每个 commit 列出修改/新增/删除的 Rust 函数名与所属文件,以及对齐的 Go 函数名(如有)。格式:

```
| commit | 文件 | Rust 函数(增/改/删) | 对齐的 Go 函数 | 用例效果 |
```

无函数级变更(纯数据/配置改动)也要显式写「无」。该表供主 agent 在轮次 FAIL 未下降时追溯改动面,缺表视为汇报不完整打回。
```

## r 轮构建修复派发模板要点(与语料模板并列,按轮型选用;r59 起固化)

r 轮(cargo check 错误清零)的 subagent 是纯文本修复,与语料轮 subagent 的关键差异:

- **派发 prompt 必须显式写明:禁止运行任何 cargo 命令、任何构建、任何测试(cargo check/build/test、rustc 一律禁止);构建与错误验证由飞轮主 agent 统一执行**。此条不可省略、不可只靠 AGENTS.md 推断(用户 2026-09-28 指示,源自 r59 派发词缺失该明示的教训)。
- 并发上限固化为 **6**(2026-09-28 r59 实证:账户级 1302 限流,8 并发两波均恰好 2 死 6 活,6 路长跑稳定;换模型不绕开账户限流。原 AGENTS.md 的 8-12 区间对本账户不可达)。
- 资源上限小节(CARGO_BUILD_JOBS=2 等)对 r 轮 subagent 不适用:他们不构建。
- prompt 必含:隔离 worktree 路径与分支名、权威清单文件路径(主 agent 给摘要并让 agent Read 清单全文)、Go oracle 路径、缺符号三选一(grep 等价接线/按 Go 补最小真实实现/保留错误记交接)、禁空壳消错、只改本片主 span 文件、每根因增量独立 commit、函数变更表汇报、30/6/10 预算契约、交接笔记文件名(不 commit)。
- 错误验证由主 agent 在合并后统一执行:`(ulimit -v 4194304; CARGO_BUILD_JOBS=8 cargo check --workspace --all-targets --message-format=json > check.json 2>check.stderr)` 再抽紧凑清单。

## 主 agent 编排注意

- 并发 ≤8;**每 agent 总预算上限 30 分钟**(固化,复杂簇不再上浮,深修拆多轮滚动+交接);`{{budget-max}}`/`{{budget-rest-min}}`/`{{budget-input-min}}` 填 30/6/10,除非主 agent 明确写更小值
- **测试全部归主 agent(用户 2026-09-29 最终指示)**:subagent 不跑任何测试不需验证;主 agent 收集各片后统一构建 + 单例/批量验证,agent 自报的验证结论一律不采信,以主 agent 实测为准。主 agent 需自建 release 测试二进制(预构建 subagent worktree 的环节随之取消)
- 派发 prompt 必须内联上述全文(含时限/提交纪律/禁改清单/函数变更表要求),不引用本文件路径代替全文
- 收集后统一 rebase 合并、全量、双表导出;agent 报告的转绿数仅作参考,以主 agent 实测为准
- 打回标准:代表例未全绿、出现清单外改动、branch 上无 commit、或汇报缺函数变更表
- **FAIL 未下降时**:轮次汇总报告必须列出各 subagent 的函数变更表(commit × 文件 × Rust 函数 × 对齐 Go 函数 × 用例效果),供人工审计改动面;未达验收线的 subagent(零转绿)同样要给全表,其改动是否入库由主 agent 在表上裁决
