> 版本：v1.0 · 2026-10-07

# fix52 波留队与回归归因

anchor 抹平波 fix52（20 例：4 重发/续修 + 16 新鲜，并发 6，含一次中断恢复）收尾记录：9 例通过、6 例零回归未收敛（已整合）、11 例留队。水位 FAIL 1289→1190（−99）、anchor 1986→1892（−94）、零回归、修好 84（两波最大单波降幅，主力 w16 洁净前缀 51 例 + w1 重发 24 例）。分支 fix52/w1..w20 保留，交接笔记 `/tmp/flywheel_verify/fix52-notes/`。

## 留队例（重发建议）

| 例 | 终态 | 归因与重发建议 |
|---|---|---|
| inferringAnyFunctionType1 | 二次零 commit | 两波静态穷尽均不可钉；断点收敛为「写入时 inferred[0] 已是 T / 读取侧缓存命中异常」二选一，须运行期插桩定位（笔记含方案） |
| arrayDestructuringInSwitch1 | 零 commit | 根因定界：Rust 缺 Go deferred 类型引用机制，别名链内数组/元组元素急切解析致 any 泄漏驻留缓存——数据模型级迁移，非单例可交付 |
| discriminantPropertyCheck | 回归携带未整合 | 修 4/6 簇但携 11 回归（布尔联合折叠波及 mappedType/spread/rest 族）；锚点机制「布尔联合折叠为 boolean」与 fix52/w5 已整合的窄口径置旗同族，重发应基于整合后基线重做 |
| controlFlowAliasedDiscriminants | 未收敛（净 3 commit 已整合） | 首轮 6 commit 已自回退（6 回归+anchor 无效）；终态 boolean 置旗窄口径零回归但自身 errors 未达 PASS，残留 f1/f5/f9/f10 无初始化式 const y=x 并集残差 |
| declarationEmitReexportedSymlinkReference | 未收敛（2 commit 已整合） | 六行分歧全修但复验轮 BUILD_ERROR→FAIL 反复；export * 链穿透与 symbolToTypeNode 全链已在基线，重发只须收敛验证侧 |
| anonClassDeclarationEmitIsAnon | 未收敛（2 commit 已整合） | 主根因已修（容器 membership 校验）；剩匿名类静态侧展开显示（getBaseTypeVariableOfClass 门+容器前缀+交集成）独立深水 |
| acceptSymbolAsWeakType | 未收敛（6 commit 已整合） | 三分歧修二；[[s,false]] 元组字面量未收敛，fresh 化修法已证伪丢弃（Go falseType 本身 fresh，checker.go:1006-1012） |
| aliasInstantiationExpressionGenericIntersectionNoCrash1 | 未收敛（1 commit 已整合） | 3/4 分歧已消；T_1 判据函数已锁定但触发路径静态不可达 |

已整合未收敛例的残留分歧随基线进入下波可选池；w16 尖端 commit fddb136（纯别名深解，搏 9 回归）弃用，但其 51 例修复价值已由洁净前缀保留。

## 观察

- 最终轮 37 例超时噪声 SKIP（jsx 族 + libCompileChecks/verifyDefaultLib_dom 等 lib 重例）：全部 timed out 分类、非新 SKIP，疑整合改动（JSDoc reparse 门控/lib 解析实装）增加 lib 处理耗时或本机负载，下波全量首轮观察是否回落。
- types_baseline.rs（crates/tsox/tests/corpus/common/）为 .types 发射器 harness（镜像 Go testutil/tsbaseline），w8 两 commit 修改该文件属渲染管线修复、非用例/基线篡改，有 4 笔历史先例，本波接受并在此留档口径。
