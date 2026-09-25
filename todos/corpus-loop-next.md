# 语料修复循环 · 下轮入口(w30)

> 版本：v1.0 · 2026-09-25
> 每轮收尾后本文件更新为下一轮唯一入口；历史轮次文档不保留，根因与论证在 commit message。

## 当前状态（w29 收官）

- 主线 `8edeef2a95`，基线锚点 **FAIL 1194 / SKIP 缺陷 435**（两次全量逐字节一致，确定性成立）。
- 合并态锚点分支 **`w29-merged`**（1423e3c687）= w28 全栈 + w29 收尾 8 支产出、**去 w29b**；该态全量 FAIL 1211。
- 分支库：`w29a-ctxreplay`（2/6 绿）、`w29b-iterbisect`（两修复 9b869421，因 relate 同符号捷径交互回归被隔离，重落前提见下）、`w29c-infwall`（1 绿+2 收敛）、`w29d-frameleft`（无 commit）、`w29e-overspec`（哨兵 genericMethodOverspecialization 关闭）、`w29f-delemit`（D2 差一段）、`w29g-hang`（挂起转确定性 FAIL）、`w29h-nondet`（SymbolTable BTreeMap 确定序）；历史分支 w27a/b/c、w25a、w28a..i 保留。
- 函数靠齐追踪：`python3 tools/gen_func_alignment.py`（单表 func_alignment.csv：exact 4028 / fuzzy 461 / go_only 7798 / rust_only 3790；靠齐后 `--mark --go <名> --status yes|partial --round w30`）。

## w30 首务（按序，全部有最小复现与入口）

1. **w29b 重落前提**：relate 同符号身份捷径。最小复现 genericSpecializations1（TS2416 族参考 49 行 → 本地 12 行漏报）：成员符号挂上函数型后，关系判定/缓存按符号身份短路。修后整支带回（jqueryInference 绿；asyncFunctionContextuallyTypedReturns、contextualTypeIterableUnions 随 arg-shell 治愈）。
2. **簇二 6 例**（arrayFrom / arrayFromAsync / mapGroupBy / nonInferrableTypePropagation2 / promiseTypeInference / underscoreMapFirst）两个断点：
   - Array 接口计算属性成员 `[Symbol.iterator]` 的成员型解析为 Number（`number[]` 取 `a[Symbol.iterator]` 后调用报 TS2349 Number 不可调用）；
   - `IteratorResult<number,·>` vs `IteratorResult<T,·>` 并集成对 T 推断（`infer_to_multiple_types_union` naked 计数路径，inference_checker_14.rs:52 附近）。
3. **JSX children 2 例**：contextuallyTypedJsxChildren2（NoInfer\<unknown\> 泄漏）、jsxChildrenGenericContextualTypes（T 收窄失败，children 返回值上下文丢失）。
4. **挂起转化 5 例**：implicitAnyFromCircularInference（TS2502 被语句级 typeof 预缓存短路，环检测的 resolve_type_query 重入路径未触发）、recursiveFunctionTypes（需签名返回型惰性化）、intersectionsOfLargeUnions×2（TS2536 卡延迟 keyof 可指派性判宽 + TS2367 comparability 按成分比较）、declarationEmitExpandoArrowFunctionParameter（推断域，交叉索引型不归约）。
5. **w29f 最后一段**：实例化 mapped type 别名传播（typenode_references_type_reference_resolution.rs:259-281 条件处），报错名取 NonReactStatics 而非 þtype。
6. **w29g relater 路径守卫**：换新递归身份守卫前，需先给 Rust 类型引用补 node 身份字段（否则 deepComparisons 回归；inference 路径 max_depth=2 已落）。

## 重落与门禁流程

1. 主线直接 `git merge --ff-only w29-merged`，其上叠加 w30 收尾 commit（每根因一 commit，验证即提交）。
2. 全量：`(ulimit -v 8388608; TSOX_SUBMODULE_LIMIT=0 cargo test --release --no-fail-fast > fullrun.log 2>&1)`。
3. 双表导出：`python3 tools/corpus_csv_export.py fullrun.log`（机械 diff，不做 AI 判读）。
4. 准入门槛：**FAIL < 1194 且相对重置态集合新增回归 = 0**；SKIP 差异同步核对。
5. 收尾后：更新本文件为下轮入口，删除本轮全部过程文档（分配表/progress_notes/handoffs 不留存）。
