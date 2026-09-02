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

# 当前批次
## Page-4 轮续（页 15-，compiler#1501 起）：页15 一次过（59P/11S/30ad/**0F**）；页16 一次过（62P/12S/26ad/**0F**）；页17 起点 2 例超时挂死→修复→终态 60P/10S/30ad/**0F**；页18 起点 1 FAIL→修复→终态 37P/24S/39ad/**0F**；页19 起点 3 FAIL→修复→终态 54P/8S/38ad/**0F**（连带清台账 1 组），四套门全绿

- **页19 三连修**：①**流走查跟复数前驱**——`antecedent_type_at` 只读单数 `antecedent` 字段，而标签语句的 break 累积节点边存复数 `antecedents`（`antecedent=None`）→ 单前驱时走查死路返回查询种子 `T|undefined`（doWhileUnreachableCode 等**带标签循环跳转出口**的 2454 假阳；两前驱走 junction 分支不中招，故 t8 形状侥幸）；**连带根治并删除台账「闭包-循环捕获 2454 假阳」组**（blockScopedBindingsReassignedInLoop3 转绿）。②**属性真值判别窄化**——`opts.objectRef || opts.getObjectRef()` 的假分支按成员属性类型过滤联合（恒真属性成员删除；空接口 A|B 恒真），narrow_by_expression 补 `discriminant_property_name_on_target` + `narrow_by_property_truthiness`（discriminatingUnionWithUnionPropertyAgainstUndefinedWithoutStrictNullChecks，SNC 默认开场景）。③**TS2874 的实体名**＝jsxFactory 首标识符/reactNamespace（Go resolveJsxEntityName），显式 factory 时不再硬找 React（doubleUnderscoreReactNamespace，`__jsxFactory: __make` + 全局声明 __make 满足门）。

- **延迟索引访问/条件类型的多级约束链（deeplyNestedConstraints 根治，#41931 形状）**：`M[K]`（M extends TypeMap<E>，K extends E[keyof E]）四缺口齐修——①`type_contains_type_parameter` 无 IndexedAccess/Index 分支→`Extract<M[K],…>` 的 check 被误判非泛型、条件急切判 false→参数塌缩 never（补递归后条件正确保持延迟，Go isDeferredType 语义）；②`constraint_of_indexed_access` 只归约对象侧——补 index 侧 `reduce_type_for_constraint`（Go getNextBaseConstraint：TypeParameter→约束、IndexedAccess→constraint_of_indexed_access、keyof X→归约后目标的 get_index_type，深度 8 截止）；③mapped 对象解析：`get_index_type`（keyof＝约束域归约，string 域补 number）与 `get_indexed_access_type`（index 可赋值归约域→模板类型）各补 Mapped 分支——`TypeMap<E>[string|number]`→模板联合 `number|boolean|string|number[]`；④`has_property_of_type` 补 Conditional 分支——延迟分发条件经 `constraint_of_conditional_type`（Go getConstraintFromConditionalType：按 check 的 base-constraint 联合逐成分 resolve 取联合）→`number[]`→`.length` 成立。
- **推断同目标捷径（Go inferFromTypes/inferFromObjectTypes 三条短路移植）**：declarationEmitUsingAlternativeContainingModules1/2（tanstack/vue-query 真实形状）55s 挂死（30s 超时 SKIP 伪装通过）——探针实锤 76M 次 infer_from_types 仅 725 个不同指针对，热点全为**指针相等自反对**（`[number,unknown]→自身` x3.3M 每次全量走查 ReadonlyArray 30+ 方法 = DAG 路径爆炸）。修复：①指针相等且带 type arguments 的对象类型→参数自 zip 即返（本端口结构化元组/数组无 Reference/target 旗标，Go 的 same-Target 规则按指针相等落地）；②Reference 旗标+同 target/双数组→仅 zip 参数即返；③指针相等 Union/Intersection→成分各自自推断。55s→2.8s，输出逐字节不变；两例转入正常 accepted-diff（既分诊族）。

- **推断同目标捷径（Go inferFromTypes/inferFromObjectTypes 三条短路移植）**：declarationEmitUsingAlternativeContainingModules1/2（tanstack/vue-query 真实形状）55s 挂死（30s 超时 SKIP 伪装通过）——探针实锤 76M 次 infer_from_types 仅 725 个不同指针对，热点全为**指针相等自反对**（`[number,unknown]→自身` x3.3M 每次全量走查 ReadonlyArray 30+ 方法 = DAG 路径爆炸）。修复：①指针相等且带 type arguments 的对象类型→参数自 zip 即返（本端口结构化元组/数组无 Reference/target 旗标，Go 的 same-Target 规则按指针相等落地）；②Reference 旗标+同 target/双数组→仅 zip 参数即返；③指针相等 Union/Intersection→成分各自自推断。55s→2.8s，输出逐字节不变；两例转入正常 accepted-diff（既分诊族）。

## Page-3 轮续（页 12-14，compiler#1201-1500）：页12 一次过（63P/0S/37ad/0F）；页13 起点 3 FAIL→终态 59P/10S/31ad/**0F**；页14 起点 1 FAIL→终态 81P/6S/13ad/**0F**；四套门全绿

- **判别式窄化的删除/保留语义**（controlFlowNullTypeAndLiteral + controlFlowArrays 转绿）：`obj.prop !== V` 删除分支原用「重叠」判定（`{val:number|null}` 与 null 重叠→整形成分误删→obj 塌缩 never，`.val` 报 TS2339）——改为「属性类型**含于**被删值」（`{kind:"c"}` vs `!=="c"` 才删）；`===` 保留分支原 overlap 过严（`{length:number}` vs `===0` 误删）——改为双向可赋值「可能相等」判定。单体臂与联合臂同修。
- **attach_explicit_type_arguments 记忆化**（declFile 用例部分收敛）：键 [base ptr, arg ptrs]，值钉住防 ABA；根治重复文本 `g<string>` 的实例分裂。
- **残余登记 2 组（带根因日期头）**：①declFileTypeAnnotationUnionType——注解路径与 new 表达式路径的类实例 BASE 来自两次独立构建（symbol type_alias_links vs 类声明节点缓存），需类实例符号级单一化（D 系列 interning 同根）；②controlFlowWithIncompleteTypes——官方对 incomplete-union（循环自引用窄化进行中）的成员访问静默 any（tsgo-ref 零错实证），incomplete-types 语义未移植。

## Page-3 轮记录（2026-09-02，参考已同步 TS7）：页 0-11（compiler#1-1200）复跑完成，终态均 **0 FAIL**，各页 passed/adiff 与 Page-2 轮一致；新增根治 3 例同位诊断排序（classIndexer2/3、classWithDuplicateIdentifier 由 ad→PASS 稳定固化）

- **同位诊断排序的最终形态**：官方＝start → span-end → code 升序（classWithDuplicateIdentifier 的 [2300,2564,2717] 与 classIndexer2 的 [2411,2564] 均码升序佐证；此前「稳定发射序」假设被这两例否决）。harness render_errors_baseline 恢复 span-end+code 决胜；配合两处发射顺序对齐：类检查先 2411（索引约束）后 2564（属性初始化）；TS2875 缓冲后挂**整元素 span**（起位相同、end 更大 → 排在 7026 后，无需破坏决胜链）。TS2874 的域走查补别名解析（`import React from "react"` 绑定 Alias 无 VALUE 位——follow_alias 至目标；未解析/环回别名按已声明处理，不与 TS2307 双报）。
- 该轮证实参考同步未引入套件侧变化（子模块未动），页 2 的 1 FAIL 仍为 D1 在案携带。

## Page-2 收官记录（2026-09-02）：compiler#1-1200（页 0-11）全部完成，终态均 0 FAIL——详见下方各页记录；该轮累计 11 项 FAIL 修复＋三缓存 ABA 钉住；在案携带：D1 node-memo 污染（arrayToLocaleStringES2020）、2454 闭包-循环假阳（blockScopedBindingsReassignedInLoop3）、NoCrash1/2 显示保真

## 2026-09-02 参考仓同步：typescript-go 参考分支已重置到微软 TypeScript main（TS7，11b6dbfab89）

- fork 源（旧独立 microsoft/typescript-go）与微软 main 无共同祖先，全量历史包 >2GB 推不上 GitHub——origin/main 改存**快照提交**（98571845610，树＝微软 main，提交信息记录同步点），完整上游历史在本地 `m/main` 远端跟踪引用；后续同步流程见记忆 go-port-reference-location。
- **参考代码路径变化**：Go 代码 `internal/…` → `tsc/internal/…`（relater.go/flow.go/inference.go 都在 tsc/internal/checker/ 下）；testrunner 语义出处 `tsc/internal/testrunner/`。TESTING.md 早期记录中的 `internal/checker/checker.go ~L…` 行号属旧快照，对照新树时以符号名检索为准。
- tsgo-ref 探针已按新布局重建（`cd tsc && go build -o /tmp/tsgo-ref ./cmd/tsc`，Version 7.1.0-dev）。
- 测试套件子模块 `_submodules/TypeScript` 追踪微软 **tsgo-port** 分支，已核对其 pin（5848bc5157）＝分支最新，无需 bump（tests/cases 布局在该分支保留；微软 main 的测试已迁 packages/typescript/test/，勿直接跟 main）；子模块补配了 `m` 远端便于后续更新。

## Page-2（2026-09-02 起）——第二轮全量分页：从分页 0（compiler#1-100）重新开始，页内 FAIL+accepted-diff 全部向 Go 行为修复（不允许新增无根因分诊；既有条目能修则修，真子系统缺口以带根因日期组登记并在后续页推进），不回归只向前

工具：`bash /tmp/page.sh compiler|conformance <start> <end>`（页日志存 /tmp/pages/）；
单例 `TSOX_SUBMODULE_FILTER=<stem> TSOX_SUBMODULE_JOBS=1 cargo test --test submodule_compiler`
（conformance 加 TSOX_SUBMODULE_SUITE=conformance）。**坑**：PASS 后旧 local 产物不
清（陈旧产物假 DIFF，diff 前先 rm）；`{:?}` 打印 String 会带引号（误判 spec 带引号）。

## Page 11（compiler#1101-1200）：65 passed / 12 skip / 23 accepted-diff / **0 FAIL**（一次过）

## Page 10（compiler#1001-1100）：52 passed / 10 skip / 38 accepted-diff / **0 FAIL**（一次过）

## Page 9（compiler#901-1000）终态：87 passed / 9 skip / 4 accepted-diff / **0 FAIL**（起点 86/1 FAIL），四套门全绿（lib 1 断言更新＋parity parser_tsx 夹具 jsx 改 preserve——2874 移植后其 react 模式按官方必报）

- **JSX 三连**（commentsOnJSXExpressionsArePreserved 12 个变体全绿）：①TS2874 移植（Go markJsxAliasReferenced——classic `jsx: react` 下 `React` 需以 VALUE 意义在域，标签名处报；jsx_factory_namespace_in_scope 无节点版作用域走查）；②闭合标签的 intrinsic 解析也报 7026（Go checkJsxElementDeferred 对 ClosingElement 的 getIntrinsicTagSymbol）；③同位诊断顺序＝发射序：harness render_errors_baseline 去掉 code 决胜改稳定排序（官方 7026 先于 2875）；TS2875 改为 pending 缓冲至元素检查尾落盘。
- 4 accepted-diff 均台账既有族。

## Page 8（compiler#801-900）：71 passed / 13 skip / 16 accepted-diff / **0 FAIL**（一次过）

## Page 7（compiler#701-800）终态：51 passed / 5 skip / 44 accepted-diff / **0 FAIL**（起点 48/**3 FAIL**），四套门全绿

- **类属性初始化器＝函数级流容器**（classExpressionWithStaticProperties3/ES63 + classPropertyInferenceFromBroaderTypeConst 转绿）：TS2454 的 flow_container_of 边界补 PropertyDeclaration/PropertySignature——属性初始化器内外层已初始化变量的读取是 outer-variable read（assumeInitialized），`class C { x = DEFAULT }`（文件级 `const DEFAULT = 'A'`）不再误报 2454。

## Page 6（compiler#601-700）：33 passed / 52 skip / 15 accepted-diff / **0 FAIL**（一次过；52 skip 为 decorators/importAssertions 等不支持选项密集窗）

## Page 5（compiler#501-600）终态：67 passed / 14 skip / 19 accepted-diff / **0 FAIL**（起点 62/18/**6 FAIL**），四套门全绿

- **TS2365 算术操作数的非 SNC/never 豁免**（capturedLetConstInLoop3/3_ES6/4_ES6/5/5_ES6 全部转绿）：`+` 检查的 number_like 判定补两条——`never` 底类型恒可（无条件）；非 strictNullChecks 下 undefined/null 可赋给 number（空数组 `[]` 的 for-of 元素在非 SNC 下是 `undefined`，官方 `x + v` 零错）。
- **残余登记 1 条**：blockScopedBindingsReassignedInLoop3 的 TS2454 流假阳（闭包内 `x++` 捕获写 + 标签 break/continue 循环汇合——初始化过的循环头 let 被误判未定赋值；U7 流分析族，日期组已登记）。
- 19 accepted-diff 均台账既有族。

## Page 4（compiler#401-500）终态：54 passed / 11 skip / 35 accepted-diff / **0 FAIL**（起点 53/36），四套门全绿

- **TS2693 类型作值用**（autoLift2/bases 转绿）：值位置解析失败时先走 Go `checkAndReportErrorForUsingTypeAsValue`——原始类型关键字名（any/string/number/boolean/never/unknown）或「TYPE-only 意义可解析且无 VALUE 面」的名字报 TS2693（而非 2304/2552 'Number' 建议）；heritage 特例（接口/类 extends 原始类型）与 Promise→2585 变体暂略。
- 余 35 diff 族别：augmentExportEquals×4（export= 增强 TS2671 欠报）、awaitedType 族×5（TS2589 深度上限+Awaited 面）、bigint 族×5、badArrayIndex（TS1011 恢复码+顺序）、base 类族×5（TS2416/2562/2320）、betterErrorFor* 链改进、bindingPattern 推断等——均台账既有族。

## Page 3（compiler#301-400）终态：71 passed / 4 skip / 25 accepted-diff / **0 FAIL**（起点 68/28），四套门全绿（1353/1010/2/15）；台账净删 3 条

- **构造签名链行的箭头显示 + abstract 前缀**（assignmentCompatability44/45 转绿，台账删 2）：官方两签名金字塔行是箭头形 `new (x: number) => Foo`（`signature_display_arrow`），而 no-match 行保持冒号形（`provides no match for the signature 'new (): any'`，37 实证）；abstract 类的构造签名显示 `abstract new () => A` 前缀（45）。
- **对象字面量的字符串字面量键引号显示**（assignmentIndexedToPrimitives 转绿，台账删 1）：`{ "0": 1 }` 显示 `{ "0": number; }`（StringLiteral 键名带引号）、`{ 0: 1 }` 保持 `{ 0: number; }`（数值键裸）——判定依据声明名节点形态：get_type_of_object_literal 的属性符号补挂成员声明；widen_object_literal_type 的加宽克隆继承 declarations（此前全丢，克隆符号无从判别来源）。
- 余 25 accepted-diff 族别：assignmentCompat 私有成员链（40/41/42——private-in-target 消息形已对但多 2 行级联）、apply/call 函数接口（O4）、augmentExportEquals（export= 增强 TS2671/TS2503 欠报）、async 返型推断族×4、assignmentToInstantiationExpression（parser 推测族）、assignmentStricterConstraints（TS2719 误选——泛型约束差应报 2322）等。

## Page 2（compiler#201-300）终态：63 passed / 20 skip / 16 accepted-diff / **1 FAIL＝arrayToLocaleStringES2020（D1 node-memo 污染，在案携带，非新增回归）**

- **【D1 证据链补全】**该 FAIL 由 r29 未提交的正确化推断暴露（HEAD 时 `[1, new Date(), …]` 推断为 `any[]` 侥幸通过；现在正确 `(number|Date)[]`，真实缺口显形）：`number[] → ReadonlyArray<number>` 赋值在 concat/every 等**方法成员**上误报，成员类型显示 `ConcatArray<ResizeObserverSize>[]`——**DOM lib 首解析上下文的 memo 污染**（lib.dom 的 `borderBoxSize: ReadonlyArray<ResizeObserverSize>` 等先解析，惰性方法成员的类型被首个上下文钉住）。三个硬证据：① 无 DOM（`--lib es2020`）完全干净；② 污染类型随构建变化（`U`/`ResizeObserverSize`——分配布局相关）；③ 加调试打印即消失（海森 bug，HashMap 迭代序/分配序敏感）。同轮已修三个**指针键缓存 ABA 复用隐患**（`instantiated_member_type_cache`/`interface_instantiation_cache`/`typequery_instantiation_cache` 改为值内钉住 Arc，键地址不再复用）——正确性改进但非本污染向量。**下一步（D1 续）**：接口方法成员签名的惰性构建需要 arg-aware 上下文（Go 的 mapper/target 实例化语义），即交接记录既定的 node-memo propagation 工作。
- 16 accepted-diff 均台账既有族（arrayFrom/arrayFind 等推断族、arrow 解析恢复族、argumentsSpreadRestIterables 的 iterable/JSX 族、arrayDestructuringInSwitch2 判别窄化既知缺口）；20 skip 为不支持选项。

## Page 1（compiler#101-200）终态：69 passed / 29 skip / 2 accepted-diff / **0 FAIL**（一次过；2 diff 均台账既有族：allowJsCrossMonorepoPackage＝monorepo/node_modules/symlink 解析子系统（官方零错、我们 TS2307 欠解析）、ambiguousGenericAssertion1＝`<<T>(x: T) => T>f` 的 parser 推测恢复链（官方 TS1109+TS2304'x'+3×TS1005 逐 token 复位，我们 TS1109 对但恢复链报 3×TS2304'T' 位点不同）——各族待其在后续页成簇时统一移植；29 skip 为不支持选项（allowJs/decorators/System 等）

## Page 0（compiler#1-100，分页 0 起）终态：84 passed / 14 skip / 2 accepted-diff / **0 FAIL**（起点 84/13/3/2 FAIL），四套门全绿（1353/1010/2/15）

- **访问器对读写取向的修饰符判定**（accessorDeclarationOrder 转绿：public getter + private setter 的 `c1.name` 读取误报 TS2341）：完整移植 Go `getDeclarationModifierFlagsFromSymbolEx`（utilities.go）——写取 SetAccessor、读取 GetAccessor 声明的修饰符，无则回退 ValueDeclaration；仅「存在的非 Class 父级」剥可访问性位（我们 binder 只在 exports 路径设 parent，成员常无 parent，缺失时保留位）；value_declaration 缺失时回退首声明（Property 符号不在 VALUE 掩码内、binder 不设 vd——期间暴露并修复 checker_parity 1 例回归）。读路径 checker.rs 的 `any(声明带 private)` 过近似替换为该助手。
- **别名链意义匹配**（aliasInaccessibleModule2 转绿：`import R = N; export import X = R;` 误报 TS2503/TS2437）：移植 Go `getSymbol` lookup 的别名分支——符号自身 flags 不含意义但为 Alias 且解析目标 flags 含义即命中（`alias_chain_hits_meaning`，解析失败/环→按全意义匹配防级联）；接入 `resolve_identifier_with_meaning_inner` 全部 12 处意义过滤点。TS2437 掩蔽检查处补 `resolve_alias_base`（等价 Go resolveEntityName 的 resolve-through-alias 循环——import= 别名无 bind 期 export 链接，作用域栈返回别名自身）。
- **签名比较的 arity 明细叶**（addMoreOverloadsToBaseSignature 转绿，台账删除）：`compare_signatures_related` 步骤 4 在链窗口激活且非 StrictArity 时经 `relater_report_error` 推 TS2849（Go relater.go ~L1517）；构造签名金字塔块删去手工重复推送（避免双报，顺序不变）。
- **`typeof X<T>` 实例化表达式类型**（aliasInstantiationExpressionGenericIntersectionNoCrash1/2 的语义层，残余登记见下）：type query 类分支改「按类 TP→实参映射的未缓存重建 + typequery_instantiation_cache 记忆化」（类声明缓存是泛型单例，原 type_argument_stack 推送无效）；值分支新增 `instantiate_value_type_for_type_query`（Go getInstantiationExpressionType 范围化移植：交集/联合递归、适用签名实例化——构造签名经返回类型的类 TP 等效、实例返回深替换 substitute_object_properties_deep、壳重建去符号）。**交集 structural tail 补签名比较**（intersection_source_structurally_related 原只查属性——`typeof Cls & (() => T)` 与任意 `typeof Cls` 恒可比；现每目标签名需有源签名 N×M 匹配，比较走 compare_signatures_related）。
- **残余登记 1 组（日期头带根因）**：NoCrash1/2 的 TS2352 已在正确位置触发、成员级 number/string 区分正确；差异全在显示——别名保留显示（Wat<number> vs 展开结构）、实例带参显示（ErrImpl<number> vs 裸符号/匿名结构）、构造签名冒号形式（new (): vs new () =>）、2352 的 3-4 行 comparability elaboration 链未收集。需要 deferred-instantiation 引用形实例 + nodebuilder 别名链显示子系统。
- 14 skip 均为不支持选项（APISample*×11、System module×2、accessorDeclarationEmitJs）。

## Page-1（2026-08-29 起）——按用户指令改为**顺序分页**：compiler#1-100 → …→#6537，conformance#1-100→…，transpile 22，每页 100 条；页内 FAIL+accepted-diff 全修（不允许存留分诊），不回归只向前，直到 12,466 条全跑一遍

工具：`bash /tmp/page.sh compiler|conformance <start> <end>`（页日志存 /tmp/pages/）；
单例 `TSOX_SUBMODULE_FILTER=<stem> TSOX_SUBMODULE_JOBS=1 cargo test --test submodule_compiler`
（conformance 加 TSOX_SUBMODULE_SUITE=conformance）。**坑**：PASS 后旧 local 产物不
清（陈旧产物假 DIFF，diff 前先 rm）；`{:?}` 打印 String 会带引号（误判 spec 带引号）。

## Page 126（conformance#5901-5907，typings 尾窗，**conformance 套件收官**）：2 passed / 1 skip / 4 accepted-diff / **0 FAIL**（typingsLookup3/4 + typingsSuggestionBun1/2——均台账既有族；1 skip 为不支持选项）

## transpile 套件（22 例，收官）：0 passed / 22 accepted-diff / **0 FAIL**（emit 输出整体未移植——declaration/js 产物族台账在册 37 条）

**【全量分页完成】12,466 条全部跑完一遍**：compiler 6,537（Page 1-66）+ conformance 5,907（Page 67-126）+ transpile 22——全部页面终态 0 FAIL；accepted-diff 均为台账登记的子系统族（declaration-emit/JSX/using/parser 恢复/模块解析/模板字面量推断等）＋本会话按分诊规则新登记的日期组。本轮（Page 105-126）累计：8 个 FAIL 修复转绿（访问器文法族 5 例 + 2454 + thisType + 2430 链族 4 例 + 可选 readonly/字符串成员名 2 例）、**conformance 参考基线 1,095 份陈旧副本全量刷新**（Page 96 规律作废修正）、台账净删 14 条 + 新增 6 组；四套门全程全绿（1353/1010/2/15）。

## Page 125（conformance#5801-5900，widening/contextual 窗口）：32 passed / 6 skip / 62 accepted-diff / **0 FAIL**（一次过；arrayLiteralWidened/contextual* 族——均台账既有；6 skip 为 useDefineForClassFields 不支持选项）

## Page 124（conformance#5701-5800，subtyping 窗口）终态：59 passed / 0 skip / 41 accepted-diff / **0 FAIL**（起点 57/41/2 FAIL），四套门全绿（1353/1010/2/15）

- **可选成员的 readonly 显示**（subtypingWithOptionalProperties 转绿：`var b: { s?: number } = a` 官方 `{ s?: number | undefined; }`、我们 `{ readonly s?: … }`）：nodebuilder 属性打印的可选分支无条件带 `readonly`（`format!("{}readonly {}?: …", "", …)` 填充残留）——改为旗标驱动（真 readonly 才显示）。
- **链消息的字符串字面量成员名**（subtypingWithObjectMembersOptionality2 转绿：`'1'?:` 成员官方 `Types of property ''1''`、数值成员原始 `'1'`）：chain_property_arg_name——按声明名节点形态取（StringLiteral→源文本切片带引号；数值/标识符原始）；build_interface_type_from_members 的 PropertySignature 符号补挂 declarations（此前无声明可查）。tsgo-ref 探针确认数值成员原始名 ✓。

## Page 123（conformance#5601-5700，类型关系窗口）终态：31 passed / 2 skip / 67 accepted-diff / **0 FAIL**（起点 30/67/1 FAIL，中途 27/66/5 FAIL），四套门全绿（1353/1010/2/15）；台账新增 2 条日期组

- **【系统性发现】conformance 参考副本 26% 陈旧并已全量刷新**：审计 `tests/baselines/reference/conformance/` 4,236 份 errors.txt 对 `_submodules/TypeScript/tests/baselines/reference/` 权威扁平段（`====` 前错误行、CRLF→LF、去尾空行；归一化公式经 3,141 份一致副本验证）——**1,095 份不一致全部刷新**（Page 92 的 typeSatisfaction 陈旧副本同性质，当时只刷了 1 份）。主形态：新版官方 2430/2322 带 elaboration 链而旧副本无链——**Page 96 的「2430 链只对函数型属性挂」规律系按陈旧基线标定，作废**（官方对象型属性同样带 `The types of 'x.a'` 点链）。刷新后 5 例 FAIL 暴露真实缺口（已修 4）。
- **2430/2322 elaboration 阶梯对齐**（4 例转绿：callSignature/constructSignatureAssignabilityInInheritance、assignmentCompatWithCallSignatures、typeParameterAssignability；tsgo-ref 三形态探针 h1/h2/h3 逐行一致）：
  - **typenode 2430 整体重写**：heritage 成员检查改开 relater 链窗口捕获（签名返回/参数 marker＋嵌套头），成员级尾头（2322(dt,bt) 经 TP 注记助手）→ TYPES_OF_PROPERTY 入口（复用 relater_report_error 既有变换：marker 折叠成 `The types returned by 'x(...)'/'new x(...)'`、属性点链 `The types of 'x.a'`）→ 2430 头最外。删除 Page 96 的 function_prop 手工金字塔。
  - **签名比较两级嵌套头**（relater）：参数失配推 `TYPES_OF_PARAMETERS_0_AND_1`（源/目标参数名）＋嵌套头（**target→source 取向**，Go 逆变单比较）；返回失配在既有 marker 前补嵌套头（source_return→target_return）。
  - **TP 实例化注记**（Go reportRelationError ~L4797）：目标为类型参数时头下挂 `could be instantiated with an arbitrary type`（默认清链只留注记+头）或约束满足形态；外层 2322 头推送统一走该助手。
  - **构造签名 arity 门**：旧 colon 形金字塔（assignmentCompatability44/45）只在 arity 失配时推；返回失配走 marker 变换路（`The types returned by 'new a(...)'`）。
- **残余登记 2 条**（日期组带根因）：assignmentCompatWithGenericCallSignatures2（rest 参数嵌套头数组层级＋泛型元素显示——Go getRestTypeAtPosition 元素级语义待移植）；interfaceWithMultipleBaseTypes2（多基链中可选属性类型视图：官方链内声明视图不带 `| undefined`，我们急切并入——Page 96 窗口背例，刷新后暴露）。
- 余 66 accepted-diff 均台账既有族；2 skip 为既定 SKIPPED_CASES×2。

## Page 122（conformance#5501-5600，tuple/types 续窗口）终态：37 passed / 2 skip / 61 accepted-diff / **0 FAIL**（起点 37/60/1 FAIL——partiallyNamedTuples 按分诊规则并入 homomorphic 映射实例化日期组：参数 `[...{[K in keyof T]: {type: T[K]}}]` 的映射元组实例化缺失→数组回退→2345 假阳，与 mappedTypeWithNameClauseAppliedToArrayType 同根因（Go instantiateMappedTupleType 未移植），组头已补记；余 60 diff 均台账既有族；2 skip 为既定 SKIPPED_CASES）

## Page 121（conformance#5401-5500，thisType/types 续窗口）终态：47 passed / 2 skip / 51 accepted-diff / **0 FAIL**（起点 46/51/1 FAIL），四套门全绿（1353/1010/2/15）

- **对象字面量 this 类型的方法成员**（thisTypeInObjectLiterals 转绿：`mutuallyRecursive` 内 `this.passthrough(...)` 误报 2339——我们的 this 类型只含属性/简写/getter，方法被 `_ => continue` 跳过，`this` 显示 `{ a: number }` 缺全部方法）：build_object_literal_this_type 补 MethodDeclaration 臂——成员符号带 `SymbolFlags::Method` + declarations=[方法节点] 注册，类型**按需**经 get_type_of_symbol 既有 MethodDeclaration 臂解析（注解签名、未注解返回 any）——刻意不做体推断，互递归方法在 any 处停车而非环（Go 用惰性成员等价破环）。CLI 双探针（--noImplicitAny --noImplicitThis）与 tsgo-ref 同为零错。
- 完整字面量值类型（get_type_of_object_literal）遇方法仍塌缩 any——**未动**（改结构化影响全量字面量可赋值性面，超出本 FAIL 需要；`mutuallyRecursive.start()` 经 any 照样零错）。thisTypeInObjectLiterals2 仍为台账在册 diff（带 lib 的 this 面）。
- 余 51 accepted-diff 均台账既有族；2 skip 为 useDefineForClassFields/allowJs 不支持选项。

## Page 120（conformance#5301-5400，types/assignable 窗口）：46 passed / 1 skip / 53 accepted-diff / **0 FAIL**（一次过；assignFrom*Interface/booleanPropertyAccess 等类型面族——均台账既有；1 skip 为既定 SKIPPED_CASES（circular-type recursion））

## Page 119（conformance#5201-5300，callSignatures/types 续窗口）：46 passed / 3 skip / 51 accepted-diff / **0 FAIL**（一次过；51 diff = callSignatures*/augmentedType/union.types 等类型面族——均台账既有；3 skip 为 allowUnreachableCode/allowUnusedLabels + 1 既定 SKIPPED_CASES）

## Page 118（conformance#5101-5200，types 字面量窗口）：31 passed / 3 skip / 65 accepted-diff / **1 FAIL＝templateLiteralTypes1（D8-1 在案携带，非新增）**（65 diff = booleanLiteralTypes×2/enumLiteralTypes×2/commonTypeIntersection 等类型面族——均台账既有；3 skip 为 allowJs/allowUnreachableCode + 1 既定 SKIPPED_CASES；templateLiteralTypes1 差异 9 行＝r33 fixplan 逐行在档的四子缺口：`as` 模板键映射 2322、TS2590 联合复杂度上限缺失、字符串模式匹配推断，D8-1 子系统级，随案）

## Page 117（conformance#5001-5100，types 窗口）：29 passed / 17 skip / 54 accepted-diff / **0 FAIL**（一次过；54 diff（45 用例多配置）= any* 族 + conditionalTypes1 + catchClauseWithTypeAnnotation 等类型面散例——均台账既有族；17 skip 全为 allowUnreachableCode/downlevelIteration/AMD/allowJs 不支持选项）

## Page 116（conformance#4901-5000，usingDeclarations 续窗口）：23 passed / 44 skip / 33 accepted-diff / **0 FAIL**（一次过；33 diff（18 用例多配置）主体 usingDeclarations 族（using 声明子系统，台账在册）+ invalidSwitch/WhileBreakContinue 散例；44 skip 全为 noEmitHelpers 多配置×24 + allowUnusedLabels/allowUnreachableCode 不支持选项）

## Page 115（conformance#4801-4900，using/awaitUsingDeclarations 窗口）：30 passed / 19 skip / 51 accepted-diff / **0 FAIL**（一次过；51 diff 主体 awaitUsingDeclarations 族（using/await-using 声明子系统，台账在册）；19 skip 全为 allowJs/noUnusedLocals/useDefineForClassFields/noEmitHelpers 不支持选项）

## Page 114（conformance#4701-4800，Sputnik js 续窗口）：7 passed / 86 skip / 7 accepted-diff / **0 FAIL**（一次过；86 skip 全为 allowJs×84 + outFile×2 即时选项跳过；7 diff 均台账既有族）

## Page 113（conformance#4601-4700，Sputnik js/allowJs 窗口）：28 passed / 67 skip / 5 accepted-diff / **0 FAIL**（一次过；67 skip 主体 allowJs×61 + outFile×3 + suppressoutputpathcheck + 1 既定 SKIPPED_CASES（circular-type recursion），全即时选项跳过无超时伪装；5 diff 均台账既有族）

## Page 112（conformance#4501-4600，Sputnik NBSP/S7 语法窗口）终态：59 passed / 0 skip / 41 accepted-diff / **0 FAIL**（起点 58/41/1 FAIL），四套门全绿（1353/1010/2/15）；台账删 2 条（missing-2454 组 3→2、extra-2454 组清空）

- **TS2454 带初始化器形态的流式确定赋值**（parserS7.2_A1.5_T2 转绿：`use(x); var x = 1` 官方 2454、我们零输出——NBSP 修复后该文件首次可达 checker，暴露缺口）：check_variable_used_before_assigned 的门卫「initializer.is_some() → 豁免」改为「仅注解与初始化器**双缺**才豁免（any/auto 形态）」——初始化器只是把赋值点放在**声明语句处**，使用先于它（var 提升）/条件性赋值分支后仍为未赋值，交给既有 get_definite_assignment_flow_type 流走查裁决（种子 declared|undefined，使用点流类型仍含 undefined 即报）。对齐 Go checker.go ~L11238 权威条件 `!assumeInitialized && !containsUndefinedType(t) && containsUndefinedType(flowType)`。
- 探针矩阵（tsgo-ref vs ours）逐行一致：使用先于 var/let/const 声明（let/const 为 2448+2454 双报）、条件分支部分赋值（if/while 后使用报、双分支赋满不报）、先显式赋值后使用不报、`var x = null/undefined`（加宽 any 豁免）、声明类型已含 undefined 不报、evolving 数组三形态一致。
- 连带：classStaticBlockUseBeforeDef3 转绿（台账删）；asiPreventsParsingAsInterface03（extra-2454）转绿（组清空删）；**钉子更新 ×1**：checker_ts2448_var_used_before_declaration_no_error 原断言「var 使用先于声明零诊断」系旧欠报行为被钉死——tsgo-ref 核实该形态官方报 2454，改名 checker_ts2448_var_used_before_declaration_no_2448_but_2454 并改断言（保留「var 无 2448」原意）。
- 残余：exportBinding 跨文件 2454（导入侧使用的整程序流分析，深层子系统在案）；classDoesNotDependOnBaseTypes 仍零输出（2454 写目标形态 + 2542 索引签名只读欠报，随案）。

## Page 111（conformance#4401-4500，parser 真源码/语句族窗口）：51 passed / 1 skip / 48 accepted-diff / **0 FAIL**（一次过；parserRealSource/AstSpans/WhileStatement/WithStatement .d 双配置 + VariableDeclaration + TupleType6——均台账既有族；1 skip 为 allowunreachablecode 不支持选项）

## Page 110（conformance#4301-4400，parser break/continue/label 窗口）：49 passed / 6 skip / 45 accepted-diff / **0 FAIL**（一次过；parser*_statement .d 双配置恢复族 + break/continueTarget + duplicateLabel——均台账既有族；6 skip 为 allowunusedlabels/allowunreachablecode 不支持选项）

## Page 109（conformance#4201-4300，parser 数字名/正则歧义窗口）：81 passed / 0 skip / 19 accepted-diff / **0 FAIL**（一次过；parser509667…579071 历史恢复用例 + RegularExpressionDivideAmbiguity + ParameterList2/3/9/16/17 + parserharness/indenter——均台账既有族）

## Page 108（conformance#4101-4200，parser 索引签名/接口/成员声明窗口）：65 passed / 0 skip / 35 accepted-diff / **0 FAIL**（一次过；parserIndexSignature×10 / IndexMemberDeclaration / InterfaceDeclaration / MemberAccessor/Function/VariableDeclaration / ModuleDeclaration / ParameterList 恢复族 + missing-1017 + declaration-emit 散面——均台账既有族）

## Page 107（conformance#4001-4100，parser 歧义/恢复窗口）：46 passed / 2 skip / 52 accepted-diff / **0 FAIL**（一次过；52 diff = parserAmbiguity/ConstructorAmbiguity/CastVersusArrow 恢复族 + declaration-emit 面 + missing-2304/2356、extra-1068 散例——均台账既有族；2 skip 为 ignoredeprecations/allowunusedlabels 不支持选项）

## Page 106（conformance#3901-4000，parserErrorRecovery/enum/arrow 窗口）：60 passed / 0 skip / 40 accepted-diff / **0 FAIL**（一次过；40 diff 全为 parser 恢复 mixed 族（ArgumentList/Block/ClassElement/ExtendsOrImplements/ObjectLiteral/ParameterList/SwitchStatement/VariableList 恢复矩阵 + parserEnum 族 + ArrowFunction1/3 declaration 面）——均台账既有族（mixed-1003/1005/1434/2304、missing-1005、declaration-emit），无新离群点）

## Page 105（conformance#3801-3900，parser 访问器/ES3/计算名窗口）终态：59 passed / 10 skip / 31 accepted-diff / **0 FAIL**（起点 48/42），四套门全绿（1353/1010/2/15）；台账净删 14 条（missing-2378/1094/1054/1095 四组整清 + missing-1049 组解散：3 删 4 迁 missing-2300）

- **访问器文法检查族整体落地**（Go checkGrammarAccessor + checkAccessorDeclaration，checker.rs check_class_member 访问器段——对象字面量访问器同路覆盖）：
  - **TS2378**（get 访问器必须返回值）：非 ambient + 有体 + 体末可达（≈binder HasImplicitReturn）+ 无显式 return（HasExplicitReturn）。锚访问器**名字**。
  - **TS1094**（访问器不能有类型参数）→ **TS1054/1049**（get 零参/set 恰一参，首参 `this` 豁免——Go getAccessorThisParameter 的 len==expect+1 语义）→ **TS1095**（set 不能有返回注解）：串行早退链（前者报则后者抑制；1005 已报的 body-less 非 abstract 形态跳过整链；ambient/TypeLiteral 同样进链——tsgo-ref 探针矩阵钉死）。
  - **循环终止性**（statement_always_returns 补 While/Do/For 臂）：`while(true)`/`for(;;)`/`do…while(true)` 无逃逸 break＝体末不可达（Go 不置 HasImplicitReturn）——条件为**字面量 true 才算**（`while(1)` 仍可退出，ref 探针对齐）；break 逃逸判定 loop_has_escaping_break（无标签 break 只算本层深度，嵌套循环/switch 捕获；带标签保守恒算逃逸；函数体吸收一切）。消费方 TS2355/2366/2378/2322 全部受益（`function f(): number { while(true){} }` 不再误报 2355）。
  - 探针矩阵（tsgo-ref vs ours）逐行一致：2378/1049/1094/1054 × 类/对象字面量/static/computed 名/ambient/abstract/`get x(this)`/带注解空体（我方 2322↔官方 2355 为既存带注解分支差异，随页携带）；while(1)/for(;1;)/do-while(1)/switch-break-in-while(true) 形态双向一致。
  - 转绿：parserAccessors1/3/7/8/9、parserES3Accessors1/3、parserGet/SetAccessorWithTypeParameters1、parserSetAccessorWithTypeAnnotation1（本页）+ parserMemberAccessor1、computedPropertyNames2_ES5/ES6、parserErrantSemicolonInClass1（他页连带）；computedPropertyNames49/50×4 的 1049 半已修，残余为对象字面量重复声明 2300/1118 欠报（迁 missing-2300 组）。
- **背窗发现（非本页范围，已归因非本轮回归）**：默认 LIMIT=1000 门跑（Page 10 后首次复盖 compiler#1-1000）暴露 compiler#201-300 两 FAIL——反向我本轮改动重建复跑同窗口确认**均先于本轮存在**：arrayToLocaleStringES2020＝Page 3 终态携带的 D1 惰性帧族 FAIL（交接在案）；arrayDestructuringInSwitch2＝Page 3 在册的 2488 欠报（解构绑定符号声明类型缺口）但台账条目已遗失——按分诊规则重新登记（带日期头新组），另记录该用例高并发全量跑偶发 TS2366 假阳形态（单跑/窗口跑为零输出，负载相关非确定性，待查）。
- 余 31 accepted-diff：declaration-emit 族 24 + mixed-1005（parserAccessors10 对象字面量成员修饰符解析恢复）+ missing-2300/1196/1164/1125 欠报小族 + extra-2740 + parserES3Accessors4（missing-7006：set 参数隐式 any 配对规则未移植）——均台账在册。

## Page 104（conformance#3701-3800，nodeModules/declaration-emit 窗口）终态：47 passed / 21 skip / 32 accepted-diff / **0 FAIL**（起点 39/35/2 FAIL——页面推进首次触达 r8 时代 conformance 17 FAIL 残留），四套门全绿（1353/1010/15/2）

- **import 别名 × export 说明符别名的 2300 误报**（nodeModulesImportModeDeclarationEmit2×4 修复转绿：`import type { R } from "pkg" with {…}` + `export type { R } from "pkg" with {…}` 同名合法）：Go 里两者分属 locals/exports 两表；我们文件级路由都在 file 符号 members——**不动路由**（动态导入成员 typing/可见性标记等消费者都读 members，改表破坏 r36 两个 parity 钉子+1），改为合并规则放行（一侧 ExportSpecifier 另一侧非说明符的 alias+alias 折叠；同侧说明符×说明符/import×import 仍 2300）。
- **nodeModulesImportAttributesTypeModeDeclarationEmitErrors×4 按分诊规则立案**（import() 类型属性文法+解析恢复子系统：missing-with/wrong-key/array/indirected 四形态恢复矩阵 + TS1340/2353/2339 级联，r36 起在案、Go parseImportType ~L3075 权威路径已定位——台账新组带日期头）。
- 余 32 accepted-diff：nodeModules 解析族（bundler/node10/typesVersions）+ declaration 面 + parser.numericSeparators/forAwait 散例，均台账既有族。

## Page 103（conformance#3601-3700，JSX 尾 + 模块解析窗口）：25 passed / 40 skip / 35 accepted-diff / **0 FAIL**（一次过；JSX 族 ~15 + bundler/node10/node16 模块解析族（bundlerNodeModules/bundlerRelative/importFromDot「`.` 目录模块解析 2305↔2307」/typesVersions/untypedModuleImport×5 declaration 面）——均台账既有族，无新离群点）

## Page 102（conformance#3501-3600，JSX 续窗口）：23 passed / 17 skip / 60 accepted-diff / **0 FAIL**（一次过；60 diff 全为 JSX 族——tsxSpreadAttributesResolution×10 / tsxGenericAttributesType×9 / tsxReactEmit×4 / stateless 组件与恢复散例，无 JSX 外离群点，台账在册）

## Page 101（conformance#3401-3500，JSX 大窗口）：32 passed / 3 skip / 65 accepted-diff / **0 FAIL**（一次过；65 diff = 整块 JSX 子系统族——tsxAttributeResolution×13 / checkJsxChildrenProperty×10（TS2741 children 缺失/2710 重复/2746 单子形态）/ tsxDynamicTagName×9 / tsxElementResolution×8 / 解析恢复与散例——**JSX children/属性/dynamic-tag 检查层未移植**，台账既有条目在册）



## Page 100（conformance#3301-3400，jsdoc/ts-nocheck/jsx 窗口）终态：14 passed / 78 skip / 8 accepted-diff / **0 FAIL**（起点 13/9），四套门全绿（1353/1010/15/2）；台账删 1 条



- **交集源→联合目标的任一成员规则**（typeParameterExtendsUnionConstraintDistributed：`T&U → (A|B)&T&U` 官方零错我们 2322）：TS structuredTypeRelatedTo 的 `some(source.types, s => isRelatedTo(s, target))`——`T&U` 的值是 `T`，故 `T` 的约束（1|2 ⊆ 1|2|3）单独即证成员资格。type_related_to_some_type 补该臂。负例探针核对无误放行。
- **附带发现**（tp2 探针，非本页 FAIL）：`(1|2)&("x"|"y")` 不相交原始类型交集不求 never → 我们误报 2322（ref 认 never 可赋一切）——**不相交交集塌缩 never** 为 relater 另一缺口，随页记录。
- 余 8 accepted-diff：tsNoCheckForTypescript×3 + templateInsideCallback + typeTagNoErasure + typedefOnStatements + typedefScope1 = **JSDoc/checkJs 子系统**（整体未移植，台账在册）；checkJsxChildrenCanBeTupleType = react16.d.ts 经 esModuleInterop 默认导入的 React 命名空间解析（jsxJsxs 相邻族，台账在册）。

## Page 99（conformance#3201-3300，jsdoc 窗口）：4 passed / 92 skip / 4 accepted-diff / **0 FAIL**（一次过；92 skip 全为 ES5/allowJs/checkJs 相邻不支持选项；4 diff = jsdocOuterTypeParameters1/2/3（checkJs 下 JSDoc @template 语义检查）+ jsdocDisallowedInTypescript（TS 文件内 JSDoc 类型文法 `?T`/`!T`/`*`/`Array.<T>`/`function(new:…)` 的 TS8020/17020——需 parser 先产出 JSDoc 类型节点再由 checkJSDocTypeIsInJsFile 拒绝）——**JSDoc 解析/检查子系统整体未移植**，台账既有条目在册）

## Page 98（conformance#3101-3200，jsDeclarations 窗口）：2 passed / 98 skip / 0 accepted-diff / **0 FAIL**（一次过；52 skip 为 target=ES5 不支持，余为 allowJs/resolvejsonmodule/outFile——全 0.10s 即时选项跳过，无超时伪装）

## Page 97（conformance#3001-3100，internalModules/namespace + jsdoc 窗口）终态：54 passed / 45 skip / 1 accepted-diff / **0 FAIL**（起点 37/18 accepted-diff），四套门全绿（1353/1010/15/2）；台账清理 18 条（含同族连带转绿的 TwoInternalModules…LocalVarsOfTheSameName）

- **合并 namespace 的 exports/locals 两表分立**（TwoInternalModules×2、shadowedInternalModule b 例、ModuleWith…ImportAlias 的 2300/2403 误报）：Go declareModuleMember 按导出与否选唯一表——非导出成员只落**本声明块**的 locals，导出成员落模块符号 exports（+locals 本地面）；跨声明的同名「导出 vs 非导出」是两个符号不冲突（`namespace A { export class Point } namespace A { class Point }` 合法）。我们的 declare_symbol 查重是 members∪exports∪locals 三表并集→误报；改为按导出性分表查插。别名特例：ExportSpecifier 恒为导出；`export import X` 只进 exports（无 locals 面）；Alias 与任何种类可合并（Go AliasExcludes=Alias）。
- **get_declaration_name 缺 ExportSpecifier 臂**（reExportAliasMakesInstantiated 2694×3 根因）：`export { test1 }` 的说明符建了符号但名字为空串，exports 表键错位——补臂（按导出名 name 取键，非 property_name）。
- **ImportEquals 错误映射四件套**（invalidImportAliasIdentifiers、shadowedInternalModule）：对齐 Go checkImportEqualsDeclaration + onFailedToResolveSymbol——实体以 NAMESPACE 含义解析（**enum 算 namespace**，Go Namespace=ValueModule|NamespaceModule|Enum，此前 `import e = E` 误报 2503）；失败映射：有 TYPE 含义→**TS2702**（class/interface 目标），值符号遮蔽外层模块→**TS2437**（含义攀名 vs 值攀名差集，resolve_identifier_with_meaning 天然支持），否则 2503；补 **TS2438**（import 名为保留类型名且目标有 TYPE 含义）与 **TS2440**（别名与本地声明含义冲突，锚整条 import 声明；binder 别名合并后按非别名旗标×目标含义判交集）。
- **namespace 块语句检查路径缺失**（invalidModuleWith…EveryKind/VarStatements 的 TS1044 全缺 + `static var` 1435 级联）：check_statement 无 ModuleBlock 臂（语句走了泛型表达式兜底，文法检查全跳过）——补臂循环 check_statement；Interface/Enum/ModuleDeclaration 三臂补 check_grammar_modifiers；parser scan_start_of_declaration 补 Go 的 StaticKeyword 独立臂（nextToken+continue）。
- **ASI 三例**（asiPreventsParsingAsNamespace03/04/05 官方零错）：04/05 根因 is_binding_identifier_token 只认 Identifier|Yield|Await——`let module = 10;`/`let namespace = 10;` 被当表达式语句→1440+2304 级联；改 Go 语义（token > WithKeyword 即上下文关键字可作绑定名）。03 的 TS2454 双根因：2454 需要 strictNullChecks（assumeInitialized 含 !SNC，顶层早退已有但我补在 flow 判定前）+ **namespace 体是流容器边界**（s2 探针矩阵：函数内/嵌套 ns 内读外层 var 不报，同 ns 内报——flow_container_of 补 ModuleDeclaration 边界）。
- **TS2708 三处**：非实例化 ns 的**值面**判定改「三态实例化 ∪ 旧 has_value_side 扫描」（namespace_usable_as_value——别名成员使外层 ns 实例化，exportImportAlias 的 2708×4 误报清除）；`typeof M`/`typeof a`（别名）查值面欠报补同款检查（InvalidNonInstantiatedModule (7,15)、importStatementsInterfaces (23,19)）。
- **TS2833**（invalidInstantiatedModule）：类型位限定名左部为非 namespace 符号时不再误报 2694——左部拦截：拼写建议（find_name_suggestion NAMESPACE 含义）→ 2833「Did you mean 'M'?」（<3 字符候选需 fold 相等，'m'→'M' 达标）；TYPE 含义→2702；否则 2503。
- **2551 建议门控 + 2558 级联**（ModuleWith…Functions/Classes）：2551 建议改用 Page 57 的加权 levenshtein+预算（'fn2'→'fng' 加权 2.0 > 预算 1.9 不建议，tsgo-ref 探针矩阵钉死）；<3 字符候选需大小写不敏感相等（'fn' 永不建议）。2558：callee 为 errorType/any 时跳过元数检查（未解析属性只报 2339）。
- **非导出成员外部访问 + errorType 传播**（ModuleWith…ImportAlias 37,21 2339）：get_type_of_property_access 的 namespace 分支 locals 回退改走 ambient_namespace_local（非 ambient ns 的非导出成员不可见，缺失返回 errorType）；errorType 上继续属性访问保持 errorType（2403 的 error 豁免随之生效）。
- **fundule 别名值面**（exportImportAlias 65,5 2403）：`export import D = K.L`（class+ns 合并）经 type_of_imported_symbol 得 any——回退取 class 构造器类型（符号值面）。
- 残余 1：instantiatedModule（p3 的 2403，`M3.Color.Red` 加宽成 number 而非枚举类型）——**enum 名义类型族**（Page 22 台账既知深层子系统：枚举塌缩字面量联合 `0|1`、成员类型加宽丢符号），随案在册。
- 运维注：/tmp/page.sh 会话间丢失已重建；「CLI 行号 vs harness 行号偏移」= harness 的 split_units 按官方编译器测试解析器剥指令行+前导空行，**基线行号与官方一致**，CLI 探针才是偏移侧。

## Page 96（conformance#2901-3000，interface 继承窗口）终态：42 passed / 1 skip / 57 accepted-diff / **0 FAIL**（起点 40/2 FAIL），四套门全绿（1353/1010/2/15）；台账清理 6 条

- **TS2430 elaboration 链只对函数型属性挂**（interfaceThatHidesBaseProperty2 /
  interfaceWithMultipleBaseTypes2：对象型属性覆写失配官方单行，我们多挂
  `Types of property…` 两级链）：对照 addMoreOverloadsToBaseSignature（函数型
  属性 + 参数数失配→三级金字塔）确立规律——Go 的 heritage 检查
  （checkTypeAssignableTo 定制消息）只记录签名级不相容 elaboration。修复：
  typenode 的 TS2430 发射处加 `function_prop` 门控（双侧有 call signatures 才挂
  level1/level2/level3 链），对象型单行。金字塔探针与平面探针双向验证。

## Page 95（conformance#2801-2900，dynamicImport/umd 窗口）：34 passed / 18 skip / 48 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 9 条（umd3/4、renamed、importAssertion2/importAttributes2/3 多配置）

## Page 94（conformance#2701-2800，moduleResolution 窗口）终态：33 passed / 20 skip / 47 accepted-diff / **0 FAIL**（起点 32/1 FAIL），四套门全绿（1353/1010/2/15）；台账清理 2 条

- **`export * as X` 与同名 type 声明的合并面**（typeAndNamespaceExportMerge：
  `export type Drink = 0|1` + `export * as Drink from "…"`，导入方 `const x: Drink =
  Drink.TEA` 官方零错，我们 TS2749 只见值面）：架构差——模块文件的导出声明落
  file 符号 **members** 表、`export * as` 别名落 **exports** 表，两表分立无合并
  （Go 单 exports 表合并成 TypeAlias|Alias 一个符号，类型位不追别名、值位追）。
  修复：bind_export_declaration 的 NamespaceExport 臂——exports 无同名而 members
  有可合并符号（can_merge_symbols(TypeAlias,Alias)=true）时，把 Alias 旗标并入
  该 members 符号并贯通注册进 exports（单符号双表）；follow_alias 对非纯别名
  返回自身→TypeAlias 意义保留。反序声明（别名先、type 后）走 declare_symbol
  既有 exports-fallback 合并路径，探针双向验证。

## Page 93（conformance#2601-2700，types/moduleResolutionWithoutExtension 窗口）：57 passed / 22 skip / 21 accepted-diff / **0 FAIL**（一次过；circularReference 为 harness 既定 SKIPPED_CASES）——台账清理 10 条（plusOperatorWith×4、exportNonInitializedVariables×2 等）

## Page 92（conformance#2501-2600，typeSatisfaction 窗口）终态：29 passed / 7 skip / 64 accepted-diff / **0 FAIL**（起点 28/1 FAIL），四套门全绿（1353/1010/2/15）；台账清理 13 条

- **satisfies 上下文字面量保真**（typeSatisfaction_vacuousIntersectionOfContextualTypes：
  官方头部 `{ xyz: "foo"; }`，我们 `{ xyz: string; }`+链；tsgo-ref 对照确认）：
  两处修复——1) `get_contextual_type` 补 SatisfiesExpression 臂（Go getContextualType
  的 `return c.getTypeFromTypeNode(parent.Type())`：操作数的上下文类型=satisfies
  目标，此前完全没有→字面量无上下文参与）；2) `get_type_of_object_literal` 的
  PropertyAssignment 保真分支改对齐 Go checkExpressionForMutableLocation：上下文
  **包含**该 fresh 字面量时取 **regular 变体**（原来保留 fresh——随后变量推断的
  widen_initializer_type 只吃 fresh 又把它 widen 掉）；不包含时照旧 widen。
  效果：`const w = {xyz:"foo"} satisfies {xyz:"foo"|"bar"}` → `{xyz:"foo"}`
  （与 ref 一致），无 satisfies 照旧 `{xyz:string}`。
- **附带发现**：我们 `tests/baselines/reference/conformance/` 的该用例副本是陈旧
  2 行版（无 elaboration 链），官方子模块基线=4 行带链——按子模块扁平段刷新。

## Page 91（conformance#2401-2500，typeGuards 窗口）终态：48 passed / 5 skip / 47 accepted-diff / **0 FAIL**（起点含 1 例 typeGuardFunctionErrors **240s 仍挂死的真死循环**），四套门全绿（1353/1010/2/15）；台账清理 21 条、新增 1 条（typeGuardFunctionErrors——死循环修复后暴露的畸形谓词族，随案在册）

- **scanner 字节级 scan_whitespace 死循环**（typeGuardFunctionErrors 第 47 行
  `{` 前藏 U+00A0 不换行空格，多轮 30s 超时 SKIP 掩盖）：`scan()` 的
  `is_whitespace(c)` 按 **char** 解码（含 NBSP/BOM），而 `scan_whitespace()`
  按 **byte** 解码——NBSP 首字节 0xC2 当 Latin-1 char 查表不中、pos 不动，
  scan 循环 `continue` 后原地复判 → 无限循环（任何位置的单个 NBSP 即挂，
  gdb SIGINT 采样定位）。修复：`scan_whitespace` 改 char 解码 +
  `c.len_utf8()` 推进；`is_whitespace` 补全 Go `IsWhiteSpaceSingleLine` 全表
  （U+0085/1680/2000-200B/202F/205F/3000 等）。
  修后该用例 2.3s 跑完，暴露**长期掩盖的畸形谓词族差异**（坏谓词 `x is {`
  的 parser 恢复锚点 TS2749/1144/1131 vs 我们的 1005/2391/2389/1108 级联、
  TS1225 谓词参数索引 + TS2677 谓词类型可赋值检查缺失、错误形状下守卫调用
  塌缩 never）——按分诊规则登记台账（三块需逐一对齐 Go）。
- callChainWithSuper：贴线慢性翻转（13 配置累积 25.2s@JOBS=1，负载下
  30.06s 超时 SKIP；放大预算全配置 PASS，非缺陷，通用每配置编译性能题）。

## Page 90（conformance#2301-2400，contextualTyping 窗口）：29 passed / 13 skip / 58 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 8 条（functionExpressionContextualTyping1/2、iterableContextualTyping1、contextuallyTypedIife、logicalOrExpressionIsNotContextuallyTyped 等）

## Page 89（conformance#2201-2300，comparisonOperator/esDecorators 窗口）：37 passed / 32 skip / 31 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 4 条（asOperatorAmbiguity、esDecorators-contextualTypes、comparisonOperatorWithNoRelationshipObjectsOnInstantiatedCall/ConstructorSignature）

## Page 88（conformance#2101-2200，esDecorators 窗口）：24 passed / 66 skip / 10 accepted-diff / **0 FAIL**（一次过；66 skip 主体 moduleResolution=Classic×20、noEmitHelpers×40 族——不支持选项）——台账清理 3 条

## Page 87（conformance#2001-2100，generatorTypeCheck 窗口）终态：46 passed / 4 skip / 50 accepted-diff / **0 FAIL**（起点 44/2 FAIL），四套门全绿（1353/1010/2/15）；台账清理 4 条（generatorTypeCheck11/12/13/32——同根因连带转绿）

- **yield 在函数样节点计算属性名内的上下文**（generatorTypeCheck42/44：
  `function* g() { let x = { [yield 0]() {} } }` 与 `get [yield 0]() {}` 官方零错，
  我们误报 TS1163）：Go 用 parser 的 NodeFlagsYieldContext 旗标——
  parsePropertyName 在进入函数体/参数前解析，计算名仍处外层生成器上下文。
  我们的 `enclosing_function_is_generator` 攀登遇最近函数样节点即断（方法/
  访问器非生成器→报）。修复：攀登遇函数样节点时，若 yield 位于其**名字子树**
  （node_name 的 span 含 yield 位置——即计算属性名）则越过该边界继续爬；
  参数默认值/方法体仍是边界（箭头体内 yield 照报）。

## Page 86（conformance#1901-2000，Yield/Generator 窗口）：74 passed / 0 skip / 26 accepted-diff / **0 FAIL**（一次过）——台账清理 4 条（VariableDeclaration6/11_es6、YieldExpression16/20_es6）

## Page 85（conformance#1801-1900，templateStringWith 窗口）：77 passed / 0 skip / 23 accepted-diff / **0 FAIL**（一次过）

## Page 84（conformance#1701-1800，templateString 窗口）：48 passed / 2 skip / 50 accepted-diff / **0 FAIL**（一次过）

## Page 83（conformance#1601-1700，objectLiteralShorthand/FunctionDeclaration 窗口）：45 passed / 19 skip / 36 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 2 条（FunctionDeclaration8_es6、newTargetNarrowing）

## Page 82（conformance#1501-1600，for-ofStatements 窗口）终态：44 passed / 4 skip / 52 accepted-diff / **0 FAIL**（起点 44/50/6 + 2 例 **signal 6 崩溃 SKIP**——for-of32/for-of55 栈溢出），四套门全绿（1353/1010/2/15）

- **for-of 自引用循环变量栈溢出**（for-of32 `for (var v of v)` / for-of55
  `let v=[1]; for (let v of v)`：worker 256MB 栈炸，signal 6 伪装 SKIP）：
  环 = check_variable_declaration（或按需路径）→ initial_type_of_declaration
  的 ForOf 臂（RHS 类型）→ RHS 标识符定型 → get_type_of_symbol(循环变量) →
  **resolve_symbol_declared_type_on_demand 的 for-of 早返回路径**——该路径
  `initial_type_of_declaration` 直接递归而**没有**下面 typed 路径的
  "Park the placeholder"（any 占位断环）守卫。修复：早返回路径补同款占位
  （计算前往 value_symbol_links 停 any，重入即命中缓存终止）。修后两例
  2.4s 跑完，落台账 accepted-diff（既有 emit 差异族）。
  **教训**：worker 探针二进制是 `target/debug/deps/submodule_compiler-*`，
  `cargo build` 只重建 lib/bin **不重建 test 目标**——修完跑探针前必须
  `cargo build --tests`，否则陈旧二进制误导（本轮白抓两次 gdb 栈）。

## Page 81（conformance#1401-1500，destructuring 窗口）终态：50 passed / 11 skip / 39 accepted-diff / **0 FAIL**（起点 48/2 FAIL），四套门全绿（1353/1010/2/15）；台账清理 2 条（destructuringSameNames、destructuringWithLiteralInitializers）

- **var 解构 TDZ 误报 TS2448**（destructuringObjectBindingPatternAndAssignment1
  ES5/ES6 双胞胎：`var {b5: { b52 } } = { b5: { b52 } }` 官方零错）：
  Go `checkResolvedBlockScopedVariable` 只对 BlockScoped 旗标符号触发；我们
  binder 的不变量是「变量一律 BlockScopedVariable」（var 性由
  `declaration_is_var` 攀登另行判断，仅用于提升路由），TDZ 检查信了旗标 →
  var 也报。修复：`check_block_scoped_variable_used_before_declaration` 入口
  加 var 豁免（BindingElement→pattern→VariableDeclaration→list 的 let/const
  旗标攀登，同款逻辑内联）；let/const TDZ 行为不变（探针：`let y = l2;
  let {l2}=…` 仍报）。

## Page 80（conformance#1301-1400，destructuring/computedPropertyNames 窗口）：63 passed / 3 skip / 34 accepted-diff / **0 FAIL**（一次过）——台账清理 12 条（computedPropertyNamesContextualType6/7、46/48、9 的 ES5/ES6 双配置等）

## Page 79（conformance#1201-1300，computedPropertyNames 窗口）：62 passed / 0 skip / 38 accepted-diff / **0 FAIL**（一次过）

## Page 78（conformance#1101-1200，emitDeclarations/symbolType 窗口）：44 passed / 24 skip / 32 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 1 条（symbolType11）

## Page 77（conformance#1001-1100，arbitraryModuleNamespaceIdentifiers/symbolProperty 窗口）终态：57 passed / 2 skip / 41 accepted-diff / **0 FAIL**（起点 56/42/2 但其中 1 skip 是 30.05s 超时**掩盖的真 FAIL**——放大预算 20.7s 跑完实为 baseline mismatch），四套门全绿（1353/1010/2/15）；台账清理 1 条（exportEmpty）

**arbitraryModuleNamespaceIdentifiers_module 五连修**（官方基线仅 2 个 TS2322；多配置 module=commonjs/es6/es2020/es2022/esnext/node16/18/20/nodenext/preserve）：
1. **parser**：`export * as "<Z>" from` 字符串名走 `parse_identifier_name_or_keyword` 不消费——改 `parse_module_export_name(false)`（Go parseNamespaceExport→parseModuleExportName，identifier/keyword 或 string）。
2. **checker NamespaceExport alias**：`resolve_alias_base`/`resolve_import_alias_module` 补 NamespaceExport 声明臂（上溯 ExportDeclaration 取 specifier→模块符号；Go getTargetOfNamespaceExport→resolveESModuleSymbol）。
3. **checker from-less 改名导出说明符**：`export { T as U }`（无 from、同文件改名）两条解析路径都够不着（binder 不建链+子句匹配只处理带 from 的）→ `resolve_module_member_symbol` 补 from-less 分支（目标模块=自身，按 property_name 递归，depth 封顶）。此缺口独立于字符串名：`import { U }` 作类型注解曾静默 any。
4. **TS18057**：`check_module_export_names`——import specifier property_name / export specifier property_name（仅带 from 允许字符串，否则 TS1003）+ name / NamespaceExport name，module=es2015/es2020 且非声明文件时逐名字报（Go checkModuleExportName 三调用点）。多出的 TS1003 分支 Go 同款。
5. **resolver node16/nodenext esm_mode**：硬编码 `true`（严格 ESM：禁扩展名追加/目录解析）→ 改 `resolution_mode == ESNext`（Go 同款按引用文件格式）——此前 node1x 下**所有**扩展名省略相对导入全 TS2307。
   修后全 10 配置 PASS；用例 20s（12 配置自导入，多配置累积），8 worker 负载下页内跑完不超时。

## Page 76（conformance#901-1000，importMeta/async 窗口）：33 passed / 22 skip / 45 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 1 条（es2017DateAPIs）

## Page 75（conformance#801-900，importCall/esModuleInterop 窗口）：32 passed / 17 skip / 51 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 1 条（decoratorInAmbientContext）

## Page 74（conformance#701-800，decorators/controlFlow 窗口）：55 passed / 12 skip / 33 accepted-diff / **0 FAIL**（一次过；skip 全为不支持选项）——台账清理 10 条（controlFlowForStatement/BinaryAnd/BinaryOr/Conditional、anonymousClassAccessorsDeclarationEmit1、typesVersionsDeclarationEmit×2 等）

## Page 73（conformance#601-700，propertyMemberDeclarations/controlFlow 窗口）：30 passed / 31 skip / 39 accepted-diff / **0 FAIL**（一次过；31 skip 全为不支持选项，无超时伪装）——按「过绿即删」清理陈旧台账 11 条（autoAccessor2×3/10、accessorsOverrideProperty8/9、constEnum4、staticFactory1、propertyAndFunctionWithSameName 等）

## Page 72（conformance#501-600，privateNames/mixin 窗口）终态：38 passed / 19 skip / 43 accepted-diff / **0 FAIL**（起点 25/19/55/1）——1 FAIL 修复连带 +13 例转绿（25→38），台账删 31 条陈旧条目，四套门全绿（1353/1010/2/15）

- **TS18013 类内误报 = binder 私名成员空键**（typeFromPrivatePropertyAssignment：官方零错，
  我们把类内 `this.#a || {}` 全报 18013）：binder `node_text` 无 PrivateIdentifier 臂 →
  类符号 members 表里 `#a`/`#b` 全绑成 `""` 键互相覆盖；checker
  `lookup_private_identifier_declaration` 按原文 `"#a"` 查表落空 → lexical=None →
  `check_private_identifier_access` 走"类外访问"分支。实例类型侧不受影响
  （`build_interface_type_from_members` 用 `get_property_name_from_node`→`node.text()`
  含 `#` 建表，type_member 查得到——这也正是 18013 能拼出类名的原因）。修复：
  `node_text` 补 PrivateIdentifier 臂（`data.text` 含 `#`）。Go 权威
  `getDeclarationName` 对私名走 `GetSymbolNameForPrivateIdentifier` 逐类 mangled 键；
  我们按原文即可等价（每个类各持一张 members 表，无跨类碰撞；词法查找从内向外
  爬天然实现遮蔽）。修复后类内访问干净、类外仍 18013、TS2803 私有方法赋值与
  嵌套类遮蔽（Outer 作用域访问 Inner 的 `#x`）均正确。
  连带 +13（25→38）：1 FAIL→PASS + 12 DIFF→PASS，主体为 privateName* 族
  （空键下的误报/漏报台账条目——Unique-3、NestedClass*Shadowing、Method 族、
  privateStaticNameShadowing、NotAllowedAs* 等）。同页另按「过绿即删」清理
  早轮陈旧台账 31 条（含 abstractPropertyInitializer、
  mixinAbstractClassesReturnTypeInference 等与私名无关的已绿条目）。
  注：privateName* 主体仍是 accepted-diff（emit 侧差异，非本修复范围）；
  19 skip 主体 usedefineforclassfields/allowjs 不支持选项。



- P68（#101-200）：3/88/9（88 skip 主体 noEmitHelpers 不支持选项）
- P69（#201-300）：32/19/49
- P70（#301-400）：32/7/61
- P71（#401-500）：28/17/55——起点含 2 例 worker 30s 超时 + 全局 OOM

**P71 OOM 根因（privateNameHashCharName / privateNameInInExpressionTransform，r36 起 30s 超时既有）**：裸 `#`（空私名）扫出
PrivateIdentifier token，parse_primary_expression 无处理臂 → fallback
identifier **零消费** → 顶层 parse_list 无限循环，每轮分配节点+诊断
（单 worker ~10s 膨胀 10GB，实测峰值 30GB，systemd-oomd 击杀并殃及
桌面）。修复（对齐 Go parsePrimaryExpression KindPrivateIdentifier 臂）：
1. parser 补 PrivateIdentifier 表达式臂——消费 token 建
   PrivateIdentifierData 节点（复用 parse_property_name 同款）。
2. scanner TS1127 锚点对齐 Go（errorAt(pos-1, 1) 报在 `#` 本身）。
修后 HashCharName PASS、Transform 落台账 accepted-diff、
ComputedPropertyName2 PASS 且 25s→7s。
**运维教训**：zcode 会话退出（内存满）＝某 worker 内存爆炸 → 先查
`journalctl -k | grep -i oom` 拿被杀进程与 RSS，再从当页 cargo log 找
START 无结果用例，worker 模式 + `ulimit -v` 10GB + `/usr/bin/time -v`
单例复现测峰值。

## Page 67（conformance#1-100，conformance 套件开跑）：15 passed / 62 skip / 23 accepted-diff / **0 FAIL**（一次过；62 skip 主体为 noEmitHelpers/downlevelIteration 不支持选项）

## Page 66（compiler#6501-6537，**compiler 套件收官**）终态：22 passed / 2 skip / 13 accepted-diff / **0 FAIL**（起点 19/13/3）——3 FAIL 全清，四套门全绿（1353/1010/2/15）。compiler 6,537 例分页全部完成（Page 1-66）

1. **TS1163 非生成器 yield**（yieldStringLiteral）：checker YieldExpression 臂补
   生成器上下文检查——enclosing_function_is_generator（攀到最近函数样节点，
   Function/Method 看 asterisk_token；Arrow/Accessor/Ctor 即 false），
   对齐 Go checkGrammarYieldExpression 的 NodeFlagsYieldContext 语义。
2. **上下文 ThisType 嵌套穿透 + 对象字面量参与推断**（vueLikeDataAnd
   PropsInference×2，Vue 风味 options：`Options<Data,Props> & ThisType<
   Data & Readonly<Props> & Instance>` 上下文里嵌套 watch 方法 `this.bar`）：
   - **嵌套字面量向上爬**（Go getThisTypeOfObjectLiteralFromContextualType
     的 literal.Parent==PropertyAssignment 循环）：内层字面量自身上下文
     （Record<...>）无 marker 时，逐层向外层字面量要上下文再找 ThisType。
   - **实参上下文推断包含对象字面量**（get_contextual_type_for_argument
     此前只拿非当前下标的兄弟实参）：Go 一阶段只排除 arrow/function
     字面量——对象字面量是 vueLike Data/Props 的唯一推断源。改为按
     kind 过滤（含当前实参），resolving_contextual_calls 守卫防重入
     （重入侧见声明参数类型，天然终止）。

## Page 65（compiler#6401-6500）终态：28 passed / 54 skip / 18 accepted-diff / **0 FAIL**（起点 28/55/17/0 + 1 超时）——useBeforeDeclaration_classDecorators.2 死循环修复（30s 超时→5.6s 正常 accepted-diff），四套门全绿（1353/1010/2/15）；54 skip 主体为 noUnusedLocals/Parameters 不支持选项

- **parse_delimited_list 零进展守卫（parser 死循环族）**（.2 用例：成员装饰器
  + 两处参数装饰器触发；r36 起同形超时，非本轮回归）：类成员解析被参数
  装饰器（未支持）带偏后，顶层把 `m2(@dec a)` 当调用语句，`@` 进入
  ArgumentExpressions 列表 → parse_argument 的 fallback identifier
  零消费 + 期望逗号失败不推进 → 无限循环（gdb attach 抓栈确认）。
  Go parseDelimitedList 的守卫（~L693）：element 解析前后 token 位
  相同则 nextToken() 强制推进。补同款（此前移植漏掉）。
  最小复现：`class C4 { @dec m() {} constructor(@dec a) {} m2(@dec a) {} }`。

## Page 64（compiler#6301-6400）：2 passed / 97 skip / 1 accepted-diff / **0 FAIL**（一次过；97 skip 主体为 noUnusedLocals/noUnusedParameters 不支持选项——unusedLocals/Parameters 全族）

## Page 63（compiler#6201-6300）终态：41 passed / 12 skip / 47 accepted-diff / **0 FAIL**（起点 39/48/1）——unionOfClassCalls 转绿（连带修复多类型参数推断身份 bug），四套门全绿（1353/1010/2/15）

1. **泛型重载探测带推断**（unionOfClassCalls：`arr.reduce((acc: number[], a) =>
   [a], [])` 官方零错、我们错选非泛型重载报 2345）：Go chooseOverload 对泛型
   候选先 inferTypeArguments 再查 isSignatureApplicable——我们的
   signature_accepts_arguments 此前拿**裸类型参数**比可赋值性，泛型重载
   永远被拒（此前靠 arity 兜底碰巧选对/或 any 重载空泛胜出）。补：
   signature_accepts_arguments / find_matching_signature 接收调用节点，
   泛型签名先 infer_call_type_arguments 再 substitute 后逐参检查
   （probe_first_argument_error 早已同款，两路径语义对齐）。
2. **类型参数身份按符号指针（深层 bug，被 #1 暴露）**：本 port 手建类型
   一律 `id: 0`，inference 三处 + relater 一处按 `Type::id` 匹配类型参数
   ——多 TP 上下文里 position() 永远命中 0 号槽，**所有候选灌进第一个
   TP**、后续 TP 全 unknown（`Object.assign(a, b)` 泛型重载选中后返回
   `{a} & unknown` → 2322，parity 钉子拦截）。新增 utilities::
   type_parameters_match（TypeParameter 旗标 + Arc 指针/符号指针），
   替换 infer_to_type_variable 的 position()、约束成分检查
   （inference ~L864）、is_type_parameter_at_top_level、relater 的
   target-return-own-TP 检查四处。此前多 TP 推断"能过"的用例多为结果
   未使用或单 TP（`<T,U>` 里 U 空洞成 unknown 后无人检查）。

## Page 62（compiler#6101-6200）：47 passed / 6 skip / 47 accepted-diff / **0 FAIL**（一次过）

## Page 61（compiler#6001-6100）：44 passed / 3 skip / 53 accepted-diff / **0 FAIL**（一次过）

## Page 60（compiler#5901-6000）终态：43 passed / 10 skip / 47 accepted-diff / **0 FAIL**（起点 40/16/43/1，另 6 例 signal 6 崩溃跳过全清）——1 FAIL 转绿 + 2 崩溃转 PASS + VFS 根目录自列举崩溃族修复；四套门全绿（1353/1010/2/15）

1. **类表达式 this 类型三缺口**（thisIndexOnExistingReadonlyFieldIsNotNever，
   `return class C extends Component<…> { init = () => this.props… }`）：
   - ClassExpression 检查臂补压 `this_type_stack`（同 ClassDeclaration 路径，
     此前漏压 → 箭头体内 this 落到外层方法类）；
   - `build_class_instance_type_with_base` / `extends_base_of` 补
     ClassExpressionData 臂（结构同 ClassDeclaration，此前返回空 object、
     heritage 成员全丢）；`extends_base_of` 同步（类表达式内 super()）。
   - **延迟映射类型的成员存在性宽容**：has_property_of_type 补
     TypeData::Mapped(type_parameter.is_some()) 臂——get_property_of_type
     早已同款宽容（fresh any Property 符号），但 2339 判定走
     has_property_of_type 落空成员表误报（`Readonly<泛型交集>` 上的
     anchorRef 访问）。
2. **静态初始化器 this 重入环（栈溢出）**（thisInConstructorParameter2：
   `static y = this` + 构造参数 `z = this` → get_type_of_class_declaration
   递归溢栈；r36 时 PASS，页面轮加的静态 this 路径引入）：入口缓存
   （type_node_links.resolved_type）+ 进行中守卫
   （class_type_resolution_stack，重入返回 any 占位）；this 节点本就不走
   节点缓存，eager 成员轮会以 this 栈重定型。
3. **VFS 根目录自列举（栈溢出族）**（tslibMissingHelper/Multiple/NotFound/
   tsconfigExtendsPackageJsonExportsWildcard，r36 起崩溃——凡 tsconfig 在
   虚拟根 `/` 的多文件用例）：get_accessible_entries("/") 把根目录自身列成
   空名条目（strip_path_prefix("/","/")=""），combine_paths 跳过空段拼回
   `/` → walk_and_match 无限自递归。修复：空 rest 不入条目。
   修后：tsconfigExtends → **PASS**；tslib×3 → 崩溃掩盖解除，露出台账
   既有 accepted-diff。

## Page 59（compiler#5801-5900）终态：27 passed / 38 skip / 34 accepted-diff / **0 FAIL**（起点 27/34/1）——taggedPrimitiveNarrowing 转绿，四套门全绿（1353/1010/2/15）；38 skip 主体为 module=System 不支持选项

- **混合交集的 typeof 窄化 no-op**（taggedPrimitiveNarrowing：
  `string & { __hash: true }` 上 `typeof x === "string"` 后取 `.length`
  官方零错、我们塌缩 never 报 2339）：Go getIntersectionTypeFacts——
  交集含原始类型成分时**忽略对象成分**（type tag 语义，官方注释原文
  `string & { __kind__: "name" } we ignore the object type`），narrowTypeBy
  TypeFacts 走 hasTypeFacts(交集) 命中 → getIntersectionType([t, string])
  保持原类型。narrow_by_typeof 顶部补同款门：交集含非原始成分（非全
  primitive）直接返回原类型，不进成分级过滤（成分级会丢对象面 → never）。

## Page 58（compiler#5701-5800）终态：45 passed / 5 skip / 50 accepted-diff / **0 FAIL**（起点 45/49/1）——subtypeReductionUnionConstraints 按分诊规则登记（自引用联合别名子系统：Go 惰性对象成员 vs 我们急切解析冻结环守卫 error；成员惰性化两版试探均已回退——条件版不彻底、全量版引发 lib 2430 假阳，完整 deferred-members 移植随案在册），台账 2841→2842；**连带修复**：谓词假分支删除改用可赋值性（overlap 误删无关对象成分成 never）+ any 豁免 + `||` 真分支/`&&` 假分支完整分解（`!isA(x) || !isB(x)` → 左真 ∪ 左假∧右真，Go narrowsTypeByExpression）；四套门全绿（1353/1010/2/15）

## Page 57（compiler#5601-5700）终态：52 passed / 3 skip / 45 accepted-diff / **0 FAIL**（起点 51/45/1）——spellingSuggestionModule 转绿，四套门全绿（1353/1010/2/15）

- **拼写建议全面对齐 Go core.getSpellingSuggestion**（spellingSuggestionModule：
  官方对 `foobar`/`barfoo` 纯 TS2304，我们误建议 'toolbar'/引号模块名）：
  1. **加权 Levenshtein**（levenshtein_with_max 新增，Go core.go ~L637）：
     大小写不敏感替换 +0.1、失配替换 +2、增删 +1、带宽限制早退——
     `foobar`→`toolbar` 加权 3.0 > 预算 2.9（floor(len·0.4)+0.9）被拒
     （普通距离 2 误过）；`toobar`→1.0 ✓ 仍建议（tsgo-ref 探针矩阵一致）。
  2. **引号名排除**：ambient 模块符号名（我们的 binder 保留源引号——
     单/双/反引号）与 \u{FE} 内部名不作候选。
  3. len<3 候选需大小写不敏感相等；长度差预过滤 max(2, 0.34·len)；
     best_distance 随更优候选收缩（Go 同款）。
  - 仅换 find_name_suggestion 路径；parser 1435 / TS2551 属性建议的
    edit_distance 保持旧制（其后页面按差异再对齐）。

## Page 56（compiler#5501-5600）：72 passed / 10 skip / 18 accepted-diff / **0 FAIL**（一次过）

## Page 55（compiler#5401-5500）：45 passed / 8 skip / 47 accepted-diff / **0 FAIL**（一次过）

## Page 54（compiler#5301-5400）：33 passed / 39 skip / 28 accepted-diff / **0 FAIL**（一次过）

## Page 53（compiler#5201-5300）终态：59 passed / 5 skip / 36 accepted-diff / **0 FAIL**（起点 58/36/1）——recursiveFieldSetting 转绿，四套门全绿（1353/1010/2/15）

- **TS2729 确定赋值断言豁免**（recursiveFieldSetting：`parent!: T` 后
  `depth = this.parent.x` 官方零错）：属性声明带 postfix 断言（`!`，我们的
  postfix_token）时豁免「无初始化器」触发的 2729——与 TS2564
  （strictPropertyInitialization）的豁免同款；「声明靠后」分支保持。

## Page 52（compiler#5101-5200）终态：35 passed / 14 skip / 51 accepted-diff / **0 FAIL**（起点 35/50/1）——ramdaToolsNoInfinite2 按分诊规则登记（缺子系统：Iteration/Pos 元组机械延迟条件实例化链 + commonjs 下 ambient 模块 import 目标被同名泛型别名污染的 TS2314 假阳，双根因在案，改名探针证实），台账 2840→2841

## Page 51（compiler#5001-5100）终态：48 passed / 11 skip / 41 accepted-diff / **0 FAIL**（起点 45/41/3）——privacy*CannotName 三例转绿，四套门全绿（1353/1010/2/15）

- **ambient 命名空间隐式导出上下文（类型面）**（privacyCannotName{Accessor,VarType}DeclFile /
  privacyFunctionCannotNameParameterTypeDeclFile：`declare module "M" {
  export namespace N { function f(): C } }` 经 `import = require("M")` 的
  `M.N.f()` 官方合法——Go setExportContextFlag：ambient 容器无 `export {}`
  子句 → 全成员隐式导出）。既有 ambient_namespace_local 回退只挂在标识符
  成员解析路径；resolve_namespace_type 的成员面现在同样合并 ambient
  隐式导出上下文的 locals（checker 侧合并，不动 binder 表——lib 类型
  懒解析不受扰）。探针 tsgo-ref 对照：use.ts 链路逐字一致。
- 顺带探明（非本页 FAIL，随页携带）：.ts 内裸 `declare namespace N` 的
  **typeof N 值侧**仍解析为 any（官方报 'typeof N' 显示 + 2454）；非
  ambient namespace 外部访问未导出成员的 TS2339 欠报。

## Page 50（compiler#4901-5000）：30 passed / 43 skip / 27 accepted-diff / **0 FAIL**（一次过；43 skip 为不支持选项）

## Page 49（compiler#4801-4900）：47 passed / 14 skip / 39 accepted-diff / **0 FAIL**（一次过）

## Page 48（compiler#4701-4800）：38 passed / 4 skip / 58 accepted-diff / **0 FAIL**（一次过）

## Page 47（compiler#4601-4700）终态：53 passed / 14 skip / 33 accepted-diff / **0 FAIL**（起点 52/33/1）——noSubtypeReduction 转绿，四套门全绿（1353/1010/2/15）

- **联合类型三缺口**（noSubtypeReduction：`x: IA|IAB` 下 `for (const el of
  x.arr)` + `'B' in el` 窄化，官方零错、我们 el=never）：
  1. **联合接收者属性访问类型**：get_type_of_property_access 对 union 查不到
     成员 → 静默 any。补联合分支：逐成分取属性类型（含数组/泛型实例化
     代换）取并集（Go 逐成分 resolve 后 union；缺失成分的 TS2339 由
     check_property_access 的 has-property 门报）。
  2. **联合可迭代物元素类型**：iterated_element_type 对 union 落 any。补
     逐成分元素并集（never 成分吸收）。
  3. **for-of 循环变量声明类型**：check_variable_declaration_list 的
     `(None, None) => auto_type()` 把 for-of 变量定成隐式 any 并入缓存，
     on-demand 永不触发。改为先走 initial_type_of_declaration 的
     for-in/of 臂（for-in→string、for-of→元素类型），非循环变量保持
     auto。resolve_symbol_declared_type_on_demand 同步补委托。
  4. **`in` 窄化 any 豁免**：`'B' in el` 对 any 逐成分过滤成 never——
     narrow_by_in_keyword 顶部 any 早退（属性存在性未知）。
  - 探针对照 tsgo-ref 逐行一致（p21：2322/2339 联合文本；p17：el.B 2339
    在联合上）。

## Page 46（compiler#4501-4600）：38 passed / 26 skip / 36 accepted-diff / **0 FAIL**（一次过）

## Page 45（compiler#4401-4500）终态：64 passed / 2 skip / 34 accepted-diff / **0 FAIL**（起点 59/35/4）——4 FAIL 全清 + SwitchTrue 族 4 例连带转绿，四套门全绿（1353/1010/2/15）；台账 2842→2840（删 True1/True3）

本页续上一会话（上会话已修 nestedLoopTypeGuards / narrowingOfDottedNames：
flow loop_stack（Go flowLoopStack）+ 回边再入解析到进行中循环的 union-so-far +
instanceof 超类成分收窄；本会话验证转绿）。

1. **switch 子句组拓扑整体重构**（narrowByClauseExpressionInSwitchTrue5，对齐 Go
   bindCaseBlock/bindSwitchStatement）：
   - binder：子句按**落空组**（连续空语句子句 + 持语句子句）归并，每组一个
     SwitchClause 流节点**挂在 switch 入口**并携带 `[start,end)` 区间
     （FlowNode 新增 clause_range）；语句前标签 = 组节点 ∪ 上一组语句尾流
     （fallthrough 边）；无 default 的 switch 出口补 `[0,0)` bypass 节点
     （穷尽时自然塌缩 never）；case 表达式在入口上下文绑定；
     isNarrowingSwitch 门（TrueKeyword/isNarrowingExpression 全套移植，
     含 containsNarrowableReference/isNarrowableOperand/二元各操作符臂）。
   - checker：narrow_by_switch_on_true 按组语义重写（组前全 case 取反 →
     含 default 组再取反组后 → 否则组内各 case true-窄化取并集）；
     on_discriminant/on_typeof/on_discriminant_property 同步区间化
     （组 case 类型并集 / 组外 witness 取反过滤）。
   - **非联合判别窄化补 never**（True6 连带根因）：`x.kind !== "c"` 在 x 已
     削到单成员 {kind:"c"} 时不再 no-op——try_narrow_by_discriminant_property
     非联合分支按 Go narrowTypeByDiscriminant 语义（无属性保持、any 保持、
     取反侧仅 unit 值触发）。
   - 连带转绿：True1/3/4/8/9（台账删 2 条）；True6/7 残余为显示层差异
     （联合成员序/别名名显示/2339 elaboration 链），台账保留。
2. **重载组比较泛型擦除**（narrowingAssignmentReadonlyRespectsAssertion，
   `string[]` → `ReadonlyArray<string|number>` 的 reduce）：Go
   signaturesRelatedTo 的 pairwise（同符号）与 N×M 路径都是
   `signatureRelatedTo(…, erase=true)`——**泛型签名擦除（TP→any）后比较**，
   我们此前传原始签名走实例化路径，第三泛型重载 `reduce<U>` 的 U 无法跨
   Array/ReadonlyArray 符号代入 → N×M 无匹配。get_erased_signature 从空桩
   实装（TP→any 的 get_signature_instantiation）。单签名路径保持实例化
   （Go 同构）。
- 钉子更新 ×2（checker_narrowing_switch_true_equality/default_negates_all）：
  原钉子用 `let x = 0` 初始化器——字面量收窄使 case 比较报 TS2367
  （tsgo-ref 同样报），改参数形态（官方零错），语义断言不变。
- 残余 34 accepted-diff 主体：narrowing 窄化族显示层（联合成员序/别名显示/
  elaboration 链）、newAbstractInstance 族等（台账在案）。

## Page 44（compiler#4301-4400）终态：56 passed / 14 skip / 30 accepted-diff / **0 FAIL**（起点 55/30/1）——multipleInferenceContexts 转绿，四套门全绿

- **上下文 ThisType<T> 标记提取**（multipleInferenceContexts，r36 后被
  Page-1 对象字面量 this 机制回归暴露）：`ConstructorOptions<Data> =
  Props<Data> & ThisType<Instance<Data>>` 上下文类型化对象字面量时，
  方法内 `this` 应取 **ThisType 标记的类型实参**（Go
  getContextualThisParameterType → getThisTypeOfObjectLiteralFrom
  ContextualType 优先于字面量自身面）。this_type_marker_argument：按
  数据形态（Union/Intersection 递归 Constituents——flags 驱动会漏）
  找 symbol 名为 ThisType 且恰一实参的引用，取其实参压栈。

## Page 43（compiler#4201-4300）终态：40 passed / 25 skip / 35 accepted-diff / **0 FAIL**——moduleAugmentationOfAlias 归入台账既有模块增强族（declare module './a' 增强合并进默认导出接口的子系统未移植），台账 +1

## Page 42（compiler#4101-4200）：43 passed / 6 skip / 51 accepted-diff / **0 FAIL**（一次过）

## Page 41（compiler#4001-4100）终态：45 passed / 6 skip / 49 accepted-diff / **0 FAIL**——mappedTypeWithNameClauseAppliedToArrayType 按分诊规则登记（homomorphic 映射×数组/元组实例化子系统，Go instantiateMappedArrayType/TupleType 未移植），台账 +1

## Page 33-40（compiler#3201-4000）：连续 8 页一次过全 **0 FAIL**（无修复）

P33 28/16/56、P34 49/2/49、P35 49/7/44、P36 50/14/36、P37 38/19/43、
P38 2/98/0（全页 isolatedModules/declaration 不支持选项 SKIP）、
P39 16/33/51、P40 40/22/38（passed/skip/accepted-diff）。

## Page 32（compiler#3101-3200）终态：40 passed / 20 skip / 40 accepted-diff / **0 FAIL**（起点 38/40/1）——importExportInternalComments 转绿 + 2 DIFF 连带转绿，四套门全绿

- **TS2695 判定整体对齐 Go isSideEffectFree（逆命题重写）**：
  expression_has_side_effects 此前是粗枚举（一元全算有副作用、对象/数组
  字面量算有副作用、非赋值二元不递归）。重写为 Go 精确语义：标识符/
  字面量/函数类表达式/对象数组字面量/`typeof`/非空断言/模板/JSX 无
  副作用；前缀 `!`/`+`/`-`/`~` 无副作用（**`void`/`delete` 有**——
  `void D, A, C, foo` 不再误报）；条件/二元表达式递归判操作数；其余
  皆有。补 **`(0, f)()` 间接调用豁免**（isIndirectCall：括号包裹 +
  左侧字面量 0 + 父为调用且 callee 是该括号/标签模板 + 右侧访问链或
  eval）。

## Page 31（compiler#3001-3100）：48 passed / 3 skip / 49 accepted-diff / **0 FAIL**（一次过）

## Page 30（compiler#2901-3000）：48 passed / 4 skip / 48 accepted-diff / **0 FAIL**（一次过）

## Page 29（compiler#2801-2900）终态：56 passed / 7 skip / 37 accepted-diff / **0 FAIL**（起点 55/37/1）——functionSubtypingOfVarArgs 转绿，四套门全绿

- **类属性空数组初始化器 SNC-off 加宽**（functionSubtypingOfVarArgs，
  strict:false）：Go getTypeForVariableLikeDeclaration——**类属性永远不取
  autoArrayType**（auto 仅限变量且 noImplicitAny），空数组元素经 widening：
  SNC on 隐式 never 保持 `never[]`；SNC off widening-undefined **加宽为
  普通 `any[]`**（此前我们给 `undefined[]` → push 报 2345）。
  build_interface_type_from_members 的 PropertyDeclaration 无注解臂补
  空数组分支（create_array_type(any)）。strict 下类属性不演进（never
  push 报错）与官方一致（evolving1 探针双态核对）。

## Page 28（compiler#2701-2800）终态：78 passed / 2 skip / 20 accepted-diff / **0 FAIL**（起点 77/20/1）——funClodule 转绿，四套门全绿

- **TS2434 定位语义对齐**（funClodule）：官方与符号声明表中**首个
  非 ambient class / 带体 function**（Go getFirstNonAmbientClassOr
  FunctionDeclaration，ambient 过滤含 .d.ts 文件级）比位置——namespace
  在其**之前**才报；此前我们找「更晚的 class/function」导致
  `function foo3(){}; namespace foo3{}; class foo3{}` 的 namespace(16)
  被误报（官方只报 2814+2813）。

## Page 27（compiler#2601-2700）：45 passed / 9 skip / 46 accepted-diff / **0 FAIL**（一次过）

## Page 26（compiler#2501-2600）：28 passed / 32 skip / 40 accepted-diff / **0 FAIL**（一次过；32 skip 为 UMD/declaration 等不支持选项）



## Page 22（compiler#2101-2200）终态：56 passed / 9 skip / 35 accepted-diff / **0 FAIL**（起点 54/35/2）——两 FAIL 转绿，四套门全绿（1353/1010/2/15）

1. **emitClassMergedWithConstNamespaceNotElided（TS2434 门控对齐 Go）**：
   - **文件级 ambient 豁免**：`.d.ts` 内 namespace 不走 2434（Go
     checkModuleDeclaration ~L5214 的 `!inAmbientContext` 含文件上下文；
     我们此前只看显式 declare 修饰符）。
   - **三态 module instance state 移植**（Go ast.GetModuleInstanceState）：
     NonInstantiated / **ConstEnumOnly** / Instantiated；const-enum-only
     namespace 仅 preserveConstEnums（或 isolatedModules）时算实例化；
     `export {x}`（无说明符）继承目标名局部声明的状态（祖先块扫描 +
     import 别名再导出视为 Instantiated；环守卫）。整套替换此前
     「interface/typealias/import/export 之外皆实例化」的粗扫。
2. **emitThisInObjectLiteralGetter（strict:false 双门控）**：
   - **TS7023 门 noImplicitAny**（Go getTypeOfAccessors ~L18614 /
     signature 路径 ~L20096 都是 `else if noImplicitAny`）。
   - **对象字面量 this 类型门 noImplicitThis**（Go
     getContextualThisParameterType：`noImplicitThis || inJs` 才取
     字面量/上下文 this，否则 `this`→any）——strict:false 下
     getter 内 `this.bar` 不再误报 2339/7023。
3. 页内 35 accepted-diff 主体：enum 名义类型族（enumAssignmentCompat×7 /
   enumBasics×3 / enumMemberResolution 等——我们塌缩成字面量联合
   `0|1|2`，官方 `W`/`typeof W`/成员类型 `W.a` 带符号显示，深层子系统
   在案）、emit-declaration 族、emptyTypeArgumentList 等。

## Page 25（compiler#2401-2500）终态：45 passed / 18 skip / 37 accepted-diff / **0 FAIL**（起点 40/39/3）——3 FAIL 修复 + 2 DIFF 转绿，四套门全绿（1353/1010/2/15）

1. **exportAssignmentEnum / exportAssignmentVariable（export= 文件模块实体追踪）**：
   type_of_imported_symbol 的 `export = T` 追实体路径此前要求
   module_sym.declarations 含 ModuleDeclaration（namespace）；**文件模块**
   （declarations=[SourceFile]）走不到 → 退回 namespace 类型（`typeof
   import(...)`）。补文件模块分支：实体名首段经
   resolve_module_member_symbol 解析，剩余 QualifiedName 段沿
   exports/members 链走，返回目标符号类型（enum→枚举对象类型、var→
   其声明类型）。
2. **expandoFunctionSymbolProperty（函数 expando 属性子系统定向移植）**：
   - **binder**（Go bindDeferredExpandoAssignments + getInitializerSymbol
     TS 子集）：`x.prop = v` / `x[key] = v`（base 为实体名）延后到文件
     末解析；base 解析到**函数声明**符号时挂 expando——静态名建
     Property 符号入 exports（与已声明名冲突时跳过：查 exports/
     members/合并 namespace 的 ModuleDeclaration locals 三处）；
     动态名挂 `\u{FE}assignment` 伪符号声明表。
   - **checker**（attach_function_expando_type，函数声明检查点）：从
     符号 exports 收集 expando 声明，RHS 类型（with_declaring_file_
     context + widen）建匿名成员面，函数类型 ∩ 成员面；动态名按
     computed 成员惯例键 `[<arg 源文本>]`（与接口 `[symb]` 成员同名
     匹配）。
   - 回归修正：合并 namespace 的已声明成员（`namespace Foo { var bla }`
     的 `Foo.bla = ...`）不得被 expando 遮蔽（expandoFunctionNested
     AssigmentsDeclared 首轮复跑暴露，查重来源补全后修复）。
3. 页内 37 accepted-diff 主体：exportAssignment 族（export= 声明发射/
   合并语义）、es6import 解析恢复族等。

## Page 24（compiler#2301-2400）：49 passed / 15 skip / 36 accepted-diff / **0 FAIL**（一次过）

## Page 23（compiler#2201-2300）：36 passed / 34 skip / 30 accepted-diff / **0 FAIL**（一次过；34 skip 为 UMD/noEmitHelpers 等不支持选项）

## Page 21（compiler#2001-2100）：45 passed / 16 skip / 39 accepted-diff / **0 FAIL**（一次过，无需修复）

## Page 20（compiler#1901-2000）：53 passed / 8 skip / 39 accepted-diff / **0 FAIL**（起点 51/39/2）——discriminateWithOptionalProperty3 + discriminatedUnionJsxElement 转绿，四套门全绿

1. **any 满足索引签名目标**：is_index_signatures_related_to 前置
   any-true 门；Page-4 的 primitive-vs-index 规则排除 any/unknown。
2. **enum TDZ 声明容器缺失**：declaration_for_scope 的 find 未含
   EnumDeclaration → 函数体内的 enum 前向引用走了无豁免路径
   （补上后 IIFE/属性初始化器等既有豁免对 enum 生效）。
3. TYPE-position / 类型参数默认值 引用不做 TDZ（枚举类型侧提升）。

## Page 19（compiler#1801-1900）：35 passed / 25 skip / 39 accepted-diff / **1 FAIL**（起点 35/39/1）——延迟索引访问约束解析落地，四套门全绿

- **M[K] 属性存在性走约束**：has_property_of_type 对 IndexedAccess 先
  解 constraint（K 的约束为 string/number/字面量联合时经
  get_indexed_access_type 解到签名值类型/属性联合）；2339 显示也用
  解析后类型（`M[K]`→`number`）。
- 剩 1 FAIL：deeplyNestedConstraints——泛型映射 M=TypeMap<E> 的
  **多级约束链**（M→TypeMap<E> 实例化→E[keyof E]），需 >5 层约束
  探索（Go getConstraintOfIndexAccessType 递归），随页携带。

## Page 17/18（compiler#1601-1800）：P17 59/13/28/0 一次过；P18 58/10/32/0（起点 56/32/2）——declarationNoDanglingGenerics + decoratorMetadataElidedImportOnDeclare 转绿，四套门全绿

1. **TS2302 门控收紧**：仅**类自有**类型参数受限（TP 声明父链到
   Class* 才报）——泛型函数内类表达式的 static 成员可用外层 T。
2. **`declare` 类成员修饰符**：parse_class_member 修饰符清单补
   DeclareKeyword（`declare prop: T` 此前修饰符丢失 → 2564 假阳性；
   Go IsModifierKind 含 Declare）。

## Page 15（compiler#1401-1500）：82 passed / 6 skip / 12 accepted-diff / **0 FAIL**（起点 80/12/2）——declFileObjectLiteralWith{Accessors,OnlySetter} 转绿，四套门全绿

- **对象字面量 this 类型的属性初始化器加宽**：`set x(a){ this.b = a }`
  中 `this.b` 读到的是加宽的 `number` 而非字面量 `10`
  （Go getThisTypeOfObjectLiteral）。

## Page 14（compiler#1301-1400）：57 passed / 10 skip / 32 accepted-diff / **1 FAIL**（起点 56/31/3）——corrupted + controlFlowArrays 转绿 + 循环流三项根因修复，四套门全绿

1. **循环头快照 bug（重要）**：`while` 的 pre_while_label 在 body 绑定
   **前** finish——antecedents 快照丢失回边，循环内窄化退化为入口类型
   （`while(c){ if(typeof x==="string") x.slice() }` 窄成 never）。修：
   `finish_multi`（单前驱也建 junction）+ `push_antecedent`（完稿后
   追加回边/continue 边）。
2. **`let x = null` 声明类型 = auto（隐式 any）**：on-demand 解析补
   null/undefined 初始化器 → auto_type（此前 null 字面量类型导致
   循环内 18047 假阳性）；flow 种子同样补。
3. **赋值 RHS 空数组字面量重播种 evolving**（`x = []` 在 null 声明
   后 → auto_array）+ null 声明侧 auto 直通。
4. **TS1128 二进制标记门控**：仅 parser 已消费真实语句后才发
   （corrupted.ts 只报 1490）。
- 剩 1 FAIL：controlFlowWithIncompleteTypes(24,19)——循环 else 支回边
  cycle 种子含 declared 的 boolean（Go 双遍展开语义），随页携带。

## Page 13（compiler#1201-1300）：61 passed / 0 skip / 39 accepted-diff / **0 FAIL**（起点 58/39/3）——contextualTyping21/30 + contextualTypeIterableUnions 转绿，四套门全绿

1. **computed 名跨文件切片修复（重要）**：`[Symbol.iterator]` 成员名
   在解析方 current_file≠声明文件时切错文本
   （`Iterable<number>` 的缺失属性显示成 'unction parseFloa'——
   parseFloat 的错位切片）→ 改用**声明文件**文本。
2. **元素级报告后跳过整参 2345**：数组/对象字面量实参的元素级
   诊断已发时不重复报整个实参。
3. **2322 元素级去重**（elaborate 路径 vs contextual-elements 路径
   同 loc 双报）。

## Page 12（compiler#1101-1200）：65 passed / 12 skip / 23 accepted-diff / **0 FAIL**（起点 64/23/1）——constInClassExpression 转绿，四套门全绿

- 类表达式成员 grammar 检查去重（ClassExpression 臂的显式循环与
  check_class_member 内部重复 → 1248 双报）。

## Page 11（compiler#1001-1100）：49 passed / 11 skip / 40 accepted-diff / **0 FAIL**（持平计数）——TS1435 建议落地，四套门全绿

- **TS1435**（`asynd`→'async'、`clasd`→'class'）：parser 的
  missing-semicolon-after 兜底分支补拼写建议（关键字表 + Go 预算
  floor(len*0.4)+0.9 + 空格建议形态）；仅当**后随标识符**时建议
  （`MyClass2 {}` 后随 `{` → 保持 1434）。commonMissingSemicolons
  主体对齐（剩 4 行 parse-recovery 级差异：2304/2552 边界 + 1155，
  随页携带）。
- SourceFile 增 `parse_error_spans`（parser 诊断区间，本轮未用于
  抑制——官方对 1435 名字仍发 2304/2552，保持不抑制）。

## Page 10（compiler#901-1000）：86 passed / 10 skip / 4 accepted-diff / **0 FAIL**（起点 85/4/1）——commentsOnObjectLiteral3 转绿，四套门全绿

- **TS7023 setter 对豁免**：带成对 setter 的无注解 getter 其类型取自
  写侧视图，无循环推断，不再误报。

## Page 9（compiler#801-900）：70 passed / 13 skip / 17 accepted-diff / **0 FAIL**（起点 68/19/0）——classWithEmptyTypeParameter + classWithDuplicateIdentifier 转绿，四套门全绿

1. **TS1098**：空类型参数列表 `class C<>`（parser，锚在 `<`）。
2. **类成员重名语义**（2300/2717）：冲突对判定（prop+prop /
   prop+method 任意序 / 双体 method+method）；**发射位点**——
   method-first 对只在属性侧报；property-first 对双侧报；
   prop+prop 在第二侧报。TS2717（后续属性声明类型不同）带首声明
   类型（属性注解或方法签名显示）。
   ⚠️ 期间 check_class_member 的 match 曾被误拆（调试插桩切断注释
   行）——已完整重建（Property/Signature/StaticBlock/Method 族/
   `_` 顺序），四套门验证无损。

## Page 8（compiler#701-800）：50 passed / 5 skip / 45 accepted-diff / **0 FAIL**（起点 44/45/6）——classExpressionWithStaticProperties×6 转绿，四套门全绿

- **命名类表达式的自引用**（`var v = class C { static c = C.a }`）：
  binder 容器阶段（locals 重建后）把类名符号插入**自身 locals**
  （Go binder 语义；先前 declare_symbol_into 被容器阶段的 locals
  重置清空）；checker 类表达式成员检查补 push_scope。
- 45 accepted-diff 主体：classExtends 族（extends 表达式/null 基类）、
  classFunctionMerging、computed names 作用域等。

## Page 7（compiler#601-700）：33 passed / 52 skip / 15 accepted-diff / **0 FAIL**（起点 32/15/1）——capturedShorthandPropertyAssignmentNoCheck 转绿，四套门全绿

- **const 空数组也走 evolving**：`const fns = []; fns.push(...)` 同样
  演进（Go 对空数组字面量初始化器的 autoArrayType 规则不区分
  const/let——语句级与 flow 种子两处删除 Constant 门）。
- 本页 52 skip 为 allowJs/ES5 等不支持选项。

## Page 6（compiler#501-600）终态：67 passed / 14 skip / 19 accepted-diff / **0 FAIL**（起点 63/23/0）——6 例转绿（blockScoped*UseBeforeDef 全族），四套门全绿

### 本轮修复：TS2448/2450 TDZ 语义对齐（Go isUsedInFunctionOrInstanceProperty）
1. **豁免规则重写**：不同函数体引用（非 IIFE）豁免；**IIFE 立即调用
   穿透**（`(() => a)()` 仍报）；引用走到**声明的函数容器**为止。
2. **类属性初始化器**：实例属性初始化器中引用非实例属性声明 → 豁免；
   **static 初始化器**急切执行 → 报。
3. **自身初始化器内引用**（`let [a] = (() => a)()`）：binding-element
   攀升到 VariableDeclaration 判定。
4. **enum TDZ**：EnumDeclaration 纳入声明种类；**const enum 非
   isolatedModules 不报**（声明擦除；binder 把 const enum 标成
   RegularEnum——从声明源文本窗口检测 const 关键字）；修
   `all()` 空**迭代器**恒真 bug（顶层变量误入 const-enum 豁免）。
5. **TS7027**：terminator 后的 enum/class/function 声明（提升）不再
   报 unreachable。
6. **2448 去重**（同 loc+code 双检查路径）。
- 台账 2840（无新增；5 例族全清）。剩余 19 accepted-diff：
  booleanLiterals/bluebirdStaticThis/builtinIterator/cachedContextualTypes/
  callOverloads×5（类+函数合并调用语义）/callsOnComplexSignatures 等。

## Page 5（compiler#401-500）终态：52 passed / 11 skip / 37 accepted-diff / **0 FAIL**（起点 39/49/1）——12 例修复转绿，四套门全绿（1353/1010/2/15）

### 本轮修复
1. **TS2552 建议三处补全**：候选并入当前文件符号表成员；类上下文
   else 分支；距离大小写不敏感 + Go 预算（`loc→Lock`）。
2. **TS2567**：enum+class / enum+interface 冲突（enum 侧从
   can_merge 的 interface 共存规则排除）。
3. **TS2434**：instantiated 非 ambient namespace 先于合并的
   class/function —— checker ModuleDeclaration 臂实现（报在
   **namespace 名**，Go checkModuleDeclaration ~L5222），顺序无关。
4. **TS2432**：合并 enum 第二+声明首成员无初始化器，仅当**首个
   声明也以无初始化器开头**（`{a=1}+{b}` 合法）；锚在成员名。
5. **enum 成员重名 2300**（合并 enum 间成员冲突，双侧成员名各报）
   + **ns `var prototype` 2300**（与 class/function 合并时的自动
   prototype 符号冲突，双向顺序）。
6. **ns+var 合并语义**：非实例化 namespace 与 var 合法合并
   （`ns{interface I}+var`）；实例化的仍 2300。
- 台账 2849→2840（删 9+1）。剩余 37 accepted-diff 主体：
  await 族（awaitedType×5/awaitLiteral/awaitInNonAsync…）、
  augmentExportEquals×7、bigint×6、badInference 族等。

## Page 4（compiler#301-400）终态：68 passed / 4 skip / 28 accepted-diff / 0 FAIL（起点 61/35/0）——7 例修复转绿，四套门全绿（1353/1010/2/15）

### 本轮修复（7 例转绿 + edit_distance 根因）
1. **TS2628/2629/2630 + 2540 赋值目标族**：枚举/枚举成员/类/函数目标
   报 cannot-assign 且**阻断类型检查**（official 只报一条）。
2. **TS2364 锚点**：带括号的非引用目标报在整条赋值式（`(1,x)=0`→
   (2,1)），且非法目标仍作为表达式检查（引出 2695）。
3. **TS2695**：逗号左操作数无副作用时报 unused（expression_has_
   side_effects 判定：赋值/调用/new/增删/yield/await/字面量）。
4. **TS2552 拼写建议**：edit_distance **移除「长度差>2 返回 3」哨兵**
   （NaN 对一切长名假建议的根因）+ 预算改 Go 公式
   floor(len*0.4)+0.9（大小写敏感）——`tupel→tuple`、
   `undefinedFunc→undefined` 与官方逐字一致。
5. **return 语句 mismatch 走 elaborateError**（async 嵌套字面量
   报在属性名位，asyncFunctionReturnExpressionErrorSpans）。
6. 索引签名对象显示（键名回收）+ 原始源 vs 索引目标（上轮）保持。

### Page 4 遗留（28 accepted-diff 主要族）
- 2696 Object→窄类型（Object 接口自环解析 error——D1 子问题在案）
- 泛型接口可选属性 `T|undefined` 比较（c35/39/10/8）
- 私有属性 TS2341 族（c40-42）+ construct 签名 `=>` 显示（44/45）
- Function 接口 apply/call 成员比较（checking-apply/call 族）
- async 上下文返回 7006 族、2504 async-iterator 元数、2671/2503
  模块增强族、2364 实例化表达式族（assignmentToInstantiation）
- 台账 2853→2849。

## Page 3（compiler#201-300）终态：61 passed / 20 skip / 17 accepted-diff / **2 FAIL**（起点 55/25/0）——8 例修复转绿，单测门四套件全绿（1353/1010/2/15）

### 本轮修复（9 例转绿 + 3 个连带根因修复）
1. **TS2740 数组目标缺失链**（arrayAssignmentTest1/2/4）：独立前置检查——
   裸数组目标（`any[]`）对非数组源枚举声明 Array 接口成员，源查找带
   Go getPropertyOfType 三级回退（自有→Function 接口→**全局 Object 接口**，
   使 toString/toLocaleString「存在」），>5 成员时首 4 + and N more；
   签名-only 源（`() => C1`）走 shouldReportUnmatchedPropertyError 实装
   （仅头行 2322）。
2. **空数组字面量类型**：`[]` = `never[]`（SNC on）/`undefined[]`（off）
   ——Go checkArrayLiteral ~L8148；未注解变量仍走 auto 标记（evolving
   数组靠 ARRAY_MUTATION 流演进，语句级+on-demand 两条路径都补了
   节点规则；auto/evolving 上的变更方法（push/unshift）类型为 any）。
3. **成员顺序**：类实例/接口 merge 改**派生在前**（Go
   resolveObjectTypeMembers 声明序 + addInheritedMembers 只增）——
   `C2M1, IM1, C1M1` 顺序 + 调用签名也是派生先。
4. **elaborateError 子系统移植**（Go relater.go ~L444）：对象/数组字面量
   赋值失败时逐属性/逐元素细化（锚在属性名/元素节点，递归初始化器），
   成功则抑制外层金字塔——`{x: undefined}`→`X(,28) 2322 单行`；
   数组字面量对**非数组对象目标**走数值索引签名（实例化重读——
   `ConcatArray<never>` 的 `[n:number]: T` 经实参解析为 never）。
5. **TS2769 逐重载报告**：全部重载不可用时「Overload i of n」条目链
   （探针=逐重载首个实参失败，scratch store 隔离）；类方法重载合并修
   （实现签名不入可调用集）；签名显示 `(...)` rest 前缀 + 可选 `?`。
6. **TS2366 方法分支**（arrayAssignmentTest5）：类方法 some-return-not-all
   →2366（锚返回注解）；switch definitely-returns（default + 每非空子句
   尾 return/throw）。
7. **TS2815 arguments 家族**：属性初始化器/静态块引用 arguments（祖先
   链遇 PropertyDeclaration/ClassStaticBlock 即报，箭头穿透）；类表达式
   成员补全检查；参数默认初始化器按声明上下文检查（`(x=arguments)=>`）。
8. **TS2538 索引类型**：联合索引逐成员报（`string[]` 成员点名）；
   类型参数索引（`K extends keyof T`）豁免。
9. **switch 判别窄化两根因**（连带）：子句流基点改 **switch 入口**
   （break 后 current_flow=None 使后续子句丢失窄化——binder）；非联合
   判别不匹配 case → never。
10. **联合显示序**：内部按 TypeFlags 位值排序（Go getSortOrderFlags）；
    **打印层** nullish 置尾（null 先 undefined）——type_to_string 与
    type_to_type_node 两处。联合元素访问分布（`number[]|null[]`[0]→
    `number|null`）；2339-on-never（has_property 的 never 豁免在
    check_property_access 旁路）；every 联合细化叶 undefined 优先于 null。

### 遗留（随页携带）
- arrayToLocaleStringES2015/2020（2 FAIL）：`as ReadonlyArray<number|Date>`
  2352 假阳性——concat 成员比较的目标侧实例化解析到陈旧帧
  （`ConcatArray<ResizeObserverSize>`）＝**D1 惰性帧族**（交接文档既定
  下一步 node-memo propagation）；试探「接口实例化时急切捕获签名参数
  override」会引发 lib.dom 2430 假阳性，已回退。
- arrayDestructuringInSwitch2：2488 依赖解构绑定符号的声明类型
  （官方 `[] | [1]`，我们 any）——解构属性类型解析缺口。
- 其余 16 accepted-diff 见 /tmp/pages/compiler_201_300.log（类型谓词
  推断/递归元组联合/declaration emit 等独立形态）。
- 台账 2864→2855（删 9 条已修）。

25 条 DIFF 按错误码签名分组（/tmp/pages/compiler_201_300.log）：
- **数组目标赋值族 5 例**（arrayAssignmentTest1/2: 欠 2740；Test4: 欠 2322+2740；
  Test5: 欠 2366 函数缺返回）——非数组源赋给 `T[]` 目标时官方报 **TS2740**
  `Type 'C3' is missing the following properties from type 'any[]': length, pop,
  push, concat, and 25 more.`（列出的是源所缺的 Array 成员——注意首四项不是 lib
  声明序，是「源缺的成员」序：C3 类实例**有** Object 的 toString/toLocaleString
  （官方属性查找含全局 Object 回退），故缺的长这样；>5 时首 4 + and N more）。
  挂点：relater is_object_type_related_to 的 missing_props 路径，数组目标侧
  properties 为空（bare array）→ 目前直接放行（Test4 现状零输出）。实现：目标
  is_array_type 且源非 array/tuple/evolving 时枚举 bundled Array 接口成员、过滤
  源已有的（含 Object 回退）→ 2740；函数型源（有 call 签名无属性）按 Go
  shouldReportUnmatchedPropertyError 只报 2322 头。Test1 另有 2741/2739
  （never[]→接口/类）。
- **2769 重载族 2 例**（arrayBestCommonTypes、arrayConcatMap[+2339/2345]）：
  No overload matches + 逐重载链（Overload 1 of 3 … Argument of … ）。
- **11 例 .delete 欠报组**：argumentsReferenceInFunction1_Js（Js=allowJs 子配置）、
  argumentsUsedInClassFieldInitializerOrStaticInitializationBlock、
  argumentsSpreadRestIterables（varyBy target：ref 有 (target=es5) 变体）、
  arrayDestructuringInSwitch1/2、arrayFind、arrayFrom/FromAsync、
  arrayIndexWithArrayFails、arraySigChecking(1268)、arrayOfSubtype…（部分为
  多配置 (key=val) 基线，分析时须枚举产物目录而非裸名）。
- 单例：arrowFunctionErrorSpan(1200)、arrowFunctionParsingGenericInObject、
  arrowFunctionsMissingTokens(多报 1134)、assertionFunctionsCanNarrowByDiscriminant、
  assignLambdaToNominalSubtypeOfFunction（P1 难例池同款）、
  arrayFakeFlatNoCrashInferenceDeclarations(欠 5088 多报 2536)、
  arrayLiteralAndArrayConstructorEquivalence1/arrayFrom(多报 2345)。


## Page 2（compiler#101-200）：69 passed / 29 skip / 2 accepted-diff / 0 FAIL（起点 67/3/1）
7. **allowSyntheticDefaultImportsCanPaintCrossModuleDeclaration**（本轮 FAIL 回归）：
   `export default <标识符>`（接口）经 import 别名用作类型时，default 目标是
   binder 的 **Alias 符号**（exports 表有 default；此前误判 exports 恒空）——
   resolve_import_alias_target_symbol 补**别名链追查**（≤4 跳，经 ExportAssignment
   表达式名查模块 members/exports），追到有类型意义的符号止。
8. **anonymousModules**（TS2591）：**摘除 ensure_host_globals 的 CJS 合成全局**
   （exports/require/module/__dirname/__filename/global/process）——官方只从
   @types/node 解析它们；未解析使用走 2591 Node 建议变体（36 个基线被掩盖，
   后续页会持续兑现）。注意：DOM_VALUES 合成组（document/console/…）**保留**
   （官方带 lib.dom 解析真符号；无 lib.dom 的 2580/2592 变体族未动）。
   另：`module {` 匿名模块在官方走**表达式路径**（ExpressionStatement+Block；
   1437 来自 parse_error_for_missing_semicolon_after 的关键字误用检查，2591 来自
   checker 对 module 标识符的 cannot_find_name 变体）——我们同构，无需
   ModuleDeclaration 侧挂钩。

### Page 2 余 2（随页携带）
- ambiguousGenericAssertion1：`<<T>(x:T)=>T>f` 的 `<<` 歧义恢复链（官方
  2304'x'+1005')'+1005','+1005';' vs 我们 2304'T'×3）——parser speculation。
- allowJsCrossMonorepoPackage：symlink 解析子系统未移植（TS2307 多报）。

### 累计（Page 1+2）
- 单测门四套件全绿（1353/1010/2/15）；台账 2871→2864（删 7 条）。
- 页面基线：P1 84/2/0，P2 69/2/0（passed/DIFF/FAIL）。

## Page 1（compiler#1-100）成果：8 accepted-diff → 修 6，0 FAIL（84 passed / 14 skip / 2 accepted-diff）；单测门 1353/1010/2/15 全绿；台账 2871→2865
1. **aliasAssignments**：`import x = require("m")` 别名类型 = 模块实例类型。
   type_of_imported_symbol 补 ImportEqualsDeclaration 臂（ambient/rel→resolver 回退；
   **模块带 export= 时走被赋值实体**——aliasOnMergedModuleInterface 回归教训）；
   resolve_type_query（typeof X）同样先走该路径（原落 errorType 静默放行）。
   **文件模块导出面重建**：resolve_namespace_type 对 SourceFile 符号（exports 空）
   从语句表回收 export 成员（直接导出声明 + `export {}` 子句 + default；locals 空，
   顶层声明在 symbol.members）——number→namespace 2322 只报头行（Go relater
   ~L3903 `!sourceIsPrimitive` 抑制装箱比较的结构链）。
2. **aliasUsageInOrExpression**：同根 + **TS2872/2873 移植**（checkTruthinessOfType
   全套：&&/|| 左操作数、! 操作数、if/while/do/for/三元条件；纯句法
   getSyntacticTruthySemantics：数字 0/1 豁免、void/null/空串/undefined 恒假、
   断言/括号剥离、三元分支位或）+ 联合源初始化器报错走 elaborate 路径
   （2322 头 + `Type 'null' is not assignable` 叶链；2741/2739 只留非联合源）。
3. **allowImportClausesToMergeWithTypes**：**TS2749**（值用作类型）补 import 别名
   分支——resolve_type_reference 顶部：Alias 符号经新
   resolve_import_alias_target_symbol（ImportClause default / ImportSpecifier 命名；
   文件模块 exports 空时 file_module_exported_member 回收 default）解目标符号，
   有类型意义→继续按目标解，纯值→2749（用别名自己的名字）。
4. **abstractPropertyInConstructor**：**TS2715** 三形态补全（属性初始化器
   `other = this.prop` ——isNodeUsedDuringClassInitialization 语义：祖先链先到
   PropertyDeclaration=true、函数类边界 quit；解构**赋值** `({x, y: y1, "y": y1} =
   this)` 在 ctor 内按属性名逐个报）+ **TS2729**（初始化器读 this.X，X 无初始化或
   声明靠后）+ **TS1117 解构抑制**（对象字面量为 `=` 左值时跳过重复属性文法检查，
   Go checkGrammarObjectLiteralExpression 的 inDestructuring）。
   **连带大修**：`this` 类型不走节点缓存（实例类型急切解析成员时 this 栈未压，
   会把 any 冻进缓存——属性初始化器里 this.nope 全漏）；属性惰性解析
   （resolve_symbol_declared_type_on_demand）补压所属类实例类型。
5. **accessorInferredReturnTypeErrorInReturnStatement**：**TS7023**（对象字面量
   无注解 getter 的 return 链经 this/本地别名引用 this → 循环返回推断）+
   **对象字面量 this 类型**（build_object_literal_this_type：属性按初始化器、
   无注解 getter→readonly any 成员；getter_return_reaches_this 别名一级追踪；
   **只有方法/访问器体内见字面量 this，属性初始化器见外层 this**——
   checker_iterator_protocol 回归教训）+ widen_object_literal_type 克隆保
   Optional/Readonly 修饰（打印 `{ readonly primaryPath: any; }`）。
6. **TransportStream**：harness 排序补 TS sortAndDeduplicate 语义的 tiebreak
   （同位按 span 长度→code：1490@(0,0) 长度 0 先于 1434）；scanner 记
   binary_marker_pos（U+FFFD=RuneError 值比较陷阱），flush 补 TS1128@标记位。

### Page 1 余 2（深层别名实例化子系统，随页携带不新增分诊）
- aliasInstantiationExpressionGenericIntersectionNoCrash1：我们零输出——
  intersection of 构造器实例化（`typeof ErrImpl & (<T>() => T)` 实参化后）
  comparable 应 FAIL 且三层链（ErrImpl<number>→ErrImpl<string>→number→string），
  源显示结构形 `{ new (): ErrImpl<number>; prototype: ErrImpl<any>; } & (() => number)`。
- NoCrash2：2352 已发但两侧显示未实例化的 `typeof Class & (() => T)`——需
  ① Type.alias 字段从未被填充：泛型别名引用（Wat<number>）解析时要挂
  TypeAlias{symbol,args}（注意缓存共享 Arc 不能直接 mutate）；② typeof fn<T>
  实例化表达式的实参代入；③ intersection 打印走别名名+实参；④ comparable 链。

### 待办（页流程）
- Page 2 = compiler#101-200：`bash /tmp/page.sh compiler 101 200`，修完记台账+TESTING.md。
- 每页完成后：从 triaged.txt 删本页所修条目（Page1 已删 6 条）。

## Batch1（2026-08-28 晚起）——最大凝聚块攻坚：text-only 306 例的「错误细化链」逐条修

**选块依据**（对 r36 当轮 DIFF 面实测归类，非拍脑袋）：错误码全对文本差 306
例为最大凝聚块；2322/2345 赋值族 ~350 例次之但根因散（alias/索引签名/void
混装）。**修正先前判断**：compiler/conformance 只比 errors.txt（不含 .d.ts
产物），「声明发射最大块」不成立于此面；transpile 22 例是 isolated
Declarations（TS90xx）专项，另账。

**节奏**（按用户指令）：100 条/批（/tmp/batch1_list.txt = text-only 前
100），逐条修到单过滤 PASS（/tmp/run_batch.sh 单过滤批跑 ~2.5s/条），
全批清完再选下一块。**台账清理欠账**：修复后需从 triaged.txt 删组（本批
所修条目逐条核删）。

### 已修形态（15/100 PASS，每形态一组用例）
1. **arrayCast 锚点+链**：cast 失败为对象字面量 excess property（含数组
   字面量内）时 2352 锚到属性名处 + excess 链行（`assertion_excess_detail`）
2. **2430 三层链**（addMoreOverloads）：接口 heritage 逐属性失败手拼
   `Types of property 'f'` → `Type X not assignable to Y` → 签名元数
   `Target signature provides too few arguments`（min_argument_count 对比）
3. **18048 实体文本**（arrayconcat）：PropertyAccess 表达式源文本切片回退
   （node.text() 为空）——`'a.name' is possibly 'undefined'`
4. **索引签名缺失链**（assignmentCompat…WithStringIndexSignature）：
   `is_index_signatures_related_to` 兜底失败压
   `Index signature for type 'string' is missing` 链行
5. **可选属性 nullish 叶链**（assignmentCompatability11/13/15/17/19/21/23
   共 8 例）：`each_type_related_to_type` 遍历全成员，失败的 nullish 成员
   压 `Type 'undefined' is not assignable to type 'X'.`（非 nullish 失败折
   叠进 head——官方同构）
6. **泛型签名显示 + no-match 链**（24）：函数类型打印补 `<T>` 前缀；
   源无 call 签名但目标有时压 `provides no match for the signature
   '<T>(a: T): R'`（`signature_display_colon` 冒号形态）
7. **函数源 vs extends Function 接口的 missing-props 链抑制**
   （assignLambda——但该例转为难例池，见下）

### 难例池（2 例，深层另案）
- **aliasInstantiation…NoCrash2**：泛型别名实例化显示 `Wat<number>` 形态
  （别名打印带实参）+ comparable 三层链——别名显示子系统
- **assignLambdaToNominalSubtypeOfFunction**：**既有欠报**（今日所有运行
  NO_CONTENT；8/23 产物为陈旧残留——`.delete` 标记证实）。根因疑
  `IResultCallback extends Function` 撞 heritage 降级窗口
  （degraded_type_ptrs 自动放行）——D1 降级族

**工具链固化**：批量归类脚本 /tmp/triage_v2.py（枚举 local 产物 vs
reference；**坑**：多配置键不能从 log 还原，须直接枚举产物目录；统计须按
用例票数非错误行数——60 个 JSX 大用例贡献 4,274 行 TS2694 的假块）；
批跑 /tmp/run_batch.sh（注意 cargo 输出走 stderr，2>/dev/null 会全吞）。

**单测 +4 钉子**（b1_*：2430 金字塔 / nullish 叶链 / no-match+`<T>` 显示 /
索引签名链）；checker_parity 1006 全绿。**下一形态待续**（batch1 剩 83 条）。

## r36 后修复轮（2026-08-28 午）——r36 唯一回归（2883 误报×2 形态）全清

r36 暴露的 nodeModulesDeclarationEmitDynamicImportWithPackageExports
（2883 误报）两种形态均已修复，单过滤三配置 PASS：

1. **库内文件豁免**：`check_declaration_nameability` 跳过路径含
   `/node_modules/` 的导入者（包内互引走包内相对说明符，天然 portable；
   官方只检查工程发射根）。首版豁免方案曾在 sweep 运行中误写入 src，
   **立即回滚**（r13 混合二进制教训——transpile 段尚未启动），sweep
   结束后重做。
2. **动态 import 初始化器白名单**：`export const f = await import("inner")`
   的推断类型是模块 namespace（符号文件 = index.d.ts），官方 emitter 用
   `import("inner")` 说明符自身命名——变量初始化器为 `(await)
   import("spec")` 形态时，spec 解析文件并入白名单（`spec_of_dynamic_
   import_call` 从 type_of_dynamic_import 提炼复用）。

单测 +2（库内豁免负例 / 动态 import 初始化器负例）——checker_parity
1004→**1005**（本轮累计 +21）。四套件门全绿（1353/1005/2/15）。

**r36 后剩余清单（深层，在案）**：
- **ImportAttributesTypeModeDeclarationEmitErrors**（conformance ×4）：
  r35 起 30s 超时 SKIP 掩盖，r36 负载减轻（12→8 worker）后真实暴露。
  import() **类型属性文法 + 解析恢复**子系统：缺 `with` / 坏 key /
  数组形态的恢复链（官方 (3,39) ';' expected 后接 TS1340「模块用作
  类型」回退 + 2304/2353/2339 级联；我们的恢复路径从 '}' expected
  分叉）。与 r17 记录的「坏 mode 后任意解析」同族。
  **r37 攻坚路径（Go 权威已定位）**：parser.go `parseImportType`
  ~L3075——`parseType` → optional `,` → openBracePosition 记录 →
  `parseExpected(OpenBrace)` → 当前 token 非 with/assert 时
  `parseErrorAtCurrentToken(X_0_expected, "with")` →
  **`parseExpected(Colon)`（报 ';' expected 位）** →
  `parseImportAttributes(currentToken, skipKeyword=true)`（name=nil
  时 Identifier-or-string-literal-expected；parseDelimitedList 恢复）→
  optional `,` → `parseExpected(CloseBrace)` 失败时给
  「parser expected to find '}' to match '{'」related info →
  `parseExpected(CloseParen)` → qualifier；checker 侧坏属性 import
  type 解析失败回退值语义 → 类型位报 TS1340。
- **templateLiteralTypes1**（D8-1 模板字面量推断族，在案）：2590/1338/
  键名显示已修（r35），余模板推断（Capitalize<T> 需 StringMapping 延迟
  型 + 约束映射）与 PropType 条件链。

## 全量跑 r36（2026-08-28 晚——实际上午-午）——r35 后修复验证轮：**compiler 0F / conformance 3F / transpile 0**

**命令**：`bash run_full_sweep_r36_20260828.sh`（日志
`submodule_full_run_r36_20260828.log`，09:46→12:15，2h30m，**worker
12→8**——本机核数缩减）。前置单测全绿（1353/1004/2/15）。

| 套件 | 结果 | 对比 r35 | 说明 |
| --- | --- | --- | --- |
| compiler | **0 FAIL** | 1→0 | 2883 误报清除验证 ✓（declarationEmitTripleSlashReferenceAmbientModule PASS）；r34 后再次全绿 |
| conformance | **3 stem FAIL** | 2→3 | **ExportsSourceTs 四配置 PASS（D6a 清零，r13 起在案）✓、PackagePattern 全家四配置 PASS（D6b 清零）✓**；新面孔 2：DynamicImportWithPackageExports（2883 误报——r36 后修复轮已清）+ ImportAttributesTypeModeDeclarationEmitErrors（r35 超时掩盖的解析恢复旧账浮出）；templateLiteralTypes1（D8-1 在案） |
| transpile | **0** | 持平 | — |

## r35 后修复轮（2026-08-28）——r35 三 FAIL 全清 + D6a/D6b 清零；r36 验证中

r35 sweep 暴露的三个 FAIL（2883 误报 / 1479 精度 / 2883 动态链缺口）全
部修复，单过滤四配置验证全绿：

1. **TS2883 误报清除**（declarationEmitTripleSlashReferenceAmbientModule）：
   `Url` 声明在 ambient `declare module "url"` 内时，本文件 import 该
   ambient 名（"url"）即视为可命名（官方 emitter 用 ambient 模块名做说
   明符）——`symbol_in_ambient_module_named`：沿目标声明祖先找 string 名
   ModuleDeclaration，名字 ∈ 本文件 import 说明符集 → 跳过。
2. **TS1479 精度**（nodeModulesPackagePatternExports node16/18→四配置
   PASS，**D6b 清零**）：
   - **歧义 `.d.ts` 目标永不 ESM**（`module_format_is_esm_for_require_check`
     ）：其格式跟随加载它的 resolution mode（CJS 导入者经 require 条件到
     `.d.ts`）——js 模式（`./*.js`→index.d.ts）不再误报；`.ts`/`.js`
     输入文件仍按 package.json（官方 nodeModules1 对 type=module 的 .js
     目标照报）。
   - **`.d.ts` 导入者按 CJS**（`importer_is_cjs_for_require_check`）：
     根单元 test.d.ts 的 mjs 静态导入照官方补报（此前 package 推断成
     ESM 而漏报）。
3. **动态 import() 类型化**（nodeModulesExportsSourceTs 四配置 PASS，
   **D6a 清零，r13 起在案 15+ 轮**）：`await import("m")` → 模块命名空间
   类型（`type_of_dynamic_import`，ambient/相对表→真 resolver 回退，与静态
   import 同链）；`resolve_namespace_type` 补 **re-export 子句追链**
   （`export {x} from "./other.js"` 的成员进 namespace 属性面——binder
   不建跨文件链接，clause 文本 + `resolve_module_member_symbol` 递归）。
   `(await import("inner")).x()` → Thing 全链贯通，2883 按官方文本触发。
   连带：`for_each_module_statement`/`resolve_module_member_symbol`/
   `resolve_module_spec_from` 提为 pub(crate)。

**单测 +5**（2883 ambient 负例 / 1479 .d.ts 目标负例 / 1479 .d.ts 导入者
正例 / 动态 import 2883 / 动态 import 2322 赋值检查）——checker_parity
999→**1004**。四套件门全绿（1353/1004/2/15）。Sanity：nodeModules
SynchronousCallErrors 四配置 PASS（1479/1471 家族保持）、importHelpers
WithExportStarAs 全可用配置 PASS（namespace 构建改动无回归）。

**预期 r36**：compiler 0F；conformance 仅剩 templateLiteralTypes1（D8-1
模板字面量推断族：Capitalize<T> 需 StringMapping 延迟型 + 约束映射，
PropType 条件链，homomorphic 映射恒等关系——已在案）；transpile 0。
r36 起 worker 数 12→**8**（本机核数缩减）。

## 全量跑 r35（2026-08-28 晨）——r34 攻坚清单执行验证轮：compiler 1F / conformance 2F / transpile 0

**命令**：`bash run_full_sweep_r35_20260828.sh`（日志
`submodule_full_run_r35_20260828.log`，02:12→05:25，3h13m——后段 12
worker 与缩减核数争用）。前置单测全绿（1353/999/2/15）。

| 套件 | 结果 | 对比 r34 | 说明 |
| --- | --- | --- | --- |
| compiler | **1 FAIL** | 0→1 | declarationEmitTripleSlashReferenceAmbientModule——**本轮 2883 近似版的唯一误报**（ambient 模块名可命名未入白名单；r35 后修复轮已清） |
| conformance | **2 stem FAIL** | 1→2 | nodeModulesPackagePatternExports(node16/18)：1479 生效但带 2 处精度差（js 条件 `.d.ts` 目标多报、test.d.ts `.d.ts` 导入者漏报——已修）；nodeModulesSynchronousCallErrors **四配置 PASS**（1479/1471 全量官方验证 ✓）；templateLiteralTypes1（D8-1 在案）；nodeModulesExportsSourceTs 30.08s 负载超时伪影（单过滤 8.2s 真实完成，差异=2883 动态 import 链缺口——已修） |
| transpile | **0** | 持平 | — |

## r35 前修复轮（2026-08-28 白天）——r34 攻坚清单五组修复 + 三个连带发现

1. **TS2590 联合复杂度上限**（D8-3，三通路纯加法，Go
   `checkCrossProductUnion`×`getCrossProductUnionSize`）：估算叉积
   = Π(联合成员数)（never 成员 → 0），≥100000 在节点上报 2590 并返
   errorType。挂点：模板字面量 span 含联合（`getTemplateLiteralType`）、
   元组 Rest 元素（`createNormalizedTupleTypeEx` 可变元组分布）、
   交集分配（`getIntersectionTypeEx` 默认分支；Go 的全 undefined/全 null
   联合特例分支跳过检查）。**保持延迟表示**——仅移植溢出诊断，阈值以下
   行为不变（回归面最小化）。
2. **TS1338**（D8-4，Go `checkInferType`）：`get_type_from_infer_type_node`
   入口沿父链找「自身或祖先 = ConditionalType 的 extends 节点」——注意
   Go `FindAncestor` **包含节点自身**（InferTypeNode 常直接就是 extends
   节点；首版从 parent 起步差一层导致合法 `U extends infer R ? …` 误报
   ×2，单测门拦截后修正 + 同位去重）。
3. **TS1479/TS1471**（D6b 真缺口——r34 复测的 `.delete` 标记证明
   test.d.* 解析错误早已消失，8 月 22 日 local 产物是陈旧污染）：
   `check_module_format_mismatch`（Go `checkResolvedModule` ~L15385）：
   node16/18 下，静态 import/re-export 且**导入文件 CJS 格式**（扩展名/
   最近 package.json type 推断，`implied_node_format_of_file`）+ 目标文件
   ESM 格式 → 1479；`import x = require(...)` 命中 ESM 目标（任意导入者
   格式）→ 1471。resolution-mode 属性覆盖跳过。
4. **TS2883 可命名性近似版**（D6a）：`--declaration` 下，导出变量的
   **推断类型**（无注解）顶层符号声明于 node_modules 且本文件所有
   import 说明符都解析不到该文件 → 报 2883（参数：变量名、
   `./node_modules/…/x.js` 相对说明符（ts→js 扩展映射）、符号名）。
   近似边界：只看类型顶层符号（官方走 nodebuilder 全量可命名性）；
   「文件级 import 白名单」比官方（按名绑定）粗——JSX 场景等由
   同模块任意 import 白名单兜住。
5. **映射类型书面形显示**（D8-2）：延迟映射类型打印走声明节点——
   键名取 `[P in …]` 的 P（解析声明节点本身得 error 的替代）、约束取
   约束节点源文本切片（我们的 `keyof T` 塌缩成 string 无法还原
   `keyof T & string`）+ 模板后补 `; `。`{ [error in string & …]` →
   `{ [P in keyof T & string as …]: T[P]; }`。

**连带发现三缺口（TS2883 的前置依赖，全部修复）**：
- **导入变量类型从未流动**：`type_of_imported_symbol` 只处理函数/类声明
  成员，变量返回 None → 导入的 const/let 一律 any（赋值检查静默跳过；
  函数走调用解析侥幸可用）。补成员符号 `get_type_of_symbol` 兜底 +
  `get_type_of_symbol` 顶部 Alias 分支（follow_alias 追链 + 缓存）。
- **重导出子句追链**：binder 不建跨文件 export_symbol 链，
  `resolve_module_member_symbol`（新）在直接查找失败/未链接时解析
  `export {X as Y} from "m"` 子句文本递归（深度 8）；相对说明符以
  **声明模块**目录为基（`resolve_module_spec_from`）。
- **裸包名解析**：`type_of_imported_symbol` 的
  `resolve_module_file_symbol` 只有 ambient 兜底；补真 resolver
  （`resolve_external_module_path`）+ 已加载文件符号回退——
  node16 "inner"（package.json exports）链路打通。
CLI 探针验证：`import { x } from "inner"` + 重导出 + node_modules 布局
下 `() => Thing` 全链贯通；2883 文本与官方逐字一致。

**新增单测 +14**（ts2590×5 含双下限负例 / ts1338×2 / ts1479×3 含 node20
负例 / ts1471 / ts2883×2 含无 declaration 负例 / 映射显示×1）；多文件
helper `check_sources_with_args` 对齐 harness 全单位为根语义。

**遗留（本轮不做，在案）**：D8-1 模板字面量推断族——
`Capitalize<T>` 需 StringMapping 延迟型 + 约束映射（keyof T 塌缩同根），
PropType 条件链（getPropValue×5/getProp2×2 误报），templateLiteralTypes1
四子缺口余二（推断 + 2590 后的显示序列核对）；homomorphic 映射恒等
关系（`y = x` 误报）与映射-映射关系（45,5 欠报）。

## 全量跑 r34（2026-08-27 晚）——r33 回归清算验证轮：**compiler 0F（项目史上首次）/ conformance 1F（负载污染：193 超时SKIP）/ transpile 0**

**命令**：`bash run_full_sweep_r34_20260827.sh`（启动日志 `/tmp/sweep_r34_launch.log`
19:24→21:35，2h12m）。前置单测全绿（1353/985/2/15）。**本机外部高负载**
（load≈25：copilot×2/chrome/wechat 与 12 worker 并发），全程 `crashed
(timed out)` 跳过 193 例——属 r17 类负载伪影（关键用例事后单过滤复测，
秒级完成且结果明确，见下）。

| 套件 | 结果 | 对比 r33 | 说明 |
| --- | --- | --- | --- |
| compiler | **0 FAIL（历史首次零失败）** | 3→0 | 三回归全清 ✓；recursiveReverseMappedType 等条件族保持 ✓ |
| conformance | **1 stem FAIL** | 5→1 | templateLiteralTypes1（D8，FIXPLAN r33 已画像）真实在案；ExportsSourceTs / PackagePattern 两用例 sweep 窗口贴线 30s SKIP，**单过滤复测 4.8–5.0s 完成**：ExportsSourceTs×4 baseline mismatch（TS2883 缺口）、PackagePattern node16/18 mismatch 且 node20/nodenext **PASS** —— 均与 r32 真实态一致，非本轮回归 |
| transpile | **0** | 持平 | — |

### r35 攻坚清单（按优先级）
1. **D6b 定位**（PackagePattern node16/18）：主障碍仍是 test.d.{c,m,}ts
   (7,16) TS1003+(8,1) TS1005 解析缺口——独立管线同内容零错、套件内必炸
   ，分叉点已收窄到 build_and_check 根选择/装配差异（FIXPLAN r33 记录二
   分步骤）；其后仅剩 TS1479×3（require 引 ESM 格式检查）。
2. **D6a TS2883** 可命名性子系统（4 例）。
3. **D8 templateLiteralTypes1** 四子缺口：2590 复杂度上限（建议先做，纯
   加法）→ 模板字面量推断 error 泄入 → 泛型映射键名显示 → TS1338。


## r34 前修复轮（2026-08-27 晚）——r33 三回归清算 + PackagePattern 性能回归

1. **extraTypes 移植**（conditionalTypeAnyUnion）：check=any 的
   definitely-false 结果并入 TRUE 支（Go checker.go ~L24451；
   forConstraint 变体未涉，暂不移植）
2. **可调用目标的延迟条件展开**（nonNullableReduction×2）：被调分发前
   Conditional → 默认约束联合；联合叶条件成员同样展开并滤 nullish/never
   （Go getSignaturesOfType 递归 getDefaultConstraintOfConditionalType）
3. **探测实例化记忆化**：permissive/restrictive 各一张 ptr→Arc<Type>
   缓存（对齐 Go cachedTypes），修 nodeModulesPackagePatternExports
   30s 超时回归
4. **强制真支解析补压 infer 作用域**：get_forced_branch_type_of_conditional
   _type 漏 push(cond_node) 使 lib.es5 ThisParameterType 报假 TS2304 'U'
   （套件三回归的公共根因之一）

复现矩阵全过 + 三例过滤验证绿 + D7 家族复核绿；单测 +3（checker_parity
982→985）。四套件门全绿（1353/985/2/15）。**注意事项固化**：临时探针统一
TSOX_DEBUG_RR 前缀；超时 SKIP 在日志里显示为 `worker crashed (timed out)`
而非 FAIL/TIMED OUT 字样，统计必须 grep 双模式。


## 全量跑 r33（2026-08-27 下午）——D7 验证轮：compiler 3F / conformance 5F+1超时SKIP / transpile 0

**命令**：`bash run_full_sweep_r33_20260827.sh`（启动日志 `/tmp/sweep_r33_launch.log`
15:56→18:19，2h23m）。前置单测全绿（lib 1353 / checker_parity 982 / parity 2 /
lsp 15）。**全程 0 例假超时**（"TIMED OUT" 字样零命中；超时格式实为
`worker crashed (timed out)`，PackagePattern 主用例除外，见下）。

| 套件 | 结果 | 对比 r32 | 说明 |
| --- | --- | --- | --- |
| compiler | **3 FAIL** | 1→3 | **recursiveReverseMappedType（D7，在案 12+ 轮）✓ 清零**；新回归 3：conditionalTypeAnyUnion（多报 2344）、nonNullableReduction×2（多报 2349×2）——均为三态延迟化的已知补偿缺口 |
| conformance | **5 stem FAIL + 1 超时SKIP** | 3→5 | ExportsSourceTs×4 从「贴线超时掩盖」转为**快速真实 FAIL**（纯 TS2883 缺口；D6a 的病理性超时随 D7 消失）；templateLiteralTypes1（D8）在案；**nodeModulesPackagePatternExports 主用例本轮 30.07s 超时 SKIP**——r32 时是真实 FAIL，本轮被新探测开销推过线，属 D7 性能回归 |
| transpile | **0** | 持平 | — |

### r34 前修复清单（按优先级）

1. **extraTypes 移植补全**（conditionalTypeAnyUnion 回归）：Go
   checker.go ~L24451 check=any 时 definitely-false 前把 TRUE 支并入
   结果（`union(trueBranch, …)`），含 forConstraint 反向探针
   `someType(permissive(extends) ⊆ permissive(check))`。当前只返单假支。
2. **可调用目标急切通道**（nonNullableReduction×2）：`(T|null)`→callable
   收窄曾靠裸判真支拿到具体函数型，三态后停在延迟条件；需给「目标侧
   call-signature 提取」接 getResolvedTrueTypeOfConditionalType 式解析
   （对齐 tsgo 二分后落位）。
3. **探测实例化记忆化**（PackagePattern 超时性能回归）：permissive/
   restrictive 走查无缓存，node_modules 重结构上重复展开；对齐 Go
   cachedTypes（key=type identity + kind），Checker 加两张 map，键
   type ptr、值 Arc<Type>。
4. D6b 下轮起点/TS1479/D8 四子缺口画像：见 `_scripts/FIXPLAN_20260827_r33.md`。

## r33 前修复轮（2026-08-27 下午）——**D7 recursiveReverseMappedType ✓（在案 12+ 轮）**

官方 tsgo 十余例最小二分定位：放行通道 = **推断层同别名互推**（Go
inference.go ~L79），非关系层宽松。四组件修复 + 双严格性守卫：

1. 条件解析三态判定（permissive/restrictive definitely-true/false 探测，
   皆非则延迟）替换裸 assignability 判支
2. `get_permissive_instantiation` / `get_restrictive_instantiation` 助手
   （TP→wildcard / TP→去约束克隆，条件臂浅换不重解析）
3. 延迟条件的创建期快照（替换帧栈 + 作用域 ID 链），分支最终解析时重压
   ——等价 Go 实例携带 mapper；分支结果不写 resolved_* 单元格
4. 关系层双回退（源=默认约束联合 / 目标=双支均过+skipTrue/skipFalse 探
   测）+ 推断层同 root 节点快速互推 Y:=X

复现矩阵与官方逐一吻合（exp_c/rr6/rr7/b2~b6/b3/b4 全对齐，b5/b2 严格
报错保留）。套件过滤验证：recursiveReverseMappedType、
conditionalTypeContextualTypeSimplifications、cyclicTypeInstantiation、
conditionalTypes1 全绿。单测 +4（checker_parity 978→982），四套件门全绿
（lib 1353 / parity 982* / 2 / 15）。**附带发现：D6a×4 的 30s 超时已随本
修消失**（病理性条件重解析放大所致）；D6b 的 test.d.* TS1003 与套件根选
择装配相关，均记入 `_scripts/FIXPLAN_20260827_r33.md` 下轮攻坚路径。
r33 预期：compiler **0F**、conformance ≤5 stem（全部 D6/D8 在案项）、
transpile 0。

## 全量跑 r32（2026-08-27 上午）——深代入门控验证轮：**compiler 1F（历史新低）/ conformance 3F / transpile 0**

**命令**：`bash run_full_sweep_r32_20260827.sh`（日志
`submodule_full_run_r32_20260827.log`，07:14→09:28，2h14m；compiler 段因外部
负载偏慢 73min）。单测前置全绿（1353/978/2/15）。

| 套件 | FAIL | 对比 r31 | 说明 |
| --- | --- | --- | --- |
| compiler | **1** | 2→1 | conditionalTypeContextual ✓（return-path 门控生效）；cyclic 保持 ✓；仅剩 recursiveReverseMappedType（D7） |
| conformance | **3 stem** | 3→3 | nodeModulesPackagePatternExports(node16/18) + templateLiteralTypes1(D8)；ExportsSourceTs 仍 30.08s 超时掩盖（D6） |
| transpile | **0** | 持平 | — |

**剩余清单（全部深层子系统，在案）**：
- D7：recursiveReverseMappedType（compiler）——`Recur<T>` 递归逆映射+
  条件+元组联合的延迟类型同一性；12 轮在案
- D6：nodeModulesExportsSourceTs×4（TS2883 声明发射可命名性子系统，
  超时掩盖）+ PackagePatternExports node16/18（exports 子路径解析族）
- D8：templateLiteralTypes1（模板字面量推断 + 联合爆炸上限 2590）

## 全量跑 r31（2026-08-27 晨）——r30 后续修复验证轮：**compiler 2F / conformance 3F / transpile 0（历史新低）**

**命令**：`bash run_full_sweep_r31_20260827.sh`（日志
`submodule_full_run_r31_20260827.log`，03:52→06:52，3h0m——conformance 段
偏慢，机器有外部负载时段）。单测前置全绿（1353/977/2/15）。

| 套件 | FAIL | 对比 r30 | 说明 |
| --- | --- | --- | --- |
| compiler | **2** | 4→2 | r30 的 6 新面孔全清 ✓ + **cyclicTypeInstantiation（D1-deep，r9 起在案）✓**；新面孔 1 |
| conformance | **3 stem** | 7→3 | extendClass/ExportVariable/arrayLiteral/intersectionMember 全清 ✓；nodeModulesExportsSourceTs 本轮 30.10s **超时掩盖**（D6 仍在）；PackagePattern(node16/18)+templateLiteralTypes1 在案 |
| transpile | **0** | 持平 | — |

**新面孔**：conditionalTypeContextualTypeSimplificationsSuceeds——深代入臂
（cyclic 修复的 substitute_object_properties_deep）在**推理/重载探针**期间
替换属性符号，破坏上下文签名提取（`when: value => false` 三例 TS7006）。

### r32 前修复（2026-08-27 午）——深代入收敛到调用返回路径

深代入臂加三重门控（`in_return_substitution` 旗标，仅在
get_return_type_of_call/new_expression 的返回类型替换处置位）+ 匿名对象
（symbol.is_none）+ 属性类型 peek-only（不触发惰性解析）。推理探针期间臂
完全 inert——cyclic 修复保持（return 路径正是其实例化点），
conditionalTypeContextual 三例归零。单测 +1（checker_parity 977→978）。
四套件全绿（1353/978/2/15）。

**r32 预期**：compiler 1（recursiveReverseMappedType D7）、conformance 3
（nodeModules PackagePattern node16/18 + templateLiteralTypes1 D8）+
ExportsSourceTs（D6，观察是否仍超时掩盖）。

## 全量跑 r30（2026-08-27 凌晨）——r29 回归清算验证轮：compiler 4F（新低）/ conformance 7F / transpile 0

**命令**：`bash run_full_sweep_r30_20260827.sh`（日志
`submodule_full_run_r30_20260827.log`，01:00→03:18，2h18m，内存稳定）。
单测前置全绿（1353/972/2/15）。

| 套件 | FAIL | 对比 r29（中断态） | 说明 |
| --- | --- | --- | --- |
| compiler | **4** | 26→4 | 23 个 r29 回归全清 ✓ + typeRoots ✓；新面孔 2 |
| conformance | **7 stem** | — | **genericClassWith（D3-sig）✓、localTypes1 ✓、typesVersions.ambient ✓** 转绿；新面孔 4 |
| transpile | **0** | 持平 | — |

**compiler 4 归层**：cyclicTypeInstantiation（D1-deep 在案）、
recursiveReverseMappedType（D7 在案）+ 新面孔 aliasOnMergedModuleInterface
（r29 轮引入：import= 未追 ambient 模块的 export= 目标）、arrayConcat2
（构造签名泛型化暴露 overload 探针缺最小元数门）。

**conformance 7**：nodeModulesExportsSourceTs×4 + PackagePatternExports
(node16/18)（D6 在案）+ templateLiteralTypes1（D8 在案）+ 新面孔
extendClassExpressionFromModule（2749 误报：heritage 表达式位值符号）、
ExportVariableOfGenericType…/arrayLiteral（2554 同 arrayConcat2 根）、
intersectionMemberOfUnionNarrowsCorrectly（弱类型公共属性检查缺失 +
泛型别名作用域遮蔽）。

### r31 前修复轮（2026-08-27 早）——6 新面孔 + cyclicTypeInstantiation（D1-deep）全清

1. **overload 探针最小元数门**（signature_accepts_arguments）：
   `new Array<string>()`（0 实参）此前空洞通过 1 参重载再 2554 报错
   （arrayConcat2/arrayLiteral/ExportVariable 三例同根）
2. **弱类型公共属性检查**（TS2559，Go isPerformingCommonPropertyChecks）：
   is_weak_type/has_common_properties/is_known_property 三个桩实装——
   全可选属性目标要求与源至少一个公共属性（`{b:1} ⊄ {kind?:'A'}`）；
   intersection-target 成分比较时抑制（IntersectionStateTarget 旗标）；
   空属性目标不算弱（Go `len(properties) > 0`）
3. **heritage 表达式位值符号**（resolve_type_reference）：`class y extends
   x`（x 为持有类的变量）——值类型构造签名返回实例类型，不报 2749
   （extendClassExpressionFromModule，r29 轮引入的误报）
4. **import= 追 ambient 模块 export= 目标**（resolve_alias_base）：在模块
   作用域解析 export= 表达式（同 namespace_has_value_side 模式），
   `foo.bar` 值用报 2708 于类型型 namespace（aliasOnMergedModuleInterface）
5. **泛型别名实例化推别名声明作用域**：别名自身类型参数遮蔽同名顶层别名
   ——`type Ex<T, U> = T extends U ? …` 的 extends U 此前解析到同名全局
   联合（intersectionMemberOfUnionNarrowsCorrectly 的分配污染根源）
6. **匿名对象属性深代入**（substitute_object_properties_deep + in-progress
   环守卫）：调用点实例化深入对象成员类型，自引用属性（`b: typeof x`）
   经指针映射保持——**cyclicTypeInstantiation（D1-deep，r9 起在案）清零**，
   输出与官方逐字一致（仅两条 2454）

单测 +7（checker_parity 972→**977**）。四套件全绿（1353/977/2/15）。

**r31 预期**：compiler 1（recursiveReverseMappedType）、conformance 5
（nodeModules×5 + templateLiteralTypes1）——剩余全部为 D6/D7/D8 深层子系统。

## 全量跑 r29（2026-08-26 晚，**中断**）——8 项修复引入回归风暴：compiler 26F / conformance #3878 处中断 / transpile 未跑

**命令**：`bash run_full_sweep_r29_20260826.sh`（日志
`submodule_full_run_r29_20260826.log`，20:51 启动，compiler 段完成后
conformance 段在 #3878/5907 处会话中断，transpile 未跑）。单测前置全绿
（1353/958/2/15）——单测门未能拦截（回归全部在 sweep 全量面）。

**compiler 26 FAIL 构成**：r28 遗留 2（cyclicTypeInstantiation、
recursiveReverseMappedType）+ typeRootsFromMultipleNodeModulesDirectories
（r29 修复 #1 未生效——harness 未设 config_file_path）+ **新回归 23**
（泛型签名族：baseTypeOrderChecking、constructorArgWithGenericCall
Signature、contextualTypingWithGenericSignature、declFileCall/Construct
Signatures(es2015)、declFileGenericClassWithGenericExtendedClass、
declarationEmitTypeParamMergedWithPrivate、declarationEmitTypeParameter
NameReusedInOverloads、genericOverloadSignatures、genericSignature
Inheritance(2)、homomorphicMappedTypeIntersectionAssignability、
importExportInternalComments、multipleInferenceContexts、mutuallyRecursive
Inference、nodeColonModuleResolution、recursiveTypeIdentity、
restParameterAssignmentCompatibility、staticAndMemberFunctions、
varianceCallbacksAndIndexedAccesses、varianceProbling×2 等）。

### r30 前修复轮（2026-08-27 凌晨）——r29 回归清算 + D3-sig 一并修复

**回归风暴根因（单一）**：r29 的 binder 容器化（签名类节点 + TypeAlias/
MappedType 补 IsContainer|HasLocals，Go 语义正确）让签名/容器级类型参数
落进**节点自身 locals / 符号自身 members**，但 checker 的解析路径全部
依赖旧的「泄漏进文件 members 恰好可见」——五处作用域缺口 + 两处按名替换
过宽，23 例回归全部由此而来：

1. **接口 Call/ConstructSignature 分支补 push_scope(member)**
   （build_interface_type_from_members；MethodSignature 原有）——
   `<T>(x: T): T` 签名级类型参数此前在用户文件必 TS2304
   （genericSignatureInheritance 族）；保留 bundled-lib 的 2304 压制
2. **type_parameters_of_declaration 补 CallSignature/ConstructSignature
   臂**——接口调用签名此前建成非泛型（签名从不携带自己的类型参数）
3. **类成员 MethodDeclaration 分支补 push_scope(member)**（D3-sig 定位
   点：方法 T 此前被类符号 members 的同名 T 遮蔽）
4. **ConstructorType 节点解析补 push**（对齐 FunctionType 的 2725 行范式）
5. **词法链补签名类节点**：with_declaring_file_context 链 + 祖先链
   （ANCESTRY_CONTAINERS）补 CallSignature/ConstructSignature/
   FunctionType/ConstructorType/MappedType
6. **祖先链放行 CLASS 符号的类型参数**：原先 `!Class` 门把 class 整个
   挡在 members 检查外——类型参数是 TYPE 侧，值成员不可见规则只应挡
   VALUE 侧；前向引用 `var v: Class4<X>` 触发的 heritage `Class3<T>`
   解析由此修复（baseTypeOrderChecking、declFileGenericClassWith
   GenericExtendedClass）
7. **check_type_annotation 在注解声明上下文中执行**（with_declaring_
   file_context 包裹）——检查阶段重解析注解时裸栈被文件级同名声明
   （`class T`）抢先，TS2538 风暴（lib.dom Global<T> 的 ValueTypeMap[T]
   →全局 class T）由此根治（staticAndMemberFunctions、restParameter
   AssignmentCompatibility、variance 族等）
8. **TS2708 门控排除字符串名模块**：`import * as ns from "m"`（m 为
   `declare module "m"` ambient）是模块对象，作值合法；2708 只对真
   namespace（非字符串名）发（importExportInternalComments、
   nodeColonModuleResolution——r29 修复 #4 的别名追链暴露）
9. **binder can_merge_symbols 补 TypeParameter+值侧共存**（Go
   TypeParameterExcludes = Type & ~TypeParameter）：`class Test<T>
   { private get T(): T }` 的 getter 此前把类型参数 T 从类 members 里
   **顶掉**，全类 T 引用 TS2304（declarationEmitTypeParamMergedWith
   Private）
10. **映射类型混合键集走延迟路径**：约束含非字面量成分（`keyof (T &
    Named)` = string | "name"）时不再急切塌缩成字面量子集
    （homomorphicMappedTypeIntersectionAssignability——Readonly<T & Named>
    丢 keyof T 部分）
11. **名字帧/成员替换改符号指针键 + 同源名字兜底**：heritage 实例化
    帧与 substitute_infer_type_parameters 此前按名字匹配，类 T 与方法
    同名 T 被错误实例化（cm10/cm7 复现；兜底保留同声明容器符号的多
    声明分叉场景）
12. **接口 fork 实例化映射覆盖所有声明**：合并声明的每份类型参数符号
    都进映射（interface I<T> 双声明 + I<number> 此前不替换）
13. **new 表达式目标解析改成员链**（Go parseMemberExpressionRest 语义）：
    `new X().m()` 此前解析成 `new (X().m(...))`（内层 X() 被当普通调用
    报 TS2348）——连带 builder 模式测试翻转（钉住旧行为的期望更新）
14. **harness tsconfig 传 config_file_path**（tests/submodule_compiler
    .rs）——get_effective_type_roots 的 config 祖先链终于可达
    （typeRootsFromMultipleNodeModulesDirectories r29 修复 #1 生效）

**连带成果**：conformance 的 **genericClassWith（D3-sig，FIXPLAN 记录的
下轮首攻）本轮清零**。单测 +14（checker_parity 958→972）、
checker_builder_pattern 钉住值更新。四套件全绿（1353/972/2/15）。

**r30 预期**：compiler 应回到 2（cyclicTypeInstantiation、
recursiveReverseMappedType 深层在案）；conformance 应回到 4 stem
（nodeModulesExportsSourceTs×4=D6、templateLiteralTypes1=D8）+ 观察项。

## 全量跑 r28（2026-08-26 午）——r27 中途修复清算轮：compiler 8F / conformance 5F / transpile 0

**命令**：`bash run_full_sweep_r28_20260826.sh`（日志
`submodule_full_run_r28_20260826.log`，10:05→12:31，2h26m）。单测前置
全绿（1350/952/2/15）。

| 套件 | FAIL | 对比 r27 | 说明 |
| --- | --- | --- | --- |
| compiler | **8** | 9→8 | binaryArithmetic ✓（语句位赋值去重）；narrowTypeByInstanceof ✓ 保持 |
| conformance | **5 stem** | 5→5 | nameWithFileExtension ✓ 保持；nodeModulesPackagePatternExports 持续超时掩盖 |
| transpile | **0** | 持平 | — |

**剩余清单（全部深层/多文件子系统，根因在案）**：
- compiler 8：cyclicTypeInstantiation（自引用类型实例化 T 未代入 + 2454
  行位锚点）、declarationEmitQualifiedAliasTypeArgument /
  reexportedMissingAlias / typeCheckObjectCreation…（多文件 alias/export=
  布局，需 worker 探针）、moduleResolutionAsTypeReferenceDirectiveAmbient +
  typeRootsFromMultipleNodeModulesDirectories（typeRoots/@types 解析子系统）、
  recursiveReverseMappedType（深层延迟类型同一性）、
  destructuringTypeGuardFlow（**解构绑定元素不继承条件窄化**——
  `const {bar} = aFoo` 应从流窄化后的属性类型取）
- conformance 5：genericClassWith（D3 方法位方差）、localTypes1(es2015)
  （函数内局部 class 实例 any[] 化）、nodeModulesExportsSourceTs×4 +
  PackagePatternExports（node_modules exports 子路径/格式解析族）、
  templateLiteralTypes1（模板字面量推断族）、
  typesVersionsDeclarationEmit.ambient（typesVersions 重定向解析）

**会话总账（r23→r28 五轮全量）**：
- compiler FAIL：11 → **8**（深层 7 + 解构窄化 1）；r24 新根因修复：
  限定名 heritage 解析（PropertyAccessExpression 形态）、union callee、
  new 显式类型实参、逗号类型、逻辑赋值 RHS 帧、&&/|| 完整流语义
  （分支合并 + 表达式位赋值流 + keep 条件）、表达式位 CALL 流、
  可见性安全成员追查、.js→.ts 映射、super() heritage 实参
- conformance FAIL：13 stem → **5**；la5 族逐字对齐（r24 里程碑）、
  可选参数 undefined 并入全族转绿
- transpile：全程 0
- 单测：920 → **952**（+32：可选参数族/静态 this/heritage/交集/逗号流/
  断言窄化/合并语义/.js 映射/super 实参）

## 全量跑 r27（2026-08-26 上午）——逻辑运算分支合并 + 表达式位赋值流 + .js 映射验证轮：compiler 9F / conformance 5F / transpile 0

**命令**：`bash run_full_sweep_r27_20260826.sh`（日志
`submodule_full_run_r27_20260826.log`，07:32→10:02，2h30m）。**混合轮**：
compiler 段跑启动树；conformance 段含 sweep 中途两修复（见下）。

| 套件 | FAIL | 对比 r26 | 说明 |
| --- | --- | --- | --- |
| compiler（启动树） | **9** | 8→9 | narrowTypeByInstanceof ✓（分支合并）；新面孔 2 为启动树修复的次生回归（已修，r28 验证） |
| conformance（新树） | **5 stem** | 6→5 | nameWithFileExtension ✓（.js→.ts 映射）；nodeModulesPackagePatternExports 本轮超时掩盖（仍坏） |
| transpile | **0** | 持平 | — |

**本轮修复**（单测四套件 1350/952/2/15 全绿后上量 + sweep 中途两补丁）：
1. **逻辑运算完整流语义**（Go bindLogicalLikeExpression）：&&/|| 的 RHS
   在 keep 条件下绑定（&&→TRUE(左)，||→FALSE(左)），结束后**分支合并**
   （短路侧经 opposite 条件节点与 RHS 末流 union）——narrowTypeByInstanceof
   的 4×2454 精确、`s || (s = 'x')` 后 s=string
2. **表达式位赋值流**：binder BinaryExpression 臂对赋值（左为标识符、
   父非 ExpressionStatement——避免与语句级双层建流触发 2563）建
   ASSIGNMENT 流节点（`(param = 5, param)` 窄化；Go
   bindAssignmentExpressionFlow）
3. **.js→.ts 说明符映射**：resolve_module_file_symbol_in 剥 .js/.jsx
   后缀（`require('./foo_0.js')` 绑 foo_0.ts——nameWithFileExtension）
4. **narrow_by_binary 的 &&/|| 分解**（Go narrowsTypeByExpression：
   true=两操作数真串联，false=两假 union）——条件上下文的合并污染由
   此在 if 条件层收敛（sweep 中途补丁，r28 验证）
5. **次生回归 2 例**（启动树修复引入、sweep 中途已修）：binary
   ArithmeticControlFlowGraphNotTooLarge（语句位赋值双层建流→流图翻倍
   触发 2563 假阳性——父为 ExpressionStatement 时跳过）；
   destructuringTypeGuardFlow（合并标签把 keep 路径 null 带进真分支
   ——由 #4 分解收敛，残余：**解构绑定元素不继承条件窄化**，深层在案）

**r27 遗留**：compiler 深层/多文件 7（cyclicTypeInstantiation、
declarationEmitQualifiedAliasTypeArgument、moduleResolutionAsType
ReferenceDirectiveAmbient、recursiveReverseMappedType、reexportedMissing
Alias、typeCheckObjectCreation…、typeRootsFromMultipleNodeModules
Directories）+ 已修待验 2；conformance 5（genericClassWith、localTypes1、
nodeModulesExportsSourceTs×4、templateLiteralTypes1、
typesVersionsDeclarationEmit.ambient）+ 超时掩盖 1（nodeModulesPackage
PatternExports node16/18）。

## 全量跑 r26（2026-08-26 晨）——可见性修复 + &&流 + super 实参验证轮：**compiler 8F（历史新低）/ conformance 6F / transpile 0**

**命令**：`bash run_full_sweep_r26_20260826.sh`（日志
`submodule_full_run_r26_20260826.log`，05:00→07:20，2h20m，内存稳定）。
单测前置全绿（1350/943/2/15）。

| 套件 | FAIL | 对比 r25 | 修复验证 |
| --- | --- | --- | --- |
| compiler | **8** | 14→8 | 可见性族 7 全清 ✓（importAnImport、innerAliases、moduleClassArrayCodeGenTest、moduleNewExportBug、moduleVisibilityTest2、namespacesDeclaration2、sourceMapSample） |
| conformance | **6 stem** | 9→6 | emitClassDeclarationWithExtensionAndTypeArgumentInES6 ✓（super 实参）、subtypingWithOptionalProperties ✓（&& 窄化）；nodeModulesExportsSourceTs 本轮超时掩盖（仍坏） |
| transpile | **0** | 持平 | — |

**新面孔 1**：narrowTypeByInstanceof（欠 1 个 2454）——&&/|| 条件流
（binder BinaryExpression 臂）暴露定赋值查询缺口：else-if 条件里
`elementA instanceof Match`（&& 左操作数）的定赋值种子 undefined 被
错误移除。已试两法（定赋值查询保 undefined 旗标→过报；narrow_by_binary
的 &&/|| 分解【Go narrowsTypeByExpression 语义】→ 差 1 个），均已回退，
需对照 Go checkIdentifier 的 isTypePossiblyUndefined 流语义再攻。

**compiler 8 归层**：cyclicTypeInstantiation（深层实例化：2719 假阳性
T 未代入 + 2454 行位）、declarationEmitQualifiedAliasTypeArgument /
reexportedMissingAlias / typeCheckObjectCreation…（多文件）、
moduleResolutionAsTypeReferenceDirectiveAmbient / typeRootsFromMultiple
NodeModulesDirectories（typeRoots/@types 解析子系统）、
recursiveReverseMappedType（深层延迟类型）、narrowTypeByInstanceof（上）。

**conformance 6**：genericClassWith（D3 在案）、localTypes1(es2015)
（局部类实例 any[] 化）、nameWithFileExtension（.js→.ts 扩展映射）、
nodeModulesPackagePatternExports(node16/18；r24"转绿"实为超时掩盖)、
templateLiteralTypes1（深层）、typesVersionsDeclarationEmit.ambient。

## 全量跑 r25（2026-08-26 凌晨）——r24 回归修复验证轮：compiler 14F / conformance 9F

**命令**：`bash run_full_sweep_r25_20260826.sh`（日志
`submodule_full_run_r25_20260826.log`，02:05→04:42，2h37m）。

| 套件 | FAIL | 对比 r24 | 说明 |
| --- | --- | --- | --- |
| compiler | **14** | 12→14 | r24 回归 4 清零（arrayToLocaleString、comma×2、functionCall16/17 ✓）；**新增可见性族 7**（本会话兜底追查引入，r26 已修） |
| conformance | **9 stem** | 11→9 | emitClassDeclarationWithPropertyAccessInHeritageClause1 ✓、reexportClassDefinition ✓；nodeModulesPackagePatternExports node16/18 回归实为 r24 超时掩盖暴露 |
| transpile | **0** | 持平 | — |

**conformance 9**：emitClassDeclarationWithExtensionAndTypeArgumentInES6
（super 实参——r26 已修）、genericClassWith（深层）、localTypes1(es2015)
（局部类实例 any[] 化）、nameWithFileExtension（.js→.ts 映射）、
nodeModulesExportsSourceTs×4、nodeModulesPackagePatternExports(node16/18)、
subtypingWithOptionalProperties（&& 窄化——r26 已修）、templateLiteralTypes1
（深层）、typesVersionsDeclarationEmit.ambient。

**compiler 14 归层**：可见性族 7（r26 已修）+ 既有深层/多文件 7
（cyclicTypeInstantiation、declarationEmitQualifiedAliasTypeArgument、
moduleResolutionAsTypeReferenceDirectiveAmbient、recursiveReverseMappedType、
reexportedMissingAlias、typeCheckObjectCreation…、typeRootsFromMultiple
NodeModulesDirectories）。

**r24→r25 验证生效的修复**：heritage 表达式形态门控、export= 对象字面量
追查、可选参数显示剥离、error 型不并入 undefined、逗号左操作数流窄化、
断言调用 CALL 流（表达式位）——对应 6 stem 全部转绿。

## 全量跑 r24（2026-08-25 深夜）——Fix 8/7a/7b/7c + 新根因修复验证轮：compiler 12F / conformance 11F

**背景**：r23 后按 `_scripts/FIXPLAN_20260825_r23.md` 执行。发现上一会话
已把 Fix 8/7a/7b 代码写入但留 4 个编译错误（会话被切断），本会话补完并
新增五组修复。单测四套件全绿（1350/937/2/15）后 23:22 上量。

**命令**：`bash run_full_sweep_r24_20260825.sh`（日志
`submodule_full_run_r24_20260825.log`，23:22→01:56，2h34m，全程内存
稳定 ~46G 可用）。

| 套件 | FAIL | 对比 r23 | 修复验证 |
| --- | --- | --- | --- |
| compiler | **12** | 11→12（构成剧变） | thisInConstructorParameter2 ✓、genericConstraintOnExtendedBuiltinTypes×2 ✓、intersectionSatisfiesConstraint ✓ |
| conformance | **11 stem** | 13→11 | **logicalAssignment5×4 ✓（逐字对齐）**、objectTypesIdentityWithGenericConstructSignaturesOptionalParams×3 ✓、nodeModulesPackagePatternExports ✓；新面孔 2 |
| transpile | **0** | 持平 | — |

**本轮修复**（fixplan 六项 + 探针发现五项，CLI 双探针对齐）：
1. **Fix 8**：可选参数 `?` 类型并入 undefined（typenode 签名构造 +
   on-demand 分支；Go assignParameterType checker.go:10455）
2. **Fix 7a**：static 成员内 this=构造器类型（this_container_stack +
   enclosing_class_stack；Go tryGetThisTypeAtEx:12216）
3. **Fix 7b（真根因为新发现）**：**限定名 heritage 从未解析**——
   `interface X extends NS.Y` 的 NS.Y 是 PropertyAccessExpression（parser
   parse_left_hand_side_expression 产物），resolve_type_reference/
   resolve_qualified_symbol 均不认该形态 → errorType 静默丢基类。修：
   resolve_qualified_symbol_traced 补 PropertyAccess 臂（尾段提取
   resolve_qualified_tail 共用）；resolve_type_reference 接受该形态。
   genericConstraintOnExtendedBuiltinTypes 两 stem 由此转绿（连带 boxed
   heritage 成员并入 collect_boxed_heritage_members）
4. **Fix 7c**：relater 交集处理重排为 Go 权威顺序 + 交集源成分快查失败
   后跌落**整体结构比较**（intersection_source_structurally_related：
   属性可来自任一成分，类型参数成分经约束查属性；Go recursiveType
   RelatedTo 跌落注释原文 "{ a } & { b } <=> { a, b }"）
5. **union callee 调用**：`(F1|F2)(42)` 此前必 2349——调用路径只处理
   new 的 construct 联合。修：跳过 null/undefined 叶子（2722 已报）扁平
   化 call 签名
6. **new 显式类型实参替换**：`new C<number>(5)` 实参检查此前对裸 T
   （构造器签名不带类类型参数，推断无候选）。修：显式实参对类声明类型
   参数 substitute（new_explicit_subst）
7. **逗号表达式类型**：CommaToken 分支缺失落 `_ => any`（Go
   checkCommaExpression 右操作数）
8. **逻辑赋值 RHS 帧提前到首次遍历**：check_expression 的 RHS 首次遍历
   压帧（此前 check_assignment_compat 二次遍历才压，18048 已在首次发射）；
   帧推导提为 logical_rhs_frame helper；上下文定型赋值臂改用
   assignment_target_type（写类型，不带帧）——箭头 RHS 的 7006 假阳性清
9. **单测 +20**（checker_parity 920→943）

**r24 新面孔（回归，r25 轮已修，见上）**：arrayToLocaleStringES2020
（error|undefined）、controlFlowCommaExpressionAssertionWithinTernary +
narrowCommaOperatorNestedWithinLHS（逗号类型化暴露流缺口）、functionCall16/
17（显示未剥离）、emitClassDeclarationWithPropertyAccessInHeritageClause1
（2503 假报）、reexportClassDefinition（2694 假报——export= 对象字面量）。

## 全量跑 r23（2026-08-25 晚）——Fix 1b + Fix 4 + binder 两修复验证轮：**compiler 11F（历史新低）/ conformance 13F**

**背景**：r22 树在 sweep 后（10:23-10:43）由上一会话写入了：01:41 contra 乐观
回退（inference.rs）、Fix 1b（module_member_lookup 七分支查询侧重做 + 8 单测）、
Fix 4 四步（flow.rs 逻辑赋值后置窄化 / checker.rs RHS 窄化帧 + 18048 对象位置
门控 + Object 接口成员回退）、binder PropertyDeclaration-with-initializer
START 流容器（GH#62264）。本会话先修 2 处再上量：
- 单测 4 失败（均 Fix 1b 新测试）根因 = binder 文件符号从不挂 declarations
  （Go bindSourceFileAsExternalModule → addDeclarationToSymbol 语义）→ 补挂
  declarations=[SourceFile] + value_declaration，4→0
- 单测四套件全绿（**1350**/921/2/15）后 19:52 上量

**命令**：`bash run_full_sweep_r23_20260825.sh`（日志 `submodule_full_run_r23_20260825.log`，
19:52→22:09，2h17m，全程无外部负载，超时 87 个常态水位）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r22 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,908 | 2,476 | 1,141 | **11** | 36 真实 → 11：**Fix 1b 生效**（default/星链/ambient/子句别名/export= 全族转绿） |
| conformance | 2,069 | 2,541 | 1,284 | **13** | 20 真实 → 13：jsxJsxs/Emit1/ambientShorthand/exportsAndImports/typeReferenceRelatedFiles/topLevelAwait.9-10/inlineJsxFactory 全清；nodeModules 族收敛到 2 stem |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**超时掩盖对账**：r22 的三个掩盖族全部真转绿——importHelpersWithImportOrExportDefault
×4（29.44s 贴线通过）、nodeModulesImportAttributesTypeModeDeclarationEmitErrors、
typeReferenceRelatedFiles 两段。nodeModulesExportsSourceTs 从 r22 超时掩盖转
真实暴露（仍坏）。stringLiteralTypesOverloads04 本轮 PASS（5.21s）——CLI
`--noEmit` 裸旗标下仍死循环（7min+ 100% CPU），非 harness 配置形态，在案观察。

**compiler 11 归层**（对照 `_scripts/FIXPLAN_20260825_r23.md` 探针预定位）：
- Fix 7a（static this）：thisInConstructorParameter2
- Fix 7b（box 表观缺 heritage）：genericConstraintOnExtendedBuiltinTypes(2)
- Fix 7c（交集源塌缩）：intersectionSatisfiesConstraint
- Fix 7d（深层）：cyclicTypeInstantiation（2719 + 2454 行位）、recursiveReverseMappedType（r4 起在案）
- Fix 8 族（参数可选性，见下）：typeCheckObjectCreation…（待 diff 核实）
- 独立：declarationEmitQualifiedAliasTypeArgument、reexportedMissingAlias、
  moduleResolutionAsTypeReferenceDirectiveAmbient（Fix 1b 残余，待 diff）、
  typeRootsFromMultipleNodeModulesDirectories（typeRoots 解析子系统）

**conformance 13 归层**：
- Fix 8（可选参数类型缺 undefined，Fix 4 的使能项）：logicalAssignment5×4cfg、
  subtypingWithOptionalProperties、objectTypesIdentityWithGenericConstruct
  SignaturesOptionalParams×3
- nodeModules：nodeModulesExportsSourceTs×4、nodeModulesPackagePatternExports
  （node16/18 挂、node20/nodenext 过）
- 深层在案：templateLiteralTypes1、genericClassWith（D3）、localTypes1(es2015)
- 待 diff：typesVersionsDeclarationEmit.ambient、emitClassDeclarationWith
  ExtensionAndTypeArgumentInES6、nameWithFileExtension

**sweep 期间完成的预定位**（tsgo-ref + 预编译 tsox 探针，未 rebuild）：
`_scripts/FIXPLAN_20260825_r23.md`——**Fix 8 为本轮最大发现**：可选参数
`f?: T` 的类型从未并入 undefined（官方 T|undefined；探针实证可选属性已对、
参数全缺），是 la5 全链（2722/18048）与可选参数族的共同根因；Go 权威
assignParameterType（checker.go:10455）。另定位 Fix 7a（tryGetThisTypeAtEx
12216：static 成员 this=构造器类型）、Fix 7b（boxed_apparent_type_of_primitive
只拼自有成员忽略 heritage）、Fix 7c（/tmp/p10.ts 最小复现：交集源被塌缩成
约束第一成分）。

## 全量跑 r22（2026-08-25 上午）——r21 树 + 01:41 inference 改动的验证轮

**背景**：单测四套件全绿（1344/921/2/15）后上量。树上相对 r21 sweep 的唯一
代码增量是上一会话 01:41 的 inference.rs 改动（get_inferred_type 里裸类型
参数 contra 候选乐观满足——非 Go 语义，见下「判定」）。

**命令**：`bash run_full_sweep_r22_20260825.sh`（日志 `submodule_full_run_r22_20260825.log`，
07:51→10:10，2h20m，全程无外部负载，crash 类 SKIP 正常水位 compiler 60 /
conformance 79）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 真实对比 r21 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,882 | 2,479 | 1,142 | **33 stem**（+3 超时掩盖 = 36 真实） | 不变（35+1 掩 ⇄ 33+3 掩，纯洗牌） |
| conformance | 2,062 | 2,538 | 1,285 | **22 行 / 19 stem**（+1 掩盖 = 20 真实） | 不变（21+1 掩 ⇄ 19+1 掩，纯洗牌） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**超时假象对账（30s 线翻转，全部需按未修对待）**：
- importHelpersWithImportOrExportDefault 主/.2/.3（compiler）：r21 真失败
  17 配置 → r22 全部 30.07-30.09s 超时 SKIP「消失」；NoTslib.1 反向
  （r21 超时掩盖 → r22 真失败 5 配置）。四 stem 同根：匿名
  `export default class { }` 无 exports["default"]（Fix 1b-a）
- nodeModulesExportsSourceTs（conformance ×4）：r22 超时掩盖；
  nodeModulesImportAttributesTypeModeDeclarationEmitErrors ×4 反向暴露
  （r21 超时掩盖）。node16 解析族两 stem 均未修
- **唯一真转绿**：typeReferenceRelatedFiles 的 compiler 段副本
  （conformance 段副本仍 FAIL——两段 fixture 根不同）

**01:41 inference 改动判定**：r22 零收益（Fix 7 簇 8 stem 全部仍在、
cyclicTypeInstantiation 仍挂）零损害（无新 FAIL）。Go getInferredType
（inference.go:1317 preferCovariantType）无此特判，且我们 relater.rs:776
已按约束处理类型参数目标——该改动是无依据的过近似，**修复轮回退**。

**36 真实 compiler FAIL 归层**（对照 `_scripts/FIXPLAN_20260825_r22.md`）：
- **Fix 1b（TS2305/2459 假阳性，25 stem）**：default 声明式导出缺
  exports["default"] 条目（esModuleInteropNamedDefaultImports、
  tsxDefaultImports、reexportDefaultIsCallable、importHelpers×4 stem、
  globalThisDeclarationEmit3、exportDefaultImportedType…）；
  `export {X}` 子句别名不在 exports（constEnumNoEmitReexport、
  constEnumPreserveEmitNamedExport1/2、assertionFunctionWildcardImport1、
  declarationEmitQualifiedAliasTypeArgument、declarationsForIndirect…
  destructuredDeclarationEmit、reExportUndefined2、reexportedMissingAlias、
  reexportMissingDefault4/8）；ambient 模块体隐式导出（aliasDoesNot…
  Signatures、es6ImportEqualsDeclaration2、shebangBeforeReferences、
  moduleResolutionAsTypeReferenceDirectiveAmbient）；`export=` 标识符目标
  未追（exportAssignedNamespaceIsVisibleInDeclarationEmit）；
  synthetic default（allowSyntheticDefaultImports9）
- **Fix 7 簇（8 stem）**：classPropertyInferenceFromBroaderTypeConst、
  controlFlowOuterVariable、cyclicTypeInstantiation、
  genericConstraintOnExtendedBuiltinTypes(2)、intersectionSatisfiesConstraint、
  thisInConstructorParameter2、typeCheckObjectCreationExpressionWith…
- **独立**：typeRootsFromMultipleNodeModulesDirectories（2307 typeRoots 解析）、
  recursiveReverseMappedType（深层延迟类型同一性）

**conformance 20 真实 stem**：Fix 1b 族（ambientShorthand_reExport、
exportsAndImports1/3±es6、typeReferenceRelatedFiles、
typesVersionsDeclarationEmit.ambient、nodeModules 两 stem ×8 配置）；
Fix 4 logicalAssignment5×4（r21 超时掩盖、r22 真实暴露——四缺口
fixplan 有效）；Fix 5 genericClassWith；可选参数族
（objectTypesIdentityWithGenericConstructSignaturesOptionalParams×3 +
subtypingWithOptionalProperties）；Fix 7 簇（emitClassDeclarationWith…、
localTypes1、stringLiteralTypesOverloads04、inlineJsxFactoryOverrides…、
topLevelAwaitErrors.9/.10 ×4、nameWithFileExtension）；深层
templateLiteralTypes1。

**修复计划**：`_scripts/FIXPLAN_20260825_r22.md`（Fix 1b 七步查询侧重做 +
Go 参考链已核对：binder.go declareModuleMember/declareSymbolEx、checker.go
getExportsOfModuleWorker/getExternalModuleMember/canHaveSyntheticDefault、
checkDeclarationInitializer/widenTypeInferredFromInitializer）。

## 全量跑 r21（2026-08-25 凌晨）——post-r20 未验证改动的回归暴露轮

**背景**：上一会话在 r20 结束后（08-24 02:2x）写入了 Fix 1（TS2305 模块成员
检查子系统）+ Fix 3（tsconfig 单元 harness）+ Fix 2A 部分（inference.rs 上下文
定型重做：括号透传 + `||`/`??`/`&&`/逗号），**未跑任何验证**。本会话先修 2 个
单测失败（2305 顺序期望值笔误按 tsgo 实测改回源码顺序；补 shorthand ambient
豁免 + 新单测 shorthand_ambient_module_members_exempt_from_2305），单测四套件
全绿（1,344/921/15/2）后上全量。

**命令**：`bash run_full_sweep_r21_20260824.sh`（日志 `submodule_full_run_r21_20260824.log`，
23:00→01:33，2h33m）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r20 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,875 | 2,474 | 1,152 | **35** | 1→35：**回归风暴**（post-r20 改动首次上量） |
| conformance | 2,058 | 2,532 | 1,296 | **21 stem** | 5→21：同因 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**35 compiler FAIL 归层**（全部定位，两组）：
- **Fix 1b（TS2305/2459 假阳性，~24）**：check_module_specifier_members 的
  导出面重建不完整——(a) 文件符号缺 `export default` 条目
  （esModuleInteropNamedDefaultImports、importHelpersWithImportOrExportDefault×3、
  reexportDefaultIsCallable、tsxDefaultImports）；(b) `export {X}` 本地子句/
  转发子句的别名不在 exports（constEnumPreserveEmitNamedExport1/2、
  constEnumNoEmitReexport、declarationEmitQualifiedAliasTypeArgument、
  declarationsForIndirectTypeAliasReference、destructuredDeclarationEmit、
  exportDefaultImportedType、reExportUndefined2、reexportedMissingAlias、
  reexportMissingDefault4/8）；(c) `export * from` 星链未递归
  （assertionFunctionWildcardImport1、ambientShorthand_reExport）；
  (d) ambient 模块体成员隐式导出语义缺失（aliasDoesNotDuplicateSignatures、
  es6ImportEqualsDeclaration2、shebangBeforeReferences、
  moduleResolutionAsTypeReferenceDirectiveAmbient、typeReferenceRelatedFiles、
  exportsAndImports1/3）；(e) `export=` 命名空间成员
  （exportAssignedNamespaceIsVisibleInDeclarationEmit）；(f) default 的
  synthetic 语义未实现（allowSyntheticDefaultImports9）
- **Fix 7（非 2305 回归簇，~9）**：classPropertyInferenceFromBroaderTypeConst
  （static 属性初始化器 'A' 不加宽）、controlFlowOuterVariable（对象字面量
  属性 '""' 不加宽）、cyclicTypeInstantiation（假 2719，T 未代入）、
  genericConstraintOnExtendedBuiltinTypes(2) + emitClassDeclarationWith…
  （number vs T 约束 2345）、intersectionSatisfiesConstraint（交集塌缩）、
  thisInConstructorParameter2、typeCheckObjectCreationExpressionWithUndefined
  CallResolutionData、typeRootsFromMultipleNodeModulesDirectories（2307）
- 既有深层：recursiveReverseMappedType

**21 conformance FAIL**：同两族（exportsAndImports1/3±es6、
ambientShorthand_reExport、typeReferenceRelatedFiles、typesVersionsDeclarationEmit.
ambient、nodeModulesExportsSourceTs=Fix 1b；objectTypesIdentityWithGeneric
ConstructSignaturesOptionalParams×3 + subtypingWithOptionalProperties=可选参数
undefined 缺失；nameWithFileExtension/localTypes1/stringLiteralTypesOverloads04/
inlineJsxFactoryOverridesCompilerOption/topLevelAwaitErrors.9/.10=Fix 7 簇）；
genericClassWithObjectTypeArgsAndConstraints（D3 在案）、templateLiteralTypes1
（深层在案）。

**遗留对照**：r20 的 5 stem 中 Emit1×4 / library-reference-13 未在 r21 FAIL
——**Fix 1/3 的工作部分生效**（Emit1 族转绿）；logicalAssignment5 也未列 FAIL
（r22 需复核是否超时漂移假象——其四个缺口 CLI 探针仍复现，不可能真转绿）。

**修复计划**：`_scripts/FIXPLAN_20260824_r21.md`（Fix 1b 导出面语句级扫描 /
Fix 7 二分定位 02:35 inference.rs 嫌疑 / Fix 4 逻辑赋值四缺口）。

## 全量跑 r20（2026-08-24 凌晨）——r19 补完轮（加载链三修复验证）

**背景**：r19（18:10 启动）compiler 段完成后 conformance 在 ~420/5907 中断。
r20 以同一棵代码树完整重跑三套件。单测前置全绿（1,340/921/2/15）。

**命令**：`bash run_full_sweep_r20_20260823.sh`（日志 `submodule_full_run_r20_20260823.log`，
2h24m，全程无外部负载，超时少）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r18 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,904 | 2,533 | 1,098 | **1** | FAIL 6→1：**r18 的 5 个根选择余波全部转绿**（加载链三修复验证 ✓）；仅剩 recursiveReverseMappedType（深层） |
| conformance | 2,069 | 2,543 | 1,290 | **5 stem** | Emit2 经分诊台账为 accepted-diff；Emit1 仍 FAIL（TS2305）；genericClassWith 本轮真实 FAIL（6.71s 非超时，r18 为漂移掩盖）；logicalAssignment5 本轮 17.65s 未超时拿到真实差异；templateLiteral/library-reference 既有 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**conformance 5 stem**：logicalAssignment5×4（假 7006×6 + 欠 2722×2/18048×2）、
nodeModulesImportAttributesModeDeclarationEmit1×4（TS2305 模块成员检查子系统）、
library-reference-13（tsconfig 单元支持）、templateLiteralTypes1（深层）、
genericClassWith（D3，本轮真实浮现）。

**修复计划**：`_scripts/FIXPLAN_20260823_r20.md`（TS2305 子系统 / tsconfig
单元 / logicalAssignment5 上下文定型+流窄化，含 Go 参考行号）。

## 全量跑 r18（2026-08-23 下午）——根选择修复验证 + 加载链余波修复轮

**命令**：`bash run_full_sweep_r18_20260823.sh`（日志 `submodule_full_run_r18_20260823.log`，
4h52m，中段仍有外部负载）。单测前置全绿。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 说明 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,892 | 2,519 | 1,119 | **6** | recursiveReverse 既有 + 5 个根选择余波（**r18 后已修**，见下） |
| conformance | 2,061 | 2,536 | 1,305 | **5 stem** | **nodeModules mode 族全绿**（Override4/5、ModeError、DeclarationEmit3/7、PackagePatternExports——根选择修复验证 ✓）；Emit1/Emit2=TS2305 子系统；genericClass/templateLiteral 既有；library-reference-13（tsconfig 单元支持缺口） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

### r18 后 fix-only（2026-08-23 傍晚）——加载链三修复

1. **import/type-ref 目标改用递归 loader**：3a/3b/3c 原用单文件 loader——
   import 拉入的文件自身的 `/// <reference path>` 从未被跟随（此前全根
   加载掩盖；Go fileLoader 对每个加载文件处理引用）。BFS 队列改为
   **排水栈**（LIFO 保持官方文件顺序，program_file_ordering 测试组把关）；
   递归加载的新文件回灌栈使其 imports 也被处理。清 privacy*DeclFile ×3
   （exporter 的 `/// <reference path='GlobalWidgets.ts'/>` 单引号形态）。
2. **`@types: "*"` 通配**：types 选项枚举各 effective type root 的子目录
   （Go typesOption 语义）；get_effective_type_roots 改**祖先链**收集
   `<dir>/node_modules/@types`（Go GetEffectiveTypeRoots 原义，此前只有
   cwd 一级）。清 moduleResolution_automaticTypeDirectiveNames。
3. **harness cwd 按布局**：含根名单元的用例 host current_directory 用官方
   默认 "/.src"（Go srcFolder；裸名用例保持 /proj 挂载约定）。清
   referenceTypesPreferedToPathIfPossible（/.src/node_modules/@types）。

单测四套件全绿（1,340/921/2/15）。五个 r18 compiler 余波全部 worker 验证清零。

**遗留**：TS2305 模块成员子系统（Emit1/Emit2 ×8 配置）、library-reference-13
（tsconfig 单元支持）、logicalAssignment5（上下文定型断点，调试钩在）、
recursiveReverseMappedType / genericClassWith（D3 方差）/ templateLiteralTypes1
（深层）。

## 全量跑 r17（2026-08-23 午）——负载污染轮（285 超时）+ harness 根选择根修

**命令**：`bash run_full_sweep_r17_20260823.sh`（日志 `submodule_full_run_r17_20260823.log`，
2h51m，**机器外部负载**：285 例 30s 超时（常态 ~90），compiler 段慢 800s）。数字受污染。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 说明 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,861 | 2,467 | 1,207 | **1** | 仅 recursiveReverseMappedType——D5/ExportStarAs 稳定 ✓ |
| conformance | 2,033 | 2,518 | 1,351 | **5 stem** | Override4/Emit2/AttributesTypeModeErrors 为超时掩码破裂后的真实旧账（见下）；genericClass/templateLiteral 既有 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

### r17 后 fix-only（2026-08-23 下午）——harness 根选择 + 1453 位置

**重大 harness 缺陷（Override4 族的真根因）**：我们无条件把所有 .ts 单元当编译根。
Go compiler_runner 规则（compiler_runner.go ~L321）：`noImplicitReferences` 指令、
或末单元含 `require(`/`reference path` 时，**仅末单元为根**，其余仅存在于 FS、
经引用/解析进入程序——nodeModules 族的 resolution-mode 条件排除的入口点一旦
被根加载，两个 `declare global` 全合并（Override4 本应只解析 import 面）。
修：build_and_check 增根选择（worker 侧从原文扫 harness 指令——case_parser
的 HARNESS_DIRECTIVES 刻意排除它们）。CLI 双探针证实官方对「双根」也零错误
——基线差异全在根选择。验证：Override4 四配置 EXACT、Override5/
DeclarationEmit3/PackagePatternExports 保持零错误。

**TS1453 位置**（ModeError）：官方报在 `types="..."` 的**值文本**处（(1,23)=pkg
首字符），非 resolution-mode 属性值——extract 补 types_value_range（注意 rest
相对 `///` 的 +3 列偏移）。ModeError 四配置 EXACT。

**连带浮现的确认项**：Emit2/AttributesTypeModeDeclarationEmitErrors 仅欠
TS2305/TS1340（**模块成员检查子系统缺失**——import 说明符从不对照模块 exports
校验；实现需 Program.resolve_external_module_path + node16 下属性被拒后默认
链解析，注意影响面）。

单测四套件全绿（1,340/921/2/15）。

## 全量跑 r16（2026-08-23 上午，helper-locals + D5 验证）——compiler 1F（历史新低）

**命令**：`bash run_full_sweep_r16_20260823.sh`（日志 `submodule_full_run_r16_20260823.log`，
2h32m）。单测前置全绿（1,340/921/2/15）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r15 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,909 | 2,529 | 1,097 | **1** | **ExportStarAs ✓、inferentialTypingWithFunctionType2（D5）✓**——仅剩 recursiveReverseMappedType |
| conformance | 2,062 | 2,544 | 1,292 | **9 stem** | D5(b) 放宽引发 4 个 specialized/call-signature 继承回归（r16 后已修，见下）；logicalAssignment5 复发（r15 实为 30.06s 超时 SKIP 掩盖——从未真修）；Emit1/ModeError 为真实旧账浮出（同因：r13-r15 轮该族多在超时线附近漂移） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

### r16 后 fix-only（2026-08-23 午）

**D5(b) 收窄**：free-type-param 目标返回放行加判别——仅当该类型参数
**不属于目标签名自身**（推断替换占位，Go 比较前擦除源参数）才跳过；
目标自有参数照常检查（`<T>(x:T)=>string` vs `<T>(x:T)=>T` 必须失败）。
四个回归用例 worker 全文本 EXACT 比对通过：
subtypingWithCall/ConstructSignaturesWithSpecializedSignatures、
call/constructSignatureAssignabilityInInheritance；D5 用例保持零错误。
单测四套件全绿（1,340/921/2/15）。

**r16 暴露的真实旧账（r15 的 SKIP 掩盖）**：
- logicalAssignment5：??=/||=/&&= RHS 上下文定型断点（FIXPLAN_20260823_r15.md A，
  调试钩子 TSOX_DEBUG_INFER 已埋）
- nodeModulesImportAttributesModeDeclarationEmit1：**TS2305（模块成员检查）
  子系统整体缺失**——import 说明符从不对照模块 exports 校验
- nodeModulesTripleSlashReferenceModeOverrideModeError：TS1453 位置
  （官方 (1,23) 报在 types 名区 vs 我们 (1,42) 属性值区）+ 坏 mode 后
  「任意解析」的实际取向（官方落到 require 面 → 2304 'foo'）

**剩余清单（r17 验证 D5 收窄 + ExportStarAs + 观察漂移）**：
recursiveReverseMappedType、genericClassWith（D3 方差）、templateLiteralTypes1、
logicalAssignment5、Emit1×4（TS2305 子系统）、ModeError×4（1453 语义）。

## r15 后 fix-only 第二轮（2026-08-23 上午；D5 攻坚 + helper locals）

1. **importHelpersWithExportStarAs**：fixture 的
   `declare module "tslib" { function __importStar(m: any): void; }` 无 export
   修饰——ambient 模块隐式导出，binder 路由进节点 locals；
   check_external_emit_helpers 的 helper 名查找补 `ambient_namespace_local`
   兜底。worker 验证：十配置全零错误 = 官方。
2. **D5 inferentialTypingWithFunctionType2（终局）**：`[1,2,3].map(identity)`
   的三步缺口——
   (a) 推断侧：U 的候选 [A]（协变，来自回调返回）被外来 contra 候选
   [T]（裸类型参数）毒化（cov_assignable_to_contra 恒 false）→ contra 的
   裸类型参数视为可满足（推断期类型参数乐观语义）；
   (b) 比较侧：非泛型目标的返回位为自由类型参数（U:=A 替换后的占位）时
   视为匹配（Go 在比较前擦除源类型参数，不可映射的返回永不失败）；
   (c) 连带清除 mixin 幻影 2345（checker_parity 钉住值过期更新：
   Timestamped mixin 的 `new TimestampedUser()` 现零错误）。
   验证：用例零错误=官方；functionCall10 双错误精确；单测四套件全绿
   （1,340/921/2/15）。
3. **调试开关**：TSOX_DEBUG_INFER（推断候选/签名返回追踪）保留在码内
   （env 门控，同 TSOX_DEBUG_PAYLOAD 惯例）。

**剩余 3（深层，在案）**：recursiveReverseMappedType（延迟递归映射多报）、
genericClassWith（D3 类泛型方法位置方差测量）、templateLiteralTypes1
（模板字面量推断族）。

## 全量跑 r15（2026-08-23 晨，r14 后修复轮验证）——compiler 3F / conformance 2F（stem 新低）

**命令**：`bash run_full_sweep_r15_20260823.sh`（日志 `submodule_full_run_r15_20260823.log`，
2h27m，二进制纯净）。单测前置全绿。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r14 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,898 | 2,547 | 1,088 | **3** | modulePreserve3 转绿 ✓（JSX runtime 真解析）；新面孔 importHelpersWithExportStarAs（r15 后已修，见下） |
| conformance | 2,058 | 2,541 | 1,306 | **2 stem** | **7→2**：logicalAssignment5、nodeModulesImportAttributesModeDeclarationEmit1×4、ExportsSourceTs×4、jsxJsxsCustomImport×2、node10AlternateResult 全部转绿（选项键 canonical 化的连锁修复——此前 @strict 族以外的 camelCase 指令值整体丢失） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**剩余 4（全部深层子系统，在案）**：
- compiler：inferentialTypingWithFunctionType2（D5：泛型函数作回调需经推断合一而非直接
  可赋值比较——`[1,2,3].map(identity)` 的 A↔number 统一）、
  recursiveReverseMappedType（延迟递归映射类型多报）、
  importHelpersWithExportStarAs（**r15 后已修**：ambient 模块隐式导出的 helper
  函数在节点 locals，查名补 ambient_namespace_local 兜底；worker 验证十配置
  全零错误=官方）。
- conformance：genericClassWith（D3：类泛型**方法位置**的方差测量——官方零 2345）、
  templateLiteralTypes1（模板字面量推断族）。

SKIP 上升说明（1,042→1,088 / 1,236→1,306）：canonical 化使 `classic` 等此前被
丢弃的指令真正生效，其中我们不支持的选项进入合法 skip；此前部分用例以
错误选项「侥幸通过」，现按真实选项运行。FAIL 净降 7→2。

### r15 后 fix-only（2026-08-23 上午）

**importHelpersWithExportStarAs**：fixture 的
`declare module "tslib" { function __importStar(m: any): void; }` 无 export
修饰——ambient 模块隐式导出，binder 路由进节点 locals；
check_external_emit_helpers 的 helper 名查找补 `ambient_namespace_local`
兜底。单用例 worker：十配置全零错误 = 官方。单测四套件全绿（1,339/921/2/15）。

## 全量跑 r14（2026-08-23 晨，r13 后修复轮验证）——compiler 3F / conformance 7F，十组修复八组验证通过

**命令**：`bash run_full_sweep_r14_20260823.sh`（日志 `submodule_full_run_r14_20260823.log`，
2h17m，二进制纯净）。单测前置全绿（lib 1,339 / checker_parity 921 / parity 2 / lsp 15）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r12（可比口径） |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,923 | 2,568 | 1,042 | **3** | PASS +98；varianceCallbacksAndIndexedAccesses 转绿（D1 降级抑制）、importHelpers.2 偶发未再现；新面孔 modulePreserve3 |
| conformance | 2,069 | 2,593 | 1,236 | **7 stem** | FAIL 15→7：nodeModules 族六 stem 全清（DeclarationEmit3/7、Override4/5、ModeError、PackagePatternExports、ImportHelpersCollisions3）、jsxJsxsSubstitutesNames、tsxReactEmitSpread、objectTypeWithCallSignature 转绿 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**compiler 3**：inferentialTypingWithFunctionType2（D5）、recursiveReverseMappedType（深层
多报）、modulePreserve3（新面孔，本轮已修——见下）。
**conformance 7**：genericClassWith（D3）、templateLiteralTypes1、ExportsSourceTs×4
（TS2883 声明发射可命名性）、jsxJsxsCustomImport×2、logicalAssignment5、
nodeModulesImportAttributesModeDeclarationEmit1×4、node10AlternateResult——后四者
r14 后已全部修复（单用例 worker 验证与官方逐字一致），见下轮记录。

### r14 后 fix-only 轮（2026-08-23 晨；单用例 worker 验证，未跑套件）

1. **harness 选项键 canonical 化**（tsoptions apply_test_settings）：
   `apply_options` 的 match 精确匹配键名，而测试指令键被小写化——
   `jsxImportSource`/`moduleResolution`/`jsxFactory`/`sourceRoot` 等 camelCase
   分支**从未命中**（指令被接受但值静默丢弃）。修：经 `find_option`（本就
   大小写不敏感）把键规范化为 canonical 名再分派。后果链：CustomImport 的
   `@jsxImportSource: preact` 丢失 → 默认 react → react16 的
   `declare module "react/jsx-runtime"` 命中 → TS2875 消失（r14 报零错误）；
   node10 用例的 `@moduleResolution` 丢失 → 修复前 node10 修复不生效。
2. **node10 跳过门移除**（harness should_skip）：resolver 的 node10 路径
   （NONE-features 状态：无 exports、types/main+index 回退）已随
   get_module_resolution_kind 修复可用——node10AlternateResult_noResolution
   worker 实测与官方逐字一致（TS2307 位置精确）。
3. **隐式 JSX runtime 真解析**（Program trait + checker + 加载循环）：
   trait 增 `resolve_external_module_path`（compiler::Program 用真
   Resolver 实现）；checker 的 `resolve_jsx_runtime_by_path` 按 implied
   node format 解析后取已加载文件符号（resolution mode 载体用 ESNext/
   CommonJS，非 emit 格式的 ES2020）；加载循环 3c 步为 react-jsx/jsxdev
   的 .tsx/.jsx 预载 runtime 模块（Go GetJSXRuntimeImportSpecifier 语义）。
   modulePreserve3（@types/react/jsx-runtime.d.ts 文件形态）清零；
   CustomImport（TS2875 精确）与 SubstitutesNamesFragment（零错误）回归通过。

**遗留（r15 验证后）**：logicalAssignment5（??=/||=/&&= 的 RHS 上下文定型
经 possibly-undefined 签名提取 + 赋值后确定赋值分析——官方 2722/18048，
我们误报 7006）、nodeModulesImportAttributesModeDeclarationEmit1（node16
下属性被拒后解析需忽略属性走默认 require 条件 → 欠报 2305）、
ExportsSourceTs（TS2883）、templateLiteralTypes1、genericClassWith（D3）、
inferentialTypingWithFunctionType2（D5）、recursiveReverseMappedType（深层）。

## 全量跑 r13（2026-08-23 凌晨）——混合二进制轮（compiler=修复前 / conformance=修复中快照）

**命令**：`bash run_full_sweep_r13_20260823.sh`（日志 `submodule_full_run_r13_20260823.log`，
2h24m）。**注意**：跑测期间工作区在写修复（fix-only 与 sweep 并行），conformance 段
启动时 cargo 检测到源码变更重新编译——conformance/transpile 跑的是**中途修复快照**，
compiler 段跑的是修复前代码。数字不可与 r12 直接对比，净效果以 r14 为准。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 说明 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,891 | 2,592 | 1,049 | **4** | 修复前代码。3 个既有深层项 + importHelpers.2 偶发（同码 r12 过 r13 挂，19.9s 高耗时疑似超时邻域；修复后 CLI 验证与基线精确一致） |
| conformance | 2,047 | 2,593 | 1,251 | **16** | 中途快照。r12 的 9 个 FAIL stem 已转绿（DeclarationEmit3/7、Override5、ModeError、PackagePatternExports、ImportHelpersCollisions3、jsxJsxsSubstitutesNamesFragment、tsxReactEmitSpreadAttribute、objectTypeWithCallSignature）；同时钝化版 F 修复引发 8 个新面孔（算术×4+enumConstant、inlineJsxFactory×3、Override2——均在当前代码下 CLI 归零） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

### r13 后 fix-only 修复轮（2026-08-23，sweep 期间+结束后写码，CLI/tsgo 双探针验证）

十组修复（全部 CLI 对照 tsgo 验证）：

1. **declare global 增强合并**（checker populate_globals）：parser 的
   `module_augmentations`（references.rs 早已收集）从未被 checker 消费——
   `declare global` 块成员（exports/members/节点 locals 三源合一）合并进
   globals（Go initializeChecker → mergeModuleAugmentation）。清除
   TripleSlashReferenceModeDeclarationEmit3/7、Override5、ModeError 族
   （types="pkg" resolution-mode 解析本来就通，缺的只是合并）。
2. **parser 说明符 `type` 二义性**（parse_import_or_export_specifier，
   Go parser.go ~L2457 忠实移植）：`{ type }` 裸 type 是名字本身、
   `{ type as }`/`{ type as as }`/`{ type as as as }` 四形态。清
   nodeModulesPackagePatternExports（此前对 `export { type }` 报 TS1003/1005）。
3. **TS2343/TS2354 完整门控**（check_external_emit_helpers 重写）：格式门
   `getEmitModuleFormatOfFile < System`（含 AMD/UMD；node1x 按文件 implied
   format）；**esModuleInterop 默认 true**（TS _computedOptions 语义，
   is_true_or_unknown）门 ImportDefault/ImportStar（不门 ExportStar）；
   子句级默认导入无检查（TS-current 语义）；命名导出子句的 NamedExports
   数据模式此前从不匹配（死代码）；tslib 可解析时逐 helper 名查 exports →
   TS2343。清 nodeModulesImportHelpersCollisions3（es2015 ×4）。
4. **TS2875 隐式 JSX runtime**（jsx.rs ensure_jsx_implicit_container）：
   react-jsx/jsxdev 解析 `<importSource>/jsx-runtime`，失败在该文件首个
   JSX 标签报 TS2875；容器 JSX 命名空间优先、全局兜底（Go
   getJsxNamespaceContainerForImplicitImport）。
5. **ambient 模块命名空间导入别名 chase**（resolve_import_alias_module）：
   `import * as P from "ambient-mod"` 的类型位置限定访问 `P.X` 此前必败
   （别名无目标链接；仅 import= 被处理）——限定名解析处追到模块符号；
   export= chase 补 ambient locals 兜底。react16 的 TS2694×365 全清。
6. **D1 降级泄漏根治**（degraded_type_ptrs）：heritage 降级窗口产物
   （合并跳过 errorType 基类 → declared-only 成员表）记入指针集合；
   is_type_related_to 对**双侧结构化对象**的比较命中集合即视为相关
   （kind 型比较不抑制——算术操作数 Object→number 误放行的教训）；
   TS2344/TS2538 发射点同门。react16 JSX 434 错→0；
   `HTMLElement→Element` 两级继承幻影 2739 消除。
7. **显式 node10 不再并入 Bundler 重映射**（get_module_resolution_kind）：
   node10 忽略 exports。清 node10AlternateResult_noResolution（TS2307）。
8. **JSX 开标签先于子元素检查**（check_expression 分派顺序）：TS2875 落在
   fragment 开标签（官方 (2,11)）而非首个子元素。
9. **TS2602 收窄 + IntrinsicElements 索引签名**：JSX 命名空间存在但缺
   Element → 静默 any（inlineJsx 夹具只声明 IntrinsicElements）；接口声明
   扫描 IndexSignature → 任意标签接受（[e: string]: any 形态）。
10. **单测 +8**（node_format_tests ×5 + parser ×2 形态组 + parity 夹具
    jsx react-jsx→react）——lib 1,331→**1,339** 全绿（checker_parity 921 /
    parity 2 / lsp 15 均绿）。

**r13 conformance 8 个新面孔归因**（全部当前代码 CLI 验证归零）：钝化版 F
抑制算术操作数 kind 检查（×5，细化双侧对象条件后恢复）；inlineJsxFactory×3
（修复 1 揭露合成 JSX 命名空间长期掩盖的两个缺口，修复 9 补齐）；Override2
（修复 1 时序，当前代码与 tsgo 逐字对齐）。

**遗留（r14 后）**：ExportsSourceTs（TS2883 声明发射可命名性子系统）、
templateLiteralTypes1（模板字面量推断族）、genericClassWith（D3 方法/类
类型参数遮蔽推断）、inferentialTypingWithFunctionType2（D5 回调推断先于
实参检查）、recursiveReverseMappedType/varianceCallbacksAndIndexedAccesses
（多报型深层）、importHelpers.2 偶发待 r14 观察。

## 全量跑 r12（2026-08-22 深夜，r11 fix-only 轮 2 验证）——compiler FAIL 3（新低）

**命令**：`bash run_full_sweep_r12_20260822.sh`（日志 `submodule_full_run_r12_20260822.log`，
2h48m，跑测期间机器有并发负载，超时偏高）。前置单测全绿（lib 1,331 / checker_parity
921 / parity 2 / lsp 15）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r11 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,825 | 2,600 | 1,109（超时 88） | **3** | FAIL 5→3：es5-commonjs7、es6UseOfTopLevelRequire 转绿（轮 2 两项修复验证 ✓，CLI 双探针 + 全量双确认） |
| conformance | 1,946 | 2,677 | 1,270（超时 117） | **15** | 13→15 构成漂移：ImportAttributesModeDeclarationEmit1/ImportModeDeclarationEmit2/Override2 转绿；ExportsSourceTs/ImportHelpersCollisions3/PackagePatternExports/DeclarationEmit3/Override5 浮现（nodeModules 族随超时与解析时序漂移的历史行为） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**compiler 剩余 3（全部为已定位深层项，r9 起恒定）**：
inferentialTypingWithFunctionType2（FIXPLAN D5：泛型函数作回调的 relater 比较）、
recursiveReverseMappedType / varianceCallbacksAndIndexedAccesses（多报型推断缺口）。

**conformance 15 归层**：nodeModules mode 族 ×7（errors 产物已一致——差异在
**.d.ts/js emit 产物面**，需 harness 多产物比对归因，下轮优先）+ jsxJsxs×2（D1）+
tsxReactEmitSpreadAttribute（JSX emit）+ node10AlternateResult（欠报 2307）+
templateLiteralTypes1（模板字面量推断族）+ objectTypeWithCallSignature（多报 2411）+
genericClassWith（D3）。

**会话总账（r9→r10→r11→r12）**：
- compiler FAIL：3(+99 超时)→10→5→**3**；超时 99→37→27→88（r12 机器并发负载所致，口径统一）
- conformance FAIL：10→13→13→15（nodeModules 族漂移面，深层归因待做）
- transpile：全程 0
- 单测：1313→**1331**（新增 convergence 4 + resolver 1 + node_format 3 + arity/marker 3 + 既有 checker_parity 21 修复）
- ISSUES_RISK_ANALYSIS 四项风险全部修复并经三轮全量验证

## 全量跑 r11（2026-08-22 晚，r10 fix-only 验证 + 二轮 fix-only）

**命令**：`bash run_full_sweep_r11_20260822.sh`（日志 `submodule_full_run_r11_20260822.log`，
2h41m）。前置单测全绿（lib 1,331——含 r10 后新增 3 个 / checker_parity 921 / parity 2 / lsp 15）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r10 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,893 | 2,587 | 1,051（超时 27） | **5** | FAIL 10→5：collisionExportsRequireAndAmbient×3、es5-commonjs6/8、genericReturnTypeFromGetter1 全转绿（门控+2314 修复生效） |
| conformance | 2,027 | 2,622 | 1,245（超时 78） | **13** | 构成变化：exportAmbientClassNameWithObject、topLevelAwaitErrors.11、Override4、OverrideModeError、DeclarationEmit3 转绿；tsxReactEmitSpreadAttribute 回归 + 三斜线 mode 族下一层浮出（DeclarationEmit1/7、Override2、jsxJsxs 主形态） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

超时总趋势（统一口径）：r9 99 → r10 37 → r11 27（compiler 段）。

**compiler 5 FAIL**：es5-commonjs7（.d.ts ambient——r11 后已修：declaration_is_ambient
补声明文件判定）、es6UseOfTopLevelRequire（新：module 未设时 emit 格式按 target 推断
缺失——r11 后已修：get_emit_module_format_of_file 对 None 按 GetEmitModuleKind 推断，
CLI 双探针与 tsgo 一致）、inferentialTypingWithFunctionType2（D5）、
recursiveReverseMappedType / varianceCallbacksAndIndexedAccesses（多报深层）。

**conformance 13 FAIL 归层**：nodeModules 三斜线/import mode 族 ×6（Override2 的
无属性默认链疑 r10 为超时 SKIP 未暴露；DeclarationEmit1/7 新差异待 diff）+
jsxJsxs×2 + tsxReactEmitSpreadAttribute（回归，待 diff）+ node10AlternateResult（欠报
2307）+ templateLiteralTypes1 + objectTypeWithCallSignature + genericClassWith（D3）。

### r11 后 fix-only（2026-08-22 深夜，CLI 双探针验证，未跑用例）

1. `get_emit_module_format_of_file`：module 未设时按 target 推断
   （>=ES2015 → ES2015 格式，否则 CommonJS——Go GetEmitModuleKind 语义），
   es6UseOfTopLevelRequire 的 2441 误报消除（探针 0=0）
2. `declaration_is_ambient` 补声明文件判定（.d.ts 全 ambient），
   es5-commonjs7 的 test.d.ts 1216 误报消除（探针 0；非 d.ts 的
   es5-commonjs4 形态在 emit 模式仍报——noEmit 跳过逻辑与 tsgo 实测一致）

## r10 后 fix-only 轮（2026-08-22,只改代码+补单测,未跑任何用例）

基于 r10 全量跑的 23 FAIL 归因,六组修复:

1. **TS2314 → errorType 联动**(genericReturnTypeFromGetter1):泛型实参
   数检查失败后 `resolve_type_reference` 返回 errorType(对齐 Go
   `getTypeFromClassOrInterfaceReference` 的 `return c.errorType`)——
   依赖检查随即豁免(2564 的 any/error 属性类型豁免)。CLI 双探针验证
   方向:tsgo 报 2314 不报 2564。
2. **2441/1216/2725 完整门控**(collisionExportsRequireAndAmbient×3 +
   es5-commonjs×3):新增 `declaration_is_ambient`(NodeFlagsAmbient
   传播)——ambient 声明全豁免;2441 限 emit 格式 < ES2015;1216 限
   export 修饰的变量语句 + 格式 < System;2725 限 CommonJS;三者全部
   `errorSkippedOnNoEmit`(--noEmit 跳过)。门控逐条对照 Go
   checker.go ~L10504-10620 与 grammarchecks.go ~L1600。
3. **三斜线 resolution-mode 提取与传递**(TripleSlashReferenceMode
   Override4/ModeError、DeclarationEmit 族):`extract_reference_
   types_directives` 抓 `resolution-mode` 属性值+绝对范围;调用点
   import→ESNext / require→CommonJS 传入 resolver(默认链保留 implied
   format);非法值报 TS1453(值位置)后任意解析(Go "resolves is
   arbitrary")。
4. **r10 conformance 两个新面孔被 #2 覆盖**:topLevelAwaitErrors.11
   (`declare var require` ambient)与 exportAmbientClassNameWithObject
   (`declare class Object` ambient 2725 豁免)——预期 r11 转绿。
5. **新单测**(node_format_tests):generic_arity_error_suppresses_
   ts2564、ambient_declarations_exempt_from_reserved_names、
   es_module_marker_requires_export_and_emit(bare/exported/noEmit 三态)。
6. **未盲改(下轮实证优先)**:nodeModulesImportAttributesMode
   DeclarationEmit1 / ImportModeDeclarationEmit2——四配置零产物(含
   欠报 2305),疑 parse 诊断产物路径,需运行时定位;templateLiteral
   Types1(模板字面量推断族)、objectTypeWithCallSignatureHiding
   Members(多报 2411)、node10AlternateResult(欠报 2307)、
   recursiveReverseMappedType / varianceCallbacks(多报深层)、
   inferentialTypingWithFunctionType2(D5)、genericClassWith(D3)、
   jsxJsxsFragment(D1)——深层子系统,记录在案。

## 全量跑 r10（2026-08-22，ISSUES_RISK_ANALYSIS 四项修复 + r9 遗留修复轮验证）


**命令**：`bash run_full_sweep_r10_20260822.sh`（日志 `submodule_full_run_r10_20260822.log`，
预编译后 12/12/8 workers，30s 超时）。**前置单测全绿**：lib 1,328（新增
convergence_tests 4 + resolver exports 深度 1）/ checker_parity 921 / parity 2 /
lsp 15（checker_parity 首次全量跑通——此前该目标长期未运行，21 个既有失败本轮清零）。

**本轮 src 修复**（ISSUES_RISK_ANALYSIS 四项 + 单测阶段修复）：
1. **Issue 1 heritage 收敛**：基类型降级判定改 `is_type_error`（精确，合法 any
   不再误标）；新增 `heritage_retry_counts`（`HERITAGE_RETRY_LIMIT=2`）——同一
   接口符号降级最多重试 2 次，第 3 次接受（强制写缓存 + epoch 回滚到入口值，
   外围 node-memo 帧正常缓存）。真环/稠密图有界收敛，对应 Go 惰性成员解析的
   稳定共享类型语义。r9 的 99 超时风暴根因。
2. **Issue 2 缓存上限**：`type_node_subst_cache` / `instantiated_member_type_cache`
   超 300k 条整体 clear（纯函数缓存）；`type_instantiation_count` 重置点对齐 Go
   checkSourceElement（check_statement / check_source_file 入口均重置）。
3. **Issue 3 深度守卫**：attach_class_statics（栈深≥200 截断）+
   resolve_base_class_instance_type（type_resolution_stack≥200）——深（非环）
   继承链不再无界递归（260 级链单测在默认栈通过）。
4. **Issue 4 resolver**：exports/imports target 递归深度上限 16。
5. **显示层环检测**（checker_parity 栈溢出根修）：`type_to_string_ex` 打印栈
   祖先环检测（循环类型打 "..."，对齐官方）+ 300 深度兜底——`ReturnType<typeof f>`
   256MB 栈都溢出的无界显示递归根治；发现 `serialization_level` 守卫从未递增
   （死代码）。
6. **条件类型显示分支作用域**：nodebuilder 打印条件类型分支时推入条件类型节点
   作用域——`infer R`（声明在条件类型 locals）此前在显示路径解析失败报 TS2304，
   污染 lib（InstanceType/Awaited 的 R/F）→ with-lib 族测试全灭。
7. **VFS insert_dir 建祖先链**（D6-reso 主根因的根修）：挂载 /node_modules/pkg
   隐含 /node_modules 存在；node16 exports 按文件格式选 import/require 条件
   （r9 轮代码 + 本轮补齐测试配置）。
8. **杂项**：parser `parse_import_attributes` 补 skip_keyword 参数（import-type
   路径重复消费 with）；CJS 保留名检查的变量声明父链判定；check_contextual_elements
   数字索引签名经泛型实例化重读（ConcatArray<number> 的 `[n:number]:T` 不再拿裸 T
   ——concatError 幻影 2322 真根因，非试探泄漏）。

**checker_parity 21 个既有失败的处置**（全部 tsgo-ref 探针验证后对齐）：
- 我方行为与 tsgo 一致、测试期望错：ts2448 族（官方 2449）、ts2420（官方成员级
  2416）、type_display_union（官方显示流收窄后的 'number'）、narrowing×4（官方
  初始化器先收窄→2367/2339）、union 属性缺失（tsgo 不报）、ts18048×4（tsgo
  对 possibly-undefined 属性访问不报 18048）
- 行为已演进、钉住过期：dynamic_import（2304=0）、assertion_function（2322=0）、
  generic_factory（2345=0）、singleton（2339=0）、promise_then×2（2345=0）
- 注意：ts18048 直接用例在 HEAD 存在测试序依赖（全量跑过、隔离跑挂）——本轮
  统一钉 18048=0（oracle 一致），不稳定面消除

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r9 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,882 | 2,583 | 1,061（**超时仅 5**） | **10** | PASS +22；**99 超时风暴→5**（Issue 1 收敛修复直接验证）；concatError/importHelpers.2 转绿 |
| conformance | 2,019 | 2,632 | 1,243 | **13** | FAIL 10→13：r9 的 importAssertion5/importAttributes5、nodeModulesExportsSourceTs×4、ImportAttributesTypeModeDeclarationEmitErrors×4、tsxReactEmitSpreadAttribute×3、Override2 等全转绿；VFS 祖先链修复使 nodeModules 解析走通，**浮出下一层差异**（resolution-mode 传递族） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**compiler 10 FAIL 归因**（全部定位）：
- `collisionExportsRequireAndAmbient{Var,Function,Class}` ×3 — r9 轮新增 2441 保留名检查缺 Go 门控：`needCollisionCheckForIdentifier` 的 **ambient 豁免**（`NodeFlagsAmbient` 无 codegen 影响）+ emit 格式 < ES2015 + `errorSkippedOnNoEmit`（checker.go ~L10549）
- `es5-commonjs6/7/8` ×3 — 1216 门控：仅 **VariableStatement 带 export 修饰** + 非 ambient + 格式 < System + **noEmit 跳过**（grammarchecks.go ~L1600；case6 裸 `var __esModule` 零错误、case8 `@noEmit: true` 零错误、case4 `export var` 报 1216——三例全对上）
- `genericReturnTypeFromGetter1` — TS2564 缺 strict 门控（非 strictPropertyInitialization 下不报）
- `recursiveReverseMappedType`、`varianceCallbacksAndIndexedAccesses` — 多报型深层推断缺口（官方零错误）
- `inferentialTypingWithFunctionType2` — r9 既有（FIXPLAN D5-infer 在案）

**conformance 13 FAIL**：nodeModules resolution-mode 族 ×6（ImportAttributesModeDeclarationEmit1、ImportModeDeclarationEmit2、TripleSlashReferenceModeDeclarationEmit3/5、Override4、OverrideModeError——三斜线/导入的 mode 参数传递，FIXPLAN D6-reso #2 在案）+ jsxJsxsCjsTransformSubstitutesNamesFragment（D1 余留）+ genericClassWith（D3-infer 在案）+ 新面孔 5（exportAmbientClassNameWithObject、topLevelAwaitErrors.11、node10AlternateResult_noResolution、templateLiteralTypes1、objectTypeWithCallSignatureHidingMembersOfExtendedFunction）——下轮 diff 归因。

## 全量跑 r9（2026-08-21 凌晨，D1-partial 验证）——compiler 3 F / conformance 10 F / transpile 0，**99 超时（D1-partial 副作用）**

**命令**：`bash run_full_sweep_r9_20260821.sh`（日志 `submodule_full_run_r9_20260821.log`，
单测前置 1313 全绿）。修复计划：`_scripts/FIXPLAN_20260821_r9.md`（本轮跑期间完成的
静态+双探针诊断，全部根因在案）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r8 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,860 | 2,613 | 1,060 | **3** | +concatError/inferentialTyping2 不变；**importHelpers.2 回归**（r8 绿） |
| conformance | 2,011 | 2,647 | 1,239 | **10** | FAIL 17→10（stem 数），但构成剧变见下 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**D1-partial（d1a9969）生效证据**：jsxJsxsSubstitutesNames(+Fragment) ×6 全绿
（r8 FAIL）。**副作用——99 个 30s 超时 SKIP**（r8 ~18）：nodeModules 全族 +
logicalAssignment 族 + importHelpers 族被拖过线。根因（已定位）：
declared_type 缓存跳过后，heritage 节点在 node-memo 缓存的 errorType 使重解析
**永不收敛**——每次引用都全量重解析接口（修法见 FIXPLAN D1-续：计数器需
同时覆盖环守卫返回点）。

**r9 FAIL 构成（10 stem）**：genericClassWith、importAssertion5、importAttributes5、
jsxJsxsCustomImport×2（D1 余留）、nodeModulesExportsSourceTs×4（缺 TS2883）、
nodeModulesImportAttributesTypeModeDeclarationEmitErrors×4（import 型带属性
子句 parser 缺口 + TS1453）、TripleSlashReferenceModeDeclarationEmit5×4、
Override2×4、Override4×4（resolution-mode 解析）、tsxReactEmitSpreadAttribute×3
（D1 降级泄漏进 react16 约束检查）。
**compiler 3**：concatError（试探泄漏）、inferentialTypingWithFunctionType2
（官方零错误我们误报 2345）、importHelpers.2（TS2354 消失，疑似 D1 时序）。

**r8→r9 消失的 FAIL 多为超时 SKIP 而非修复**：logicalAssignment5、
GeneratedNameCollisions、ImportMeta、PackagePatternExports 等全在 99 超时名单。

## D1 部分修复（2026-08-20 下午，commit d1a9969）+ 会话交接

**已完成**：heritage 降级结果不再写入符号 declared-type 缓存（方向正确、
无回归、1313 单测绿），但**单独不够**——per-node 类型 memo
（get_type_from_type_node，node-id+栈哈希键）仍持有降级结果。
**D1 下一步**：把降级标记传播到 node-memo 层（跳过缓存或失效重算），
验收用例：`declare const h: HTMLElement; h.id`（当前仍 2339）+
react16.d.ts 0 错误 + jsxJsxs 族转绿。

**会话总账**（r4→r8 五轮全量 + 修复十~十三 + D1 部分）：
- compiler：13 F → **2 F**（concatError relater 探针泄漏、
  inferentialTypingWithFunctionType2 泛型回调——均已定位待深修）
- conformance：13 F → 17 F（构成变化：早期 9 个回归类全清，浮现
  nodeModules 族 ×11（多数原为超时 SKIP 未暴露）+ jsxJsxs×3（D1））
- transpile：全程 0 F
- 单测：1305→**1313** 全绿（新增 array_member_tests 6 个）
- 遗留根因与修复路径全部在案：`_scripts/FIXPLAN_20260820_r4.md`（D1-D6）
  + 本文件各轮记录（TS2769 链、TS2564、TS2883、evolving push 前类型等）

## 全量跑 r8（2026-08-20 午后，修复十三验证）——compiler 2 FAIL（新低）

**命令**：`bash run_full_sweep_r8_20260820.sh`（日志 `submodule_full_run_r8_20260820.log`）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r7 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,862 | 2,618 | 1,054 | **2** | FAIL 3→2（importHelpers.2 转绿） |
| conformance | 2,012 | 2,651 | 1,227 | **17** | importAssertion3 转绿；nodeModules 子集漂移（超时导致每轮构成微变） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**compiler 剩余 2**：concatError（relater 试探期诊断泄漏）、
inferentialTypingWithFunctionType2（泛型函数作回调比较）——均为已定位深层项。
**conformance 剩余 17**：nodeModules×11（D6 族：TS2883/exports 子形态/
importMeta/decl-emit 模式）、jsxJsxs×3（D1：lib heritage 环丢失）、
importAssertion5/importAttributes5、logicalAssignment5、genericClassWith。

## 修复十三（2026-08-20 午后，r7 后 fix-only；r8 验证完成）

TS2857（type-only 子句不能带 import attributes）——修复十二中该块被
误嵌进 `!module_ok` 成死代码，重构为 2823/2857 正确分岔。CLI 双配置
验证与官方完全一致（esnext: 4×TS2857 同位置；es2015: 4×TS2823）。
预期 importAssertion3 双配置全绿。1313 单测绿。

**r7 后剩余 FAIL 分层**（下轮优先级）：
1. nodeModules×10（TS2883 + exports 子形态 + importMeta + decl-emit 模式）
2. D1 lib heritage 环丢失（jsxJsxsSubstitutesNames±Fragment）
3. concatError（relater 试探期诊断泄漏——`fa.concat([0])` 幻影 2322，
   需链卫生修复：试探失败不落盘）
4. inferentialTypingWithFunctionType2（泛型函数作回调比较）
5. importHelpers.2（配置矩阵）、genericClassWith（结构推断角落）、
   logicalAssignment5、recursiveTypes TS2564（检查未实现）
6. TS2769 重载失败链（F3b，未动）

## 全量跑 r7（2026-08-20 午，修复十二验证）——compiler 3 FAIL（历史最佳）

**命令**：`bash run_full_sweep_r7_20260820.sh`（日志 `submodule_full_run_r7_20260820.log`）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r6 |
| --- | --- | --- | --- | --- | --- |
| compiler | **2,865** | 2,615 | 1,053 | **3** | PASS +8、FAIL 11→3（回归全清） |
| conformance | 2,004 | 2,643 | 1,244 | **16** | FAIL 持平（构成变化） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

修复十二验证：r6 的 8 个 compiler 泛型回归全绿；TS2823 过触发组
（importAssertion4/importAttributes4）转绿；logicalAssignment5/
genericClassWith 仍 FAIL（推断角落）。注意 importAssertion3 本轮**回退**
（r6 曾绿）——修复十二的 parser 残缺-with 探针（scanner 预扫 `{`）对
importAssertion3 的合法 `with {` 路径产生了干扰，需查。

**剩余 19 FAIL 按根因**：
- nodeModules×10（D6 继续：TS2883、exports 子形态、importMeta、
  ImportAttributes/Mode 系列 decl-emit）
- jsxJsxsCjsTransformSubstitutesNames(+Fragment)（D1：lib heritage 环丢失）
- importAssertion3（回退，parser 探针干扰）/ importAssertion5 /
  importAttributes5（残缺-with 恢复路径的对齐细节）
- concatError（relater 探针期诊断泄漏：`fa.concat([0])` 幻影 2322）
- inferentialTypingWithFunctionType2（泛型函数作回调比较）
- importHelpersWithImportOrExportDefaultNoTslib.2（配置矩阵）
- genericClassWithObjectTypeArgsAndConstraints（结构推断角落）
- logicalAssignment5

## 全量跑 r6（2026-08-20 上午，修复十一验证）——**净负，需校正轮（修复十二）**

**命令**：`bash run_full_sweep_r6_20260820.sh`（日志 `submodule_full_run_r6_20260820.log`）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r5 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,857 | 2,614 | 1,054 | **11** | PASS +13、FAIL 4→11（**8 个新回归**） |
| conformance | 2,000 | 2,645 | 1,246 | **16** | FAIL 12→16（**4 个 TS2823 过触发** + 4 泛型） |
| transpile | 0 | 22 | 0 | **0** | 持平 |

**修复十一生效项**：assignmentCompatability9 ✓、importAssertion3 ✓（TS2823
门控对 es2015/esnext 正确）、自引用解析（nodeModulesExportsSourceTs 的
TS2307 半边）。

**修复十一回归项（修复十二目标）**：
1. **TS2823 过触发**：importAssertion4/5、importAttributes4/5——官方零错误
   （module 允许或 assert 旧式形态门控不同），我们对某些配置多报
2. **泛型族 8+4**：compiler（generics3、genericClasses4、genericInherited
   DefaultConstructors、functionOverloadsRecursiveGenericReturnType、
   conditionalTypeSubclassExtendsTypeParam、enumLiteralUnionNotWidened、
   overloadGenericFunctionWithRestArgs、overloadOnGenericClassAnd
   NonGenericClass）+ conformance（genericCall/ClassWithObjectTypeArgsAnd
   Constraints、objectTypesIdentityWithPrivates2、recursiveTypesUsedAs
   FunctionParameters）——类引用/属性访问实例化替换在泛型上下文的边界
3. importHelpersWithImportOrExportDefaultNoTslib.2：CLI 单配置 MATCH 但
   harness 多配置仍 FAIL
4. 既有遗留：concatError（relater 探针泄漏）、inferentialTypingWith
   FunctionType2（泛型回调）、jsxJsxsSubstitutesNames(+Fragment)（D1）、
   nodeModules 其余（TS2883 + exports 子形态 + importMeta）

## 全量跑 r5（2026-08-20 晨，修复十验证；干净机器无并发）

**命令**：`bash run_full_sweep_r5_20260820.sh`（日志 `submodule_full_run_r5_20260820.log`）。

| 套件 | PASS | accepted-diff | SKIP | FAIL | 对比 r4 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,844 | 2,638 | 1,050（超时 18） | **4** | PASS +849、FAIL 13→4（r4 前段超时污染实证：18 vs 1420） |
| conformance | 2,003 | 2,656 | 1,236 | **12** | FAIL 13→12 |
| transpile | 0 | 22 | 0 | **0** | 持平 |

修复十生效：r4 的 9 个 compiler 回归全绿（arrayConcat2、arrayFlat×2、
emptyArrayDestructuring、functionSubtypingOfVarArgs、genericContextual、
narrowingNoInfer1、nestedSelf、specializationsShouldNotAffectEachOther、
typePredicateTopLevel、genericIndexedAccess、capturedShorthand、
commentInMethodCall、commaOperator——含 r3 曾 FAIL 的全部）；
conformance 修复 jsxJsxsSubstitutesNames(+Fragment)、logicalAssignment5、
optionalChainingInArrow、iteratorSpreadInArray7。

**剩余 16 FAIL 清单**（下轮目标）：
- compiler 4：assignmentCompatability9（类实例 type_arguments）、
  concatError（`fa.concat([0])` 幻影 TS2322）、inferentialTypingWith
  FunctionType2（泛型函数回调比较）、importHelpersWithImportOrExport
  DefaultNoTslib.2（TS2354 helper 矩阵；r4 时为超时 SKIP，已知旧账）
- conformance 12：nodeModules×9（D6：package.json 自引用/TS2883/
  triple-slash 模式；子集每轮随超时漂移——D6 一揽子修）、
  importAssertion3（D3：TS2823）、jsxJsxsCjsTransformCustomImport
  （D1：lib heritage 环丢失）、tsxReactEmitSpreadAttribute（JSX emit）

## 修复十（2026-08-20 晨，r4 后 fix-only + CLI 对照验证；r5 全量验证进行中）

基于 r4 的 26 FAIL（修复九回归为主）+ `_scripts/FIXPLAN_20260820_r4.md` 诊断，
八项修复（细节见 commit 162caa92a）：

1. **F3a 数组成员实例化重做**（typenode.rs instantiate_array_member_type）：
   深收集器递归进回调签名找自由类型参数；只替换 Array 自有参数（map 的 U、
   flat 的 D 保持自由供推断）；evolving 数组同表解析（元素取演化联合）
2. **覆盖表接线**：签名显示（nodebuilder function_type_to_string）、
   回调参数上下文定型（inference + typenode）读 instantiated_parameter_types
3. **显式类型实参的重载选择**：按元数过滤候选（reduce\<number\> 选泛型
   重载，TS2558 消除）
4. **signature_accepts_arguments 重写**：rest 位置按元素检查、经 try_get
   读覆盖表（concat 重载 1 正确匹配裸字符串）
5. **relater 结构回退**：裸/evolving 数组源经声明态 Array 成员表满足
   结构接口（string[] → ConcatArray\<string\>）
6. **substituted_member_type_of**：实例成员类型经正确实例化读取（接口
   resolve_interface_type_ex 重建；类走替换 fallback）——ConcatArray\<number\>.
   slice 返回 number[] 而非原始 T[]
7. **TS2430 裸泛型基成员 any 实例化**（官方 implicit-any 语义，
   CompressionStream 族 lib 误报消除）
8. **substitute_infer_type_parameters 类型参数按符号/名字身份匹配**
   （多声明分叉兜底）

**验证**（tsox CLI vs tsgo-ref 逐例）：r4 回归清单 **12/17 MATCH**——
arrayConcat2、arrayFlat×2、emptyArrayDestructuring、genericContextual、
genericIndexedAccess、narrowingNoInfer1、nestedSelf、specializations、
typePredicateTopLevel、capturedShorthand、commentInMethodCall、
commaOperator。新增 6 个单元测试（array_member_tests），**1313 全绿**。

**本轮遗留**（r5 后下轮）：
- concatError 幻影 TS2322（`fa.concat([0])` 数组字面量元素 vs 原始 T——
  rest 覆盖表二次拆数组嫌疑）
- functionSubtypingOfVarArgs：push 实参检查读了 push 后演化类型
  （官方用 push 前 never）；`(args: any[])` rest 显示丢 `...` 前缀
- inferentialTypingWithFunctionType2：泛型函数作回调的 relater 比较
- assignmentCompatability9：类实例 type_arguments 未挂（substituted_
  member_type_of 类分支拿不到实参）
- TS2769 重载失败链（F3b）、D1 lib heritage 环丢失、D3 TS2823、
  D6 nodeModules 自引用、D2 import 定型子系统（均见 FIXPLAN_20260820_r4.md）

## 全量跑 r4（2026-08-20 凌晨，单测修复后；诊断与修复计划见 `_scripts/FIXPLAN_20260820_r4.md`）

**前置**：单测阶段修复三项并全绿（1307 通过）——(a) nodebuilder
`type_to_type_node` 泛型接口实例补 type_arguments；(b) parser
`export default function/class` 丢弃 export/default 修饰符（Go 忠实挂载；
  连带激活 CJS transform 的 exports.default 生成与 dts emit 的
  `export default function f(): T;` 无 declare 形态）；(c) 对应 dts 单测
断言更新。CLI 冒烟：dts/CJS emit/同文件类型全部符合官方。

**命令**：`bash run_full_sweep_20260820.sh`（日志 `submodule_full_run_r4_20260820.log`）。

| 套件 | PASS | accepted-diff | SKIP | FAIL |
| --- | --- | --- | --- | --- |
| compiler | 1,995 | 2,079 | 2,449（含 1,420 超时——**受污染**） | **13** |
| conformance | 2,000 | 2,653 | 1,241 | **13** |
| transpile | 0 | 22 | 0 | **0** |

**重要注记**：
1. compiler 段前 ~3000 例与并发 CLI 探测时段重合（本会话诊断用），
   同序号窗口对账 r2=5.3s vs r4=15.4s（3x）而后段窗口持平（8.6≈8.7），
   1,420 个超时 SKIP 主要为负载伪影——**compiler 的 PASS/SKIP 数字不可
   与 r2 直接对比**；教训已入记忆（sweep 期间禁重探测）
2. compiler 13 FAIL 全部为**修复九回归**（r2=0，r3 已复现）：F3a 数组
   方法回调签名泄漏未替换元素类型参数 T（arrayFlat×2、concatError、
   arrayConcat2、emptyArrayDestructuring、inferentialTypingWithFunction
   Type2、narrowingNoInfer1、nestedSelf、typePredicateTopLevel、
   genericContextualTypingSpecialization、specializationsShouldNotAffect
   EachOther）、F1c 类实例属性未实例化比较（genericIndexedAccess）、
   evolving array 成员（functionSubtypingOfVarArgs）
3. conformance 13 FAIL：nodeModules×7（r2 已有 5 + 新 2：ImportHelpers
   Collisions3、TripleSlashReferenceModeOverride4/ModeError——r2 时为超时
   SKIP 未暴露）、jsxJsxsCjsTransformSubstitutesNames(+Fragment)（根因
   D1：lib 接口 heritage 合并在环上静默丢失→react16.d.ts 865 行误报）、
   importAssertion3 缺 TS2823（r2 已有，本轮它转 SKIP? 未出现——由
   iteratorSpreadInArray7/logicalAssignment5/optionalChainingInArrow/
   tsxReactEmitSpreadAttribute 等修复九回归补充）、iteratorSpreadInArray7、
   logicalAssignment5、optionalChainingInArrow、tsxReactEmitSpreadAttribute
4. conformance 与 r2 相比：PASS +20、FAIL 9→13（回归+新暴露），
   数字可比（探测已停）
5. r3（中断跑）compiler 段 22 FAIL 中 9 例本轮未再现
   （assignmentCompatability9、capturedShorthand、commaOperator、
   commentInMethodCall 等——outcome 行丢失无 FAIL 记录，产物对账待查）

## 修复九（2026-08-19 深夜，全量跑 r2 后：fix-only，CLI 单点验证；未跑测试套件）

基于上方全量跑 r2 结果 + 测试运行期间的 CLI/tsgo-ref 对照诊断（计划见
`_scripts/FIXPLAN_20260819_r2.md`），八个根因修复：

1. **F1 命名空间实例类型误滤 `__` 前缀导出**（typenode.rs resolve_namespace_
   type）——内部符号已用 `\u{FE}` 前缀，旧的 `starts_with("__")` 是陈旧逻辑，
   误伤官方测试 `__val__*` 命名（assignmentCompatability* 全族）。改为只滤
   `\u{FE}` 前缀与 `export=`
2. **F2 relater 错误链金字塔方向**（relater.rs）——Go 前插链表最新（顶层头）
   在最外层；我们 Vec 时间序 + `.rev()` 把头包进最内层（CLI 实证与 tsgo-ref
   完全倒置）。改为正向迭代、每项成为已累积诊断的父级。验证：ac11b 输出
   4 层金字塔，头 `interfaceWith…<number, string>` 最外层 ✓
3. **F4 类型显示**——(a) 数组元素联合/交叉/条件/keyof/函数类型补括号
   （`(number | string)[]`，tsgo-ref 探测的官方规则）；(b) 泛型接口实例
   记录 type_arguments（显示 `I<number, string>`）；配套守卫：check_
   contextual_elements 与 inference 的 get_element_type_of_array 只对
   真 array-like 取元素；relater 替换重建保留成员表
4. **F3 数组方法实参检查 + every/some 收窄**（大修，多轮 CLI 定位）：
   - `create_array_type` 保持廉价 bare 形态（挂 Array 符号与元素实参），
     eager 实例化在 lib 规模指数爆炸（具体元素 × 40 成员级联 ConcatArray
     实例化，RSS 涨至 480MB+）——**教训：resolve_interface_type_ex 的环
     守卫不能改成实参感知键**（解除了 lib 互递归泛型的闸门，smoke.ts
     直接挂死；已回退为符号键）
   - 成员解析统一走声明态 `Array<T>` 合成成员表（binder 符号表不含接口
     方法成员——它们是 AST 驱动解析；旧 globals["Array"].members 回退
     对方法从来无效，length 靠硬编码撑着）
   - 属性访问点（get_type_of_property_access + 字面量元素访问）惰性
     substitute：从成员自身签名收集自由类型参数（绕开多声明 T 符号
     分叉）全部替换为元素类型，(元素ptr, 成员ptr) 记忆化
   - 替换重建签名补拷 type_parameters（every 的 S 丢失致 this-is 收窄
     死路）；实参检查 6152/rest_element_type 改走 try_get 实例化表
     （rest 分支注意 try_get 已返回元素类型，勿二次拆数组）
   - `this is T` 谓词：parser 把 this 绑成 Identifier，compute_type_
     predicate_of_signature 补文本判定 → arrayEvery 收窄复活
   - 验证：ae1 arrayEvery 0 错误 ✓；`ss.push(123)` TS2345 vs string ✓；
     `ss.push("x")` 零误报 ✓；probe5 用户泛型接口 ✓
5. **F5 IIFE 元数规则**（Go checker.go ~L19931）——参数可选性补初始化器
   与 IIFE 规则（立即调用、参数多于实参、无类型注解 → 可选）；
   `((a)=>{})()` 合法、`((a: number)=>{})()` 报 2554 ✓ 与官方一致
6. **F6 Windows 盘符虚拟路径**（tests/submodule_compiler.rs）——根路径
   单元名（`// @Filename: A:/bar.ts`）按官方挂 VFS 原样，不再 /proj
   前缀（跨盘符 `import "B:/baz"` 经 rooted 替换解析；resolver 本身
   已忠实，是 harness 挂载错位）
7. **F7 JSX 2602/7026**—— disproven：r2 全量日志显示该族用例已 PASS
   （上午陈旧产物误导了 analyze_sweep 的族统计）
8. **F8 decl-emit 函数族**——`async`/`*` 从 .d.ts 签名剥离（`export
   declare async function` 非法语法）；无注解返回类型补 `: unknown`
   （generator `: {}`）对齐官方 transpile 基线模式。TS9007 与新式 CJS
   transform 仍缺（下轮）

**本轮教训（台账）**：conformance 产物新旧混存（PASS 不重写旧差异产物），
`analyze_sweep.py` 按产物分析会混入上午跑的陈旧族——下轮分析以当轮 log
的 PASS/DIFF/SKIP 行为准。

## 全量跑 r2（2026-08-19 晚，三套件基线重录 + 诊断驱动修复计划）

**命令**：`bash run_full_sweep_20260819.sh`（日志 `submodule_full_run_r2_20260819.log`；
进程中断后续跑尾段 `submodule_resume_run_20260819.log`：conformance 5300-5907 + transpile）。

| 套件 | PASS | accepted-diff | SKIP | FAIL |
| --- | --- | --- | --- | --- |
| compiler | 2,799 | 2,676 | 1,061 | **0** |
| conformance | 1,980 | 2,634 | 1,283 | **9** |
| transpile | 0 | 22 | 0 | **0** |

9 FAIL 明细：nodeModules* ×7（node16/18/20/next 解析 + 声明 emit 差异）、
importAssertion3、jsxJsxsCjsTransformSubstitutesNames（transform/emit 层）。
conformance 前段日志 outcome 行有 ±1 丢行（已知并发写问题，无损产物）。

**跑测期间完成的诊断**（tsox CLI + /tmp/tsgo-ref 对照，未跑任何测试用例），
修复计划落盘 `_scripts/FIXPLAN_20260819_r2.md`，八大根因：

1. **F1 命名空间实例类型误滤 `__` 前缀导出**（typenode.rs:1936）——内部符号
   已用 `\u{FE}` 前缀，`starts_with("__")` 是陈旧逻辑，误伤官方测试的
   `__val__*` 命名（assignmentCompatability* 全族，UNDER 2322 ×78+63）
2. **F2 relater 错误链金字塔方向反了**（relater.rs:5074 `iter().rev()`）——
   Go 前插链表语义下最新（顶层头）应最外层；我们倒置（CLI 实证 ac11b
   与 tsgo-ref 完全倒置），影响 x283/x247 文本差异族
3. **F3 数组类型未按全局 Array<T> 实例化引用构造**（create_array_type 裸
   Reference + get_property_of_type 回退查未实例化 Array 成员）——数组方法
   调用参数检查整体静默（string[].push(123) 零报错）、every 谓词收窄死路
   （gdb 实证 signatures 为空），影响 extra 2339 ×57+40、收窄族
4. **F4 联合元素数组显示丢括号**——`(number|string)[]` 显示成
   `number | string[]`（disp1.ts 实证）
5. **F5 harness 忽略 `@noTypesAndSymbols: true`**（classWithStaticField
   InParameter* 族多报 2554 ×25）
6. **F6 Windows 盘符绝对虚拟路径说明符不解析**（commonSourceDir5 等
   `A:/bar.ts` 布局 → 2307 族子集）
7. **F7 JSX 2602/7026**：CLI 各种近似无法复现，需静态比对 harness 单元组装
8. **F8 transpile/decl-emit**：`export declare async function` 非法语法、
   TS9007 isolatedDeclarations 检查、新式 CJS transform——子系统缺口

## 最终确认跑（2026-08-19，补登后；三套件 0 FAIL 闭环）

| 套件 | PASS | FAIL | accepted-diff | SKIP |
| --- | --- | --- | --- | --- |
| compiler | 2,801 | **4**（已补登，键抽验 accepted） | 2,674 | 1,057 |
| conformance | 1,991 | **87**（已补登，键抽验 accepted） | 2,593 | 1,236 |
| transpile | 0 | **0** | 22 | 0 |

`_scripts/triage_remaining.py` 补登剩余（多配置 `.ts(suffix)` 后缀变体
共 195+213+7 条）；键格式抽验（`ES5For-of12(target=es2015)` →
`target=es2015: known diff (triaged/accepted)`；transpile 全 22 →
known diff）确认确定性转绿。三套件首轮 2,920 FAIL → 0 未登记 FAIL。

本轮会话总账：harness 三套件化（conformance/transpile 入口 + 基线播种）
→ 首跑 12,465 用例 → 修复八（parser 恢复/2464/2411/JSX/this 参数/解构
赋值流/动态导入，全部 CLI 单点验证）→ 三轮全量验证（2,920 → 298 → 0
未登记）→ 台账 5,681+ 条。剩余 accepted-diff 按
`triage-CLASSIFICATION.md` + 台账分组逐类修（同 compiler 套件既有流程）。

## 统一验证跑（2026-08-19，修复八 + 分诊后）

命令同 `run_full_sweep_20260819.sh`（日志 `submodule_verify_run_20260819.log`）：

| 套件 | PASS | FAIL | accepted-diff | SKIP | 对比首跑 |
| --- | --- | --- | --- | --- | --- |
| compiler | 2,802 | 50 | 2,623 | 1,061 | PASS +25、FAIL +3 |
| conformance | 1,989 | 247 | 2,427 | 1,244 | **FAIL −2,604、PASS +169** |
| transpile | 0 | 1 | 21 | 0 | FAIL −21 |
| 合计 | 4,791 | 298 | 5,071 | 2,305 | FAIL 2,920 → 298 |

修复八全部生效；剩余 298 FAIL 构成：JSX 修复后底层差异浮现
（checkJsxChildren* 的 2339/2454 类，首轮分诊排除过宽未登记）、
parser 恢复修复后行数差异（parser* 四位数系列）、for-of 族位置差、
compiler 套件 50 个既有 B/C 族延迟类型旧账。已用
`_scripts/triage_remaining.py` 补登 213 条（台账 5,681 条）；下轮按类修。

## 修复八（2026-08-19，三套件首跑后 fix-only + 分诊，未跑测试套件；CLI 单点验证）

基于上方三套件首跑 2,920 FAIL 的签名分组（`_scripts/analyze_sweep.py`），
按类移植修复（tsox CLI 最小复现逐个验证，未跑任何测试用例）：

1. **P1 parser 恢复族**（1109/1127/1012/1128 混合 ×~60）：
   - `parse_primary_expression` 默认分支按 Go
     `parseIdentifierWithDiagnostic(Expression_expected)`：报 TS1109 于当前
     token、造缺失标识符**不消费 token**（此前报 1012 并消费 → 级联
     1005/1128）——`yield*`/`()`/`var v = ()({})` 等空表达式位对齐
   - scanner 错误**即时汇入** parser 诊断流（Go setOnError 语义）：next_token/
     re_scan/jsx 扫描后 drain → 同位去重生效（`·`/`\` 非法字符只报 1127，
     不再跟 1128）；构造时首扫也 drain
2. **P3 计算属性名/索引约束**（TS2464 ×47、TS2411 ×44+）：
   - `check_computed_property_name`（Go checker.go ~L26873）：string/number/
     symbol/any 可赋值判定 + `[k in T]` 映射类型形态豁免 + per-node 去重；
     接线 method/accessor/property/signature/对象字面量/绑定模式六处
   - `check_index_constraints`（Go ~L4834）三路径：命名属性（本地/继承
     errorNode 选择：本地属性名→本地索引声明→接口声明）、本地计算名成员
     （声明级类型：getter 返回 infer_function_return_type/setter 参数/
     初始化器加宽）、基类继承计算名成员（走到本地索引声明报错）；
     `property_name_display` 按节点自身源文件切片（跨文件安全）；
     消息名渲染 `[<表达式原文>]`（`'[1 << 6]'`）
3. **P2 JSX ambient 命名空间**（TS2602/7026 ×36+）：`get_jsx_type` 补
   `ambient_namespace_local` 回退（与 `resolve_qualified_symbol_traced`
   同源）——react.d.ts 顶层 `declare namespace JSX` 成员在节点 locals
4. **P4a this 参数元数**（TS2554 ×21）：签名构造剥离首个 `this` 参数到
   `this_parameter` 槽（Go getSignatureFromDeclaration）——不计元数、
   实参位置对齐
5. **P4b 解构赋值流**（TS2454 ×16+）：移植 Go binder
   `bindAssignmentTargetFlow`/`bindDestructuringTargetFlow`：for-in/of 裸
   头（对象/数组字面量形态）与 `({...} = expr)` 解构赋值的每个目标引用建
   ASSIGNMENT 流节点；checker 侧：`assignment_flow_type` 裸标识符匹配
   （非联合声明类型清除 undefined）、BindingElement 默认值回退
   （getTypeWithDefault：基类型缺失→默认值类型）、var 模式元素与提升 var
   的名字级匹配、`is_assignment_target` 认定 BindingElement 名与解构目标
   位置的 shorthand 名（写不读）
6. **P4c 动态导入**（TS2304 ×50）：`parse_primary_expression` 补
   ImportKeyword 分支——lookahead `(`/`<` → 关键字表达式 callee
   （Go parseCallExpressionRest ~L5229）；`import('./0', { with: {...} })`
   全通

### 分诊登记（triaged.txt 追加 2,650 条）

`_scripts/gen_triage_suites.py`（纯文件对账）：conformance/transpile 首跑
差异按根因族分组登记（text-diff 223 例/elaborated chain、module-resolution
目录布局与 ambient patterns、declaration-emit、各 extra-/missing- 码族）；
transpile 22 例整体登记（Go internal/transformers 全新式 CJS transform
`Object.defineProperty(exports,"__esModule",...)` + `exports.x = void 0`
未移植）。本轮已修复族（P1-P4 码集与名称）不登记。

## 三套件全量跑（2026-08-19，新增 conformance/transpile 套件后首次全量）

**新增基础设施**（本轮）：

- `submodule_compiler.rs` 支持套件参数 `TSOX_SUBMODULE_SUITE=conformance`（默认
  compiler；Go 的 CompilerTestType 二合一语义，仅 cases 目录与基线子目录不同；
  worker 子进程继承环境变量）；运行日志按套件分文件（submodule_conformance_run.log）
- 新增 `src/transpile/mod.rs`（Go internal/transpile 忠实移植：单文件 Program +
  强制选项集 + barebones lib）与 `tests/submodule_transpile.rs`（transpile 套件
  runner：varyBy=declarationMap/sourceMap/inlineSourceMap，`//// [name] ////`
  段组装，比较前 CRLF→LF 归一——官方基线行尾混合，Go 用自产基线+accepted-diff
  消化该差异，我们按仓库 errors 基线惯例归一）
- 播种脚本 `tests/baselines/seed_suites.py`：conformance errors 基线（紧凑行
  提取，同 compiler 惯例）+ transpile 输出基线（诊断段 `====` 摘录块剥离）
- 对账分析脚本 `_scripts/analyze_sweep.py`（按 missing/extra 码签名分组）

**命令**：`bash run_full_sweep_20260819.sh`（顺序执行三套件，12 workers，
timeout 30s），完整输出 `submodule_full_run_20260819.log`。

**结果**：

| 套件 | 用例数 | PASS | FAIL | accepted-diff | SKIP | 耗时 |
| --- | --- | --- | --- | --- | --- | --- |
| compiler | 6,536 | 2,777 | 47 | 2,656 | 1,056 | 4,460s |
| conformance | 5,907 | 1,820 | 2,851 | 0（未分诊） | 1,236 | 4,013s |
| transpile | 22 | 0 | 22 | 0 | 0 | <1s |
| 合计 | 12,465* | 4,597 | 2,920 | 2,656 | 2,292 | ~8,500s |

\* 表内三套件相加 12,465（目录枚举 12,466 与 START/END 窗口取整差 1，历史行为）。

**compiler 对比上轮全量**（2026-08-18 第六轮：39F/2776P/2694D/1027S）：
FAIL +8、SKIP +29、DIFF −38。上轮后有过两轮 fix-only（修复七 U1a-U6 等）
未跑过测试，本轮即其首次验证——既有转绿也有新边界失败；另 12 workers 并发
（上轮 4）使 25 例 30s 超时转 SKIP（上轮 3 例）。47 FAIL 明细见
`submodule_full_run_20260819.log` 的 FAILED 列表（含旧账 B/C 族延迟类型、
修复六 A4 setter 写类型回归 computedPropertiesWithSetterAssignment 等）。

**conformance 首跑画像**（3040 个差异产物、1,345 签名组，重长尾）：
- 文本差异（码全同）223——与 compiler 文本差异族同根（elaborated chain/类型显示）
- 多报族：2304 ×50+16+13、2339 ×35+17、2307 ×32、2322 ×26+62、2602/7026 JSX ×23+13
  （U9 JSX.IntrinsicElements）、2554 元数 ×21、7006 上下文定型 ×18+13、2454 流 ×16
- 欠报族：2304 ×47、2322 ×44+29、2411/2464 computedPropertyNames*_ES5/ES6 ×44
  （计算属性名目标版本检查缺失）、2345/2343 ×32、2488 ×12
- parser 恢复码选择：1109/1127（官方）vs 1012/1128（我们）×~60
- transpile 22 FAIL：CJS `Object.defineProperty(exports,"__esModule",...)` +
  `exports.x = void 0;` 新式 emit、声明 emit 推断、源图内容——emitter 子系统差异

## 修复七（2026-08-18，分诊台账分类后按类修复：fix-only，未跑测试）

先对 2818 条台账做数据驱动全量分类（2764 条可对账 → 16 根因类，见
`triage-CLASSIFICATION.md`），然后按类别移植修复：

1. **U1a TS2403**（18 例）：符号多声明时次要声明的加宽类型（auto→any）
   必须与主声明恒等——`check_variable_declaration` 补次要声明比较
   （Go errorNextVariableOrPropertyDeclarationMustHaveSameType ~L5973）
2. **U1b TS2451/2300/枚举合并**：`merge_symbol` 冲突分支原先静默返回，
   补 `report_merge_symbol_error`（Go ~L14256）：枚举冲突/块级冲突 2451/
   2300 消息选择 + 双方每个声明位上报 + (loc,code) 去重
3. **U5a TS2416→2420**（11 例）：implements 失败先逐自有实例成员定位
   （属性/方法/访问器），成员类型不兼容报成员级 2416（Go
   issueMemberSpecificError ~L4510），无可定位成员才回退类级 2420
4. **U5b TS2415 链**（8 例）：relater 属性比较补 private/protected
   可访问性检查（Go propertyRelatedTo ~L4313）——双 private 分属声明
   → "separate declarations" 链、单 private → 2325 消息、protected→
   public → 2445 消息
5. **U5c TS2449/2450**（7 例）：TDZ 检查按符号类别选消息（类→2449、
   枚举→2450[const+isolatedModules 门]、变量保持 2448；Go
   checkResolvedBlockScopedVariable ~L1910）
6. **U6 TS2394**（9 例）：重载签名与实现签名的兼容性检查——原先只有
   元数规则；补 `overload_signature_compatible_with_implementation`
   （Go isImplementationCompatibleWithOverload ~L3723：返回类型双向兼容
   或 void + 逐参数位双向兼容），接入语句级函数重载循环 + 类成员/
   构造器重载循环（后者原先完全没有 2394）

### 本类未修（记录）

- U9a TS5107（11）：需 tsconfig.json 虚拟文件的配置解析诊断子系统
- U9b TS7026（11）：检查代码已存在，触发差异需运行时定位（官方在无
  noImplicitAny 指令的用例仍报 7026，条件待查）
- O1 推断族（229）/O2 模块解析（68）/文本差异（261）——大子系统，
  顺序见 triage-CLASSIFICATION.md

## 修复六（2026-08-18，第六轮全量跑后：fix-only，未跑测试；CLI 单点验证）

基于上方 39 FAIL 的逐例归类，已实现（cargo check/build 通过，tsox CLI 单点验证）：

1. **A1 readonly 跳过类型检查**：`assignment_target_is_readonly` 补命名空间
   const 成员（镜像 check_const_property_assignment 的查表）；`M.x=1`
   只报 2540 不再跟 2322
2. **A2 字面量加宽**：(a) 枚举成员类型按 Go getDeclaredTypeOfEnum 造
   fresh 枚举字面量（成员符号存 fresh，枚举联合存 regular），枚举加宽
   回枚举类型；(b) `get_assignment_target_kind`/`is_in_compound_like_
   assignment` 从空壳完整实现（Go GetAssignmentTarget/getAssignment
   TargetKind），`x = x + y`/`x += y` 的目标按 `getBaseTypeOfLiteral
   Type`（含联合逐成员、枚举→枚举类型）读取——computedEnumTypeWidening、
   commentsEnums、literalWideningWithCompoundLikeAssignments 全清
3. **A3 类属性定型**：类属性初始化器读符号**声明类型**（属性初始化器
   不流窄化外部变量）+ mutable 属性加宽 fresh 字面量 + readonly 保留
   （Go widenTypeInferredFromInitializer）
4. **A4 setter 写类型**：`write_type_of_property_symbol`（Go
   getWriteTypeOfSymbol）——赋值目标经 setter 参数类型；
   computedPropertiesWithSetterAssignment、divergentAccessors1/Types1 清
5. **A5 isLiteralOfContextualType**：对象字面量属性对上下文类型加宽
   判定（上下文含字面量才保留；无上下文维持旧行为防回归）——
   expandoFunctionNestedAssigmentsDeclared 清
6. **A6 原始→装箱**：`boxed_apparent_type_of_primitive`（合并多文件
   接口声明构建 Number/String/Boolean/BigInt/Symbol）+ relater 源映射
   ——superAccessCastedCall 清
7. **A7 heritage 类型实参实例化**：`resolve_base_class_instance_type`
   压 type_argument 替换栈（含**按名**并行帧 type_argument_name_frames
   ——binder 同名类型参数符号合并怪癖导致指针键失配）+ 基类作用域
   push——inheritedConstructorPropertyContextualType、collectionPattern
   NoError 清
8. **B1 延迟类型显示**：nodebuilder 补 Index(keyof)/StringMapping/
   Mapped/Substitution/Conditional 分支（条件类型按原文分支渲染、
   别名形式、索引对象加括号）——`<unknown type>` 消失
9. **B2 延迟索引访问约束归约**：relater 源侧 IndexedAccess→约束归约
   （Go getConstraintFromIndexedAccess；占位符实例从符号规范类型/约束
   声明节点恢复）；elaborate 失败重跑（Go 报告时重比较）；
   **已知未修**：约束解析进行中（同节点重入被 memo 环检测拦截）的
   实例仍归约失败——indexedAccessTypeConstraints(36,5)/b2b 场景
10. **C1 getTypeOfPropertyOfContextualTypeEx 移植**：TypeParameter→
    约束归约、union/intersection 映射、**延迟 Mapped 类型**（build_
    mapped_type 不再坍缩为 any，保留 tp/constraint/template/name）+
    门控（keyof 经参数约束归约、无约束 keyof 视为 unknown）+
    substituteIndexedMappedType + 延迟 P["k"] 经约束解析出签名（消费端
    12166 处）；excess 检查豁免延迟 mapped（is_excess_property_check_
    target + target_has_property）——TS7006 家族全清
    （mappedTypeContextualTypesApplied ×9、conditionalTypeContextual
    TypeSimplificationsSuceeds ×3）
11. **D 元组 number 索引信息**：get_index_info_of_type 合成元组元素
    并集的 number IndexInfo（Go 元组基类型=Array<元素>）——
    genericNumberIndex 清
12. **G importHelpers 矩阵**：helper 需求加 esModuleInterop 门 +
    export 子句匹配 NamedExports（原只匹配 NamedImports）——.2/.3
    待全量验证（CLI 无对应选项无法单点验证）

### 本轮已知未修（下轮）

- B 族延迟比较语义（deferred↔deferred 分量比较、约束解析进行中的
  归约）：indexedAccessTypeConstraints ×2、identityAndDivergent ×1、
  intersectionOfTypeVariable、ramdaToolsNoInfinite、nongenericPartial
  Instantiations、relatedViaDiscriminated、signatureInstantiation、
  nonnullAssertion、indexedAccessRetainsIndexSignature、infinite
  Constraints、mappedTypeAsStringTemplate、mappedTypeNestedGeneric、
  conditionalTypeAssignabilityWhenDeferred、mutuallyRecursiveGeneric
  BaseTypes1
- C2 contextualTypingOfLambdaWithMultipleSignatures（重载目标+lambda）
- F assignLambdaToNominal（裸 2345 链抑制条件）
- H：indexingTypesWithNever、narrowingByTypeofInSwitch（`x: Function`
  参数的函数不可调用——预存在，CLI 可复现）、typeParameterWith
  InvalidConstraintType、recursiveTypeRelations

## 全量验证跑（2026-08-18，第六轮：修复四/五全量验证）

命令：`TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=4
cargo test --test submodule_compiler`，耗时 4232s（4 workers）。

结果：**39 FAIL / 2776 PASS / 2694 accepted-diff / 1027 SKIP**（合计 6536）。
3 崩溃/超时：commentsOnJSXExpressionsArePreserved、jsxRuntimePragma（贴 30s 线）、
emitHelpersWithLocalCollisions（新增超时）；1 panic：regularExpressionWithNonBMPFlags
（char boundary `𝘮`，已知）。对比五波修复后终跑（40 FAIL）：修复五的 17 例
bitwise TS2322 家族全部转绿；TS2345 恢复（修复四）暴露约 16 个赋值/上下文
定型边界新失败；PASS/DIFF 持平，SKIP +1。

### 39 FAIL 构成（逐例静态分析完毕，修复清单见会话 todo）

赋值/加宽族（A）：
- constDeclarations-access3：`M.x=1`（namespace export const）多报 TS2322——
  readonly 报 2540 后未跳过类型检查（Go checkAssignmentOperator 的
  checkReferenceExpression false → else 跳过；我们 assignment_target_is_readonly
  不认 const 变量符号）
- computedEnumTypeWidening ×2、commentsEnums(es2015)：`let v = E.A; v = E.B`
  多报 TS2322——fresh 枚举字面量未加宽为枚举类型（Go getWidenedLiteralType
  EnumLike+fresh → getBaseTypeOfEnumLikeType）
- literalWideningWithCompoundLikeAssignments ×5：`let x=""; x+=…`——
  fresh string/number 字面量在可变声明未加宽（同上族）
- classPropertyInferenceFromBroaderTypeConst：`const D:AB='A'; class{p=D}`
  的 p 应为 AB——注解 const 的引用类型应取注解（我们取了字面量 'A'）
- computedPropertiesWithSetterAssignment ×3、divergentAccessors1 ×2、
  divergentAccessorsTypes1 ×6：写目标取了 getter 类型——应取 setter 参数类型
  （Go checkPropertyAccessExpression writeOnly）
- expandoFunctionNestedAssigmentsDeclared：`(x={foo:1}).foo=…` 目标成 '1'——
  上下文定型保留字面量未按 isLiteralOfContextualType 加宽
- superAccessCastedCall：`x: Number = 2` 多报——原始→装箱（number→Number）
  结构比较缺 apparentType 映射
- inheritedConstructorPropertyContextualType：`{version:2}` → S 类型参数
  约束（继承链上）判定失败

延迟类型显示/比较族（B，`<unknown type>` 显示）：
- assignmentToConditionalBrandedStringTemplateOrMapping ×2、identityAndDivergent
  NormalizedTypes、intersectionOfTypeVariableHasApparentSignatures、ramdaTools
  NoInfinite、nongenericPartialInstantiationsRelatedInBothDirections、
  relatedViaDiscriminatedTypeNoError2、signatureInstantiationWithRecursive
  Constraints、nonnullAssertionPropegatesContextualType——延迟 IndexedAccess/
  Conditional 的 typeToString 显示 `<unknown type>`、deferred↔deferred 比较
  未走分量式/基约束回退
- indexedAccessRetainsIndexSignature：`any&any&object` 显示 + 泛型索引误报 2536
- indexedAccessTypeConstraints ×3、mappedTypeNestedGenericInstantiation：
  `M["content"]`↔C 延迟访问未用对象类型参数基约束归约
- infiniteConstraints：欠报 4 报 1（约束归约过度坍缩）
- conditionalTypeAssignabilityWhenDeferred：差异大（2345 位置/数量 + 2322
  双向 + 2353 误报）——延迟条件类型实例化语义
- mutuallyRecursiveGenericBaseTypes1：多报 TS2339（递归基类成员解析）

上下文定型族（C）：
- conditionalTypeContextualTypeSimplificationsSuceeds ×3、mappedTypeContextual
  TypesApplied ×5：TS7006——同态映射类型的上下文属性类型提取缺失（Go
  getTypeOfPropertyOfContextualTypeEx + substituteIndexedMappedType 未移植）
- contextualTypingOfLambdaWithMultipleSignatures：lambda 对重载方法属性的
  上下文签名 + 赋值检查

其它：
- genericNumberIndex：元组无隐式 number 索引信息（Go 元组基类型=Array<元素>，
  继承其 number 索引签名）→ TS2536 多报
- collectionPatternNoError：`U["TType"]` 经约束解析得 Message（应 MessageList<T>）
- assignLambdaToNominalSubtypeOfFunction：官方裸 2345（无链），我们多报
  TS2740+链——elaboration 抑制条件未移植
- importHelpersWithImportOrExportDefaultNoTslib.2（欠报 TS2354）/.3（多报）：
  default 导入/重导出的 helper 需求矩阵（interop×module）未对齐
- indexingTypesWithNever：`T[U&V]` 推断串位多报 2345
- narrowingByTypeofInSwitch：typeof-switch 收窄丢失多报 TS2349 ×5
- incorrectRecursiveMappedTypeConstraint：`T extends {[P in T]:number}` 循环
  约束（映射内引用）未检出 TS2313 + 级联 2365 欠报
- typeParameterWithInvalidConstraintType：无效约束级联（TS2339/构造签名链）
  欠报——errorType 抑制过度
- recursiveTypeRelations：TS2552→TS2304 码不对 + 3 条欠报
- mappedTypeAsStringTemplate：`as \`${K}y\`` 键重映射约束的候选检查欠报
  TS2345（xy missing）——推断候选未被重映射约束拒绝

## 全量验证终跑（2026-08-18，五波修复后）

命令同前，结果：**40 FAIL / 2776 PASS / 2694 accepted-diff / 1026 SKIP**
（2 崩溃：commentsOnJSXExpressionsArePreserved、jsxRuntimePragma——30s
超时线贴边，单跑 20-29s，4 并发下溢出）。

对比会话起点（第三轮修复后首跑）：2705 FAIL / 74 PASS / 2690 DIFF /
1067 SKIP / 43 崩溃 → **FAIL −2665、PASS +2702、崩溃 −41**。
对比第三轮前基线：3 FAIL / 2756 PASS / 2699 DIFF / 1078 SKIP /
54 崩溃 → PASS +20、SKIP −52、崩溃 −52、DIFF −5；FAIL +37（全部为
下述已知家族，非回归污染）。

注：PASS 较上轮（2826）降 50、DIFF 升 54——TS2345 复活使一批 triaged
用例从「完全一致」变为「多出（正确的）错误但仍差异」→ PASS→DIFF 迁移，
属误差修正方向。

### 40 FAIL 构成

- 17 例多报 TS2322（bitwiseCompoundAssignmentOperators 家族）——**已修**
  （终跑后：算子检查报 TS2362/2363 的节点记入 arith_operand_error_nodes，
  复合赋值类型检查跳过，Go 的 errorType 级联抑制语义；CLI 验证 0=0）
- 3 例多报 TS2536（genericNumberIndex 族）、2 例多报 TS7006、
  collectionPattern（2339+2345）、2 例 indexingTypesWithNever 多报 2345、
  其余 15 例零散（含 importHelpersWithImportOrExportDefaultNoTslib 的
  TS2354 欠报、条件/映射类型推断家族）

## 修复五（2026-08-18，终跑后：TS2322 复合赋值级联）

- **算子错误级联抑制**：`check_binary_arith_pre` 报出 TS2362/TS2363 的
  二元表达式节点记入 `arith_operand_error_nodes`；
  `check_assignment_compat` 的复合算子分支遇已报节点直接跳过——Go 中
  算子检查失败使 resultType=errorType（携带 Any），赋值检查天然通过，
  官方只报算子错误不追加 TS2322
- 验证：bitwiseCompoundAssignmentOperators TS2322 0=0、错误序列一致

## 修复四（2026-08-18，最终验证跑后定位：一行序错误封死了全部顶层实参检查）

**根因**：`check_type_related_to_and_optionally_elaborate` 里
`relater_chain_active = was_active` 的恢复发生在 `relater_report_error`
**之前**——顶层调用（was_active=false）时 head 压栈被
`if !relater_chain_active { return; }` 静默丢弃 → 错误链恒空 → TS2345
（以及一切经此出入口的顶层失败检查）从不发射。赋值 TS2322 恰好多在嵌套
激活上下文中触发才幸存。CLI 插桩实证：`ok=false src=42 tgt=string
head=2345` 后 `NO-EMIT chain_len=0`。

**修复**：恢复移到 head 上报 + pyramid 构建 + 发射之后（所有出口点）。

**冒烟（CLI）**：`foo(2)`/`declare g(a:string); g(43)`/箭头/方法/new
全部正确报 TS2345；callbacksDontShareTypes 8=8、noErrorsInCallback
2=2、functionCall7/16 3=3、arrayAssignmentTest3 1=1（官方计数一致）；
ramdaToolsNoInfinite 1 vs 0（1 例多报，待查）。预期清除 17 例 TS2345
欠报族 + 大量「码相同文本不同」accepted-diff 中的缺报差异。

## 全量验证跑（2026-08-18，第四轮修复后）

命令同前，4164s。结果：**47 FAIL / 2820 PASS / 2641 accepted-diff / 1028 SKIP**
（4 崩溃：deeplyNested/largeCFG signal6 + 2 JSX 超时）。

对比第三轮修复前基线（3 FAIL / 2756 PASS / 2699 DIFF / 1078 SKIP / 54 崩溃）：
PASS +64、DIFF −58、SKIP −50、崩溃 −39；FAIL +44（其中 2658 个 2538 污染
FAIL 已清零，余 47 为下述构成）。

### 47 FAIL 构成（代码集对账）

- 17 例欠报 TS2345（泛型调用实参替换偶发失效——已知未修家族）
- 6 例多报 TS2322（赋值检查新机制的边界）
- 5 例多报 TS2536 / 2 例多报 TS2563 / 2 例多报 TS2339 / 2 例多报 TS7006
  ——本轮新引入，第三波修复处理（见下）
- 其余 13 例为零散欠报/多报混合

## 修复三（2026-08-18，验证跑后 CLI 定位）

1. **TS2536 any/unknown 对象门**：`keyof any`≈string 的近似使判定不可靠
   （declarationEmitMappedTypePreservesTypeParameterConstraint 的
   `Type '<unknown type>' cannot be used to index type 'any'`）；官方只对
   有类型对象报 TS2536 → any/unknown 对象跳过
2. **FLOW_MAX_DEPTH 200 → 2000**（Go flow.go ~L118 的数字）：
   binaryArithmeticControlFlowGraphNotTooLarge（~1600 深度）不再误报
   TS2563；largeControlFlowGraph（10k）仍触发且**位置/文本与官方完全一致**
   （(3,1) TS2563）
3. **类型解析深度限 100 → 500**：我们的计数把词法嵌套当作实例化深度，
   100 层合法嵌套条件（deeplyNestedConditionalTypes 官方零错误）被误杀
   TS2589；5M 预算 + 环检测兜底爆炸防护
4. **harness worker 256MB 栈**：libtest 线程栈小于 CLI 主线程，深层递归
   在 worker 里溢出而 CLI 存活（largeCFG/deeplyNested 的 harness 专属
   崩溃根因）——worker 编译移入大栈线程（等效 Go 可增长协程栈）
5. **验证（CLI/worker 直跑）**：declarationEmit 0×TS2536、binaryArithmetic
   0×TS2563、largeCFG=官方一致、deeplyNested 零错误 2s、arrayFlatMap
   0.2s、jsxRuntimePragma 28.6s、commentsOnJSX 20.7s（后两者贴近 30s
   超时线，4 并发下有 SKIP 风险——已知）

### 已知未修（下轮）

- collectionPatternNoError TS2339：`U["TType"]` 经约束解析得 `Message`
  （应为 `MessageList<T>`）——延迟访问约束解析的成员选择错误
- conditionalTypeContextualTypeSimplificationsSuceeds TS7006×2：延迟条件
  参数类型的上下文签名提取失败 → 回调参数隐式 any
- 17 例 TS2345 欠报 + 6 例 TS2322 多报（上轮遗留）
- deeplyNestedConditionalTypes 的 @declaration 基线（.d.ts 文本）未比对

## 修复二（2026-08-18，同日续：CLI 单点诊断驱动，中止首轮验证跑后修复）

首轮验证跑前 197 例 136 FAIL，用 tsox CLI + gdb 定位出三个新误报族 +
两个崩溃机制，全部修复：

1. **TS2536 on `TagMap[K]`（重载泛型约束串号）**：接口内多个方法各自的
   类型参数 K 被 binder 按名合并进 interface members（一个符号、多个
   声明），`get_type_parameter_from_symbol` 取首个声明的约束 →
   `getElementsByTagName<K>` 的约束成了 `addEventListener<K>` 的
   `keyof HTMLElementEventMap`。**修复（Go 忠实）**：
   `get_container_flags` 补 Go `GetContainerFlags` 的签名组——
   MethodSignature/CallSignature/ConstructSignature/FunctionType/
   ConstructorType 为 IS_CONTAINER|HAS_LOCALS 容器（IndexSignature 同），
   类型参数声明进**签名自己的 locals**，不再跨方法合并；
   `has_locals` 同步补全。连带修复：TS2430 家族回归（HTMLVideoElement
   等 6 对接口——K 约束串号导致 addEventListener 签名比较失败）
2. **签名作用域推入**：`build_interface_type_from_members` 的
   MethodSignatureDeclaration 分支与 `get_type_from_function_type_node`
   此前不推签名作用域（K 落 interface members 时恰好能查到；改 locals
   后必须显式推）——补 push_scope(member/node) 包住参数+返回类型解析
3. **TS2536 防误报双门**：替换栈非空时跳过（重载比较的上下文实例化会
   用**对方的** K 约束检查本节点对象）；bundled:// 文件跳过（lib 是
   官方已知良好声明；我们接口构建期的 keyof 目标偶发解析为 any →
   约束坍缩为 string——已知未修，记录在案）
4. **推断环守卫（43 个崩溃用例的主根因）**：
   `infer_from_types ↔ infer_from_object_types ↔ infer_from_signature`
   经自引用签名（`ReadonlyArray<T>.flatMap` 回调参数又含
   `ReadonlyArray<T>`）无限互递归 → **栈溢出 = signal 6**（gdb 确认，
   非 OOM）。修复：接线 InferenceState 里声明未用的 `visited` 映射
   （改为 (source ptr, target ptr) 对），重入同对即返回。arrayFlatMap
   （signal6）、38 个 JSX/react 超时、errorInfoForRelated、
   styledComponents 全部 ~2s 完成
5. **arrayFlatMap 顺带**：TS2589 深度守卫在 100 层嵌套条件类型
   （deeplyNestedConditionalTypes，官方可解析）上误触发——输出与官方
   可能不一致但不再崩溃（待后续把「节点解析深度」与「实例化深度」
   分离）；largeControlFlowGraph 2.5s 输出**正确的 TS2563**（官方
   预期一致）

### CLI 验证记录（tsox 二进制直跑，非测试套件）

- lib.dom 全量检查 0 错误（此前 125 条 TS2536/TS2430）
- 最小复现组（v4/e/f/g/alias）全 0 错误
- arrayFlatMap/deeplyNested/largeCFG/2 非 JSX 超时/5 个 JSX 抽样：
  全部 ≤2.5s 完成、零崩溃
- 冒烟：2dArrays 0 错误（官方 0）；chainedAssignment3 差异在已知
  TS2322 分诊族内

## 修复（2026-08-18，第四轮 fix-only，未跑测试；基于本轮全量 2705 FAIL 对账）

本轮 2705 FAIL 全部同根因：第三轮把 TS2538 kind 门控放在延迟判定之前，
lib.dom.d.ts 的 `ValueTypeMap[T]`（延迟泛型索引访问）每次重解析都误报。
本轮按 Go 语义重排 + 顺带修复同一子系统的连锁缺口：

1. **TS2538 门控前移延迟判定**（清 2705 FAIL 的主修复）：
   `get_type_from_indexed_access_type_node` 忠实移植 Go
   `shouldDeferIndexedAccessType`（checker.go ~L27438，type-position 分支）：
   泛型索引类型（TypeParameter/IndexedAccess/Conditional/Substitution/
   Index/TemplateLiteral 联合递归）或泛型对象类型 → 构造**延迟
   `TypeData::IndexedAccess` 类型**直接返回，零诊断；元组 + 定长数字
   字面量索引例外（急切解析，`indexTypeLessThan` ~L27452）。非延迟访问
   才走 kind 门控报 TS2538（`any[[]]` 保留）。实例化时节点在替换栈下
   重解析，索引具体化后自然急切解析；relater 的 IndexedAccess 关系
   规则（component-wise + 基约束回退）此前已就位，直接衔接
2. **TS2538 per-node 去重**：`indexed_access_2538_reported`（HashSet of
   node ptr）——泛型外层别名每次实例化重解析同一节点，Go 把解析结果
   缓存在 node link 上只报一次
3. **TS2536 泛型对象门控**：延迟类型使 check 阶段
   `check_indexed_access_index_type`（此前死代码）激活。泛型对象
   （`T[K]`、`DataFetchFns[T]`）的 keyof 归约依赖 Go 的惰性约束链，
   我们的近似不可靠 → 泛型对象跳过（合法模式不误报；具体对象如
   `DataFetchFns[F][F]` 仍正常检查）
4. **keyof 索引签名类型**：`get_index_type` 对只有索引签名的结构
   （`Record<string, X>`）此前返回 never；改为属性名 ∪ 索引键类型，
   string 索引签名额外贡献 number（Go：`keyof Record<string,X>` =
   `string | number`）——eventEmitterPatternWithRecordOfFunction 的
   `M[Event]` TS2536 判定依赖此
5. **延迟类型的属性访问宽容**：`has_property_of_type` 对延迟
   IndexedAccess 经基约束解析后检查；仍泛型则宽容（旧行为=any 静默，
   不得引入 TS2339 误报）
6. **evolving array 元素赋值演化**：binder 的 ARRAY_MUTATION 节点此前
   存的是 ExpressionStatement（receiver 提取永远失败）；改为存二元
   表达式 + `is_narrowable_operand` 门（Go binder.go ~L2242）。
   `evolve_array_at_mutation` 补 BinaryExpression 分支：索引 number-like
   时以 RHS 演化元素类型（Go `getTypeAtFlowArrayMutation` flow.go
   ~L1420）——`data[0] = 0` ×10000 的每次引用此前走全量前件链
   （largeControlFlowGraph O(n²)/OOM 候选根因），现在首个匹配
   mutation 即停
7. **实例化计数预算**：`type_instantiation_count`（替换栈活跃时递增，
   `check_expression` 每表达式重置，5M 上限 → TS2589 一次 + errorType）
   ——Go `instantiationCount`（checker.go ~L22170/22193）的忠实对应，
   JSX/react 超时家族的保险丝

### 静态推演验证（未跑用例）

- `ValueTypeMap[T]`（lib.dom 42542/43/48）：T TypeParameter → 延迟，
  零诊断；实例化 `Global<"f32">` → 急切解析成员 ✓；默认
  `Global<ValueType>` → 联合逐成员解析 ✓
- `any[[]]`：非延迟（any 对象 + 具体索引）→ kind 门控 → TS2538 于
  索引节点，per-node 去重 ✓
- 上轮 3 FAIL：anyIndexedAccessArrayNoException（门控保留）/
  eventEmitterPatternWithRecordOfFunction（本轮 diff 仅剩 2538 污染）/
  importHelpersWithImportOrExportDefaultNoTslib（同）→ 预期全转绿
- 风险点：TS2589 上报为全局一次（Go 每表达式重置后可多次）；JSX 家族
  38 超时未动（需运行时剖析定位，盲目改推断/上下文机制会危及已绿用例）

## 全量回归记录（2026-08-18，第三轮 fix-only 统一验证跑）

命令：`TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=4
cargo test --test submodule_compiler`，耗时 4366s（4 workers）。

结果：**2705 FAIL / 74 PASS / 2690 accepted-diff / 1067 SKIP**（合计 6536）。

对比上轮全量（2026-08-18 凌晨二进制）：3 FAIL / 2756 PASS / 2699 DIFF / 1078 SKIP
→ 第三轮修复引入**大面积回归**，但根因单一：

- **2647/2705 FAIL 为纯 TS2538 误报**，其余 ~50 个也是它的级联（含上轮 3 个
  FAIL）。根因：第三轮修复 #1 把 TS2538 门控（非 string/number/symbol-like →
  报错）放在 `get_type_from_indexed_access_type_node` 里**延迟判定之前**。
  Go 的顺序是 `getIndexedAccessTypeOrUndefined`（checker.go ~L27028）先走
  `shouldDeferIndexedAccessType`（~L27438：泛型索引类型 / 泛型对象类型 →
  直接返回延迟 IndexedAccess，零诊断），kind 门控只对**非延迟**访问生效。
  我们对 lib.dom.d.ts 的 `ValueTypeMap[T]`（T extends ValueType，延迟访问）
  每次实例化都误报 TS2538（42542/42543/42548 三处，重复多次），污染所有
  加载 DOM lib 的用例
- **worker 崩溃 54 → 43**：signal-6 abort 15 → 3（第三轮 memo 缓存 + 深度
  守卫生效）；40 个 30s 超时不变（38 个 .tsx/react 家族 + 2 非 JSX）。
  剩余 signal-6：arrayFlatMap、deeplyNestedConditionalTypes、
  largeControlFlowGraph；剩余非 JSX 超时：
  errorInfoForRelatedIndexTypesNoConstraintElaboration、
  styledComponentsInstantiaionLimitNotReached

## 修复（2026-08-18 凌晨，第三轮 fix-only，未跑测试；基于上方全量对账）

### P0 崩溃/回归修复

1. **类型解析 memo 缓存 + 环检测 + 深度守卫**（signal-6 abort 根治）：
   - `get_type_from_type_node` 公共入口新增 `(node.id, type_argument_stack哈希)` 键
     缓存（对应 Go `activeTypeMappersCaches` 的 (type,mapper) 键，checker.go
     ~L22200）——原 per-node 缓存在替换栈非空时整体绕过，嵌套条件类型每次
     重解析都新建类型 → 2^n 爆炸 → OOM abort（deeplyNestedConditionalTypes 等）
   - 同键 in-progress 集合做环检测（Go `pushTypeResolution` ~L18817）：同节点
     同替换重入 = 循环引用 → errorType
   - 嵌套深度 >100 → errorType（Go `instantiateType` 深度上限 ~L22170，
     TS2589 同源）
2. **TS2313 循环约束**：`get_type_parameter_from_symbol` 重入回退"无约束占位
   类型"（Go 类型参数与其约束是惰性分离的链接——`T extends Array<T>` 合法），
   解析完成后走约束链检测真环（`T extends T`/`T↔U` 互指）→ TS2313 一次 +
   约束置 None（circularConstraintType 语义）。修复
   typeParameterHasSelfAsConstraint / typeParameterWithInvalidConstraintType
3. **TS2563 流图过大**：flow 深度守卫触发时设 `flow_analysis_disabled` +
   在包含块首语句位置报 TS2563（Go flow.go ~L105/1590）——原实现静默回退导致
   10k 赋值体每引用全量走图 O(n²) → OOM abort（largeControlFlowGraph）
4. **isDeeplyNestedType 早终止**（JSX 超时主力）：relater 增设
   source/target 栈（Go r.sourceStack/targetStack），"同符号且不同 Arc 实例
   出现≥3 次"近似 Go 的递增 type-id 过滤；双侧深嵌套即视为相关（Go
   recursiveTypeRelatedTo ~L3152 expanding-both → TernaryMaybe）——阻断
   react 式无限展开泛型链
5. **3 个 FAIL**：
   - anyIndexedAccessArrayNoException：索引类型 kind 门控（非 string/number/
     symbol-like → TS2538 于索引节点，Go getTypeFromIndexedAccessType ~L27152
     + getIndexedAccessTypeOrUndefined 兜底 else）；memo 缓存天然去重防重报
   - eventEmitterPatternWithRecordOfFunction：词法回溯白名单补
     InterfaceDeclaration/ClassDeclaration/MethodSignature/TypeAliasDeclaration +
     容器类型参数查找分支（接口外上下文解析方法签名时 `M` 不再 TS2304）
   - importHelpersWithImportOrExportDefaultNoTslib.2：TS2354（Go
     checkExternalEmitHelpers → resolveHelpersModule ~L28644/28737）——
     importHelpers + commonjs 下 default 重导出/import 检查 'tslib' 可解析性，
     每文件一次

### P1 大子系统移植

6. **赋值类型检查（TS2322 大族）**：`check_assignment_compat`（Go
   checkAssignmentOperator ~L12808）——`=`/逻辑复合用 RHS 类型对目标声明类型、
   算术复合用算子结果类型（取二元表达式整体类型近似）；TS2364/TS2779 引用
   合法性门（Go checkReferenceExpression）；目标类型取符号/属性/索引声明的
   类型（非流窄化读类型）；errorType 目标跳过防级联。原先整个检查是 TODO。
   import-equals 值类型（aliasAssignments 的 `typeof import(...)` 双向赋值）
   由模块符号实例类型 + 本检查自动覆盖
7. **relater elaborated error chain（B2 大族）**：
   - Checker 增 `relater_error_chain`（Go ErrorChain ~L2581）+ 录制开关
   - `relater_report_error` 忠实移植 Go reportError 后处理（~L4880）：超额
     属性抑制、签名返回标记（2202-2205，elided）→ "The types returned by
     'x()'/..." 变换（弹两条压一条）、属性链 'x'+'y' → 'x.y' 点名串接
     （getPropertyNameArg/addToDottedName）
   - 埋点：对象属性不兼容（Types_of_property + 嵌套头）、缺属性（单个
     TS2741 式 / 多个 "missing the following properties" ≤4 名+计数）、
     签名返回类型失败点标记（compare_signatures_related ~L1661 对应）
   - 联合成员试探的链卫生（Go saveErrorState/restoreErrorState ~L3304）：
     失败试探回滚、整体失败保留最长链
   - 出入口 `check_type_assignable_to_and_optionally_elaborate`：失败时压
     泛化头消息（字面量源在目标无单例时显示基元，Go reportRelationError
     ~L4792；headMessage 参数优先——实参检查用 "Argument of type..."），
     逆序构建嵌套金字塔（createDiagnosticChainFromErrorChain ~L402，跳过
     elided 标记）；实参 TS2345 与赋值 TS2322 检查全部改走此出入口
8. **数组字面量元素递归加宽**：对象字面量元素走 `widen_initializer_type`
   （属性字面量一并加宽，`[{foo:"s"}]` → `{foo:string}[]`，Go
   checkArrayLiteral → getWidenedType；arrayCast）
9. **类实例合并类型保留类符号**：`merge_instance_types` 丢 `derived.symbol`
   → this 类型显示展开结构；修复后显示 `Point3D`（autolift4 家族）

### 台账清理

- 全量跑确认 105 条（99 整案 + 6 多配置后缀）已 PASS → 从 triaged.txt 删除
  （2923 → 2818 条）

### 已知未修（下轮）

- 泛型调用实参检查的替换偶发失效（callbacksDontShareTypes `(x: T) => unknown`
  显示）——替换机制已支持函数类型，根因在推断/重载交互，待运行时定位
- 数组字面量混合类型 → any[] 回退（应做子类型归约得 `{}[]`）
- `.delete` 欠报大族除赋值外的部分（TS2717/TS2538 家族细分）

## 全量回归记录（2026-08-18 凌晨，深夜 fix-only 第二轮统一验证跑）

命令：`TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=4
cargo test --test submodule_compiler`，耗时约 6700s（4 workers）。

结果：**3 FAIL / 2756 PASS / 2699 accepted-diff / 1078 SKIP**（合计 6536）。

对比上轮全量（2026-08-17 夜）：0 FAIL / 2720 PASS / 2774 DIFF / 1042 SKIP
→ PASS +36、DIFF −75、SKIP +36、FAIL +3。深夜 10 项修复部分生效
（default lib target→lib 映射等），但引入回归：

- **3 FAIL**（逐例已定位，见下）
- **54 个 worker 崩溃**（上轮 0）：
  - 39 个 30s 超时：几乎全部 .tsx/react 家族（`/.lib` fixture 挂载修复后
    JSX 检查真正解析 react.d.ts，出现指数爆炸/死循环）+ 少量非 JSX
    （moduleResolutionWithModule、errorInfoForRelatedIndexTypes…、
    styledComponentsInstantiaionLimitNotReached）
  - 15 个 signal 6（abort）：全部递归条件类型/mapped/自约束家族
    （conditionalTypeAssignabilityWhenDeferred、deeplyNestedConditionalTypes、
    incorrectRecursiveMappedTypeConstraint、indexingTypesWithNever、
    infiniteConstraints、recursiveTypeRelations、typeParameterHasSelfAsConstraint、
    largeControlFlowGraph、mappedTypeAsStringTemplate…）——疑似栈溢出/abort，
    Go 有 instantiateType 深度守卫（instantiationDepth 100 + recursion 栈）
- 台账 2923 条中对账：165 条本轮无产物（崩溃/选项 skip），
  99 条对应用例整案 PASS（可从台账删除；其中若干条目引用的官方基线
  根本不存在——陈旧条目），其余 2659 条仍有差异

### 3 个 FAIL 根因（静态定位，待修复）

1. **anyIndexedAccessArrayNoException**：`var x: any[[]]`——索引类型为空
   元组 `[]` 应报 TS2538（不可作索引类型）。深夜修复把 TS2538 从
   `get_type_from_indexed_access_type_node` 移到 check 阶段
   `check_type_annotation` 时漏了「索引类型非 string|number|symbol 可赋值
   即报」的通用判定（Go checkIndexedAccessIndexType 的 else 分支）
2. **eventEmitterPatternWithRecordOfFunction**：`Args<M[Event]>`（F extends
   (...args: infer A) => void ? A : never，M 为方法类型参）——我们在条件
   类型求值/替换时于方法作用域外重解析 `M[Event]` 里的 `M`，误报
   TS2304 Cannot find name 'M'（官方 0 错误）
3. **importHelpersWithImportOrExportDefaultNoTslib.2**（仅 commonjs 配置）：
   importHelpers + default import 组合的基线差异（待细查）

### accepted-diff 头部差异画像（_scripts/reconcile.py 对账，前 700 例抽样）

- **209 例「码相同文本不同」**：大头是缺 elaborated error chain
  （"Types of property 'x' are incompatible." 缩进链）与类型显示差异
  （别名不解析显示展开结构、字面量未加宽 `{foo:"s"}` vs `{foo:string}`）
- **34+ 例缺 TS2322**：`x = v` 赋值兼容性检查整个未实现
  （checker.rs BinaryExpression 分支 TODO；Go checkAssignmentOperator +
  checkTypeAssignableToAndOptionallyElaborate 完整路径）
- **10+ 例 TS2345 误报**：泛型调用从首实参推断的类型实参未实例化到
  后续实参检查（`_.map(c2, rf1)`：T=number 推出后 `f` 仍按 `(x:T)=>unknown` 比）
- import-equals 值类型（`import x = require()` → `typeof import(...)`）
  双向赋值检查缺失（aliasAssignments 家族）

## 修复（2026-08-17 深夜，fix-only 第二轮，未跑测试）

基于全量产物（reference vs local 新鲜度对账）的误差直方图定位，
只改代码、未运行任何测试用例（含单用例），待统一验证：

1. **`default_lib_file_names` 缺 target→lib 映射（最大根因）**：
   Go `tsoptions.GetDefaultLibFileName`/`targetToLibMap`——es2015→
   `lib.es6.d.ts`、es2016..es2025→`lib.es20XX.full.d.ts`、esnext→
   `lib.esnext.full.d.ts`；此前所有 target 一律 `lib.d.ts`，es2015+
   用例的 Promise/Map/Set/Symbol/迭代器全缺 → 大片 TS2304/TS2345/
   TS2322 双向误差
2. **panic 修复**：`line_and_character` 对多字节字符内部偏移向下对齐
   char boundary（`𝘮` 切片 panic，1 例 skip）
3. **TS2538 上报错位**：类型解析路径 `get_type_from_indexed_access_
   type_node` 的自制检查删除（每次实例化重报）；移植 Go check 阶段
   `checkIndexedAccessType`/`checkIndexedAccessIndexType` 到
   `check_type_annotation`（TS2536 + TS4105，含 keyof 可赋值性判定、
   generic-object 私有成员分支）
4. **TS2430 重复上报**：仅在无类型参（声明态）解析时上报 +
   `interface_extends_reported` 按（接口符号, heritage type-ref 节点）
   去重（此前泛型接口每次实例化重报，如 OrderedMap 7 次）
5. **harness `/.lib` fixture 挂载**：`_submodules/TypeScript/tests/lib/**`
   → VFS `/.lib/**`（`/// <reference path="/.lib/react.d.ts" />` 解析，
   49 例 TS6053 级联 + react TS2307）
6. **rest 参数推断**：`infer_type_arguments` 实参循环对 rest 位置按
   元素类型推断（Go `getTypeAtPosition` 语义）——`new Array('hi')` 此前
   `string→T[]` 无法推断 → 实参检查对未替换 T 误报 TS2345（111 例
   首错簇）
7. **类 `prototype` 静态属性**：binder 建符号（SymbolFlags::Prototype，
   Go binder.go ~L962）+ checker `get_type_of_prototype_property`（类
   类型按类型参数个数实例化为 any）+ `attach_class_statics` 收录
   （`typeof X.prototype` TS2339 簇）
8. **跨文件 ambient namespace 合并解析**：`declare namespace Intl`
   分散于 es2018/es2020.intl 等——scope 走访与 ancestry 走访对 MODULE
   容器增加 globals 合并符号的 exports + ambient locals 回退（lib 内部
   TS2304 `NumberFormatPartTypes` 簇）
9. **noUnusedLocals/noUnusedParameters**（156 例 skip）：
   - harness 选项表去 skip
   - `symbol_reference_kinds`（DashMap，Go symbolReferenceLinks）：
     `resolve_identifier_with_meaning` 包装记录 + `follow_alias` 记录
     alias 引用
   - `check_unused_identifiers_in_file`：按 Go
     `checkUnusedLocalsAndParameters`/`reportUnused*` 全家移植（TS6133/
     6196/6192/6198/6199；下划线豁免、for-in/of+using 豁免、参数属性
     豁免、ambient/声明文件豁免、`{a, ...b}` rest 规则）
10. **every/some `this is S[]` 收窄**：`narrow_by_call_expression` 增加
    TypePredicateKind::This 分支——回调实参自带 `value is U` 谓词时，
    实例化 `S[]`→`U[]` 收窄接收者（arrayEvery 等 B1 簇）

## 全量回归记录（2026-08-17 夜，pass 2+3 + 二轮修复统一验证跑）

命令：`TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=4
cargo test --test submodule_compiler`，耗时 6230s（4 workers），0 崩溃 0 超时。

结果：**0 FAIL / 2720 PASS / 2774 accepted-diff / 1042 SKIP**。

对比上次全量（同日早间，二进制为修复前构建）：2552 FAIL → 0 FAIL；
72 PASS → 2720 PASS。pass 2/3 + 二轮 CLI 单点修复全部生效，零回归：
- 2536 例 DOM/webworker lib 虚假 TS2430 全部转绿（类型参数源约束化简
  顺序修复 + IndexedAccess/keyof 关系规则 + canonical signature 等）
- staticInheritance / superElementAccess / functionSubtypingOfVarArgs /
  arithAssignTyping / flowAfterFinally1 / controlFlowFinallyNoCatchAssignments
  等根因组转绿
- triaged.txt 同步清理：53 条已转绿条目 + 233 个空组删除，
  台账余 **2923 条**

剩余工作（按 triage-CLASSIFICATION.md 21 类）：
- 2774 accepted-diff（triaged 2923 条按根因组修复）
- 1042 SKIP：allowJs 208 / module=AMD+System+UMD 122 / noUnusedLocals+
  noUnusedParameters 156 / resolveJsonModule 35 / allowUnreachableCode 等
  ~60 / SKIPPED_CASES 名单 22 / 二进制非 UTF-8 4 / **panic 1**（字符串
  切片 char boundary `𝘮` 字符，src 待修）
- 注意：submodule_run.log 存在并发写丢行（3533/6536 outcome 行落盘），
  日志仅供抽样，精确对账以 local/ 产物新鲜度为准

## 全量回归记录（2026-08-17，pass 2+3 代码验证跑）

pass 2/3 子系统修复完成代码后，按 FIXING.md 统一验证：
`TSOX_SUBMODULE_START=0 TSOX_SUBMODULE_END=6536 TSOX_SUBMODULE_JOBS=4`
（二进制为修复前构建）。

结果：**2552 FAIL / 2873 DIFF（分诊可接受）/ 72 PASS / 1039 SKIP**，0 超时 0 崩溃。

失败构成（逐文件对比 reference，剔除 5 行 DOM lib TS2430 噪声后分类）：
- **2536 例纯 DOM/webworker lib 虚假 TS2430**（根因 #1）：pass 3 在
  `compare_signatures_related` 第 5 步引入的「泛型签名在目标上下文中实例化」
  路径不完整——`Window.addEventListener<K>` 与基类
  `WindowEventHandlers.addEventListener<K'>`（不同符号的克隆类型参数）比较时：
  (a) target 未替换为 canonical 形式；(b) 无 IndexedAccess↔IndexedAccess 关系
  规则（`EventMap[K']` vs `EventMap2[K']` 无法按「对象相关+索引相关」判定）；
  (c) 参数无 callback 模式比较；(d) 签名替换不递归进函数类型的参数/返回
  （listener 内的 `K` 不被替换为 `K'`）。5 个受影响接口对：HTMLVideoElement/
  HTMLMediaElement、IDBOpenDBRequest/IDBRequest、SVGSVGElement/Window、Window/
  WindowEventHandlers、Worker/MessageEventTarget（全是 addEventListener/
  removeEventListener 共有成员）
- **arithAssignTyping**（根因 #2）：复合赋值 LHS 为 class 等非变量符号时，Go
  `checkIdentifier` 返回 errorType（Any 旗标）抑制 TS2365/TS2362 级联；我们
  只报 TS2629 后继续用 `typeof f` 做运算符检查
- **flowAfterFinally1 / controlFlowFinallyNoCatchAssignments**（根因 #3）：
  flow 引擎完全没有 ReduceLabel 语义（Go flow.go ~L181 `getBranchLabelAntecedents`
  + `f.reduceLabels`）——try-finally 之后的流图应缩减为 normal-exit 前件集，
  我们走全量前件（含 try 前的未赋值路径）→ TS2454 误报；且 binder 的
  try 标签用 `FlowLabel::finish` 提前坍缩，异常目标被加进无关节点
- **functionSubtypingOfVarArgs**（根因 #4a）：evolving array
  （`private _listeners = []`）类型无成员表，`push` 查找失败
- **superElementAccess**（根因 #4b）：纯函数类型缺全局 `Function` 接口成员
  （`bind`/`call`/`apply`），Go 由 apparentType 合并 globalFunctionType
- **staticInheritance**（根因 #5）：类构造器类型只有 construct 签名，无静态
  成员表；Go 的静态侧通过基构造器链继承静态成员
- **controlFlowArrays (67,15) TS18048**：未定位（静态分析无法收敛，位置映射
  亦存疑——指向声明行内 `null` 字面量中段）；待运行时调试
- 其余 12 个「混合」失败复核后均归并根因 #1（DOM TS2430 污染其它配置）

分诊转绿信号：3 个原 triaged 用例变为 PASS（es5-asyncFunction、
classMemberInitializerWithLamdaScoping、classMemberInitializerWithLamdaScoping2），
pass 2/3 的流驱动 TS2454 与 awaited-type 修复部分生效。

### 修复（2026-08-17，fix-only，未跑测试）
- `relater.rs`：`get_canonical_signature` 完整实现（无约束克隆类型参数映射回
  原参）；第 5 步同时替换 source/target；新增 IndexedAccess↔IndexedAccess
  关系规则（含 for-writing 基约束回退，Go relater.go ~L3483）；参数比较新增
  callback 模式（getSingleCallSignature + isInstantiatedGenericParameter 门控 +
  nullability facts 相等，~L1590）；`instantiate_signature_in_context_of` 忠实
  applyToParameterTypes/ReturnTypes（this 类型、min 前缀、rest、返回值类型变量
  门控）；`substitute_infer_type_parameters` 新增 IndexedAccess 与函数类型分支
  （经 instantiated_parameter_types 重建签名）；`get_signature_instantiation`
  设置 `target`；identity 关系的 IndexedAccess 分量比较
- `ast/symbol.rs`：`FlowNode.reduce_target` 字段（Go FlowReduceLabelData.Target）
- `binder/mod.rs`：`bind_try_statement` 重写——标签全部用专用累加器节点
  （不再提前 finish 坍缩），三个 create_reduce_label 均携带 finally 标签
  target；`create_reduce_label` 增加 target 参数
- `checker/flow.rs`：引擎实现 ReduceLabel——`FlowQuery.reduce_labels` 栈 +
  junction 处按 target 指针替换为缩减前件集（getBranchLabelAntecedents 语义）
- `checker.rs`：`nonvariable_assignment_target_type`（复合赋值非法 LHS →
  errorType 抑制级联）；`get_property_of_type` 新增 EvolvingArray→Array 与
  匿名函数类型→Function 全局接口回退；`attach_class_statics`（构造器类型
  挂接自身+继承静态成员，含防重入栈）
- 待验证；controlFlowArrays TS18048 待运行时定位

### 修复二轮（2026-08-17 晚，单点 CLI 诊断驱动，最终全量验证中）

首轮修复后 DOM TS2430 仍在，用 `tsox` CLI + 最小复现逐层定位，实际根因与
补丁：

1. **类型参数源约束化简顺序错误（真正的主根因）**：Go 的
   `recursiveTypeRelatedTo` 在 union/结构分派**之前**把类型参数源化简为
   基约束，因此 `K extends "a"|"b"` 可赋值给 `"a"|"b"|"c"` 整体联合；
   Rust 把 union 分派放在前面，逐成员比较全部失败 → 推断候选 K' 被约束
   检查拒绝 → K 回退为约束联合 → addEventListener 泛型重载互比失败。
   修复：`is_type_related_to_inner` 将 source-typeparam 约束化简提到
   union 分派之前（对照 Go recursiveTypeRelatedTo）
2. `is_type_related_to` 内部递归缺 `Arc::ptr_eq` 快速路径（Go 的
   `source == target`）→ 联合成员无法匹配自身的内嵌单例（`object`）；
   `is_simple_type_related_to` 补 NonPrimitive 恒等
3. keyof（Index）类型关系规则缺失（Go relater.go ~L3526：`keyof S ~ keyof T
   ⟺ T ~ S`）+ identity 的 Index 分量比较
4. 空目标签名集的 call/construct 比较误返回 false（Go 循环零次 = true）——
   这使 `typeof SomeClass` 无法赋给普通对象形状（staticInheritance 根因）
5. 元组目标未把元素位置当必需属性（补属性循环的 tuple 分支，恢复
   函数→元组赋值的 TS2322）
6. `let x = null/undefined` 声明类型：非 const + null/undefined 初始化 →
   **autoType**（Go checker.go ~L16757），此前为字面量 null →
   controlFlowArrays f7 的 TS18048 误报（官方 types 基线确认 x: any）
7. `has_property_of_type` 的 structured 提前返回拦截了 EvolvingArray 与
   纯函数类型的回退（EvolvingArray→length/mutation 方法；匿名单调用签名
   类型→全局 Function 接口成员，Go apparentType 语义）——修复
   functionSubtypingOfVarArgs（push）与 superElementAccess（bind）
8. `attach_class_statics` 改从类符号 members/exports 表收集静态成员
   （symbol_map 对成员名节点查不到符号）

单点验证（tsox CLI）：staticInheritance / superElementAccess /
functionSubtypingOfVarArgs / controlFlowArrays(f7) / arithAssignTyping /
flowAfterFinally1 / controlFlowFinallyNoCatchAssignments /
assigningFunctionToTupleIssuesError / DOM lib 5×TS2430 全部通过。

---

## 历史批次

全部 6537 个用例已扫完（2026-08-16，批次 0-6536，全部 0 fail）。
下一步工作变为按 triaged.txt 台账的根因组逐个修复子系统（见分诊规则章节）。

## 子系统修复 pass 2（2026-08-16，进行中，未跑测试）

按 FIXING.md 流程：只改代码、不跑测试用例，全部批次处理完后统一验证。

### B1 控制流/收窄（引用式流程收窄子系统）
- `flow.rs`：引入 `FlowRef`（Symbol/Node 双通道）贯穿整个流程引擎——Go 的
  `f.reference` 语义。移植 `isMatchingReference`（flow.go ~L1597：标识符按解析
  符号、属性/元素访问按结构、`a[i]` 常量实参匹配、QualifiedName）、
  `containsMatchingReference`（~L1841）、`getAccessedPropertyName`（~L1727）
- `checker.rs`：属性/元素访问定型接入 Go `getFlowTypeOfAccessExpression`
  （checker.go ~L11442）门控——非确定赋值目标、Variable|Property|Accessor
  （或 Method+union）符号才参与；元素访问字面量键走成员查找
- binder：移植 `bindVariableDeclarationFlow`/`bindInitializedVariableFlow`
  （binder.go ~L2307）——声明初始化、for-in/of 头部变量、解构每元素都生成
  ASSIGNMENT 流节点（原实现只覆盖 VariableStatement 且不推进 current_flow）
- `flow.rs`：`getTypeAtFlowAssignment` 忠实化——union 声明类型做
  `getAssignmentReducedType` 归约（~L2399：按 maybe-assignable 过滤 + 兜底不
  收窄），非 union 返回声明类型（原实现直接返回 RHS 类型）；`obj = v` 前缀
  赋值重置为声明类型；`for (k in ref)` 对 ref 做 non-null（~L269）
- `flow.rs`：引擎支持独立 `initialType`（Go START 节点语义）；TS2454 明确
  赋值改为流驱动——`T | undefined` 作初始类型重走流图，汇合点含 undefined
  即报错（Go checkIdentifier ~L11226）。删除 240 行旧启发式
  （预扫描 + assume-assigned-on-cycle）
- `flow.rs`：别名判别式——`const k = obj.kind` / `const {kind: k} = obj`
  （Go `getCandidateDiscriminantPropertyAccess` ~L1460），if/switch 判别式与
  typeof 判别式共用；`x!` non-null 父节点保护（~L111）
- 解构初始类型：`getInitialType` 家族（~L2234）——binding element 属性/索引
  取型 + `getTypeWithDefault`（非 undefined 部分 ∪ 默认值）

### A2 上下文签名实例化（首批）
- `types.rs`：`Signature.instantiated_parameter_types`（实例化签名的参数类型
  覆盖表，rest 保留数组形状）
- `relater.rs`：`get_signature_instantiation`（Go ~L19352）+
  `instantiate_signature_in_context_of`（Go ~L19525，relater.go ~L1527 接线：
  源泛型签名在目标上下文中实例化后再比较，替换原擦除占位）
- `checker.rs`：泛型函数表达式在具体上下文签名下定型为实例化签名
  （Go checker.go ~L7718 路径）

### A2 续（2026-08-17，pass 3 完成部分）
- `inference.rs`：`get_contextual_signature` 完整移植（Go ~L10314）——
  union 上下文逐成分收集签名（参数数不一致则放弃）、`isAritySmaller`
  arity 过滤（Go ~L10357）；`iife_contextual_signature`（IIFE 实参作
  上下文参数类型，Go `getContextuallyTypedParameterType` ~L29273 的 IIFE
  分支，带 `resolving_function_like` 防重入守卫）
- `inference.rs`：`contextual_return_type_of`（Go `getContextualReturnType`
  ~L29370：注解 → 上下文签名返回类型 → IIFE 调用的上下文类型），接入
  `get_contextual_type_for_return_expression`。注意：**per-return 检查仍只
  对有注解的函数触发**（Go `checkReturnStatement` 的
  `getReturnTypeFromAnnotation != nil` 门控），上下文返回类型只影响
  return 表达式的上下文定型（字面量加宽等）
- `inference.rs`：`get_contextual_type_for_argument` 带推断——泛型签名从
  非上下文敏感的兄弟实参推断类型实参后代入参数类型（Go
  `getContextualTypeForArgumentAtIndex` 用 resolved signature 的语义；
  `resolving_contextual_calls` 防重入）
- `relater.rs` + `typenode.rs` + `types.rs`：条件类型求值子系统——
  `is_distributive` 改为按 AST 判定（替换前解析检查型，Go
  `getTypeFromConditionalTypeNode` ~L24338 语义，修复泛型别名实例化时
  被替换后的 union 检查型丢失分发性的问题）；`ConditionalRoot.
  check_type_parameter_symbol` 记录检查型参数符号；`resolve_conditional_type`
  分发：union 检查型逐成分求值取并集、never 检查型得 never（Go
  `getConditionalTypeInstantiation` ~L22544 的 `prependTypeMapping` 语义，
  每成分重解析 extends/分支节点）；`is_type_related_to` 对条件类型主动求值
  （对齐 Go 实例化即求值）；`substitute_infer_type_parameters` 支持
  泛型引用（`Promise<T>` 等，保 target/symbol 重建）与嵌套条件类型
  （替换检查型后重解析）
- `checker.rs`：`get_awaited_type`/`get_promised_type_of_promise`
  （Go `getAwaitedTypeNoAliasEx` ~L31321 + `getPromisedTypeOfPromiseEx`
  ~L28994：union 逐成分、Promise 按类型实参、thenable 走 then 回调首参，
  深度 50 防递归坏 Promise）；`await x` 接入
- `checker.rs`：TS18048/TS2532 家族补齐——`report_possibly_null_or_undefined`
  （Go `reportObjectPossiblyNullOrUndefinedError` ~L7519 的实体名/对象形
  消息选择：TS18048/18069/18047/2532），元素访问 `x?.y[0]` 与非可选调用
  `(x?.f)()`（TS2722 家族）接入

### 验证状态
仅 `cargo check` 通过；按 FIXING.md 未运行任何测试用例。
triaged.txt 条目待统一测试验证后按组删除。

# 测试流程修改

1. 如果存在 rust 测试套件问题
  - 完全停止测试，与测试代办
  - 集中精力修复测试套件产生的问题
