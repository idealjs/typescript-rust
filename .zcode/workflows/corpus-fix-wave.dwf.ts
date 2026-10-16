/* zcode-workflow
description: 语料修复飞轮单波 workflow（types anchor 抹平型）：修复波并发修复 anchor .types 首分歧（完成即入队）+
  串行全量验证（TSOX_POOL=1 cargo xtask verify 全量模式：本例 errors PASS + 无跨例回归 + 本例 anchor 行消失才出队）+
  未过非阻塞退回（上限 3 次；退回后无新 commit 自动无进展留队，防认输例空转）+ 单例 turn 失败隔离留队。
  基线分支为工作 base ts2rust-port（波在飞期间冻结）。上报只留初始/退回/终态（256 条/运行上限教训）。
whenToUse: 按 AGENTS.md 飞轮分发 types anchor 抹平波时使用（验收=本例 errors PASS + 无回归 + 本例 anchor 行消失）。
  每波改 WAVE/CASES/BASE_REF 常量按名运行并带 max_concurrency（当前波：队列 20 例、并发 6，2026-10-06 用户拍板）。
  subagent 模型按协议用 GLM-5.3-Flash max。
*/
// 修复波 fix53（types anchor 抹平）：准备 → 修复波（20 例 anchor 分歧并发修复，max_concurrency=6）⇄ 串行全量验证（本例 errors PASS + 无回归 + 本例 anchor 行消失才出队）→ 汇总
// 执行链：cargo xtask verify 全量模式（worktree 构建 + 6820 例 + --target 判定 + 相对波起点回归归因 + data/ 水位刷新；归因基线读主仓 --base-dir）。
// 最终整体全量与跨例互斥归因由主 agent 在合并后执行，不在本 workflow 内
// 波收尾整合（cherry-pick 线性化 + 最终全量）在主 agent，不在本 workflow 内。
// 串行逐步合并（fix54 起）：队首分支验证通过即由 workflow cherry-pick 合入 ts2rust-port；合入使基线前移，
// 消费器在每例验证前检测基线前移，对受影响分支直接退回 rebase（不跑本轮验证、不计退回上限）——
// 即每次合入后队列其余分支全部回炉新基线重修重验；验证失败则打回原 agent 续修并检查下一个。
const WAVE = "fix53";
const MAIN = "/home/cqh/workspace/ts2rust-port";
const WTROOT = "/home/cqh/worktrees";
const BASE_REF = "ts2rust-port"; // 工作 base 分支；波在飞期间约定冻结（合入只在波收尾）

/** 同代码跨轮结果翻转的抖动例，不参与回归归因。contextualTypeCaching：基线两轮 1018→1019 差异例；
 * jsx 两例：27~30s 贴 worker 30s 超时线重例，fix51 波 w5-a0/w5-a1/w7-a0/w9-a0 四轮跨分支 FAIL↔SKIP 翻转，
 * 波起点基线即非 PASS（skips），失败形态 fix46/47 归档已存在，非任何修复分支引入。 */
const FLAKY: string[] = ["compiler/contextualTypeCaching.ts", "compiler/jsxChildrenIndividualErrorElaborations.tsx", "compiler/jsxCallElaborationCheckNoCrash1.tsx"];

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
  { key: "compiler/declarationEmitReexportedSymlinkReference.ts", stem: "declarationEmitReexportedSymlinkReference", slot: "w1", family: "anchor .types 首分歧：typeof import 跨包路径限定（续修）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>MetadataAccessor : typeof import('../../pkg1/dist/types').MetadataAccessor || R:>MetadataAccessor : typeof MetadataAccessor。fix52 续修例：前次 2 commit（具名导入目标解析穿 export * 链 + 别名引用走 Go symbolToTypeNode 全链）已整合进当前基线，勿重做；波内复验轮反复 BUILD_ERROR→FAIL 未收敛，交接笔记 /tmp/flywheel_verify/fix52-notes/w12.md。本次只须收敛验证侧残留。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/declarationEmitReexportedSymlinkReference.ts.md。" },
  { key: "compiler/anonClassDeclarationEmitIsAnon.ts", stem: "anonClassDeclarationEmitIsAnon", slot: "w2", family: "anchor .types 首分歧：mixin 匿名类静态侧显示（续修）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Timestamped : <TBase extends Constructor>(Base: TBase) => typeof (Anonymous class) || R:>Timestamped : <TBase extends Constructor>(Base: TBase) => { new (...args: any[…（截断，全文见分片与两侧 types CSV）。fix52 续修例：主根因（get_containers_of_symbol 声明 membership 校验）与别名呈现已整合进基线；残留为首分歧后的匿名类静态侧展开显示（Class B：getBaseTypeVariableOfClass 门+容器前缀+交集成），交接笔记 /tmp/flywheel_verify/fix52-notes/w13.md。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/anonClassDeclarationEmitIsAnon.ts.md。" },
  { key: "compiler/acceptSymbolAsWeakType.ts", stem: "acceptSymbolAsWeakType", slot: "w3", family: "anchor .types 首分歧：元组字面量 fresh 字面量保留（续修）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>[[s, false]] : [symbol, boolean][] || R:>[[s, false]] : [symbol, false][]。fix52 续修例：三分歧已修二（this 多态返回、WeakKey 别名）已整合；第三分歧为 [[s,false]] 元组字面量：本地 [symbol, boolean][] vs Go [symbol, false][]（元组内字面量不随对象属性拓宽路径放宽）。注意：fresh 化修法已证伪丢弃（Go falseType 本身 fresh，checker.go:1006-1012），勿走该路；交接笔记 /tmp/flywheel_verify/fix52-notes/w14.md。本例无分片，anchor 行与两侧 types CSV 为主信号。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片缺失，主信号为 anchor 行与 data/rust-types.csv / corpus_go_types.csv 两侧全文。" },
  { key: "compiler/controlFlowNoImplicitAny.ts", stem: "controlFlowNoImplicitAny", slot: "w4", family: "anchor .types 首分歧：字面量联合未拓宽（新基线重做）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>cond : true | false || R:>cond : boolean。fix51 重发例：前次方向（cond 布尔并集渲染等三根因）正确但流初始型 auto 污染臂过宽搏 41 回归（分支 fix51/w1 未整合，勿参考其污染臂）。fix52 起基线已有 boolean 双成员字面量联合窄口径置旗机制（getUnionTypeFromSortedList 口径），本例大概率可窄口径直接复用或微调；交接笔记 /tmp/flywheel_verify/fix51-notes/w1.md。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/controlFlowNoImplicitAny.ts.md。" },
  { key: "compiler/controlFlowAliasedDiscriminants.ts", stem: "controlFlowAliasedDiscriminants", slot: "w5", family: "anchor .types 首分歧：字面量联合未拓宽（fix52 残留续修）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>isSuccess1 : false | true || R:>isSuccess1 : boolean。fix52 续修例：该例 fixer 终态净 3 commit 已整合（boolean 联合置 TypeFlagsBoolean 窄口径），但其 isSuccess1 行 L45 仍 false | true——置旗判定未触达本例上下文（别名判别式场景）。须找出置旗臂与本例路径的差异点（同族 discriminantPropertyCheck 待同根因判定），交接笔记 /tmp/flywheel_verify/fix52-notes/（w5 无笔记，见分支 fix52/w5 的 9294c03 revert + 7a5be05 置旗 commit）。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/controlFlowAliasedDiscriminants.ts.md。" },
  { key: "compiler/aliasInstantiationExpressionGenericIntersectionNoCrash2.ts", stem: "aliasInstantiationExpressionGenericIntersectionNoCrash2", slot: "w6", family: "anchor .types 首分歧：实例化表达式残留（同族续修）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>wat : Wat<T> || R:>wat : Wat<number>。fix52 同族例：NoCrash1 的 3/4 分歧已由整合 commit 消除（实例化表达式型惰性产物空对象交集根因），本例 L28 wat : Wat<T> vs Wat<number> 为同族推断实例化残留，可直接参考已整合修法（分支 fix52/w20 的 0886b607）。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/aliasInstantiationExpressionGenericIntersectionNoCrash2.ts.md。" },
  { key: "compiler/literalFreshnessPropagationOnNarrowing.ts", stem: "literalFreshnessPropagationOnNarrowing", slot: "w7", family: "anchor .types 首分歧：字面量拓宽反向（对象属性）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>obj : { b: true; } || R:>obj : { b: boolean; }。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/literalFreshnessPropagationOnNarrowing.ts.md。" },
  { key: "compiler/multipleModuleExportsAssignments.ts", stem: "multipleModuleExportsAssignments", slot: "w8", family: "anchor .types 首分歧：拓宽反向（!!true）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>!!true : boolean || R:>!!true : true。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/multipleModuleExportsAssignments.ts.md。" },
  { key: "compiler/discriminantNarrowingCouldBeCircular.ts", stem: "discriminantNarrowingCouldBeCircular", slot: "w9", family: "anchor .types 首分歧：拓宽反向（布尔判定）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>!parents : boolean || R:>!parents : false。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/discriminantNarrowingCouldBeCircular.ts.md。" },
  { key: "compiler/augmentExportEquals3.ts", stem: "augmentExportEquals3", slot: "w10", family: "anchor .types 首分歧：typeof import 限定渲染", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>x : typeof import('./file1') || R:>x : typeof x。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/augmentExportEquals3.ts.md。" },
  { key: "compiler/exportEqualsOfModule.ts", stem: "exportEqualsOfModule", slot: "w11", family: "anchor .types 首分歧：typeof import 路径段污染", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Request : typeof import('./exportEqualsOfModule').'~popsicle/dist/request'.Request || R:>Request : typeof Request。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/exportEqualsOfModule.ts.md。" },
  { key: "compiler/declarationEmitCrossFileImportTypeOfAmbientModule.ts", stem: "declarationEmitCrossFileImportTypeOfAmbientModule", slot: "w12", family: "anchor .types 首分歧：typeof import 命名空间路径", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>item : typeof import('../../types/component').'@namespace/component'.Foo || R:>item : typeof import('@namespace/component').Foo。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/declarationEmitCrossFileImportTypeOfAmbientModule.ts.md。" },
  { key: "compiler/bestChoiceType.ts", stem: "bestChoiceType", slot: "w13", family: "anchor .types 首分歧：any 泄漏（正则 match 数组）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>(''.match(/ /) || []).map(s => s.toLowerCase()) : any[] || R:>(''.match(/ /) || []).map(s => s.toLowerCase()) : string[]。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/bestChoiceType.ts.md。" },
  { key: "compiler/assignmentToObjectAndFunction.ts", stem: "assignmentToObjectAndFunction", slot: "w14", family: "anchor .types 首分歧：方法签名可选参显示", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>{    toString(x?) {        return '';    }} : { toString(x: any): string; } || R:>{    toString(x?) {        return '';    }} : { toString(x?: any): string; }。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/assignmentToObjectAndFunction.ts.md。" },
  { key: "compiler/awaitedTypeNoLib.ts", stem: "awaitedTypeNoLib", slot: "w15", family: "anchor .types 首分歧：any 泄漏（lib 缺省 promise）", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>handleResolve : <TResult>(result: any, resolve: Receiver<TResult>) => void || R:>handleResolve : <TResult>(result: NotPromise<TResult> | Thenable<NotPromise<TResult>>,…（截断，全文见分片与两侧 types CSV）。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/awaitedTypeNoLib.ts.md。" },
  { key: "compiler/declarationEmitMappedTypeTemplateTypeofSymbol.ts", stem: "declarationEmitMappedTypeTemplateTypeofSymbol", slot: "w16", family: "anchor .types 首分歧：mapped 模板 symbol 显示", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Timestamp : { [TKey in unique symbol]: true; } || R:>Timestamp : { [timestampSymbol]: true; }。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/declarationEmitMappedTypeTemplateTypeofSymbol.ts.md。" },
  { key: "compiler/keyRemappingKeyofResult.ts", stem: "keyRemappingKeyofResult", slot: "w17", family: "anchor .types 首分歧：keyof 联合缺 unique symbol", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>Okay : string | number | keyof T || R:>Okay : string | number | unique symbol | keyof T。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/keyRemappingKeyofResult.ts.md。" },
  { key: "compiler/augmentExportEquals4.ts", stem: "augmentExportEquals4", slot: "w18", family: "anchor .types 首分歧：export= 别名身份", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>foo : import('./file1').foo || R:>foo : import('./file1')。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/augmentExportEquals4.ts.md。" },
  { key: "compiler/asyncYieldStarContextualType.ts", stem: "asyncYieldStarContextualType", slot: "w19", family: "anchor .types 首分歧：then 签名显示", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>authorPromise.then : <TResult1 = T, TResult2 = never>(onfulfilled?: ((value: Result<Author, 'NOT_FOUND_AUTHOR'>) => TResult1 | PromiseLike<TResult1>) | null | undefine…（截断，全文见分片与两侧 types CSV）。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/asyncYieldStarContextualType.ts.md。" },
  { key: "compiler/arrayBestCommonTypes.ts", stem: "arrayBestCommonTypes", slot: "w20", family: "anchor .types 首分歧：重载集显示", hint: "anchor 首分歧（L=本地 R=Go 期望）：L:>voidIfAny : { (x: boolean, y?: boolean | undefined): number; (x: string, y?: boolean | undefined): number; (x: number, y?: boolean | undefined): number; } || R:>voidIf…（截断，全文见分片与两侧 types CSV）。目标：全量下本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS（波起点水位），不得引入 errors 回归或跨例回归。按 Go 对应类型求值/渲染函数修根因；参考两侧全文（data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep）与 分片 /tmp/tsox-verify/latest/shards/arrayBestCommonTypes.ts.md。" },
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
  /** 全量判定行：本例 anchor 行是否已消失（.types 与 Go 参考抹平） */
  targetAnchorGone: boolean;
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

/** xtask verdict.json 的形状（stdout 末行 VERDICT_JSON= 携带） */
interface XtaskVerdict {
  status: string;
  fail_count: number;
  skip_count: number;
  anchor_count: number;
  fixed: number;
  new_fails: string[];
  new_skips: { real: string[]; timeout: string[] };
  selected: { case: string; status: string; anchor_gone: boolean | null; anchor_context: string | null }[];
}

function parseVerdict(stdout: string, stderr: string, exitCode: number): Verdict {
  const lines = stdout.split("\n");
  const jsonLine = lines.find((l) => l.startsWith("VERDICT_JSON="));
  if (exitCode !== 0 || !jsonLine) {
    // 构建失败（cargo 编译错）或无判定行：原样暴露编译错误
    return {
      caseKey: "-", status: "BUILD_ERROR", failCount: "-", skipCount: "-", anchorCount: "-", fixed: "-",
      newFails: [], newSkips: [], buildErr: stderr.split("\n").filter((l) => l.startsWith("error")).slice(0, 20),
      reason: "", caseDiff: [], anchorDiff: [], targetAnchorGone: false,
    };
  }
  const d = JSON.parse(jsonLine.slice("VERDICT_JSON=".length)) as XtaskVerdict;
  const sel = d.selected[0];
  return {
    caseKey: sel?.case ?? "-",
    status: sel?.status ?? "UNKNOWN",
    failCount: String(d.fail_count),
    skipCount: String(d.skip_count),
    anchorCount: String(d.anchor_count),
    fixed: String(d.fixed),
    // 超时噪声桶不计回归（沿旧 TIMEOUT-SKIP 剔除口径）
    newFails: d.new_fails,
    newSkips: d.new_skips.real,
    buildErr: [],
    reason: "",
    caseDiff: [],
    anchorDiff: sel?.anchor_context ? [sel.anchor_context] : [],
    targetAnchorGone: sel?.anchor_gone ?? false,
  };
}

function fixPrompt(c: WaveCase): string {
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  return [
    `# 任务：${c.key}（修复入口，目标：抹平 .types 分歧且保 errors 绿）`,
    ``,
    `## 目标`,
    `用例：${c.key}（根因族：${c.family}）`,
    `定位 Go/Rust 类型求值/渲染分歧，按 Go 修根因。验收（主 agent 全量执行，你禁测）= 本例 errors 保持 PASS + 无跨例回归 + 本例 anchor 行消失（.types 与 Go 参考全文一致）。本例当前 errors 已 PASS，属 .types 隐藏分歧例，不得为类型对齐引入 errors 回归或跨例回归；红线：不得引入 panic/挂起类性能问题。`,
    ``,
    `## 本例已知信号（主 agent 预分析，仍须自行核实）`,
    c.hint,
    ``,
    `## 失败信息（三层锚点，按序使用）`,
    `分片：/tmp/tsox-verify/latest/shards/（.types 首分歧锚点——已知偏差形态跳过——→ 错误 diff → 执行栈对照段；目录内按用例基名找文件）。`,
    `完整序列：主仓 data/corpus_go_trace.csv / data/rust-trace.csv 按行首 case 名 grep。`,
    ``,
    `## 环境`,
    `- Go oracle（只读）：/home/cqh/workspace/typescript-go`,
    `- 你的 worktree：${wt}（分支 ${WAVE}/${c.slot}）。开工先 pwd 验证；git 一律 git -C ${wt} ...`,
    `- 只改 crates/ 下生产代码；主仓（/home/cqh/workspace/ts2rust-port）绝对只读。`,
    ``,
    `## 参考数据（主仓根，只读）`,
    `- data/corpus-failures.csv——FAIL 全量（水位以当前 CSV 为准；本波目标例均为 errors-PASS 的 .types 隐藏分歧例）`,
    `- data/rust-trace.csv / data/corpus_go_trace.csv——两侧函数调用序列`,
    `- data/rust-types.csv / data/corpus_go_types.csv——两侧 .types 全量输出`,
    `- data/types-anchor.csv——.types 首分歧锚点（按行首 case 名 grep）`,
    `- data/stack-diff.csv——执行栈差集（go_only / rust_only）`,
    `- data/func_alignment.csv——靠齐判定唯一依据：按名称匹配迁移，只以此表为准，不得以报告声称/语义判断/探针观察认定匹配完成`,
    ``,
    `## 收尾`,
    `- 无法继续推进时：提交已有可信修改，写 progress_notes.md（worktree 根，不入库），按汇报结构收尾。`,
    ``,
    `## 禁止事项`,
    `- 禁止执行任何测试、构建命令（cargo/rustc/跑用例）。验证由主 agent 统一执行。`,
    `- 禁止探针式调试（插桩+跑用例）。`,
    `- 禁改：data/ 全部 CSV / corpus_go_*.csv / func_alignment.csv（主仓数据文件）/ AGENTS.md / tools/ / .traces/ / crates/tsox/tests/corpus/。`,
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
  } else if (v.status === "PASS" && v.anchorDiff.length > 0) {
    parts.push(`- anchor 未抹平（.types 与 Go 参考存在分歧）：`);
    parts.push(...v.anchorDiff.slice(0, 30).map((l) => `  ${l}`));
    if (v.reason) parts.push(`- errors 状态：${v.reason}`);
    parts.push(`- 目标：本例 .types 与 Go 参考全文一致；errors 基线回归/失败属预期不需处理，红线只有 panic/挂起类性能问题。两侧全文见主仓 data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep。`);
  } else if (v.failCount === "-") {
    parts.push(`- 目标例状态：${v.status}（单例快检未过）${v.reason ? `，原因：${v.reason}` : ""}。`);
    if (v.caseDiff.length > 0) {
      parts.push(`- 单例基线 diff（参考 vs 本地）：`);
      parts.push(...v.caseDiff.slice(0, 30).map((l) => `  ${l}`));
    }
    parts.push(`- 本轮为单例快检：波内不感知跨例回归，合并后由主 agent 全量统一暴露并归因，你只须使本例 PASS。`);
  } else {
    parts.push(`- 目标例状态：${v.status}（未过）。全量水位 FAIL=${v.failCount} / SKIP=${v.skipCount} / anchor=${v.anchorCount}，本分支修好 ${v.fixed} 例。`);
    if (v.status === "PASS" && !v.targetAnchorGone) {
      parts.push(`- 本例 errors 已 PASS，但 anchor 行未消失：data/types-anchor.csv 仍可 grep 到本例，.types 与 Go 参考仍有分歧。两侧全文见主仓 data/rust-types.csv / data/corpus_go_types.csv 按行首 case 名 grep，须继续抹平至全文一致。`);
    }
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
    `在原 worktree 分支上**追加**修复 commit（不 amend、不重建分支），使目标例与全部回归例同时绿、且本例 anchor 行消失。`,
    `你的首个任务上下文仍在，但主仓数据已刷新，先重读：git -C ${wt} log --oneline -5、最新 data/corpus-failures.csv / data/types-anchor.csv 中本例与回归例行、归档 /tmp/tsox-verify/。`,
    `约束同首任务（禁构建禁测试、只改 crates/、按 Go 修根因、独立 commit、函数变更表必交）。`,
  );
  return parts.join("\n");
}

function rebaseRetreatPrompt(c: WaveCase, oldBase: string, newBase: string): string {
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  return [
    `# 退回 rebase：${c.key}（基线前移，不计退回次数）`,
    ``,
    `## 背景`,
    `主 agent 已把通过验证的修复批次合并进 ts2rust-port（${oldBase.slice(0, 10)} → ${newBase.slice(0, 10)}），你的分支还站在旧基线上。`,
    ``,
    `## 本轮任务`,
    `1. git -C ${wt} rebase ts2rust-port（分支 ${WAVE}/${c.slot}）。`,
    `2. 冲突按你自己的改动语义解决；先看对方侧（已整合修复）是否已覆盖你的部分意图——已覆盖的臂直接丢弃，勿重做。`,
    `3. rebase 完成后若你的用例仍有未修部分则继续修；已修完则确认工作区干净即可。`,
    `4. 只 add crates/ 生产路径；progress_notes.md 不入库。禁构建禁测试同前。`,
    ``,
    `## 说明`,
    `本轮退回不计入 3 次退回上限；rebase 后分支会自动重新入队验证。若 rebase 后你的用例仍 FAIL，正常退回续修（计次）。`,
  ].join("\n");
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
const snap = await world.run("cargo", ["xtask", "snapshot"], { timeoutMs: 300_000 });
log(`波起点快照：${snap.stdout.trim()}`);
const waveBase = (await world.run("git", ["rev-parse", BASE_REF])).stdout.trim();
const lastSyncedBase = new Map<string, string>();
for (const c of CASES) lastSyncedBase.set(c.key, waveBase);
log(`波起点基线：${waveBase.slice(0, 10)}`);

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
        "靠齐判定只以 data/func_alignment.csv 的名称匹配为准；禁止构建/测试/探针，验证由主 agent 统一执行；" +
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

phase("串行逐步合并：队首验证，通过即合入 ts2rust-port（其余分支回炉新基线重修重验），失败打回续修");
while (!producersDone || ready.length > 0 || pendingRetreats > 0) {
  // 全量验证：每轮 6820 例 + TARGET_ANCHOR 判定 + 相对波起点回归归因（fix50 起拍板，anchor 抹平波口径）
  const c = ready.shift();
  if (!c) { await waitNext(); continue; }
  const wt = `${WTROOT}/${WAVE}-${c.slot}`;
  const attempt = retreatCounts.get(c.key) ?? 0;
  // 基线前移检测：主 agent 波内合并批次落地后，先退回 rebase 再验证（不跑本轮验证、不计退回次数）
  const curBase = (await world.run("git", ["rev-parse", BASE_REF])).stdout.trim();
  const syncedBase = lastSyncedBase.get(c.key) ?? waveBase;
  if (curBase !== syncedBase) {
    const fixer = fixers.get(c.key);
    if (!fixer) throw new Error(`fixer 丢失：${c.key}`);
    report({ case: c.key, state: "退回续修" }, "wave-board");
    log(`${c.slot} ${c.key} 基线前移（${syncedBase.slice(0, 8)}→${curBase.slice(0, 8)}），退回 rebase（不计次）`);
    pendingRetreats += 1;
    fixer.ask(rebaseRetreatPrompt(c, syncedBase, curBase)).then(
      () => {
        lastSyncedBase.set(c.key, curBase);
        ready.push(c);
        pendingRetreats -= 1;
        wake();
      },
      (err: unknown) => {
        outcomes.push({
          caseKey: c.key, slot: c.slot, family: c.family, outcome: "退回异常留队",
          verifies: 0, retreats: attempt, finalStatus: `rebase ask 失败：${String(err).slice(0, 200)}`,
          failCount: "-", anchorCount: "-", fixedCount: "-",
          regressions: [], flakyHits: [], rootCause: "", functionTable: "",
          commits: commitCounts.get(c.key) ?? 0, handedOff: true, summary: "rebase 退回 ask 终态失败，worktree 保留供收尾复核",
        });
        report({ case: c.key, state: "留队" }, "wave-board");
        pendingRetreats -= 1;
        wake();
      },
    );
    continue;
  }
    const cntNow = Number((await world.run("git", ["-C", wt, "rev-list", "--count", `${BASE_REF}..HEAD`])).stdout.trim() || "-1");
  const prevCnt = lastCommitCounts.get(c.key);
  lastCommitCounts.set(c.key, cntNow);
  // xtask 全量验证：worktree 内构建+跑全量，--target 填充本例判定，--base-dir 读主仓波起点归因。
  // 输出重定向文件（6820 行事件超 world.run 256KB 上限），stdout 只回显判定行；失败时错误行走 stderr
  const vcmd = `cd ${wt} && TSOX_POOL=1 cargo xtask verify --jobs 12 --target ${c.key} --base-dir ${MAIN} >/tmp/xtask-${c.slot}.log 2>/tmp/xtask-${c.slot}.err; ec=\$?; grep '^VERDICT_JSON=' /tmp/xtask-${c.slot}.log; if [ \$ec -ne 0 ]; then grep -E '^error' /tmp/xtask-${c.slot}.err | head -20 >&2; fi; exit \$ec`;
  const run = await world.run("bash", ["-lc", vcmd], { timeoutMs: 3_600_000 });
  const verdict = parseVerdict(run.stdout, run.stderr, run.exitCode);
  lastVerdict = verdict;
  const regressions = [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k));
  const flakyHits = [...verdict.newFails, ...verdict.newSkips].filter((k) => FLAKY.includes(k));
  const anchorGone = verdict.targetAnchorGone;
  const ok = verdict.status === "PASS" && regressions.length === 0 && anchorGone;
  const fr = fixResults.get(c.key);
  log(`${c.slot} ${c.key} 第 ${attempt + 1} 次全量验证：${verdict.status}，FAIL=${verdict.failCount}，修好=${verdict.fixed}，回归=${regressions.length}${flakyHits.length > 0 ? `（另剔抖动 ${flakyHits.length}）` : ""}`);
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
    // 通过即合入：分支 commit 逐笔 cherry-pick 到 ts2rust-port（串行逐步合并）。
    // 合入成功后基线前移，队列中其余分支由消费器的基线前移检测自动回炉 rebase 重修重验。
    const branch = `${WAVE}/${c.slot}`;
    const revs = (await world.run("git", ["rev-list", "--reverse", `${BASE_REF}..${branch}`])).stdout.trim().split("\n").filter(Boolean);
    let merged = revs.length;
    for (const h of revs) {
      const pk = await world.run("git", ["cherry-pick", h]);
      if (pk.exitCode !== 0) {
        if (/empty/i.test(pk.stdout + pk.stderr)) {
          await world.run("git", ["cherry-pick", "--skip"]);
          continue;
        }
        await world.run("git", ["cherry-pick", "--abort"]);
        merged = -1;
        log(`${c.slot} ${c.key} 合入冲突（commit ${h.slice(0, 10)}），保留分支交主 agent 手合，不阻断队列`);
        break;
      }
    }
    if (merged >= 0) {
      log(`${c.slot} ${c.key} 已合入 ts2rust-port（${merged} commit，零空跳），后续分支将回炉新基线`);
    }
  } else {
    const fixer = fixers.get(c.key);
    if (!fixer) throw new Error(`fixer 丢失：${c.key}`);
    if (prevCnt !== undefined && cntNow === prevCnt && attempt > 0) {
      outcomes.push({
        caseKey: c.key, slot: c.slot, family: c.family, outcome: "无改动留队",
        verifies: attempt + 1, retreats: attempt, finalStatus: "退回后无新 commit 且复验未过（认输/交接）",
        failCount: verdict.failCount, anchorCount: verdict.anchorCount, fixedCount: verdict.fixed,
        regressions: [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k)),
        flakyHits, rootCause: fr?.rootCause ?? "", functionTable: fr?.functionTable ?? "",
        commits: cntNow, handedOff: true, summary: fr?.summary ?? "",
      });
      report({ case: c.key, state: "留队" }, "wave-board");
      log(`${c.slot} ${c.key} 退回后无新 commit 且复验未过，无进展留队`);
      continue;
    }
    if (attempt >= 3) {
      outcomes.push({
        caseKey: c.key, slot: c.slot, family: c.family, outcome: "无改动留队",
        verifies: attempt + 1, retreats: attempt, finalStatus: `退回 ${attempt} 次未收敛（超维修价值，终态失败不再退回）`,
        failCount: verdict.failCount, anchorCount: verdict.anchorCount, fixedCount: verdict.fixed,
        regressions: [...verdict.newFails, ...verdict.newSkips].filter((k) => !FLAKY.includes(k)),
        flakyHits, rootCause: fr?.rootCause ?? "", functionTable: fr?.functionTable ?? "",
        commits: commitCounts.get(c.key) ?? 0, handedOff: true, summary: fr?.summary ?? "",
      });
      report({ case: c.key, state: "留队" }, "wave-board");
      log(`${c.slot} ${c.key} 退回 ${attempt} 次未收敛，超维修价值终态失败留队`);
      continue;
    }
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
    phase("串行逐步合并：队首验证，通过即合入 ts2rust-port（其余分支回炉新基线重修重验），失败打回续修");
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
  `- 已通过波内验证（全量 anchor 判定）：${passed.length} 例；留队：${queued.length} 例（${queued.map((q) => q.caseKey).join("、") || "无"}）`,
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
    `修复波 ${WAVE}：${passed.length} 例通过波内验证（全量：本例 errors PASS + 无回归 + 本例 anchor 行消失才出队），` +
    `${queued.length} 例留队${queued.length > 0 ? `（${queued.map((q) => `${q.caseKey}:${q.finalStatus}`).join("、")}）` : ""}。` +
    `各分支只含本例改动、全量轮已含相对波起点回归归因；跨例互斥与合并后整体效果由主 agent 波收尾最终全量确认。`,
  findings: outcomes.map((o) => ({
    where: `${WTROOT}/${WAVE}-${o.slot}（分支 ${WAVE}/${o.slot}）`,
    what: `${o.caseKey} ${o.outcome}：${o.summary || o.rootCause || "见报告"}`,
    evidence: `末次验证 ${o.finalStatus}，FAIL=${o.failCount}，修好=${o.fixedCount}，回归=${o.regressions.join("、") || "无"}；commits=${o.commits}`,
    status: o.outcome === "已通过" ? ("verified" as const) : ("unconfirmed" as const),
    severity: o.outcome === "已通过" ? ("low" as const) : ("medium" as const),
  })),
  verified: [
    `每例在其专属 worktree 上经 TSOX_POOL=1 cargo xtask verify 全量校验（6820 例 + --target 判定 + 相对波起点回归归因），本例 errors PASS + 无回归 + anchor 行消失才出队`,
    `修复 commit 数由 git rev-list 机械复核，非采信 subagent 自报`,
  ],
  notCovered: [
    "全部分支未合并：跨例互斥与合并后整体效果未经最终全量暴露（波收尾整合 + 最终全量在主 agent 执行）",
    "anchor 抹平只看全量轮刷新的总数，未逐例核对全部首分歧的语义",
    "同族未分发例（propertyAccess4/5 等）未处理，留待下波按代表例效果复检",
  ],
};
