# corpus 修复循环轮次状态（2026-09-21，主 agent 会话）

> 本文件是主 agent 中断后的接续锚点。数据以仓库根 corpus_results.csv 为准。

## wave3 主 agent 直修战果（无 subagent，逐根因提交）

- P0 回归修复 665860d4e9：conditionalTypeRelaxingConstraintAssignability（条件分支推断裸类型参数后置，NakedTypeVariable 优先级）
- P1 a68558db12：destructuringAssignmentWithDefault（联合归约不吞并元组/数组源 + 联合成员级 nia 抑制）
- P2 36d90e1212：typeArgInference（rest 位参数型双重下钻去除）

## 未清账诊断（下轮直接从这里开工，均已实证到代码位）

1. **A4 nonInferrableTypePropagation3（根因已锁定，架构级）**：签名实例化对 rest 位类型参数产生新克隆，破坏推断同一性。证据链：探针 ITV found=false，签名侧 Args=(tid 506831, sym 313183) ≠ 追踪 (506835, sym 313188)/(506834, sym 313187)；同签名内 R 保持同一性（found=true）而 Args（rest 位）被克隆 → 克隆点在签名实例化的 rest 参数处理。修复方向：对齐 Go 的类型参数同一性（substitute 时 identity 映射，不克隆）。最小复现 /tmp/mini_a4b.ts（mk((age: number) => 1)）。
2. **C11 recursiveLetConst**：TDZ 自引用（`let [x1] = x1 + 1`）本地解析落具体类型（number/{}），Go 落 errorType 使解构不报 TS2488。已试并回退：二元运算 error 传播（假设未命中，操作数本身不是 error）。病灶在 CFA 环解析回退（getFlowTypeOfReference 环路时的类型），不在二元运算。
3. **A1 contextualTypingOfOptionalMembers**：TS7006 缺报（可选属性上下文签名未达箭头参数），未开工。
4. **skipopts 残留 10 例**：unusedTypeParameters6/7/8、unusedLocalsAndParameters×2 等，聚类与入手点在 wt4-skipopts/progress_notes.md；stretch（7027/7028/7030 选项族 85 例）未动。
5. **抖动 ×2**：noImplicitThisFunctions（lib TS2502 循环误报随机触发，Rust HashMap RandomState 每进程随机种子；候选修法：类型解析序敏感的 HashMap 换 BTreeMap/索引序）、conditionalTypeDoesntSpinForever（同族）。

## wave1/wave2 累计战果（背景）

FAIL 1604 → wave1 收官 1472 → wave2 收官 1394 → wave3 进行中（P0 回归已清 + 2 例转绿）。

- wave0 挽救：parser-1005(78 绿)、assign-2345(12 绿)、assign-2502(7 绿)；csv 导出脚本轮转 bug 修复
- wave1（并发 4）：reg2502(+2)、jsany-7022(整簇 10/10)、members-2339(7/32)、skipopts nounused(37→136 PASS)
- wave2（并发 4）：regfix12(7/12)、assign-2322(5 代表例+簇内净绿)、skipopts-residue(→151 PASS)、typeval-2307(5/13)

## 环境备注

- 全量运行 lib/lsp/dbg1 的内存分配失败为既有环境问题（与多轮日志一致），非移植回归
- corpus 二进制必须用 `ls -t target/release/deps/corpus-* | grep -v '\.d$' | head -1` 选最新（glob 会命中旧二进制）
- TDBG 探针法：env 门控 + /tmp 文件追加 + TSOX_PROBE_PHASES=1，交付前必须清除（本轮已清）
- 抖动例判定法：同一提交连跑 3 次

## 流程资产（沉淀）

- `tools/subagent_prompt_template.md` v2：时限/熔断/提交纪律/故障协议/禁改清单，派发全文内联
- AGENTS.md「派发并发与退出契约」+ decisions/subagent-corpus-repair.md v1.2
- 回归判定必须 key 级（LC_ALL=C sort + comm），seconds 噪声会污染行级 diff

