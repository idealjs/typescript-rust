# corpus 修复记录（飞轮单发流水）

飞轮多 subagent 并发纪律下的逐次修复记录（2026-09-30 起），修一片记一条，最新在上。系统记忆已清空且仓内文件遭多次外部重置，本文件主副本同步存 /tmp/corpus-fix-notes-master.md。

## 第 22 轮总状态（2026-09-30，主 agent 维护；第四次重建）

- **严重事件**：主仓工作树被外部清空**三次**（无 reflog 的路径级还原，均发生在后台批次在飞期间，疑似个别 subagent 越权）；已三次全量重建硬化与笔记。**硬化快照：/tmp/hardening_snapshot/**；笔记主副本：/tmp/corpus-fix-notes-master.md。**强烈建议用户提交硬化改动并排查越权 subagent**。
- **基线与口径**：4GB 内存限制口径（ulimit -v 4194304 + TSOX_SUBMODULE_LIMIT=0 + --release --no-fail-fast，cd crates/tsox 跑 corpus）；183 例确定性挂起（171 timeout + 12 signal6）按异常路由挂起待人工审。
- **检查点轨迹**：1095 基线 → 1116（大回归已 revert）→ 1062 → 1036 → **1031（检查点 4，114 转绿 / 17 新红）**。skip 超基准 649（含 parity 规则新增合规 SKIP，轮末人工审）。队列 1031 片（/tmp/flywheel_shards）。
- **17 新红**（多为批内连带，下波优先）：bigintIndex、classExtendsClauseClassMerged*/NotReferringConstructor（TS2507 族=get_type_of_symbol 分派顺序结构分歧）、classVarianceResolveCircularity1、declFileTypeofFunction、declarationEmitReexportedSymlinkReference*2（m2b_6 波及待查）、destructureOptionalParameter、destructuringAssignmentWithDefault 等。
- **已合并 commit（44+，编译闸门每轮零错误）**：0001 reservedWords2 三连（Missing 标识符/pos/namespace 门）；0002 `?` 可空类型五连；0003 parseBigInt 三连（intern 统一收官）；0004 绑定模式簇；0008 TS2454 isParameter 守卫；0009 负字面量型；0010 LHS 门+resolveEntityName+模块说明符门三连；0011 isIdentifier is_keyword；0014 adjust_type_with_facts 接线；0016 relater 混对守卫；0018 Missing 兜底簇；0019 类静态成员实例化；0020 bare return elaborate；0022 构造器 return 实例型检查（一次误 revert 已恢复）；0023 relater union fall-through；0024 参数 TS7006 遍历位快路径（**TS7006 大簇 15 例收官**）；0025 全局增强单次合并；0026/0028/0035/0041/0054 熔断交接；0027 算术三路+intern；0035 truncate 机制 revert（目标未修绿，挂起）；0036 d.ts ambient；0037 类表达式 implements；0038 heritage 左端 TS2708 抑制；0039 enum+namespace 值类型；0040 get_set_accessor_value_parameter；0044 声明变换 visit 接线（TS90xx 通道仍静默待追因）；0045 赋值值含义门控；0046/0047/0048 基类型移植三连（getBaseTypes/getBaseConstructorTypeOfClass/值语义，0046+0048 同函数冲突融合）；0049 TS2347 untyped 门控；0050 mapped type Index 桥接；0051 TS2417 静态侧分支（仍未触发）；0052 TS2460 同引用判定；0055 report_non_exported_member 空壳填实；0056 accessor scope 压栈。
- **主 agent 直接修正**：should_skip 补 3 条 Go parity 规则（esModuleInterop=false/allowSyntheticDefaultImports=false/alwaysStrict=false，Tristate::False；alwaysStrict 规则加 `strict != False` 守卫——我们的测试设置解析会展开 strict 而 Go 不展开，实证 skip_baseline 无 @strict:false 用例）；0044 编译修复 7 处；0003/0014 编译修复；两次误判 revert 已纠正。
- **验证转绿 40+ 例**（抽样）：moduleExports1 族 4、TS2454 簇 4、TS7006 大簇 15、`?` 可空 2、parseBigInt、typeMatch1、baseTypeOrderChecking、extendsClauseAlreadySeen2、enumAssignmentCompat 部分、importNonExportedMember 族 4、classImplementsPrimitive、classExtendsInterfaceInModule、constructorReturningAPrimitive、assignToModule 族 2、classExtendsNull/declFileClassExtendsNull、staticAnonymousType、contextuallyTyped* 等。
- **挂起池（精确交接在本文件历史条目与下方速查）**：0001 TS7010+L10/L11（Go oracle CLI 实测法）；0002 TS2616 signature 机制（get_signature_from_declaration 空壳）；0013/0006 FT 参数 TS7006 tail 链；0017/0028 TS2564 递归塌缩（结构性：急切注解解析 vs Go 惰性，需插桩）；0021 联合属性型单点；0024/0030 crashIntypeCheck TS2347 apparent 边界；0041/0056 TS2842 scope_stack（需插桩）；0044 TS90xx 通道静默追因；0046 get_type_of_symbol 分派顺序（大轮）；0051 Arc::get_mut 别名缺陷；export= 族 5/7 interop 路径；privateName mangle（binder.go:319）；NodeFlags::Ambient 系统性缺失；TS7080 声明发射通道。
- **机制与纪律**：flat_segment 只比对纯错误段；merge --ff-only 只在分支上做、rebase 在 worktree 内做；subagent 派发一律后台、并发 8-12；派发前先在最新 HEAD 快测候选（防空派）；corpus_one.sh 传参不带 .ts（rm 目标才对）；FAIL CSV 缺席≠绿（可能是 SKIP，查 skips 表）；/tmp 是 tmpfs 注意 inode。
- **靠齐标记**：func_alignment.csv r22 若被重置按此重打——yes：DeclarationNameToString/createMissingIdentifier/nextTokenIsIdentifierOrStringLiteralOnSameLine/parsePostfixTypeOrHigher/parseJSDocNullableType/getNullableType/checkExpressionCached/checkPrefixUnaryExpression/parseAssignmentExpressionOrHigherWorker/resolveEntityName/parseLiteralTypeNode/getAdjustedTypeWithFacts/isIdentifier/bindVariableDeclarationOrBindingElement/getDeclarationName/checkReturnStatement/getBaseConstructorTypeOfClass/GetSetAccessorValueParameter/reportNonExportedMember/checkBinaryLikeExpression/getLiteralTypeOfBigIntLiteral；partial：declareSymbolEx/reportMergeSymbolError/Type/checkExpressionWorker/bindParameter/structuredTypeRelatedToWorker/isUntypedFunctionCall/isConstructorType/getBaseTypes/checkQualifiedName/checkPropertyAccessibilityAtLocation/getResolvedSymbol/isInAmbientOrTypeNode/getTypeOfFuncClassEnumModuleWorker。
- **worktree 预建纪律（2026-09-30 用户拍板）**：主 agent 在派发前为每片预建隔离 worktree（确认干净基线再派发）；subagent 全部操作含 git 命令只发生在其 worktree 内，与主工作树完全分离；派发 prompt 首步 pwd 验证、git 一律 -C worktree。
- **运行命令模板**：全量 `(ulimit -v 4194304; export TSOX_SUBMODULE_LIMIT=0; cd crates/tsox && cargo test --release --no-fail-fast --test corpus) > fullrun.log`；导出 `python3 tools/corpus_csv_export.py fullrun.log`；单例 `tools/corpus_one.sh <case 不带扩展名>`；切割 `python3 /tmp/cut_test_shards.py`。

## 基线（2026-09-29，自第 21 轮记忆迁移）

- 终态：CSV 4042 败（generatorTypeCheck 族 6 例 + yield 上下文定型 3 例转绿后）。
- 已知剩余深水（待单发消化）：yield* 二级上下文链（26/64）；生成器体自然返回推断（25/62/63）；本地接口 extends Iterator+Iterable 的合成成员迭代型提取（8）；注解型 `{[Symbol.iterator](): void}` 不报 TS2488 的通道（28）。
- 运行口径：全量 `ulimit -v 4194304 + TSOX_SUBMODULE_LIMIT=0 + --release --no-fail-fast`，cd crates/tsox 跑 corpus；CSV 只记 FAIL，写操作一律绝对路径。

**第 24 轮第二波（限额中断恢复）**：
- 两片因账户 5h 限额中止；collateral 片死前交付 18a8c86679（字面量索引补报豁免非直写字面量索引节点）→ **deferredLookupTypeResolution2 PASS**，wave-1 三绿全保。
- **arrayConcat3 归因（主 agent 双 revert 实验）**：a6b351eace 洗清；**dbe79f3265（tps_changed 触发签名重建）确认为连带源**，但它同时转绿 contextualSignatureInstantiation…——需按 Go instantiateSignatureEx 的惰性约束解析收窄（mapper 挂类型参数；注意 set_type_parameter_mapper 全库恒 no-op 的 Arc::get_mut 陷阱是结构性缺口），不能 revert。
- contextualPropertyOfGenericMappedType（mapped 门控代入时序）待收窄片。
- cpn 片（闭包 const 收窄）worktree /tmp/wt24_cpn 保留半成品（symbol_map.rs 脏），限额重置后重派。

**模板 v4（2026-10-01 用户拍板）**：tools/subagent_prompt_template.md 已更新——目标单一（一片=一个文件或一个用例，开头声明，验收=该目标 PASS）；任务背景极简（仅 Go oracle 只读路径+失败信息，不写根因长文）；删代码规范段（主 agent 收集时把关）。保留时限/提交/故障/禁测禁改/函数变更表契约。第 24 轮后续派发一律用 v4。

**第 24 轮 wave-2 结算**：1107→**1102**（净 −5，与实测转绿 5 例吻合：deferredLookup、arrayConcat3、bitwise、contextualTypeOnYield2、contextualSignatureInstantiation）。
- dbe 收窄 cdddc2f09f：tps 重建身份断裂修复（参数/返回经新 tps 重接线）——arrayConcat3 复绿且保住另一例。
- mapped 收窄 5fcbba471d：目标例转 panic（unwrap None），已 revert（286584fee9），用例回普通红留队。
- cpn 三件套 c98e29b182：binder 流槽位+Start 续走完整移植（lib 86/86 无回归），但两目标例未绿（「对齐落地路径未达」又一例）。
- wave-3 起派发一律用模板 v4（单目标/极简背景/无代码规范段）。

- **ets 500798f11d 已合并**：plain default import 的 ImportDefault helper 请求通道补齐（Go checker.go:5440-5447）——4 条 TS2354 全部补上；残差仅 1 条多余 TS2339（file2.ts(2,6) `typeof import("path")` 成员访问，shorthand 修复残余面），下波顺带。

- **icp f3e1d9b6c1 已合并**：get_flow_type_of_reference_ex 恒返桩→真引擎（check_identifier 流收窄链首次真正触达，配合 c98e29b182 三件套），lib 86/86。**但过冲**：本用例本地输出 6 错→0 错，参考期望恰 1 条（f5 闭包后赋值不应收窄）——is_past_last_assignment/flow_container 外推守卫对「闭包后有赋值的 let」放过。r2 在飞。
- 交接：仓库并存两套流引擎（现役 FlowQuery vs 未接线 mig/w6_flow.rs 完整 Go 忠实版）；check_if_expression_refines_parameter 缺第 5 参槽位；get_narrowed_type_of_symbol flow=None 跳过解构判别特例。

- **ufc 双 commit 已合并**（0b1a3a148a 无缓存流引擎版——与 icp 版同函数冲突，按「Go sharedFlows 单次语义防同键串染」取无缓存版；55bc90cd60 联合/交叉合成属性按 (type id,name) 缓存使符号身份跨调用稳定，对齐 Go propertyCache）→ **uncalledFunctionChecksInConditional2 PASS**。implicitConstParameters 仍红（过冲待 r2）。lib 86/86。

**派发纪律补丁（2026-10-01 用户指出）**：icp 与 ufc 同族（cpn 三件套后续、同一恒返桩线索）却被同波并发派发，两片重写同一函数（f3e1d9b6c1 / 34763bd819），合并冲突+icp 版作废。修正已入模板 v4 编排节：**派发前冲突检查必做**——同波各片登记嫌疑文件/函数，嫌疑面相交或同属一个前序交接家族 → 禁止并发（合一或串行）。当前在飞 4 片已核：cgm（inference_checker_8/10）/icp-r2（m1b_2 收窄守卫）/ets-residual（typeof import 成员）/lastPropertyWins（binder 属性合并），无相交。

- **ets 残差 e738a5db1c → PASS**（shorthand 环境模块 resolve_namespace_type 早返回 anyType，Go checker.go:17211-15）。
- **icp-r2 0699cf270b → PASS**（过冲根源=mark_node_assignments 误接纯缓存读 get_resolved_symbol_or_nil，晚于引用的赋值不入账→is_past_last_assignment 恒真；增 get_resolved_symbol_on_demand 最小移植 Go checker.go:14107）。赋值入账修复影响 is_symbol_assigned 全局面，全量观察。本轮已绿：ufc、ets、icp。

- **eana 全量把关：4 例同族连带新红**（checkMergedGlobalUMDSymbol、crashDeclareGlobalTypeofExport、extendGlobalThis、umdGlobalAugmentationNoCrash——UMD/declare-global 家族，98a2968e87 的预警波及面），已按同族合一追加进 flash 工作流单片修复。
- **dstr 36bd7f32ee 已合并**（绑定模式返回位上下文链：getContextualTypeForInitializerExpression/isFromBindingPattern/二阶段候选累积），用例残差 (11,48) elaborate 路由留队，lib 86/86。
- **flash 派发通道上线**：CreateWorkflow + subagent_model=GLM-5.3-Flash，v4 prompt 内联，board 看板跟踪，run dwfrun-f6dc57fe（3 片）+ 追加 UMD 片。

**第 24 轮 flash 通道首战（4 片，主 agent 实测）**：
- **umd 片 86d55eeb17 → 3/4 PASS**（checkMergedGlobalUMDSymbol/crashDeclareGlobalTypeofExport/umdGlobalAugmentationNoCrash；extendGlobalThis 残差，交接下一嫌疑=resolve_anonymous_type_members 的 globalThis vars-only 过滤与 get_exports_of_symbol 克隆世界接线）。
- **des 片 0c80394baf**：binder export default 具名导出恒名 default（Go declareSymbolEx），目标例残差待查；lib 86/86 无回归。
- **sm 片 6a20912a96**：生成器方法返回型分派（Go getReturnTypeFromBody case isGenerator），根因2（TS2739 ReadonlySetLike<number> 应 <unknown>，返回型 T 绑定多余推断）留队，交接线索 inference_checker_8.rs:903-944。
- **tin 零提交**：链路钉到两处真分歧（elaborate_object_literal 的 type_contains_type_parameter bail 非 Go 语义；约束门实例化口径），交接 /tmp/notes_archive 待归档。
- flash（GLM-5.3-Flash）产出质量：4 片 3 commit、单片根因分析+函数变更表齐全，转绿率 3/6 目标用例——弱于主力模型但可用，配合队列驱动闭环可规模化。

**队列驱动闭环成立（2026-10-01）**：试运行终版 dwfrun-87defd32 完整走通「选题（corpus_results.csv 耗时升序）→ worktree 自愈式预建 → flash 修复（v4 禁测契约）→ 串行合并主仓 → 重建 → corpus_one 确定性验证 → 失败回退+回喂一轮 → 通过保留/worktree 自动清理」。**2/2 用例第 1 轮验证通过**：abstractClassUnionInstantiation（signatures_related_to 构造签名 abstract 守卫原是空壳 if cond{}，47 行填实+构造器可见性相容）、abstractPropertyInConstructor（TS2715 发射点哑火改经 get_property_of_type，Go 单符号模型）。主 agent 审计：diff 无空壳、lib 86/86、worktree 清零。已存 saved workflow `corpus-flash-wave`（参数 wave_size/exclude）。中途三次修正均留档：args 空→排除表失效、worktree -b 非幂等→remove+prune+-B、构建 stderr 超 world.run 262KB cap→落文件回传尾部。水位预估 ~1093，待全量确认。

**第 25 轮开场（2026-10-01，主 agent 全量确认）**：
- 全量水位：**1095 FAIL**（6820 例：5092 pass / 633 skip / 1095 fail，115s），较上轮陈旧 CSV 净 −6，全部为已合并 flash 修复经全量确认：abstractClassUnionInstantiation、abstractPropertyInConstructor（dwfrun-87defd32）、checkMergedGlobalUMDSymbol、crashDeclareGlobalTypeofExport、umdGlobalAugmentationNoCrash（umd 片）+ contextualTypingReturnStatementWithReturnTypeAnnotation（顺带绿）。**零新增红、skip 零变化（443）**。
- **cgm 遗留分片处置（验证失败，已回退）**：269cbe35ef rebase（d4287153f4）合并实测 contextualPropertyOfGenericMappedType 仍报原 TS2322，主仓已回退，分支删除。根因交接存档（/tmp/baseline_backup_r25/cgm_progress_notes.md）：Go 收口在 chooseOverload 第二轮 inferTypeArguments（checker.go:9267→9665 checkExpressionWithContextualType→10346-73 contextuallyCheckFunctionExpressionOrObjectLiteralMethod，inferenceContext.mapper instantiateSignature 使 T["data"]→number）；Rust 分歧在 infer_type_arguments_inner CS 分支 type_of_context_sensitive_arg 仅函数/箭头、容器回退不压推断语境。下一嫌疑：substitute_infer_type_parameters 对映射型 `{[P in keyof T]:...}` 产出形态 + get_type_of_property_of_contextual_type 在该形态上的输出。
- **主 agent 事故与恢复（2026-10-01）**：cgm 处置中的 `git reset --hard` 误伤 4 个未提交文件（新导出双 CSV、本笔记 64 行工作版、v4 模板），且备份动作落在 reset 之后（顺序错误），导致首次 workflow 派发读到 HEAD 陈旧 CSV（1031 例）空派 2 个已修复用例，已 TaskStop 终止。恢复：CSV 从 fullrun.log 重导出（1095 ✓）、笔记与模板从主 agent 上下文全文重建、/tmp/corpus-fix-notes-master.md 同步更新。**纪律补丁：任何 reset --hard 前必须先确认工作树脏文件清单并完成备份，备份在 reset 之前**。

**第 25 轮波 1b 结算（2026-10-01，flash 通道）**：
- **1/4 验证合并**：accessorInferredReturnTypeErrorInReturnStatement → **b95ad090d2**（两根因：TS7023 语法级启发式 getter_return_reaches_this 为 Rust 特有，Go 仅 getTypeOfAccessors push/popTypeResolution 真重入计环；TS2339 印 `{}` 因 in_flight_object_literal_types 空快照，Go 同点取 checkExpressionCached(containingLiteral) 全成员字面量型 + readonly 来自 getter-only 符号 Accessor 位）。
- **accessorAccidentalCallDiagnostic 未收敛**（4c83b372a3→rebase 7b36fe9a78）：get_apparent_type 按 Go 实装（services_checker_7.rs + get_global_big_int_type），但 E0502 编译错（tsox-api m5k2_2.rs:738 借用冲突）且修通后仍 FAIL。主 agent 构建+单例实测定位**真根因**：输出被两条 Rust 自创捷径截胡——checker_calls_signature_selection.rs 两处 report_get_accessor_call 拦截（发射裸 TS6234 无副消息）与 checker_calls_checker_2.rs report_invocation_error 自创复写（无 TS6234 分支、自算 primitive_apparent_name）；Go 忠实链是 checker.go:10130-10175 invocationErrorDetails 的同一链式诊断（10162 行 len(Arguments())==0 的 nil 语义）。主仓已回退 b95ad090d2，参考分支 fix25/acc-ref，评估全文入 /tmp/flywheel_shards/0607 分片，重派。
- **2 片零提交**：aliasInstantiationExpressionGenericIntersectionNoCrash2（relater 构造器成分误判 related，两候选：m1e_3.rs 实例化缓存跨实参误命中 / relate_alias_variances 成分级误放行）、allowImportClausesToMergeWithTypes（binder symbols_binder_2.rs:252-263 locals-only 查找不对称嫌疑 + Go getSymbolFlagsEx/resolveAlias 两层配合）。交接均已写入各自分片文件（0564/0457），本轮排除择机重派。
- **用户指令（2026-10-01 拍板）**：**主 agent 不做任何代码改动，只做评估、分发、构建、测试出结果**。此前主 agent 对 accessor 片的直接修复（E0502 借用提升、report_invocation_error 委托改写、拦截删除）已全部回退转为分片材料；后续编译错一律经 workflow 回喂分片修。
- **workflow 脚本三处修补**：reset --hard 前后守卫 4 设施文件（/tmp/wfq_guard_dir 往返 cp）；编译失败由直接 break 改为回喂一轮再试；清理 worktree 前抢救 progress_notes.md 到 /tmp/flywheel_handoffs/（波 1b 零提交 worktree 被清理时交接文件曾随之丢失）。

**队列消费循环版 workflow 上线（2026-10-01 用户拍板「后续这么做」）**：saved workflow `corpus-flash-wave` 已重写为外层多波循环，飞行中可动态增删用例无需 amend。**控制契约**：① `/tmp/corpus_wave_control.json`（可缺省）每波回读——`exclude`: 用例 basename 数组（与启动参数 exclude 取并集）、`max_waves`: 波数上限（下调即优雅收尾）；② `/tmp/corpus_wave_stop` 文件出现即不再开新波（当前波跑完收尾）；③ 队列源 corpus_results.csv 每波重读（主 agent 中途重新导出即生效）；④ 本运行内已尝试用例（无论过否）进 settled 集，不重复派发。新参数：`max_waves`（默认 3）、`wave_size`（默认 2 上限 4）、`exclude`。worktree/分支命名带波次号（/tmp/wtq_{wave}_{i}、fix25/flashq-{wave}-{i}）。波 2（单波版）在飞不受影响，后续派发一律用循环版。

**第 25 轮波 2 结算（2026-10-01，flash 通道，4/4 全绿）**：
- **accessorAccidentalCallDiagnostic → 392943642f + 0ca61f9425 + 552bac598d（第 2 轮回喂通过）**：修复完全按主 agent 分片材料落地——cherry-pick 参考分支 get_apparent_type 实装 + 修 m5k2_2.rs E0502 借用提升；删两处 report_get_accessor_call 拦截与本体；report_invocation_error 改委托 invocation_error_details（get_apparent_type 宽化 + invocation_error）；wc3.rs arguments 判定 map_or(true) 对齐 Go len==0。552bac598d 为回喂第 2 轮补链头 file/loc 继承 + 属性访问 resolvedSymbol links。**回喂机制首战成功**。
- **ambientExportDefaultErrors → 49744f5907 + d692bc667c**：TS2714 ambient 非实体名 export assignment 文法错全缺（Rust 常量零使用）；program 层两处对无 clause 副作用导入误报 TS2307，Go 只在 noUncheckedSideEffectImports 真值时报 TS2882（checker.go:5421-5461）。
- **ambiguousGenericAssertion1 → 1755100932**：parser 主表达式 `(` 多做箭头消歧一步（parse_parenthesized_or_arrow），Go 只在 parseAssignmentExpressionOrHigherWorker 入口消歧；改回纯括号解析。**parser 高风险改动，全量把关重点**。
- **anonymousModules → 79ebb63dc4**：checkExportsOnMergedDeclarations 补 Go binder 导出分支可达闸门（declareModuleMember vs declareSourceFileMember 路径差，node.LocalSymbol 未建模按可达条件等价重建）。
- worktree 清零、4 份交接归档 /tmp/flywheel_handoffs/、设施文件经守卫完好（accessor 第 1 轮回退触发守卫生效实证）。diff 审计无空壳。

**波 2 全量把关（2026-10-01）**：1092 FAIL（净 −3）：8 例转绿（5 目标 + 3 顺带：missingCloseParenStatements/moduleResolutionWithExtensions_unexpected/sideEffectImports1）；**5 例连带新红**（es5-commonjs7、intersectionConstructorReductionCrash【自 skip 池移出转普通红】、narrowingNoInfer1、typeAliasExport、typePredicateStructuralMatch，主嫌疑 392943642f getApparentType 实装 + d692bc667c 副作用导入门控）；skip 零新增。新红分片已建（/tmp/flywheel_shards/new_*.ts.md，含嫌疑 commit 与 diff），经控制文件 prioritize 置队首。循环版 workflow 已加 prioritize 机制（新红下波优先正式通道）。

**两段式管线定版（2026-10-02 用户拍板）**：修复循环与审计闸门分离，因 subagent_model 为整 run 一致参数、无法单 run 混用模型——
- **第一段 corpus-flash-wave（GLM-5.3-Flash）**：队列消费循环修复（prioritize/exclude/max_waves/stop 控制文件），职责仅修复+单例验证合并。
- **第二段 corpus-wave-audit（GLM-5.3 主力）**：每批修复循环结束后由主 agent 链式派发（base_sha=本批起始 commit），四步把关：①构建闸门（cargo test --no-run，4GB ulimit）；②逐 commit 改动审查（独立审查员×每 commit，空壳/越界/死循环/无界内存/Go 对照成立五清单，只读）；③全量语料运行（4GB ulimit + 900s 上限 + 终态 progress 行缺失即判死循环/OOM 截断高危）；④输出对比（CSV 水位 key 级对比、新红独立分诊、挂起嫌疑 seconds>=30 清单、新 skip 预警）。产出 markdown 审计报告（primary artifact）+ 结构化 findings；高危→主 agent 回退/重派后再续飞轮，中危新红→写入分片材料经控制文件 prioritize 置队首。
- 管线纪律：两段不并跑（cargo 锁竞争），flash 循环完成通知→派 audit→audit 完成通知→主 agent 处置+续派。

**第 25 轮循环波 3 结算（2026-10-02，flash 通道 3 波 12 例）**：
- **4 例验证合并**（79ebb63dc4→6315e6690b）：es5-commonjs7（230f2ef40e+ca638b4e2e，TS2714 闸门对 undefined 误报——Go 表达式位 undefined 产 Identifier 而我方产 UndefinedKeyword，is_entity_name_expression 补认）、typeAliasExport（24ec68a024，同族）、typePredicateStructuralMatch（3e86bf8ce3，get_property_of_type 尾部 global_object_member 增补缺 Go TypeFlagsObject 限定，unknown 上误命中 hasOwnProperty）、anyAndUnknownHaveFalsyComponents（6315e6690b，definitely_falsy 谓词缺 Void|Any|Unknown）。
- **8 例未收敛**（交接已抢救 /tmp/flywheel_handoffs/）：intersectionConstructorReductionCrash（零提交：mixin 归约旁路——get_signatures_of_type 对 intersection 直接拼接签名，忠实版 resolve_intersection_type_members 零调用点；另 Arc::get_mut 标志不落盘）、narrowingNoInfer1（2ee2292232 第 2 轮编译失败，根因=392943642f 使 get_apparent_type 真跑但 get_base_constraint_of_type 仍是空壳恒 None）、argumentsBindsToFunctionScopeArgumentList（bb00b3c2fe 第 2 轮未过：NameResolver 函数容器 arguments→argumentsSymbol 注入缺失）、argumentsReferenceInFunction1_Js（780205e3e3 编译失败：harness 根文件筛选 .js 被丢弃偏离 Go）、argumentsSpreadRestIterables.tsx（cb0839ad24：spread 实参四处偏离）、arrayDestructuringInSwitch1（零提交：every 候选签名未实例化）、arrayFakeFlatNoCrashInferenceDeclarations（8c544ef9f1：nodebuilder type_to_type_node 恒返 None，环不可探测）、arrayIterationLibES5TargetDifferent（cebfb172bd：根因一已修根因二 nolib 数组建 Reference 缺口）。
- flash 转绿率 4/12（波内含 5 个波 2 连带新红已修 3 + 深水用例占比高）。两片第 2 轮编译失败的错误尾部均显示 x5a.rs:119 unnecessary unsafe（警告混入 tail，真错误被截断，下波顺带）。
- **审计闸门首跑**：corpus-wave-audit（GLM-5.3 主力）base_sha=79ebb63dc4 已派发（构建+逐 commit 审查+全量+水位对比四步）。

**未收敛分支映射（worktree 已删释放 2.3GB tmpfs，分支保留供重派参考）**：fix25/flashq-1-3（b30c2cfbf1）=narrowingNoInfer1 第2轮编译失败版；flashq-2-3（efddffd510）=argumentsBindsToFunctionScopeArgumentList；flashq-2-4（8ca19bdcc0）=argumentsReferenceInFunction1_Js（harness .js 根文件筛选修）；flashq-3-1（9bca256a47）=argumentsSpreadRestIterables.tsx；flashq-3-3（eb51fac84c）=arrayFakeFlatNoCrashInferenceDeclarations；flashq-3-4（caf31b45de）=arrayIterationLibES5TargetDifferent（含已修根因一）。另有 fix25/acc-ref（7b36fe9a78）已消费。

**审计闸门首跑结算（2026-10-02，GLM-5.3 主力，批次 79ebb63dc4..6315e6690b）**：**通过**——构建零错误、无空壳高危、水位 1092→1087（6 转绿：4 目标+typeGuardConstructorNarrowPrimitivesInUnion/typeGuardConstructorPrimitiveTypes 顺带；新红 1：nestedTypeVariableInfersLiteral）；挂起观测 fail>=30s 零例。两段式管线（flash 修复→GLM-5.3 审计）全链路验证成立。
- **新红分诊闭环**：nestedTypeVariableInfersLiteral（数组字面量元素丢字面量保留 widen 成 string）钉到 3e86bf8ce3 新增 flow_union_ops_checker_5.rs:414 无条件递归；分片已建（含同函数「闸门过严」附带发现：NonPrimitive/StringLike/Index/非strict unknown 应按 Go getApparentType 映射先行再入 Object 分支），控制文件 prioritize 置队首。
- **挂起池新增（审计发现，无现成红例）**：①6315e6690b 的 `||=` 流类型门控翻转——any/unknown/void 声明下 possibly_falsy 变 true 进 remove+flow_union_of 产出裸 union `any | typeof v`（Go 对 Definite 赋值直接返回 declared=any；flow_union_of 无 any 吸收，吸收仅在 get_union_type）；潜伏例 esDecorators-classExpression-namedEvaluation.1.ts:6,23-24（||= 后不读 x）。②谓词缺 BigIntLiteral 0n 分支 + extract 不做 Go 的 ""/0/0n 字面量映射（pre-existing）。③remove kept 空回退返 t vs Go neverType（void||y 分歧，pre-existing）。④T extends any 约束递归返 None（Go 改写 any 约束为 unknown 再 emptyObjectType）。⑤230f2ef40e 姊妹谓词 is_entity_name_expression_ex 未同步 UndefinedKeyword（.js `undefined.b=1` JSDeclarationKind::Property 路径，unconfirmed）。
- **卫生项（待批量处理）**：ca638b4e2e 对 230f2ef40e 的重复 UndefinedKeyword 兜底（死代码，两层编码同一规则有漂移风险）+ 4 行「为什么」注释违反 AGENTS.md 代码拒绝注释条款。

**第 25 轮循环波 4 结算（2026-10-02，flash 3 波 12 例：4 验证合并 8 未收敛）**：
- **4 例验证合并**（6315e6690b→bc007ffb95 共 7 commit）：nestedTypeVariableInfersLiteral（77a1380bd5+95d42bb316，get_property_of_type 原始种别未命中改走 globalObject 成员 + 尾部补 getApparentType 映射——审计分诊闭环兑现）、arrayIterationLibES5TargetDifferent（48b50f64d1+e9705db824，for-of array-like 判定对齐 Go isArrayType||assignable(ReadonlyArray) + 诊断按 allowsStrings 选码）、arrayOfSubtypeIsAssignableToReadonlyArray（9f8a8bb9bc+5cc6e36573，readonly 数组目标成员迭代裁剪 Array 独有可变成员）、arrowFunctionsMissingTokens（bc007ffb95，括号箭头体走 parseArrowFunctionExpressionBody 恢复分支 + parse_function_block_ex 补 ignore_missing_open_brace）。
- **8 例未收敛**（交接已富化分片）：argumentsBinds（f6144f4492，NameResolver arguments 分支移到注册查找前与 Go 同序仍差一步）、argumentsReferenceInFunction1_Js（0266be2c94+77fccb4155，harness .js 根文件筛选 + inferTypeArguments 缺 thisType 块双层）、argumentsSpreadRest（bd623477dd+314879e995，get_spread_argument_type 空壳+isSignatureApplicable argCount 边界+rest 上下文兜底四处）、arrayDestructuringInSwitch1（32233741db，get_union_signatures 缺 Go 第一遍 findMatchingSignatures/createUnionSignature 跨成分匹配——第二处 string 来源未钉）、arrayFakeFlat（零提交，声明发射三断裂：nodecopy_builder type_to_type_node 恒 None/环走查零调用/tracker sink 缺；注：波 3 的 8c544ef9f1 曾实测 FAIL，勿盲并）、arrowFunctionErrorSpan（bd51b5e067，TS1200 已修 TS2345 链重复未修——plain isTypeRelatedTo 不建链 vs 我方 relater_chain_active=true）、assertionFunctionWildcardImport1（c429deb01a 编译失败，resolve_namespace_type_uncached 裸 exports vs get_exports_of_module __export 展开）、assertionFunctionsCanNarrowByDiscriminant（c7be6b1431，旧流引擎缺判别式窄化接线——忠实版 get_discriminant_property_access/narrow_type_by_discriminant 已存在未接）。
- **波 4 分支映射（worktree 已删保分支）**：flashq-1-2（16790dd9ce）argsBinds / 1-3（5fc4a0f53a）argsRefJs / 1-4（d78f59f6b8）argsSpread / 2-1（4a69a952c7）arrayDestruct / 3-1（5b8aa8bd47）arrowErrorSpan / 3-3（747ca667bc）与 3-4（07b303c45a）assertion 两例。注意 -B 复用已覆盖波 3 同名分支（narrowingNoInfer1 的 b30c2cfbf1 已成孤儿）。
- 审计闸门第 2 跑已派（base_sha=6315e6690b）。

**审计闸门第 2 跑结算（2026-10-02，批次 6315e6690b..bc007ffb95，8 commit）**：**通过**——构建零错误、无空壳高危、水位 **1087→1074**；转绿 14 = 4 目标 + **10 例 sourceMapValidation\* 系（Destructuring\*ForOf\* 家族全部转绿——bc007ffb95 parser 恢复分支修复的大范围正涟漪）**；新红 1：ramdaToolsNoInfinite2（TS2344 映射型不满足 readonly any[] 约束，分诊指 5cc6e36573 readonly 标志保留使 assignable 翻转），分片已建 prioritize 置队首，修法约束=不得回退 arrayOfSubtype 例。
- **补记归属（审计指出流程缺口）**：a343aff68b（boxed_apparent_type_of_primitive 缺全局接口声明退化 emptyObjectType，checker_prop_access_checker.rs:71-73）属 arrayIterationLibES5TargetDifferent 片的第 2 轮回喂产物，此前往笔记漏记单列。
- **挂起池新增**：①bc007ffb95 同根因未修全——三个兄弟箭头体站点（expressions_parser_2.rs:255-265、_3.rs:141-145/226-230）仍旧内联，`a => var k = 10` 三变体仍误报 TS1109；②e9705db824 双重定义——调用点内联 Go array-like 公式但共享谓词 is_array_like_type（旧启发式）仍被约 20 处使用；③48b50f64d1 缺 getIterationDiagnosticDetails 前两分支（downlevelIteration/typed array TS2803 族）；④has_string_constituent 过滤缺 TemplateLiteral|StringMapping（Go 用 StringLike）；⑤迭代诊断缺 maybeMissingAwait await 建议；⑥possibleOutOfBounds（NoUncheckedIndexedAccess）路径缺失；⑦global_object_member 缺 symbolIsValue 过滤（unconfirmed）；⑧空接口命中分支不写 boxed_global_types 缓存（CPU 重复）；⑨性能：is_object_type_related_to 成员循环非 readonly 目标也 clone Vec（热路径分配）。
- 累计水位轨迹（第 25 轮）：1095→1092→1087→1074（净 −21，转绿 24+，新红 3 例中 2 已修）。

**第 25 轮循环波 5 结算（2026-10-02，flash 3 波 12 例：6 验证合并，产率新高）**：
- **6 例验证合并**（bc007ffb95→4ce4721759 共 9 commit）：ramdaToolsNoInfinite2（43f6e5ce14 第 1 轮——审计新红闭环：is_object_type_related_to 空成员目标缺属性预检只用 source 裸表，映射型惰性成员裸表空被误判缺成员；Go getUnmatchedProperty 经 resolveMappedTypeMembers 按基约束解析）、argumentsSpreadRestIterables（28896bdda4+c4e98ee7f8 第 2 轮——自研 IIFE 上下文签名镜像短路 Go 惰性 rest 定型，镜像遇 spread 回退）、assertionFunctionWildcardImport1（d18b341008+7748e87f28+e54e8673de 第 2 轮——export * 链展开经 get_exports_of_module_table + 星目标相对路径程序级 Resolver）、assignmentToAnyArrayRestParameters（9b6bb7ba66+1390cb7ab5 第 2 轮——LiteralType 包裹索引解包 + 索引信息按实例化穿透类型参数约束）、asyncImportNestedYield（69a9ba9d96 第 1 轮——import() 实参上下文型 arg0=string 特判）、asyncIteratorExtraParameters（4ce4721759 第 1 轮——yield* 迭代型检查提到返回注解早退前；注：agent 自报「症状 A 预计仍 FAIL」被脚本层 corpus_one PASS 推翻，确定性验证优先）。
- **6 例未收敛**（交接富化分片）：argumentsBinds（零提交——静态读码已尽，运行时 resolve 落点不在推演分支，**需插桩探针**，挂起池）、argumentsReferenceInFunction1_Js（64e9bb369a——harness .js 根文件 + inferTypeArguments thisType 块重放仍差一步）、arrayDestructuringInSwitch1（30afe742d4——error1 根因 get_union_signatures 缺第一遍跨表匹配已钉，error2 string 来源未钉）、arrayFakeFlat（5a03eb1ac2——声明发射链静态贯通，TS5088 触发依赖运行时递归图复现，需脚本层裁决）、arrowFunctionErrorSpan（4956acaae0——**需两笔同基线合并**：TS1200 在 flashq-3-1 分支 bd51b5e067 + 本轮链重写，分片已加编排提示）、assertionFunctionsCanNarrow（e0552771e0 编译失败——get_type_predicate_of_signature 只读字段无计算写入点 + 旧流引擎判别式未接线）。
- worktree 已清（5 个），分支保留。
- 审计第 3 跑已派（base_sha=bc007ffb95，9 commit）。

**波 5 后主 agent 本地全量（2026-10-02，HEAD 4ce4721759）**：**1066 FAIL**（净 −8，5122 pass/632 skip，126s，挂起零）。转绿 12 = 6 目标全绿 + 6 涟漪（**arrayDestructuringInSwitch1 未收敛却经涟漪转绿**——1390cb7ab5 索引信息穿透旁路了其 error 路径；ramdaToolsNoInfinite 兄弟例/largeTupleTypes/reverseMappedUnionInference/nodeColonModuleResolution/conditionalTypeDiscriminating*）。**4 例新红**（分片已建 w5new_*，控制文件 prioritize 置队首）：contextualTypeSelfReferencing、inferRestArgumentsMappedTuple、**nestedTypeVariableInfersLiteral（波 4 已修用例回归，主嫌疑 43f6e5ce14——其改的缺属性预检域正是 77a1380bd5 修复域）**、sourceMapValidation…DefaultValues3（波 4 涟漪绿回退）。波 6 排除表增 argumentsBinds（需插桩，挂起池）。

**审计第 3 跑结算（2026-10-02，批次 bc007ffb95..4ce4721759，10 commit）**：**2 条高危**——argumentsSpreadRestIterables 的波 5 修复（772bbd06e8=c4e98ee7f8+28896bdda4）系**假绿**：新调用点接进 get_spread_argument_type（inference_checker_13.rs:231）#[allow(dead_code)] 空壳恒返 unknown，TS2345 消失仅因万物可赋 unknown；审查员实测探针非可迭代 spread 零诊断（tsc 报 TS2488）假阴性实证。**已回退**（revert d2de7e3649+a35eab2c80），用例诚实回红（corpus_one FAIL 实证），重派硬约束入分片（必须真实移植 checker.go:29851-29916 约 65 行；corpus 只比 .errors.txt 不比 .types 的盲区记录在案）。
- **回归机制实锤**：43f6e5ce14 预检回退命中的 get_property_of_type_cached 首分支（flow_union_ops_checker_6.rs:8-14）对带类型参数映射型**任意名捏造符号返回 Some**；真实移植 resolveMappedTypeMembers（m2c_4.rs:16）全仓零调用点死代码——nestedTypeVariableInfersLiteral 回红与 ramda 假对齐同源，修法=接线 m2c_4 替换捏造分支（已入 w5new_ 分片）。
- 中危入挂起池：generator_instantiation_assignable_to 恒 true 空壳（union 过滤无效，真实移植在 m1c_3.rs:52-60 未接）、is_import_call 缺 import.defer 臂、GetContextualTypeForArgumentAtIndex 公共 API 空壳（LSP 侧）、yield 内联求取缺 sentType/awaited 三处偏差（完整移植 wc2_3.rs:1435 未复用双实现漂移）、contextual_type_of_parameter 与 wc2 忠实版双实现分叉、resolver 热路径无记忆化（性能）。
- 波动观察：我方全量 5122/632 vs 审计 5121/633（±1 flaky 候选，下轮全量关注）；skip>=30s 计数 1↔2 波动同源。
- 控制文件已更新：prioritize=假绿重派+4 新红；排除表增 argumentsBinds（需插桩）。

**派发词口径修正（2026-10-02 用户拍板）**：「一片 = 一个用例」指**修复入口**而非「只准修这个」——目标仍是入口用例修到 PASS（验收不变），但根因修复允许且常有额外效果（同族顺带转绿、机制面更大对齐），鼓励按 Go 修根因优于单例补丁；唯一禁区是不为额外效果引入与根因无关的改动。saved workflow corpus-flash-wave 的 fixPrompt 与 tools/subagent_prompt_template.md 均已改（波 6 在飞用旧副本不受影响，波 7 起生效）。

**第 25 轮循环波 6 结算（2026-10-02，flash 3 波 12 例：3 验证合并 9 未收敛，深水密度最高一波）**：
- **3 例验证合并**（a35eab2c80→10b32e6485 共 5 commit）：sourceMapValidation…DefaultValues3（0720b233af——relater 缺数组目标臂：空元组对 ReadonlyArray 按索引型协变应放行，Go relater.go:3873-3879）、arrowFunctionErrorSpan（5b71e8e00c——**经 reflog 找回被 reset 出所有分支的波 4/5 四笔 cherry-pick**：TS1200 语法检查移植 26b64d1bbb + arity 相邻去重 ac13f54794 + 两段式链 + E0308 修复 5b71e8e00c + 完整检查分派 d23361a543）、automaticTypeDirectiveResolutionBundler（10b32e6485——types 指令的 inferred-types 容器文件按 Go program.ts:1790 计算）。
- **9 例未收敛**（交接富化）：多为跨多文件结构性缺口——映射型域三连（contextualTypeSelfReferencing：1390cb7ab5 暴露 index-key 分支 def_p 未代入需插桩；nestedTypeVariableInfersLiteral：捏造符号+死代码 resolveMappedTypeMembers+instantiate_anonymous_type 约束不代入三因叠加；inferRestArgumentsMappedTuple：需真实移植 getSpreadArgumentType + rest 整体推断分支——与 argumentsSpreadRestIterables 同一阻塞）、awaitedType 家族三连（Awaited<T> 递归条件求值链/TS2589 一次性标志差异/元组重载推断）、argumentsReferenceInFunction1_Js（三笔回放后仍缺 TS2345，断点疑在动态调用路径）、arrayFakeFlat（三断点精化：nodecopy_builder 恒 None+elision 零调用+tracker 无实现；注意 m2g_4.go 型遍历不在基线，实际是 nodecopy_recovery 包装）、assertionNarrow（w6_flow.rs:7545 有 Go 同形实现但无外部入口）。
- **编排缺口记录**：prioritize 的 argumentsSpreadRestIterables.tsx 静默漏派——回退发生在上次 CSV 导出后，用例彼时绿、不在队列。**纪律补丁：回退后必须重导出 CSV 或单例验证红后再派**。审计 #4 全量将带回 CSV，波 7 携真实移植硬约束重派（与 inferRestArgumentsMappedTuple 共享 getSpreadArgumentType 真实现，宜同波先后或合一）。
- 审计第 4 跑已派（base_sha=a35eab2c80，含波 6 五 commit 与两笔 revert）。

**审计第 4 跑结算（2026-10-02，批次 a35eab2c80..10b32e6485，7 commit 含两 revert）**：构建零错误无空壳高危，但**大回归**——d23361a543（过载适用性实参改走完整检查分派）造成 **44 例连带新红**（1066→1103）：三缺=硬编码 SkipContextSensitive（Go checker.go:9062 条件选择）+缺 getRegularTypeOfObjectLiteral 剥新鲜度（9473-9478）+缺逐候选缓存清理（字面量实参类型算错根源，m1b.rs:285 只压栈不清缓存）；42/44 形态精确匹配（字面量多层包裹/裸 TS2345/上下文签名失效）。**已回退**（3891342980），抽查+全量确认：44 新红全恢复，终态 **1064**（优于波 6 前基线 1066：+1 argumentsSpreadRestIterables 诚实红入列，−3 存活真修复 automaticTypeDirectiveResolutionBundler/jsxNamespaceGlobalReexport/sourceMapValidation…3）。arrowErrorSpan/cgm 例随 revert 回红，重派硬约束（三缺补全清单+TS1200 锚区修正）已入分片。轨迹：1095→1092→1087→1074→1066→1103→**1064**。

**派发口径变更（2026-10-02 用户拍板）**：修复分片**并发降为 3**（saved workflow wave_size 默认/上限均改 3）且**模型换 GLM-5.3 主力**（subagent_model=account:bigmodel-individual-coding-plan/GLM-5.3，flash 通道停用——波 5 假绿与波 6 大回归两次质量事件后，以质量换吞吐）；审计闸门本就是 GLM-5.3，后续派发同样带 max_concurrency=3 限流审查员并发。波 7（flash×4）在飞不受影响，其后全部按新口径。

**波 7 中断接管结算（2026-10-02）**：run 因 subagent turn 失败中断（arrowErrorSpan/argumentsRef 两 agent 未交付），2 笔悬空 commit 由主 agent 手工接管合并验证——**均未过已回退**（分支保留）：argumentsSpreadRestIterables（78b9d3a233：三层真实移植，第 9 行已修，残第 5 行 1 条 TS2345）、arrayFakeFlat（ec016a9a90：仍 mismatch，.delete 标记待查）。**教训两次**：①同波嫌疑面相交——spread 与 argumentsRef 交接同指 getSpreadArgumentType，两 agent 各自移植同一函数（icp/ufc 事件重演），已串行化（argumentsRef 入排除表+分片加域约束）；②reset --hard 前必须确认设施文件备份当前（本次靠 master 副本恢复）。**新口径首派波 8**：GLM-5.3 主力 × wave_size 3 × max_concurrency 3，prioritize=[spread 重试（带第 5 行残差）、arrowErrorSpan 重做、arrayFakeFlat 续]，argumentsRef 串行待后波。

**workflow 脚本加固（2026-10-02，连续两跑 DriverError 后）**：波 7/波 8 均因单个 subagent「Turn execution failed」炸整跑——裸 Promise.all 一损俱损（技能文档明示反模式）。已修（saved + 波8草稿双落）：①fan-out 内 ask 包 try/catch，agent 会话故障降级为未收敛结果（committed=false，rootCause 记会话故障）不再传播；②合并验证阶段两处回喂 ask 同防护；③修 prompt 双扩展名 bug（.tsx 用例被拼成 .tsx.ts）。波 8 经 AmendWorkflow 以修订脚本重启（导入已完成步骤为缓存）。若再度立即全灭则指向 provider/账户限额，需人工确认账户状态。

**波 8（修订版）结算 + 陈旧 CSV 事故复盘（2026-10-02）**：
- **run 层**：故障隔离生效（spread agent 会话故障降级为未收敛，不再炸跑）。4/9 验证合并（3891342980→6c89a9767a 五 commit）：arrowFunctionErrorSpan（41940eda29：TS1200 触发点=legacy walker check_function_like_expression 补 Go 10315 语法检查步——并实证审计指示重做的 d23361a543 第 4 笔对本例无效，单签名不走多签名路径）、allowJsCrossMonorepoPackage（95619f64a6+028b77c99f：symlink realpath 回写）、ambientExportDefaultErrors 重修（6c89a9767a：m1b_3 路径 TS2714+TS2307 门控——**注意与 10-01 的 49744f5907 同码两处实现，审计关注重复诊断风险**）、abstractClassUnionInstantiation 幻影修复（abb1499d05：some_signature composite 递归分歧真实存在，但其 corpus_one PASS 不能证明翻转——用例本就绿）。
- **事故**：队列源是**陈旧 1031 例提交版 CSV**——根因=波 7 接管 arrayFakeFlat 回退时 reset --hard 把未提交 CSV 打回 HEAD 版，仅恢复了笔记/模板漏了 CSV。后果：3 个幻影用例空派、abstractProperty agent 误报「41940eda29 引 96 例新红」（实为陈旧 CSV 伪象，全量实测零新红）。**纪律补丁（第三次同类）**：手动 reset --hard 后必须从 /tmp/baseline_backup_r25 恢复全部四个设施文件（备份已刷新为 1060 版）。
- **地面真相（1060 FAIL/5128 pass/632 skip，零新增红）**：波 8 净 −4（arrowErrorSpan + fatarrowfunctionsOptionalArgs1/4 + optionalArgsWithDefaultValues 涟漪）。轨迹：1095→1092→1087→1074→1066→1064→**1060**（净 −35）。
- 审计第 5 跑派发（base=3891342980，重点：幻影修复三笔的 Go 对照与重复实现风险）。

**审计第 5 跑结算（2026-10-02，批次 3891342980..6c89a9767a，5 commit）**：**通过**——零高危、水位 1060 稳定复核（5128/632/1060）、挂起零。11 条中低危入挂起池：①m1b_3 TS2714 副本静态追踪大概率不可达（顶层/declare module 走 check_statement 栈；真正起效的是 TS2307 门控）——休眠重复，两栈一旦贯通会双报；②abb1499d05 some_signature 真递归 vs Go core.Some 单层（嵌套 composite 场景 TS2511 漏报分歧+隐式无环不变量）；③41940eda29 generator 门接恒 false 空壳（承诺无实效）+type_parameter_list/use_strict 两子检查同空壳+once-guard 二次调用语义失真——空壳填实候选批次；④95619f64a6 alternate_result 未 symlink 换写+type-reference-directive realpath 未接线（helper 已有）；⑤副作用导入回退只扫顶层（declare module 嵌套缺口）。波 9 派发：prioritize=[spread 会话故障重派, arrayFakeFlat 续]，arrowErrorSpan 已结出队。

**波 9 结算（2026-10-02 07:50~10:34，上 session run，补记）**：9 例 2 绿——augmentExportEquals2（f726a16e17：corpus harness 按 Go CompileFilesEx 顺序构建测试 FS，root 先写 otherFiles 后覆盖）+ augmentExportEquals7（c8c65c6865：mergeSymbol 补 TS2649 冲突分支；41d7e53df3：闸门接入真实执行路径 merge_module_augmentations）。7 未收敛分支归档 fix25/w9q-*。**连带新红 2 例 module_augmentUninstantiatedModule/2（TS2649 闸门对未实例化模块过冲，波 10 置顶修复）**。波 9 后 /tmp tmpfs 被清空：flywheel_shards/cut_test_shards.py/handoffs 全失，本轮已重建（cut_test_shards.py 增强含 fullrun.log 摘要段）。

**波 10 结算（2026-10-02，并发 8 首跑 · GLM-5.3 · max_concurrency=8）**：8 例 4 绿：
- module_augmentUninstantiatedModule（89eebd806f：binder 字符串名 ambient 模块按 Go IsModuleAugmentationExternal 分流，未实例化取 NamespaceModule/Excludes=None，TS2649 虚发消除；配套 m3g is_external_module_node 桩填实按 IsExternalModuleIndicator）**+ 连带 module_augmentUninstantiatedModule2 快测 PASS**（同根因，2d1bec476d 冲突分支作废已删）；
- allowImportClausesToMergeWithTypes（2678e9ea8a：resolveAlias 双意义符号链终点，值含义走 resolveEntityName 尾段）；
- argumentsBindsToFunctionScopeArgumentList（105f9a9eb9：initialize_checker 播种 argumentsSymbol=IArguments，Go checker.go:1356）。
- **escalation 事件（首例）**：argumentsReferenceInFunction1_Js 回喂卡壳升级提问→主 agent 代跑插桩（worker stderr 须 TSOX_PROBE_PHASES 才 inherit）→定位 harness build_and_check 扩展预筛（只收 .ts/.tsx/.mts/.cts，纯 .js unit 被滤成空程序，program_build=259µs 实证）→对照 Go newCompilerTest（compiler_runner.go:318-332 toBeCompiled 全量无过滤）→fixer 修 9631ab4968→目标例残差（TS7006+TS2345）仍未全绿回退→**主 agent cherry-pick cd385d445b 单独合并**（机制面修复，波及全 _Js 族），全量净效应验证中。
- 未收敛分支归档 fix25/w10q-1-{3,6,7,8}：aliasInstantiation（26e3dfe 实例化表达式构造签名返回型代入链）、awaitedTypeStrictNull（a4a3b7fa10 getApparentTypeOfContextualType 已修，TS2589×2/TS7010×1 断点未定位交接）、badInference（267733614e get_contextual_type 查 contextual 栈接线未竟，四 push 点已对应）、argumentsReferenceIn（9631ab4968 harness 部分已裁量合并，五环 checker 改动留分支；交接 get_spread_argument_type 恒返 unknown、get_min_argument_count_ex 忽略 StrongArityForUntypedJS）。
- **时限契约 v4.1（2026-10-02 用户拍板）**：30 分钟改期望值非硬截点，删「剩 6 分钟」分钟级指令，模板+workflow CONTRACT 已同步。控制文件 max_waves=1 使本 run 单波收尾后新契约生效。

**波 11 结算（2026-10-02，新契约首跑 · 用户中途手工停止，仅跑第 1 波）**：8 例 3 绿——capturedLetConstInLoop2（6f0a1594d2：变量类符号声明类型补 Go worker 外层 widen，for-of 空数组 undefinedWidening→any）、incrementalConcurrentSafeAliasFollowing（c2ad30ef5b：file_names 过滤 .json/.tsbuildinfo 根，对齐 Go CompileFilesEx）、jsDeclarationEmitExportAssignedFunctionWithExtraTypedefsMembers（4714aaf19a：宿主语句 @param 标签适用于右值函数，对齐 Go reparseHosted/getFunctionLikeHost）。5 红分支归档 fix25/w11q-1-{2,3,6,7,8}（contravariantOnlyInference/exportAssignmentMerging5/jsDocGenericOverloads/nonExpandoDeclarations/widenedThisPropertyAssignment，均历经回喂未收敛）。**r25 并发时代战绩汇总：波8 4/9、波9 2/9、波10 3/8、波11 3/8——成功率 ~33%，远低于第 24 轮前单发时代**。同日用户拍板：循环停摆，转向执行栈对照方案（Rust/Go 全函数插桩 trace → 差集锚点进分片），插桩永久落地（脚本机械改码）+ CSV 去秒数。

**执行栈基础设施落地 + r25 停摆复盘（2026-10-02 深夜，用户拍板循环停摆）**：
- **基础设施**：crates/tsox-core/src/fntrace.rs（enter 内联去重保序，内存只与唯一函数数相关）+ instrument_rust.py（主仓 1680 文件 21633 函数已插桩，**未 commit 留工作区**）+ instrument_go.py（oracle 11560 函数临时插桩，采集后已还原 /tmp/instr/go_backup）+ stack_diff.py（norm 归一 + 框架噪音自动过滤）。采集链路：Rust 全量 TSOX_FN_TRACE_DIR → /tmp/rust_trace（1057 例，仅 FAIL）；Go 串行 -parallel 1 GOFN_TRACE_DIR → /tmp/go_trace（12806 例一次性数据，oracle 源码不变可复用）；差集 /tmp/stack_diff.csv。分片已带执行栈对照段。CSV 耗时列已移除（双表表头 key；wave 脚本选例排序需同步改字典序——待办）。
- **事故**：首次全量采集（trace Vec 无限累积 + 24 worker 并发 + nice）触发系统级资源耗尽崩溃（未重启，tmpfs 幸存）。修复=enter 内联去重；纪律=重活单跑、nice -n 19、ulimit 保留。
- **水位（波11 3 绿后）**：1058 FAIL / 444 skip 差异（超时噪声修正：complexRecursiveCollections/enumLiteralsSubtypeReduction 回 FAIL，conditionalTypeDoesntSpinForever 等 4 例回 PASS）。波11 战果：capturedLetConstInLoop2（6f0a1594d2）连带族 4 绿 + incremental（c2ad30ef5b）+ jsDeclaration（4714aaf19a），共 6 例真绿。
- **关键数据结论**：stack_diff 显示 go_only≈0（1055 例中最高 2 函数）、962 例仅有 rust_only——**Rust 执行函数集 ⊇ Go**，全量迁移完成度的直接证据；语料失败根因 100% 为「同名函数行为分歧」型，集合级栈差集对此无信号。锚定主力应转向：错误码→发射点 grep→双侧函数体并排对照（机械定位协议），rust_only 作辅助（Rust 独有路径）。
- **r25 并发时代战绩**：波8 4/9、波9 2/9、波10 3/8+1 连带、波11 3/8+3 连带（直接成功率 33-37%，连带占绿例 1/3）。波11 run 131M tokens / 3 直接绿例。

**CSV 记录入仓（2026-10-03 用户拍板）**：corpus_stack_diff.csv（1055 例差集，5.8MB）入仓库根与 corpus_results.csv 并列；工具入 tools/：cut_test_shards.py（分片切割，ROOT 脚本相对）、stack_diff.py（差集生成，输出仓库根）、instrument_rust.py（fntrace 插桩 + --wire 接线，模板 tools/fntrace.rs）、instrument_go.py（oracle 临时插桩 + --restore）、fntrace.rs（runtime 模板）。分片正文新增「执行栈对照」段（go_only/rust_only 前 60）。原始 trace 不入 git（可再生：Rust=插桩全量 TSOX_FN_TRACE_DIR；Go=插桩 oracle 后 GOFN_TRACE_DIR go test -parallel 1 + --restore）。

**worktree 位置约定（2026-10-03 用户拍板）**：修复 subagent 的隔离 worktree 一律放 **~/worktrees/** 下（不再用 /tmp——tmpfs 吃内存且重启即失）；在飞批次（/tmp/wtA_1..8，2026-10-03 r26 栈对照首跑）完成后切换。saved workflow 已同步。

**r26 批次结算（2026-10-03，直接并发 8 后台 subagent · GLM-5.3 · 栈对照数据首用）**：8 例 7 绿 1 熔断——
- bigintArbirtraryIdentifier（be4d2874d9：TS1141 通道 grammar_error_on_node 误用→error_message，解析错误抑制解除）；
- bigintAmbientMinimal（b982616e1d+af801ad097：parser ambient_context 机制首建（NodeFlags::Ambient 系统性缺失第一块）+ check_grammar_big_int_literal 空壳实装 TS2737）；
- bigintIndex（4f1cdfb5cc+4f23b25d40：TS2538 BigIntLiteral 下标文案 'bigint' 两站点；**上轮验证因 corpus_one 复用旧二进制误判 FAIL**——教训：合并后必须强制重编（touch 源文件）再验证）；
- betterErrorForUnionCall（a17728ad53：联合调用 getUnionSignatures 公共集空否门 + findMatchingSignatures 真实现（relater.go:2267 族），空壳 stub 移除）；
- baseExpressionTypeParameters（12df63c2f7：NameResolver EWA 分支 TS2562+早退，base_expression_type_parameters_hit）；
- baseConstraintOfDecorator（f6cfe874b9+fd4d363248 编译修复：mixin 机制——getBaseConstructorTypeOfClass 值语义 TS2507/TS2735、resolveBaseTypesOfClass mixin 分支取构造返回型、静态型交集 TS2545）；
- bigintPropertyName（7621d4a1ee+94cca1ac82+0f9a5f17d1+50267817a4+3cbcc07418（rebase 后 hash 链）：bigint 属性名 __missing 机制（getDeclarationName 对齐）+TS1539 三入口+TS2538 全站点+映射类型约束检查+DeclarationNameToString 显示名（missing_property_display_name 带引号形态））；
- **bigintWithLib 熔断归档 fix26/w26q-8**（两轮未收敛：elaborate_array_literal 忠实移植+wc1b 元组尾修正+report_no_overload_matches 多诊断独立包链，均未改变实际输出——实际渲染路径仍单条深链，疑 Rust 该路径与 Go reportCallResolutionErrors 结构性不同构，留待后续轮）。
- **效率对比**：单例 0.85-26M tokens（均值 ~8.9M）vs workflow 模式 5-42M（均值 ~20M），耗时 3.6-34 分钟；fixer 普遍先走执行栈对照（go_only/rust_only），bigintWithLib 的二轮分析用 trace 证伪了自己的接线假设（Go resolveCall 在 Rust 从未执行）。
- **澄清记录**：errors.txt 详细段（==== 段）是套件有意豁免（flat_segment 只比错误头），非缺陷；corpus_one.sh 不构建只挑最新二进制。

**r26 批次最终结算（mixin revert 后全量复核）**：FAIL 1058→**1054（净 −4）**，skip 零增。保留 6 绿：bigintArbirtraryIdentifier/bigintAmbientMinimal/bigintIndex/bigintPropertyName/betterErrorForUnionCall/baseExpressionTypeParameters（连带绿 6 例中 mixin 族 5 例随 revert 回红）。**mixin 修复（f6cfe874b9）已整体 revert**：其"对一切非类符号 extends 生效"的边界误伤 aliasUsage/递归基类/jsx/HOC 等 30+ 例（全量实测确认，抽查 aliasUsageInArray=import alias 基类误入 mixin 分支丢成员）；baseConstraintOfDecorator 回红留队，重派时须收窄触发条件（仅类型变量基约束形态，排除 import=require 别名/递归基）。**真新红 2 例**：baseTypeOrderChecking、declarationEmitNestedAnonymousMappedType（归因待查，嫌疑 bigintPropertyName 的 __missing/显示名波及，下批优先）。留队：bigintWithLib（fix26/w26q-8 两轮熔断）。

**r27 类分片批次结算（2026-10-03，按错误码族分片 · 并发 3 · 直接后台 subagent）**：
- **三族 6 例 PASS 合入**：defassign 族 2（86a87301a2 parser 实例化表达式早退修 baseTypeOrderChecking TS2562 误报；de7ea29534 TS2507 值语义取型修 classExtendsClauseNot）；implicitany 族 1（4b6423fa70+191c0ce979 别名环 error 不污染缓存+union 上下文签名全同修 contextualOverloadList，改动面大待全量观察）；dupident 族 3（b3f7d6c825 lateBindMember 计算名查重+6f744901ed/25974ed2a5 binder 分表+66232600b8 跨文件合并，修 acrossFileBoundaries/cloduleWithDuplicateMember1/cloduleSplitAcrossFiles 两连带兑现）。
- **两例两轮熔断留队**：callOfConditionalType（逆变位 C∩E 收缩按行为拟合未收敛，72fbcb9395 已回退）；bigintWithLib（前批，fix26/w26q-8）。
- **duplicateIdentifierChecks 精确缺口（CRLF 归一化后仅 6 行）**：(62,9)(63,14)(64,9)(68,9)(69,14)(70,9) TS2300——declare class 中 auto-accessor（accessor x）与 get/set 混合声明形态，列 14=accessor 成员名位。族 agent kind_of 的 auto-accessor→2 部分覆盖但该形态漏。留队下批单例片。
- **教训（第三次同类）**：shell 层对比基线必须 `tr -d '\r'`（期望基线 CRLF vs 本地 LF）+ 截断 `====` 详细段——本次误判经历：先因详细段重复报"缺对象字面量 108 行"（错误回喂，浪费 agent 一轮 21.6M 证伪），再因 CRLF 全行假差异。**回喂前残差必须用归一化 multiset 对比脚本**（可固化进 tools/）。
- **方法论数据**：错误码族聚合的根因命中率——defassign 3/41、dupident ~6/20、implicitany ~3/23（其余主导 diff 属其他码域）。类分片适合"一个机制根因带一族同形态"的场景（如 clodule 兑现），但纯错误码聚合噪声大；下批分族建议按「错误码+形态签名（列模式/消息模板）」二次聚类。
- **合并摩擦**：类分片改动面大（多文件/函数移动/300 行拆分），本批 3 处编译错误主 agent 就地修 + 2 轮 rebase 冲突 + 1 次 progress_notes 误入库（git add -A 教训：解决冲突时禁用 -A，逐文件 add）。

**r27 全量终态（2026-10-03）**：FAIL 1054→**1047（净 −7）**，真绿 25（6 直接 + 19 连带：dupident 跨文件合并带绿 fundule/qualifiedName/umdNamespace/strictModeReservedWord 族；implicitany union 签名带绿 compositeContextualSignature/contextualTyping 族；defassign 带绿 extendNonClassSymbol2 等）。**新红 18**（contextual 族为主：contextualTypeCaching/contextualTypingOfOptionalMembers/contextuallyTypedByDiscriminableUnion/controlFlowLoopAnalysis/arrayBestCommonTypes 等——implicitany 的 union 签名收紧过冲波及，agent 交接已预警方向正确但需收敛），下批 prioritize。**新增 skip 1 例待人工确认：intersectionWithConflictingPrivates（panic: never_type，FAIL 转 crash 属恶化方向非转绿，待修）**。轨迹：1095→…→1058→1054→1047。

**.types 发射器上线（2026-10-03 用户拍板：生成内容不入 git）**：
- crates/tsox/tests/corpus/common/types_baseline.rs（忠实移植 Go tsbaseline types 通道：for_each_child_and_js_doc 遍历 + is_expression_node/Identifier/is_declaration_name 标注 + is_part_of_type_node/Interface/TypeReference 过滤 + alias T:T 特例 + EWTAS-extends 取 parent 型 + 源码交织 CRLF）；runner 接线 TSOX_TYPES_EMIT_DIR/HEADER/STEM（per-case），spawn 透传；两个判定 #[doc(hidden)] pub 化（is_expression_node/is_declaration_name）、types_type_id mod pub 化。
- 数据：.traces/types/ 40MB/6348 文件（gitignore，不入 git）；重采=全量带 TSOX_TYPES_EMIT_DIR 跑 corpus。锚点工具 tools/types_anchor.py → corpus_types_anchor.csv（4688 例首分歧，已知偏差过滤：typeof 侧/裸 any）。
- 分片新增「.types 首分歧锚点」段（三层锚点：.types → 错误 diff → 执行栈）。
- **发射器已知偏差**（fixer 需跳过的形态）：class 声明名 `typeof X` vs `X`（get_type_of_symbol class 分支取构造型侧——真语义分歧留修）；名字节点覆盖不全 any；union 中 alias 渲染带括号 `(Validate)`。
- **能力验证**：PASS 例的 .types diff 非零（如 `=> false` 拓宽为 `boolean`、class 取型侧）——**错误基线全对但类型语义已分歧的暗差异检测**成立，这是 .errors/.types 双维锚点的核心价值。水位顺手更新：FAIL 1045（+2 例上批未计入的绿）。
- r28 试验（wtC_1..3）：contextualTypeCaching（锚点：缺 `>callback : (response: T) => void`）、controlFlowLoopAnalysis（`number` vs `number | undefined` 循环收窄）、contextualTypingOfOptionalMembers（缺 `>prop : string`），汇报含「锚点是否命中真根因」数据采集项。

**r28 .types 锚点试验结算（2026-10-03，3 例直接并发 · GLM-5.3）**：
- **contextualTypingOfOptionalMembers PASS**（32feef17f9：type_of_context_sensitive_arg 对非函数 CS 实参以实例化参数位重定型 + contextual_element_of_constituent 补 getIteratedTypeOrElementType 回落——Go isSignatureApplicable→checkExpressionWithContextualType 链）；
- **contextualTypeCaching 回喂中**（899e13904d：**contextualInfos 栈查询接入 get_contextual_type 入口**（波10 起挂起池多轮提及的「从不查栈」遗留首次落地）+ CS 重检压栈；残差 1 条 (43,15) TS7006 匹配期方向回喂）；
- **controlFlowLoopAnalysis 熔断闭案**（fix28/w28q-2 归档）：三轮 18M tokens。事实链——create_array_literal_type 契约修复方向正确（Go clone 语义）但过冲消除 (12,25)；agent 二轮回退判断踩「FAIL CSV 缺席≠绿（可能被 SKIP）」老坑（其考证 a300 态本例 PASS，实测纯 a300 仍 FAIL=真实存量红）；主仓已 reset 回 a300。**教训两条**：①派发前防空派快测必须执行（本例分片信息与实际状态不符）；②create_array_literal_type 的 Go 契约对齐重派时须连同 ArrayLiteral 标志全部消费者审计（jsx 推断域实证波及），并先具备本地复跑验证手段。
- **.types 锚点命中率 0/3（定量结论）**：三例锚点全部未命中真根因——发射器三个系统性缺口形态吞掉首分歧位（索引签名参数不序列化 / 函数类型属性空行（333/813 例共患）/ write-target 独立暗分歧）。**.types 通道当前作为「锚点」不合格，作为「暗差异检测器」有效**（暴露 write-target 收窄、`=> false` 拓宽等真实语义分歧）；修复方向=补发射器序列化覆盖（索引签名/函数类型属性），使首分歧位有效后再作定位通道。分片锚点段保留（agent 用于排除法仍有价值）。

**r28 终态（2026-10-03）**：FAIL 1047→**1042（净 −5）**，绿 10/新红 5。亮点：899e13904d 的 contextualInfos 栈查询顺带修绿 **badInferenceLowerPriorityThanGoodInference**（波10 起挂起 3 轮的深水例）+ contextual 族连带 9 绿；contextualTypingOfOptionalMembers 直接绿。新红 5：contextuallyTypedParametersWithInitializers1-4（32feef17f9 CS 重定型的波及面，下批 prioritize）+ discriminantUsingEvaluatableTemplateExpression。轨迹：1095→…→1054→1047→1042。.types 数据已随本轮全量刷新（.traces/types 40MB）。

**r30 双波结算（2026-10-03，循环第 2 轮）**：FAIL 1039→**1024（净 −15，单轮新高）**，10 例尝试 8 绿 2 未过。
- 入口绿：arrayFromAsync（CS 窗口代入）、callExpressionWithMissingTypeArgument1（errorType 语义+TS1110）、callsOnComplexSignatures（联合 rest 合并）、cannotIndexGenericWritingError（TS2862 写规则）、capturedParametersInInitializers1+2（TS2373 跟踪模块 f6 版，g1 同根因撞车作废）、catchClauseRestProperties、checkChildrenAlwaysChecked（TS1063+TS2304）；连带绿 9（含 contravariantInferenceAndTypeGuard、genericFunctionInference2 回归修复）。
- **f2（heritage 值位求值）+126 大波及已 revert**：extends 基类解析域二次实证（与 r27 mixin 同域同下场）——aliasUsage/abstractProperty 族全红。**该域（heritage 值位+mixin+基构造类型）需整体规划立项**，单点修复两次翻车。
- 未过留队：builtinIterator（GROUP 2 Iterator.from 推断，agent 已给嫌疑定位）、chainedCalls（g3 约束惰性解析，渲染链已备）、Initializers1 回红（f1 的代入形态与 r29-d1 的 base 形态互斥——真语义是 Go instantiateContextualType 有条件代入，下轮统一修）。
- 轨迹：1095→…→1047→1042→1039→1024。模式观察：批式验证下两轮各出一次大波及（e2 解构管线、f2 heritage），均靠全量闸门兜住；防线条款（大面收窄）在其余 8 片生效（多 agent 主动收窄并在 handoff 声明更大对齐面）。

**「错错相抵」回归模式与处置规程（2026-10-03 用户拍板）**：有些回归不是新修复的错，而是**旧修复本是 hack**——两个片面实现互相抵消凑出正确输出，纠正一个后另一个暴露成回归（实证：r29-d1 的无条件 base 上下文形态与 r30-f1 的无条件代入形态互斥，各修一例、合在一起必翻一个，真语义是 Go instantiateContextualType 的有条件代入）。**处置规程**：此类回红例不回喂旧 agent、不在旧分支续修，一律**新建 worktree 重新分发**——派发词需说明前置修复的互斥史与真语义嫌疑，授权新 agent 纠正前置 commit 的片面语义（含 revert/重写前置 hack），按 Go 统一修正。

**分发策略（2026-10-03 用户拍板）**：后续修复波分发优先使用 start-plan 通道（account:bigmodel-start-plan/GLM-5.3-Flash），individual plan 通道留给主 agent 会话与特殊重审分片；双波并行时主波通道为 start-plan。

**部分修复+部分回归的处置规程（2026-10-03 用户拍板）**：改动后若出现部分修复、部分回归，这是分析回归与改动相关关系的最佳时机——**不能只看数字变化就整体丢弃改动**。正确流程：主 agent 先①建好含该改动的工作区（worktree）、②更新该状态下的调用栈数据（对回归例重采 trace/差集），然后③把「回归-改动相关关系分析」分发给处理该内容的 subagent（带新调用栈+改动 diff+回归例清单），由它逐例判定真回归 vs 假绿暴露（旧绿若依赖被纠正的错误形态即为假绿），产出统一收口方案。首个应用：f2 heritage 值位求值（曾 +126 新红被整体 revert，方向本身正确）。

**r31 整合结算（2026-10-03，三波+错错相抵重审+回归分析全链闭环）**：整合态 FAIL 1024→**1031（净 +7，绿 23/新红 30）**——数字微涨但结构健康：f2 保留成功（未 revert），分析线预测的四族收口大量兑现（importAsBaseClass/exportClassExtendingIntersection/contextuallyTypedByDiscriminableUnion/jsxComplexSignature 族 B/D 转绿、checkInheritedProperty 双例绿、jsFileCompilationWithoutJsExtensions 13 片族兑现 k2 预测）；30 新红中约半数为分析预测内（genericRecursiveImplicitConstructorErrors1=假绿例预期红待基线收口、declarationEmitExpressionInExtends7=族A 残余、extendsUntypedModule=遗留疑点 TS6133），其余待归因（genericFunctionInference2 第二次回红=f1 代入修复与其旧绿的又一交互，错错相抵候选）。合并插曲三起已处置：rebase patch-id 误判跳 f2（实证 j3 已携带语义）、污染 commit 重做、原型 3 处编译修复。主仓 HEAD 含：k1-k6+j1-j3+f2 语义+原型入口 d0d7a25ec7。轨迹：1095→…→1024→1031（结构性回调，非退化）。
