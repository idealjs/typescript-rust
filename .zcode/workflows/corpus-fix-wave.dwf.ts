/* zcode-workflow
description: 语料修复飞轮单波 workflow：修复波并发修复（每个 fixer 完成即入队；并发上限由提交参数 max_concurrency
  控制，当前协议 8）+ 波内自适应串行验证（待验证队列>1 走 tools/wave_verify.sh --single 单例快检约 1
  分钟；无排队走全量含回归归因）+ 未过非阻塞退回原 agent 续修（无固定退回上限）。跨例回归与整体效果由主 agent 合并后最终全量统一暴露。依赖
  tools/wave_verify.sh / corpus_one.sh 与仓库根数据 CSV；波收尾整合在主 agent。
whenToUse: 按 AGENTS.md 语料修复飞轮分发修复波时使用。每波先编辑脚本内 WAVE/CASES/FLAKY 常量（选例来自最新
  corpus_results.csv 与 corpus_types_anchor.csv），再以 saved 按名运行并带
  max_concurrency（协议当前 8）；波中扩容/修脚本用 AmendWorkflow（注意：已有 live 验证后再 amend 会使已完成
  fixer 缓存失效重跑）。subagent 模型按协议用 GLM-5.3-Flash max。
*/
// 修复波 fix36：准备 → 修复波（20 例并发修复，max_concurrency=8 由提交参数控制，完成即入队）⇄ 串行验证（自适应：待验证>1 单例快检，无排队全量+回归归因）→ 汇总
// 执行链：tools/wave_verify.sh（自适应：待验证>1 走 --single 单例快检；无排队全量+五份 CSV 刷新+回归归因。
// 最终整体全量与跨例归因由主 agent 在合并后执行，不在本 workflow 内）
// 波收尾整合（cherry-pick 线性化 + 最终全量）在主 agent，不在本 workflow 内。
const WAVE = "fix36";
const MAIN = "/home/cqh/workspace/ts2rust-port";
const WTROOT = "/home/cqh/worktrees";
const VERIFY_SH = `${MAIN}/tools/wave_verify.sh`;
const BASE_REF = "ts2rust-port";

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
  { key: "compiler/errorOnInitializerInObjectTypeLiteralProperty.ts", stem: "errorOnInitializerInObjectTypeLiteralProperty", slot: "w1", family: "类型字面量属性初始化器诊断缺失（TS1247）", hint: "本地零输出；Go 发 TS1247 'A type literal property cannot have an initializer.'（2,19）。与 fix35 已修的 TS1246（interface 成员）同族不同检查：type literal 成员遍历路径。Go 侧 grep 'cannot have an initializer' 定位。" },
  { key: "compiler/newFunctionImplicitAny.ts", stem: "newFunctionImplicitAny", slot: "w2", family: "new 目标缺构造签名隐式 any 诊断缺失（TS7009）", hint: "本地零输出；Go 发 TS7009 ''new' expression, whose target lacks a construct signature, implicitly has an 'any' type.'（4,12）。" },
  { key: "compiler/declarationFileOverwriteError.ts", stem: "declarationFileOverwriteError", slot: "w3", family: "输出覆盖输入文件诊断缺失（TS5055）", hint: "本地零输出；Go 发 TS5055 'Cannot write file ... because it would overwrite input file.'（配置级无文件定位）。fix35/w10 已通 emitted_output_file_names 路径，本例缺诊断发射本身。" },
  { key: "compiler/recursiveInheritance.ts", stem: "recursiveInheritance", slot: "w4", family: "递归基型诊断缺失（TS2310）", hint: "本地零输出；Go 发 TS2310 'Type 'I5' recursively references itself as a base type.'（1,11）。同族 recursiveInheritanceGeneric 未分发。" },
  { key: "compiler/typeofEnum.ts", stem: "typeofEnum", slot: "w5", family: "enum 场景使用前未赋值诊断缺失（TS2454）", hint: "本地零输出；Go 发 TS2454 'Variable 'e1' is used before being assigned.'（7,1，enum 场景 definite assignment 检查）。" },
  { key: "compiler/gettersAndSettersAccessibility.ts", stem: "gettersAndSettersAccessibility", slot: "w6", family: "访问器可访问性诊断缺失（TS2808）", hint: "本地零输出；Go 发 TS2808 'A get accessor must be at least as accessible as its setter.'（2,14）。" },
  { key: "compiler/inheritedStringIndexersFromDifferentBaseTypes.ts", stem: "inheritedStringIndexersFromDifferentBaseTypes", slot: "w7", family: "多基类 string indexer 冲突诊断缺失（TS2430）", hint: "本地零输出；Go 发 TS2430 'Interface 'E' incorrectly extends interface ...'（13,11，多基类继承的 string indexer 不兼容）。" },
  { key: "compiler/isolatedModulesAmbientConstEnum.ts", stem: "isolatedModulesAmbientConstEnum", slot: "w8", family: "isolatedModules 下访问 ambient const enum 诊断缺失（TS2748）", hint: "本地零输出（多文件 file1.ts）；Go 发 TS2748 'Cannot access ambient const enums when 'isolatedModules' is enabled.'（file1.ts 2,16）。" },
  { key: "compiler/exportAsNamespaceConflict.ts", stem: "exportAsNamespaceConflict", slot: "w9", family: "UMD 导出命名空间循环别名诊断缺失（TS2303）", hint: "本地零输出（多文件 /a.d.ts）；Go 发 TS2303 'Circular definition of import alias 'N'.'（/a.d.ts 2,1，exportAsNamespace 冲突）。" },
  { key: "compiler/moduleResolution_relativeImportJsFile_noImplicitAny.ts", stem: "moduleResolution_relativeImportJsFile_noImplicitAny", slot: "w10", family: "相对导入 JS 无声明诊断缺失（TS7016）", hint: "本地零输出；Go 发 TS7016 'Could not find a declaration file for module './b'...'（/src/a.ts 1,20）。" },
  { key: "compiler/newAbstractInstance.ts", stem: "newAbstractInstance", slot: "w11", family: "abstract 实例化诊断选择（TS2351 vs TS2511）", hint: "Go 期望 TS2351 'This expression is not constructable.'（3,5），本地发 TS2511 'Cannot create an instance of an abstract class.'（3,1）：abstract 构造签名检查的先序/诊断选择。" },
  { key: "compiler/moduleAsBaseType.ts", stem: "moduleAsBaseType", slot: "w12", family: "命名空间作基型的值/型诊断选择（TS2708 vs TS2709）", hint: "Go 期望 TS2708 'Cannot use namespace 'M' as a value.'（2,17），本地发 TS2709 'as a type'（3,21）：命名空间作 extends 基型的诊断路径。" },
  { key: "compiler/multipleExportAssignmentsInAmbientDeclaration.ts", stem: "multipleExportAssignmentsInAmbientDeclaration", slot: "w13", family: "ambient 多 export 赋值重复标识符名（TS2300）", hint: "Go 期望 TS2300 duplicate 'export='（4,14），本地报 duplicate 'a'：ambient 模块多 export assignment 合并时的符号名选择。" },
  { key: "compiler/constEnumBadPropertyNames.ts", stem: "constEnumBadPropertyNames", slot: "w14", family: "const enum 非法属性名成员类型（TS2339 vs TS7015）", hint: "Go 期望 TS2339 'Property 'B' does not exist on type 'typeof E'.'（2,11），本地发 TS7015 隐式 any：const enum 非法属性名成员的类型求值。" },
  { key: "compiler/parseErrorInHeritageClause1.ts", stem: "parseErrorInHeritageClause1", slot: "w15", family: "heritage 子句解析错误恢复（TS2304 vs TS1127）", hint: "Go 在 (1,17) 发 TS2304 'Cannot find name 'A'.'，本地在 (1,19) 发 TS1127 'Invalid character.'：heritage 子句解析错误的恢复路径。" },
  { key: "compiler/indirectSelfReference.ts", stem: "indirectSelfReference", slot: "w16", family: "自引用基型报错节点选择（TS2506 位置差）", hint: "两侧同发 TS2506，但 Go 定位 (1,7) 本地 (1,1)：'a' is referenced directly or indirectly 的报错节点选择。" },
  { key: "compiler/classInheritence.ts", stem: "classInheritence", slot: "w17", family: "类自引用声明序列完整诊断集（TS2449 族）", hint: "首行两侧同 TS2449 'Class 'A' used before its declaration.'（1,17），后续差异见分片：类声明提升检查的完整诊断集合对齐。" },
  { key: "compiler/moduleResolution_automaticTypeDirectiveNames.ts", stem: "moduleResolution_automaticTypeDirectiveNames", slot: "w18", family: "automaticTypeDirectiveNames 模块解析过报（TS2451）", hint: "Go 期望零错误；本地在 /node_modules/@types/.a/index.d.ts(1,15) 过报 TS2451 redeclare block-scoped 'a'：automaticTypeDirectiveNames 与 @types 目录的模块解析/符号重复。" },
  { key: "compiler/mixedExports.ts", stem: "mixedExports", slot: "w19", family: "混合导出重载签名过报（TS2383）", hint: "Go 期望零错误；本地过报 TS2383 'Overload signatures must all be exported or non-exported.'（3,22）：mixed exports（export= + named）的重载导出判定。" },
  { key: "compiler/definiteAssignmentWithErrorStillStripped.ts", stem: "definiteAssignmentWithErrorStillStripped", slot: "w20", family: "verify_compiler_options 过报 removed 选项（TS5108）", hint: "Go 期望 TS1264（2,6）definite assignment；本地首行却是 TS5108 'Option 'alwaysStrict=false' has been removed.'——fix35/w10 接线 verify_compiler_options 后的过报噪声（Go 中 alwaysStrict 是合法三态项非 removed）。同因例 topLevelLambda4 未分发。" },
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
    `定位 Go/Rust 语义分歧，按 Go 修根因。验收 = 复跑 PASS（主 agent 执行，你禁测）。`,
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
    const commitCount = Number(cnt.stdout.trim());
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
    report({ case: c.key, state: "待验证" }, "wave-board");
    wake();
  }),
);
producers.then(
  () => { producersDone = true; wake(); },
  (e) => { producerError = e; producersDone = true; wake(); },
);

phase("串行验证：在 worktree 上跑校验（排队>1 单例快检，无排队全量）");
while (!producersDone || ready.length > 0 || pendingRetreats > 0) {
  // 自适应策略：待验证队列长度 >1（有排队积压）→ 单例快检压缩单轮时长；
  // 无排队（fixer 仍在修，验证器有余量）→ 全量验证并取波内回归归因
  const fast = ready.length > 1;
  const c = ready.shift();
  if (!c) { await waitNext(); continue; }
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  const attempt = retreatCounts.get(c.key) ?? 0;
  const tag = `${c.slot}-a${attempt}`;
  report({ case: c.key, state: "验证中" }, "wave-board");
  const run = await world.run(
    "bash",
    fast ? [VERIFY_SH, "--single", wt, c.key, tag] : [VERIFY_SH, wt, c.key, tag],
    { timeoutMs: fast ? 900_000 : 3_600_000 },
  );
  const verdict = parseVerdict(run.stdout);
  lastVerdict = verdict;
  const regressions = [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k));
  const flakyHits = [...verdict.newFails, ...verdict.newSkips].filter((k) => FLAKY.includes(k));
  const ok = verdict.status === "PASS" && regressions.length === 0;
  const fr = fixResults.get(c.key);
  log(`${c.slot} ${c.key} 第 ${attempt + 1} 次验证（${fast ? "单例快检" : "全量"}）：${verdict.status}，FAIL=${verdict.failCount}，anchor=${verdict.anchorCount}，修好=${verdict.fixed}，回归=${regressions.length}${flakyHits.length > 0 ? `（另剔抖动 ${flakyHits.length}）` : ""}`);
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
        report({ case: c.key, state: "待验证" }, "wave-board");
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
    phase("串行验证：在 worktree 上跑校验（排队>1 单例快检，无排队全量）");
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
  `- 已通过波内验证（自适应快检/全量）：${passed.length} 例；留队：${queued.length} 例（${queued.map((q) => q.caseKey).join("、") || "无"}）`,
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
    `修复波 ${WAVE}：${passed.length} 例通过波内验证（自适应：排队>1 单例快检、无排队全量含回归归因；目标例 PASS 才出队），` +
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
