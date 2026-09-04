# 分诊台账分类（2026-08-18，数据驱动重分类）

数据源：`_scripts/reconcile.py` 思路对账 —— 2818 条台账 vs 最新全量跑产物
（tests/baselines/local/）。可对账 2764 条（54 条本轮未跑/skip）、
843 个错误码签名，聚合为 16 个根因类别。

类别按「欠报 / 多报 / 文本差异」三分，括号为案例数。

## 欠报族（官方有错、我们零输出或少报，共 ~800）

| # | 类别 | 码 | 数量 | 根因（对照 Go） | 修复锚点 |
|---|------|----|------|------------------|----------|
| U1 | 变量重声明合并诊断 | 2403/2451/2300/2303 | 47 | 符号多声明时次要声明类型恒等检查（checker.go ~L5973 errorNextVariableOrPropertyDeclarationMustHaveSameType）+ binder mergeSymbol 跨文件块级冲突（binder.go ~L215） | checker::check_variable_declaration + binder::declare_symbol |
| U2 | 赋值兼容检查缺口 | 2322 | 75 | 各赋值位点（属性/解构/返回/数组元素）未走 check_assignment_compat | checker.rs 各 check_* |
| U3 | 调用实参检查缺口 | 2345 | 23 | 泛型推断后实参替换失效 / 重载选择缺比对 | inference.rs |
| U4 | 属性存在检查缺口 | 2339 | 23 | 基类/命名空间/索引签名的成员查找缺报 | flow.rs get_property_of_type 族 |
| U5 | 类成员继承检查 | 2449/2415/2416→2420/2320 | 33 | abstract 成员缺失（checkClassLikeDts）、private 非 derived（2415）、override 属性兼容（2416 而非类级 2420）、接口 extends 签名不兼容（2320） | checker.rs 类检查 |
| U6 | 重载族 | 2394/2709 | 17 | 重载表与实现签名不匹配（checkFunctionOrConstructorDeclaration）；方法赋值无匹配重载 | checker.rs |
| U7 | 流分析 | 2454/2872 | 18 | definite assignment（2454 漏报位）；always-truthy/falsy 谓词语义（2872） | flow.rs |
| U8 | 模块解析欠报 | 2305/2304 | 27 | 命名导入成员不存在（2305）；声明文件/全局名缺报（2304，多为 declarationEmit*） | resolver |
| U9 | 选项层 | 5107/7026 | 22 | moduleResolution=node10 弃用消息（5107）；JSX.IntrinsicElements 缺失（7026） | tsoptions + jsx 检查 |
| U10 | 杂项欠报 | 2353/2661/2394… | ~60 | 解构 shorthand excess、导出说明符引用外层声明等 | 各处 |

## 多报族（我们多报官方没有的错，共 ~570 大类，推断族为主）

| # | 类别 | 码 | 数量 | 根因 |
|---|------|----|------|------|
| O1 | 调用/赋值/属性/隐式 any 多报（推断族） | 2345+2322+2339+7006 | 229 | 泛型调用实参替换、上下文签名提取、延迟类型比较的连锁误报——与 39 FAIL 中 B 族同根 |
| O2 | 模块解析多报 | 2307 | 68 | composite/monorepo/node_modules/符号链接解析未移植——目录布局类用例 |
| O3 | 命名空间成员多报 | 2694 | 18 | 跨文件 ambient namespace 合并解析缺口（第四轮修过 globals 合并，残余） |
| O4 | 不可调用多报 | 2349 | 18 | Function 接口参数/联合签名的可调用性判定（`x: Function` 传参后不可调用——预存在） |
| O5 | 解析恢复多报 | 1005/1434 | 26 | emit 辅助文件的解析错误（errors-only 对比仍计入）；tagged template 修饰符 |
| O6 | 重复标识符多报 | 2300 | 15 | 声明文件 emit 场景的合并符号双报 |
| O7 | 杂项多报 | 2348/2741/2420 | 32 | spread 元数、缺属性链码、variance 码选择 |

## 文本差异族（码全同、文本/顺序不同，261）

elaborated error chain（嵌套金字塔）缺层/多层的显示差异 + 类型显示
（别名展开 vs 名称、字面量加宽显示）。与 relater_error_chain 的录制
埋点覆盖度同根，无独立逻辑错误。

## 修复顺序（按 权重=数量×机械度）

1. U1 合并诊断族（47，纯机械，Go 逻辑单点）
2. U9 选项层（22，机械：弃用消息 + JSX 探测）
3. O5 解析恢复（26，看样本可裁剪）
4. U5 类成员继承（33）
5. U6 重载族（17）
6. U7 流分析（18）
7. O1 推断族（229，最大但最难——B 族延迟比较为前置）
8. O2 模块解析（68，目录布局子系统）
