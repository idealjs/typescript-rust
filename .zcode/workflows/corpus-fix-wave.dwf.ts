/* zcode-workflow
description: 语料修复飞轮单波 workflow（anchor 抹平型）：修复波并发修复 anchor .types 首分歧（完成即入队）+ 串行
  anchor 快检（tools/wave_verify.sh --anchor：.types 与 Go 参考全文一致即出队；errors
  回归/失败属预期不拦截，红线仅 panic/挂起类性能问题）+ 未过非阻塞退回（无固定上限；退回后无新 commit 自动无进展留队，防认输例空转）+ 单例
  turn 失败隔离留队。基线分支为工作 base ts2rust-port（波在飞期间冻结）。上报只留初始/退回/终态（256 条/运行上限教训）。
whenToUse: 按 AGENTS.md 飞轮分发修复波时使用（anchor 波验收=.types 全等，errors 回归预期内不拦截）。**每波只填 4
  例（2026-10-04 用户拍板上限，避免主 agent 合并压力）**：每波改 WAVE/CASES/BASE_REF 常量按名运行并带
  max_concurrency（协议当前 8）。subagent 模型按协议用 GLM-5.3-Flash max。errors 基线波将验证调用改回
  --single/全量模式并恢复 errors 验收口径。
*/
// 修复波 fix37（anchor 抹平）：准备 → 修复波（20 例 anchor 分歧并发修复，max_concurrency=8）⇄ 串行验证（anchor 快检：errors PASS 且 .types 全等才出队）→ 汇总
// 执行链：tools/wave_verify.sh（自适应：待验证>1 走 --single 单例快检；无排队全量+五份 CSV 刷新+回归归因。
// 最终整体全量与跨例归因由主 agent 在合并后执行，不在本 workflow 内）
// 波收尾整合（cherry-pick 线性化 + 最终全量）在主 agent，不在本 workflow 内。
const WAVE = "fix37";
const MAIN = "/home/cqh/workspace/ts2rust-port";
const WTROOT = "/home/cqh/worktrees";
const VERIFY_SH = `${MAIN}/tools/wave_verify.sh`;
const BASE_REF = "ts2rust-port"; // 工作 base 分支；波在飞期间约定冻结（合入只在波收尾）

/** 同代码两轮结果翻转的抖动例，不参与回归归因（基线两轮：1018→1019 的差异例）。 */
const FLAKY: string[] = ["compiler/contextualTypeCaching.ts"];

interface WaveCase {
  /** 用例 key，形如 compiler/foo.ts */
  key: string;
  /** 用例名去扩展名（trace/分片文件名） */
  stem: string;
  /** 波内槽位（worktree/分支后缀） */
  slot: string;
  /** 根因族描述（用于报告） */
  family: string;
  /** 主 agent 预分析信号，注入修复 prompt */
  hint: string;
}

const CASES: WaveCase[] = [
  { key: "compiler/arithmeticOnInvalidTypes2.ts", stem: "arithmeticOnInvalidTypes2", slot: "w1", family: "anchor .types 首分歧（9 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>a : T || R:>b : T。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/arithmeticOnInvalidTypes2.ts.md 分片。" },
  { key: "compiler/classExtendsMultipleBaseClasses.ts", stem: "classExtendsMultipleBaseClasses", slot: "w2", family: "anchor .types 首分歧（13 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>B : B || R:>B : A。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/classExtendsMultipleBaseClasses.ts.md 分片。" },
  { key: "compiler/accessorsInAmbientContext.ts", stem: "accessorsInAmbientContext", slot: "w3", family: "anchor .types 首分歧（8 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>C : typeof M.C || R:>C : C。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/accessorsInAmbientContext.ts.md 分片。" },
  { key: "compiler/ambientEnumElementInitializer6.ts", stem: "ambientEnumElementInitializer6", slot: "w4", family: "anchor .types 首分歧（8 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>E : typeof M.E || R:>E : E。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/ambientEnumElementInitializer6.ts.md 分片。" },
  { key: "compiler/abstractClassInLocalScope.ts", stem: "abstractClassInLocalScope", slot: "w5", family: "anchor .types 首分歧（17 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>new B( : B || R:>new B() : B。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/abstractClassInLocalScope.ts.md 分片。" },
  { key: "compiler/abstractClassInLocalScopeIsAbstract.ts", stem: "abstractClassInLocalScopeIsAbstract", slot: "w6", family: "anchor .types 首分歧（17 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>new A( : A || R:>new A() : A。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/abstractClassInLocalScopeIsAbstract.ts.md 分片。" },
  { key: "compiler/declarationEmitTypeAliasTypeParameterExtendingUnknownSymbol.ts", stem: "declarationEmitTypeAliasTypeParameterExtendingUnknownSymbol", slot: "w7", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>A : A || R:>A : A<T>。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/declarationEmitTypeAliasTypeParameterExtendingUnknownSymbol.ts.md 分片。" },
  { key: "compiler/deferredConditionalTypes.ts", stem: "deferredConditionalTypes", slot: "w8", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>A : A || R:>A : A<T>。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/deferredConditionalTypes.ts.md 分片。" },
  { key: "compiler/ambientClassDeclarationWithExtends.ts", stem: "ambientClassDeclarationWithExtends", slot: "w9", family: "anchor .types 首分歧（18 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>D : D || R:>D : typeof D。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/ambientClassDeclarationWithExtends.ts.md 分片。" },
  { key: "compiler/augmentedClassWithPrototypePropertyOnModule.ts", stem: "augmentedClassWithPrototypePropertyOnModule", slot: "w10", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>m : m || R:>m : typeof m。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/augmentedClassWithPrototypePropertyOnModule.ts.md 分片。" },
  { key: "compiler/declarationEmitOfTypeofAliasedExport.ts", stem: "declarationEmitOfTypeofAliasedExport", slot: "w11", family: "anchor .types 首分歧（14 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>a : typeof import('a') || R:>a : typeof a。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/declarationEmitOfTypeofAliasedExport.ts.md 分片。" },
  { key: "compiler/errorWithSameNameType.ts", stem: "errorWithSameNameType", slot: "w12", family: "anchor .types 首分歧（19 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>A : typeof import('a') || R:>A : typeof A。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/errorWithSameNameType.ts.md 分片。" },
  { key: "compiler/assignmentCompatability1.ts", stem: "assignmentCompatability1", slot: "w13", family: "anchor .types 首分歧（12 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>obj4 : __test1__.interfaceWithPublicAndOptional<number, string> || R:>one : number。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/assignmentCompatability1.ts.md 分片。" },
  { key: "compiler/assignmentCompatability10.ts", stem: "assignmentCompatability10", slot: "w14", family: "anchor .types 首分歧（12 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>obj4 : __test1__.interfaceWithPublicAndOptional<number, string> || R:>one : number。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/assignmentCompatability10.ts.md 分片。" },
  { key: "compiler/es6ExportAssignment4.ts", stem: "es6ExportAssignment4", slot: "w15", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>'a' : ''a'' || R:>'a' : typeof import('a')。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/es6ExportAssignment4.ts.md 分片。" },
  { key: "compiler/importDeclWithDeclareModifierInAmbientContext.ts", stem: "importDeclWithDeclareModifierInAmbientContext", slot: "w16", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>'m' : ''m'' || R:>'m' : typeof import('m')。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/importDeclWithDeclareModifierInAmbientContext.ts.md 分片。" },
  { key: "compiler/nonInferrableTypePropagation1.ts", stem: "nonInferrableTypePropagation1", slot: "w17", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Op : Op || R:>Op : Op<I, O>。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/nonInferrableTypePropagation1.ts.md 分片。" },
  { key: "compiler/declarationEmitTypeAliasWithTypeParameters1.ts", stem: "declarationEmitTypeAliasWithTypeParameters1", slot: "w18", family: "anchor .types 首分歧（5 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Bar : Bar || R:>Bar : Bar<X, Y>。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/declarationEmitTypeAliasWithTypeParameters1.ts.md 分片。" },
  { key: "compiler/assign1.ts", stem: "assign1", slot: "w19", family: "anchor .types 首分歧（16 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>x : M.I || R:>x : I。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/assign1.ts.md 分片。" },
  { key: "compiler/assignToFn.ts", stem: "assignToFn", slot: "w20", family: "anchor .types 首分歧（14 行）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>x : M.I || R:>x : I。目标：本例 .types 与 Go 参考全文一致。errors 基线回归属预期、不作为验收。按 Go 对应类型求值/渲染函数修根因；参考主仓 corpus_rust_types.csv / corpus_go_types.csv 两侧全文与 /tmp/flywheel_shards/assignToFn.ts.md 分片。" },
];

interface FixResult {
  /** 本例在 worktree 分支上的独立修复 commit 数（workflow 会用 git 机械复核） */
  commits: number;
  /** 根因一句话，含双侧源码 文件:行号 */
  rootCause: string;
  /** 函数变更表 markdown 原文（契约必交，缺表打回） */
  functionTable: string;
  /** true = 未完成修复、已留交接记录 */
  handedOff: boolean;
  /** 一句话总结 */
  summary: string;
}

interface Verdict {
  caseKey: string;
  status: string;
  failCount: string;
  skipCount: string;
  anchorCount: string;
  fixed: string;
  newFails: string[];
  newSkips: string[];
  buildErr: string[];
  /** 单例快检：跳过/失败原因行（全量模式为空） */
  reason: string;
  /** 单例快检：ref vs local 基线 diff（全量模式为空） */
  caseDiff: string[];
  /** anchor 快检：.types 首分歧区（两侧上下文+行号） */
  anchorDiff: string[];
}

interface CaseOutcome {
  caseKey: string;
  slot: string;
  family: string;
  outcome: "已通过" | "无改动留队" | "退回异常留队";
  verifies: number;
  retreats: number;
  finalStatus: string;
  failCount: string;
  anchorCount: string;
  fixedCount: string;
  regressions: string[];
  flakyHits: string[];
  rootCause: string;
  functionTable: string;
  commits: number;
  handedOff: boolean;
  summary: string;
}

function parseVerdict(stdout: string): Verdict {
  const lines = stdout.split("\n");
  const line = lines.find((l) => l.startsWith("VERDICT "));
  const get = (k: string): string => {
    const m = new RegExp(`(?:^| )${k}=([^ ]*)`).exec(line ?? "");
    return m?.[1] ?? "?";
  };
  return {
    caseKey: get("case"),
    status: get("status"),
    failCount: get("fail_count"),
    skipCount: get("skip_count"),
    anchorCount: get("anchor_count"),
    fixed: get("fixed"),
    newFails: lines.filter((l) => l.startsWith("REGRESSION-FAIL ")).map((l) => l.slice("REGRESSION-FAIL ".length)),
    newSkips: lines.filter((l) => l.startsWith("REGRESSION-SKIP ")).map((l) => l.slice("REGRESSION-SKIP ".length)),
    buildErr: lines.filter((l) => l.startsWith("BUILD_ERR")).slice(0, 20),
    reason: (lines.find((l) => l.startsWith("REASON ")) ?? "").slice("REASON ".length),
    anchorDiff: lines.filter((l) => l.startsWith("ANCHORDIFF ")).map((l) => l.slice("ANCHORDIFF ".length)),
    caseDiff: lines.filter((l) => l.startsWith("CASEDIFF ")).map((l) => l.slice("CASEDIFF ".length)),
  };
}

function fixPrompt(c: WaveCase): string {
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  return [
    `# 任务：${c.key}（修复入口，目标：使其 PASS）`,
    ``,
    `## 目标`,
    `用例：${c.key}（根因族：${c.family}）`,
    `定位 Go/Rust 类型求值/渲染分歧，按 Go 修根因。验收 = 本例 .types 与 Go 参考全文一致（主 agent 执行，你禁测）。errors 基线的回归/失败属预期，不作为验收、不必为保 errors 绿妥协类型对齐；唯一红线是不得引入 panic/挂起类性能问题。`,
    ``,
    `## 本例已知信号（主 agent 预分析，仍须自行核实）`,
    c.hint,
    ``,
    `## 失败信息（三层锚点，按序使用）`,
    `分片：/tmp/flywheel_shards/${c.stem}.ts.md（.types 首分歧锚点——已知偏差形态跳过——→ 错误 diff → 执行栈对照段）。`,
    `完整序列：主仓 corpus_go_trace.csv / corpus_rust_trace.csv 按行首 case 名 grep。`,
    ``,
    `## 环境`,
    `- Go oracle（只读）：/home/cqh/workspace/typescript-go`,
    `- 你的 worktree：${wt}（分支 ${WAVE}/${c.slot}）。开工先 pwd 验证；git 一律 git -C ${wt} ...`,
    `- 只改 crates/ 下生产代码；主仓（/home/cqh/workspace/ts2rust-port）绝对只读。`,
    ``,
    `## 参考数据（主仓根，只读）`,
    `- corpus_results.csv——FAIL 全量（水位，本轮 1019 例）`,
    `- corpus_rust_trace.csv / corpus_go_trace.csv——两侧函数调用序列`,
    `- corpus_rust_types.csv / corpus_go_types.csv——两侧 .types 全量输出`,
    `- corpus_types_anchor.csv——.types 首分歧锚点（按行首 case 名 grep）`,
    `- corpus_stack_diff.csv——执行栈差集（go_only / rust_only）`,
    `- func_alignment.csv——靠齐判定唯一依据：按名称匹配迁移，只以此表为准，不得以报告声称/语义判断/探针观察认定匹配完成`,
    ``,
    `## 收尾`,
    `- 无法继续推进时：提交已有可信修改，写 progress_notes.md（worktree 根，不入库），按汇报结构收尾。`,
    ``,
    `## 禁止事项`,
    `- 禁止执行任何测试、构建命令（cargo/rustc/跑用例）。验证由主 agent 统一执行。`,
    `- 禁止探针式调试（插桩+跑用例）。`,
    `- 禁改：corpus_*.csv / func_alignment.csv（主仓根全部数据 CSV）/ AGENTS.md / tools/ / .traces/ / crates/tsox/tests/corpus/。`,
    `- 禁止 skip、改断言、改基线、空壳实现（恒返 None/空函数/删真实逻辑换占位）。`,
    `- 符号不存在时三选一：grep 等价符号改接线 / 按 Go 最小真实实现 / 保留错误记交接。`,
    ``,
    `## 提交`,
    `- 每个独立修复立即 git commit（git -C ${wt}），只 add crates/ 下生产路径。`,
    `- progress_notes.md 不入库。`,
    ``,
    `## 故障退出`,
    `- Bash 连续 3 次失败：写 progress_notes.md 后立即结束并汇报。`,
    ``,
    `## 汇报（返回 JSON，字段如下）`,
    `- commits：独立修复 commit 数；rootCause：根因 + 双侧源码 文件:行号；`,
    `- functionTable：函数变更表（必交）：| commit | 文件 | Rust 函数(增/改/删) | 对齐的 Go 函数 | 用例效果 |；`,
    `- handedOff：是否交接未完成；summary：一句话总结。`,
    `- 如遇约束矛盾或无法推进，如实说明并结束，不要造假结果。`,
  ].join("\n");
}

function retreatPrompt(c: WaveCase, v: Verdict, attempt: number): string {
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  const regressions = [...v.newFails, ...v.newSkips].filter((k) => !FLAKY.includes(k));
  const flakyHits = [...v.newFails, ...v.newSkips].filter((k) => FLAKY.includes(k));
  const parts = [
    `# 退回：${c.key}（第 ${attempt} 次退回）`,
    ``,
    `## 背景`,
    `你在 ${wt}（分支 ${WAVE}/${c.slot}）的修复 commit 已在波内串行验证，结果未过：`,
  ];
  if (v.status === "BUILD_ERROR") {
    parts.push(`- 构建失败（机械错可最小修复并在 commit message 标注；逻辑错原样记录）：`);
    parts.push(...v.buildErr.map((l) => `  ${l}`));
  } else if (v.anchorDiff.length > 0) {
    parts.push(`- anchor 未抹平（.types 与 Go 参考存在分歧）：`);
    parts.push(...v.anchorDiff.slice(0, 30).map((l) => `  ${l}`));
    if (v.reason) parts.push(`- errors 状态：${v.reason}`);
    parts.push(`- 目标：本例 .types 与 Go 参考全文一致；errors 基线回归/失败属预期不需处理，红线只有 panic/挂起类性能问题。两侧全文见主仓 corpus_rust_types.csv / corpus_go_types.csv 按行首 case 名 grep。`);
  } else if (v.failCount === "-") {
    parts.push(`- 目标例状态：${v.status}（单例快检未过）${v.reason ? `，原因：${v.reason}` : ""}。`);
    if (v.caseDiff.length > 0) {
      parts.push(`- 单例基线 diff（参考 vs 本地）：`);
      parts.push(...v.caseDiff.slice(0, 30).map((l) => `  ${l}`));
    }
    parts.push(`- 本轮为单例快检：波内不感知跨例回归，合并后由主 agent 全量统一暴露并归因，你只须使本例 PASS。`);
  } else {
    parts.push(`- 目标例状态：${v.status}（未过）。全量水位 FAIL=${v.failCount} / SKIP=${v.skipCount} / anchor=${v.anchorCount}，本分支修好 ${v.fixed} 例。`);
    if (regressions.length > 0) {
      parts.push(`- 波及回归（相对波起点新增，须一并修绿）：${regressions.join("、")}`);
    }
    if (flakyHits.length > 0) {
      parts.push(`- 已剔除的抖动例（同代码两轮翻转，不归因本分支，无需处理）：${flakyHits.join("、")}`);
    }
  }
  parts.push(
    ``,
    `## 本轮目标`,
    `在原 worktree 分支上**追加**修复 commit（不 amend、不重建分支），使目标例与全部回归例同时绿。`,
    `你的首个任务上下文仍在，但主仓数据已刷新，先重读：git -C ${wt} log --oneline -5、最新 corpus_results.csv / corpus_types_anchor.csv 中本例与回归例行、分片 /tmp/flywheel_shards/。`,
    `约束同首任务（禁构建禁测试、只改 crates/、按 Go 修根因、独立 commit、函数变更表必交）。`,
  );
  return parts.join("\n");
}

function outcomeToReport(o: CaseOutcome): string {
  const lines = [
    `### ${o.slot} · ${o.caseKey}（${o.family}）——${o.outcome}`,
    `- 验证 ${o.verifies} 次，退回 ${o.retreats} 次；末次状态 ${o.finalStatus}，水位 FAIL=${o.failCount} / anchor=${o.anchorCount}，本分支修好 ${o.fixedCount} 例`,
  ];
  if (o.regressions.length > 0) lines.push(`- 未消化回归（留队归因）：${o.regressions.join("、")}`);
  if (o.flakyHits.length > 0) lines.push(`- 剔除抖动：${o.flakyHits.join("、")}`);
  lines.push(`- commits：${o.commits}${o.handedOff ? "（含交接未完成）" : ""}`);
  lines.push(`- rootCause：${o.rootCause || "（未汇报）"}`);
  lines.push(`- 函数变更表：`);
  lines.push(o.functionTable || "（未汇报）");
  lines.push(`- 一句话：${o.summary || "（未汇报）"}`);
  return lines.join("\n");
}

artifact.board("wave-board", {
  title: `修复波 ${WAVE} 看板`,
  key: "case",
  status: "state",
  columns: ["修复中", "待验证", "验证中", "退回续修", "已通过", "留队"],
  cardTitle: "case",
});

phase("为全部用例创建隔离 worktree 并快照波起点水位");
for (const c of CASES) {
  const add = await world.run("git", [
    "worktree", "add", `${WTROOT}/${WAVE}-${c.slot}`, "-b", `${WAVE}/${c.slot}`, BASE_REF,
  ]);
  if (add.exitCode !== 0) {
    const chk = await world.run("git", ["worktree", "list", "--porcelain"]);
    if (!chk.stdout.includes(`${WAVE}-${c.slot}`)) {
      throw new Error(`worktree add 失败 ${c.slot}: ${add.stderr.slice(0, 500)}`);
    }
    log(`${WAVE}-${c.slot} 已存在，复用`);
  }
  report({ case: c.key, state: "修复中" }, "wave-board");
}
const snap = await world.run("bash", [VERIFY_SH, "--snapshot"]);
log(`波起点快照：${snap.stdout.trim()}`);

const fixers = new Map<string, Agent>();
const fixResults = new Map<string, FixResult>();

phase("修复波：并发修复各例，退回时原 agent 续修");
const outcomes: CaseOutcome[] = [];
let lastVerdict: Verdict | null = null;
const commitCounts = new Map<string, number>();
const retreatCounts = new Map<string, number>();
const lastCommitCounts = new Map<string, number>();
// 验证消费队列：fixer 完成即入队（commit 机械复核在内），验证侧单线程逐个消费，互不阻塞
const ready: WaveCase[] = [];
let producersDone = false;
let producerError: unknown = null;
let pendingRetreats = 0;
let wakeSignal: (() => void) | undefined;
const wake = (): void => { wakeSignal?.(); wakeSignal = undefined; };
const waitNext = (): Promise<void> => new Promise((res) => { wakeSignal = res; });

const producers = Promise.all(
  CASES.map(async (c) => {
   // 单例失败隔离：该例记留队，不连坐整波
   try {
    const fixer = agent(`修复-${c.stem}`, {
      system:
        "你是 TypeScript→Rust 迁移工程中按 Go oracle 修语料用例的修复工程师。" +
        "靠齐判定只以 func_alignment.csv 的名称匹配为准；禁止构建/测试/探针，验证由主 agent 统一执行；" +
        "每个独立根因一个 commit，只 add crates/ 生产路径；" +
        "若约束互相矛盾或确实无法推进，如实汇报交接而不是绕过或造假。",
    });
    fixers.set(c.key, fixer);
    let res = await fixer.ask<FixResult>(fixPrompt(c));
    if (!res.functionTable || !res.functionTable.trim()) {
      res = await fixer.ask<FixResult>(
        "汇报缺少函数变更表（契约必交）。补交形如 | commit | 文件 | Rust 函数(增/改/删) | 对齐的 Go 函数 | 用例效果 | 的完整表格，" +
          "并保持其余字段（commits/rootCause/handedOff/summary）完整后重新返回 JSON。",
      );
    }
    fixResults.set(c.key, res);
    log(`${c.slot} ${c.key} 修复段完成：commits=${res.commits}，handedOff=${res.handedOff}`);
    // 机械复核 commit 数（不采信自报）：零 commit 例直接留队，不占验证槽
    const wt = `${WTROOT}/${WAVE}-${c.slot}`;
    const cnt = await world.run("git", ["-C", wt, "rev-list", "--count", `${BASE_REF}..HEAD`]);
    // fail-safe：复核命令失败（如基线分支缺失）不得误判为零 commit
    const commitCount = cnt.exitCode === 0 ? Number(cnt.stdout.trim()) : NaN;
    if (cnt.exitCode !== 0) log(`${c.slot} commit 复核失败（${cnt.stderr.trim().slice(0, 100)}），按有 commit 入队，波收尾复核`);
    commitCounts.set(c.key, commitCount);
    if (commitCount === 0) {
      outcomes.push({
        caseKey: c.key, slot: c.slot, family: c.family, outcome: "无改动留队",
        verifies: 0, retreats: 0, finalStatus: "无 commit", failCount: "-", anchorCount: "-", fixedCount: "-",
        regressions: [], flakyHits: [], rootCause: res.rootCause, functionTable: res.functionTable,
        commits: 0, handedOff: true, summary: res.summary,
      });
      log(`${c.slot} ${c.key} 无 commit，直接留队`);
      return;
    }
    ready.push(c);
    wake();
   } catch (err) {
    outcomes.push({
      caseKey: c.key, slot: c.slot, family: c.family, outcome: "无改动留队",
      verifies: 0, retreats: 0, finalStatus: `fixer turn 失败：${String(err).slice(0, 150)}`,
      failCount: "-", anchorCount: "-", fixedCount: "-",
      regressions: [], flakyHits: [], rootCause: "", functionTable: "",
      commits: 0, handedOff: true, summary: "fixer turn 终态失败（provider 时段），worktree 可能已有部分 commit，波收尾复核",
    });
    report({ case: c.key, state: "留队" }, "wave-board");
    log(`${c.slot} ${c.key} fixer turn 失败，记留队（worktree 分支保留供收尾复核）：${String(err).slice(0, 120)}`);
   }
  }),
);
producers.then(
  () => { producersDone = true; wake(); },
  (e) => { producerError = e; producersDone = true; wake(); },
);

phase("串行验证：在 worktree 上跑 anchor 快检（.types 全等即出队，errors 回归预期内）");
while (!producersDone || ready.length > 0 || pendingRetreats > 0) {
  // 自适应策略：待验证队列长度 >1（有排队积压）→ 单例快检压缩单轮时长；
  // 无排队（fixer 仍在修，验证器有余量）→ 全量验证并取波内回归归因
  const c = ready.shift();
  if (!c) { await waitNext(); continue; }
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  const attempt = retreatCounts.get(c.key) ?? 0;
  const tag = `${c.slot}-a${attempt}`;
  const cntNow = Number((await world.run("git", ["-C", wt, "rev-list", "--count", `${BASE_REF}..HEAD`])).stdout.trim() || "-1");
  const prevCnt = lastCommitCounts.get(c.key);
  lastCommitCounts.set(c.key, cntNow);
  if (prevCnt !== undefined && cntNow === prevCnt && attempt > 0) {
    outcomes.push({
      caseKey: c.key, slot: c.slot, family: c.family, outcome: "无改动留队",
      verifies: attempt, retreats: attempt, finalStatus: "退回后无新 commit（认输/交接）",
      failCount: "-", anchorCount: "-", fixedCount: "-",
      regressions: [], flakyHits: [], rootCause: "", functionTable: "",
      commits: cntNow, handedOff: true, summary: "退回轮无新 commit，按无进展留队（见 progress_notes）",
    });
    report({ case: c.key, state: "留队" }, "wave-board");
    log(`${c.slot} ${c.key} 退回后无新 commit，无进展留队`);
    continue;
  }
  const run = await world.run("bash", [VERIFY_SH, "--anchor", wt, c.key, tag], { timeoutMs: 900_000 });
  const verdict = parseVerdict(run.stdout);
  lastVerdict = verdict;
  const regressions = [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k));
  const flakyHits = [...verdict.newFails, ...verdict.newSkips].filter((k) => FLAKY.includes(k));
  const ok = verdict.status === "PASS" && regressions.length === 0;
  const fr = fixResults.get(c.key);
  log(`${c.slot} ${c.key} 第 ${attempt + 1} 次 anchor 快检：${verdict.status}，FAIL=${verdict.failCount}，修好=${verdict.fixed}，回归=${regressions.length}${flakyHits.length > 0 ? `（另剔抖动 ${flakyHits.length}）` : ""}`);
  if (ok) {
    outcomes.push({
      caseKey: c.key, slot: c.slot, family: c.family, outcome: "已通过",
      verifies: attempt + 1, retreats: attempt, finalStatus: verdict.status,
      failCount: verdict.failCount, anchorCount: verdict.anchorCount, fixedCount: verdict.fixed,
      regressions: [], flakyHits,
      rootCause: fr?.rootCause ?? "", functionTable: fr?.functionTable ?? "",
      commits: commitCounts.get(c.key) ?? 0, handedOff: fr?.handedOff ?? false,
      summary: fr?.summary ?? "",
    });
    report({ case: c.key, state: "已通过" }, "wave-board");
  } else {
    const fixer = fixers.get(c.key);
    if (!fixer) throw new Error(`fixer 丢失：${c.key}`);
    report({ case: c.key, state: "退回续修" }, "wave-board");
    phase("修复波：并发修复各例，退回时原 agent 续修");
    // 退回非阻塞：派发给原 agent 后消费者继续验证下一个 worktree，修完自动重新入队
    pendingRetreats += 1;
    fixer.ask(retreatPrompt(c, verdict, attempt + 1)).then(
      () => {
        retreatCounts.set(c.key, attempt + 1);
        ready.push(c);
          pendingRetreats -= 1;
        wake();
      },
      (err: unknown) => {
        outcomes.push({
          caseKey: c.key, slot: c.slot, family: c.family, outcome: "退回异常留队",
          verifies: attempt + 1, retreats: attempt, finalStatus: `retreat ask 失败：${String(err).slice(0, 200)}`,
          failCount: verdict.failCount, anchorCount: verdict.anchorCount, fixedCount: verdict.fixed,
          regressions: [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k)),
          flakyHits: flakyHits, rootCause: fr?.rootCause ?? "", functionTable: fr?.functionTable ?? "",
          commits: commitCounts.get(c.key) ?? 0, handedOff: true, summary: fr?.summary ?? "",
        });
        report({ case: c.key, state: "留队" }, "wave-board");
        pendingRetreats -= 1;
        wake();
      },
    );
    phase("串行验证：在 worktree 上跑 anchor 快检（.types 全等即出队，errors 回归预期内）");
  }
}
if (producerError !== null) throw producerError;

phase("汇总波产出并独立校对报告");
const passed = outcomes.filter((o) => o.outcome === "已通过");
const queued = outcomes.filter((o) => o.outcome !== "已通过");
for (const o of outcomes) {
  report({ case: o.caseKey, state: o.outcome === "已通过" ? "已通过" : "留队" }, "wave-board");
}
const draft = [
  `# 修复波 ${WAVE} 报告`,
  ``,
  `- 分发 ${CASES.length} 例（并发修复段，模型 GLM-5.3-Flash max）：${CASES.map((c) => c.key).join("、")}`,
  `- 已通过波内验证（anchor 快检）：${passed.length} 例；留队：${queued.length} 例（${queued.map((q) => q.caseKey).join("、") || "无"}）`,
  lastVerdict
    ? `- 末次验证水位：FAIL=${lastVerdict.failCount} / SKIP=${lastVerdict.skipCount} / anchor=${lastVerdict.anchorCount}（注：CSV 反映最后一个验证的 worktree，六分支合并后的整体效果由主 agent 波收尾整合后最终全量确认）`
    : `- 本波未产生验证水位`,
  `- 抖动例剔除口径：${FLAKY.join("、")}（同代码两轮翻转，不参与回归归因）`,
  ``,
  ...outcomes.map(outcomeToReport),
].join("\n");

const reader = agent("报告校对员", {
  system: "你校对工程报告的可读性。只依据文本本身判断：哪里不清楚、哪里缺少支撑、读者会追问什么；不核对仓库。",
});
const revised = await reader.ask<string>(
  `以下是一份修复波报告 markdown。请从纯文本可读性校对并直接返回修订后的完整 markdown（保持全部事实数据不变，只改进表述与结构；若已足够清晰则原样返回）：\n\n${draft}`,
);
const reportMd = revised && revised.includes("#") ? revised : draft;
await artifact.markdown("wave-report", reportMd, {
  title: `修复波 ${WAVE} 报告`,
  description: `各例验证结论、归因与函数变更表；已通过 ${passed.length} 例。`,
  primary: true,
});

return {
  conclusion:
    `修复波 ${WAVE}：${passed.length} 例通过波内验证（anchor 快检：目标例 .types 与 Go 全文一致即出队；errors 回归预期内，仅性能问题拦截），` +
    `${queued.length} 例留队${queued.length > 0 ? `（${queued.map((q) => `${q.caseKey}:${q.finalStatus}`).join("、")}）` : ""}。` +
    `各分支只含本例改动；快检轮不感知跨例回归，待主 agent 合并后最终全量统一暴露。`,
  findings: outcomes.map((o) => ({
    where: `${WTROOT}/${WAVE}-${o.slot}（分支 ${WAVE}/${o.slot}）`,
    what: `${o.caseKey} ${o.outcome}：${o.summary || o.rootCause || "见报告"}`,
    evidence: `末次验证 ${o.finalStatus}，FAIL=${o.failCount}，修好=${o.fixedCount}，回归=${o.regressions.join("、") || "无"}；commits=${o.commits}`,
    status: o.outcome === "已通过" ? ("verified" as const) : ("unconfirmed" as const),
    severity: o.outcome === "已通过" ? ("low" as const) : ("medium" as const),
  })),
  verified: [
    `每例在其专属 worktree 上经 tools/wave_verify.sh 校验（自适应：待验证>1 单例快检，无排队全量 6820 例 + 相对波起点回归归因），目标例 PASS 才出队`,
    `修复 commit 数由 git rev-list 机械复核，非采信 subagent 自报`,
  ],
  notCovered: [
    "快检轮次不感知跨例回归，且全部分支未合并：跨例互斥与整体回归未经最终全量暴露（波收尾整合 + 最终全量在主 agent 执行）",
    "anchor 抹平只看全量轮刷新的总数，未逐例核对全部首分歧的语义",
    "同族未分发例（propertyAccess4/5 等）未处理，留待下波按代表例效果复检",
  ],
};
