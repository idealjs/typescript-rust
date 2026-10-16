> 版本：v1.0 · 2026-10-07

# fix51 波留队与回归归因

anchor 抹平波 fix51（20 例分发、并发 6、全量验收）收尾记录：7 例通过、5 例零回归未收敛（已整合）、8 例留队。水位 FAIL 1306→1289、anchor 2174→1986、零回归。修复分支 fix51/w1..w20 保留，worktree 已清理，交接笔记归档 `/tmp/flywheel_verify/fix51-notes/`。

## 留队例（重发建议）

| 例 | 终态 | 归因与重发建议 |
|---|---|---|
| arrayConcatMap | 零 commit | 反向偏差（本地 U[] vs Go any[]）：参数函数型返回位推断 U←errorType 实例化路径，静态无法定钉单行，笔记含探针方案与已排除假设 |
| inferringAnyFunctionType1 | 零 commit | 泛型实参推断不落、返回型保留 T；同族 5 例+fixingTypeParametersRepeatedly2/3 同根因，高杠杆，笔记含三点插桩方案 |
| reverseMappedIntersectionInference1 | 零 commit | 反向映射推断未实例化（res:[T,E]）；候选已排除 12 项，断点依赖运行期状态 |
| contextualTypeBasedOnIntersectionWithAnyInTheMix5 | 零 commit | 多层差异面（any 求值污染+交集 empty-object 移除+签名渲染 fallback+this 链），笔记含四条交接候选线 |
| controlFlowNoImplicitAny | 回归携带未整合 | 9 commit 修 cond 布尔并集渲染等三根因但末轮 41 条回归（capturedLetConst 系列/forOf 系列等），FAIL 1306→1342；流初始型 auto 污染臂过宽，方向对但门控不足，重发须带回归清单收窄 |
| excessPropertyCheckWithMultipleDiscriminants | 回归携带未整合 | 8 commit 修对象字面量上下文型管线，修好 22 例但携 1 回归（enumLiteralAssignableToEnumInsideUnion），判别联合逐成分取型的归约边界需对齐 |
| typePredicateFreshLiteralWidening | 回归携带未整合 | 4 commit 修 isLiteralOfContextualType 全 InstantiableNonPrimitive 臂，修好 3 携 1 回归（correlatedUnions） |
| typeParameterExtendsPrimitive | 回归携带未整合 | 7 commit 接 getWidenedType 全量拓宽修首分歧，修好 1 携 1 回归（mappedTypeGenericIndexedAccess），索引签名拓宽波及面需排查 |

未收敛已整合例（残留分歧下波续修，改动已在 ts2rust-port）：esNextWeakRefs_IterableWeakMap（残留 this 返回型/generator 推断）、declarationEmitCrossFileCopiedGeneratedImportType（6 commit 零回归）、contextuallyTypedSymbolNamedProperties（unique symbol 建链五连修 +9）、abstractClassUnionInstantiation（联合构造签名+排序）、accessorInferredReturnTypeErrorInReturnStatement（readonly 谓词）。

## 已确认抖动例（不参与回归归因）

jsxChildrenIndividualErrorElaborations / jsxCallElaborationCheckNoCrash1、reactReadonlyHOCAssignabilityReal、unknownSymbolOffContextualType1、conditionalTypeDiscriminatingLargeUnionRegularTypeFetchingSpeedReasonable：贴 30s worker 超时线重例，跨分支 SKIP↔FAIL 翻转，波起点基线即非 PASS。前二入 workflow FLAKY 名单；skip→fail 翻转已由 wave_verify.sh 结构性剔除。
