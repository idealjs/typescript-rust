> 版本：v1.0 · 2026-09-20

# corpus 家族修复的多 subagent 工作流

主 agent 与 subagent 的分工编排：主 agent 负责定位、清单、验证与合并，subagent 负责清单内用例的修复。任务边界的决策与两个否决项的完整理由见 [decisions/subagent-corpus-repair.md](../decisions/subagent-corpus-repair.md)。

## 流程

```mermaid
flowchart TD
    A[主 agent: 全量跑 corpus 得到 runlog] --> B[tools/corpus_families.py 按错误码家族聚类]
    B --> C[每家族建 worktree 与分支 corpusN/family]
    C --> D[主 agent: 提取家族当前剩余失败集写入 assigned_cases.txt]
    D --> E[按波次派发 subagent, 每波 4 个]
    E --> F[subagent: 修清单内用例, 家族批量自验, 小步提交, 写 progress_notes]
    F --> G[主 agent: 收回各 worktree 提交]
    G --> H[主 agent: 全量跑 corpus 与基线对比]
    H --> I{有回归或未修完?}
    I -- 有回归 --> J[回归用例写入对应 assigned_cases.txt 重新派发]
    J --> H
    I -- 否 --> K[逐家族合并主分支, 汇总报告]
```

## 分工

| 角色 | 职责 | 禁止 |
|---|---|---|
| 主 agent | 全量验证、失败聚类、清单生成、派发、回归裁决、合并、汇总 | 逐例修复 |
| subagent | 修 assigned_cases.txt 内用例、单例调试、家族批量自验、小步提交、交接日志 | 全量 corpus、全量 fourslash、修清单外用例、merge 或 rebase |

## 派发 prompt 要素

- worktree 绝对路径与分支名，禁止触碰其他 worktree 与主仓
- assigned_cases.txt 为唯一工作清单，修完即止
- 长命令后台化与轮询纪律：预计超过 3 分钟的命令禁止阻塞等待
- 资源上限：cargo 构建 -j 4、家族批量 TSOX_SUBMODULE_JOBS=2、lib 测试 RUST_TEST_THREADS=1，全部包 ulimit -v 8388608
- 提交纪律：验证单元（5 到 20 例）家族批量净绿即提交，message 风格 fix(corpus): 家族 根因 手法，N 例转绿
- 交接日志 progress_notes.md 的读写约定
- 代码规范引用 AGENTS.md：禁解释性注释、300 行拆分、不新增内联测试

## 波次与资源

- 每波最多 4 个 subagent，波内家族按剩余失败数从大到小优先
- 并行负载合计不超过 24 核的 70%，避免内存带宽瓶颈；主 agent 的全量验证在波次间执行，不与波内并行叠加
- subagent 被配额终止时，从 progress_notes.md 与分支提交恢复现场，重新派发即可，已提交进度不丢失
