# TS2300 重复标识符错误族（fix27/fam-2）进度笔记

## 提交（时间序）
1. `e0a9..`（见 git log）fix(checker): 类/接口成员查重接入 lateBindMember 计算名解析与 mergeSymbol 冲突链
2. `6f744901ed` fix(binder): declareSymbol 成员表与命名空间导出表严格分表查重
3. `66232600b8` fix(checker): 跨文件 class/function 合并补 TS2813/TS2814 与 namespace 跨文件 TS2433
4. `25974ed2a5` fix(binder): mixed_exportness 过滤沿变量声明链上溯到语句层（对 2 的回归修正）

## 三入口判断
- duplicateIdentifierChecks：缺的全部是计算名（[foo]/[sym]）成员的 TS2300 与 I12 的
  related-info 组，Go 链路 = checker lateBindMember（名字解析）→ combineSymbolTables/mergeSymbol
  （早/晚绑定符号冲突，带 TS6203/TS6204）→ checkObjectTypeForDuplicateDeclarations（状态机）。
  修复 1 在 Rust 成员查重处按该链路补齐。
- duplicateIdentifiersAcrossFileBoundaries：缺 TS2813×2 / TS2814×2 / TS2433×1（两条 TS2300 已对）。
  修复 3 补 Go checkFunctionOrMethodDeclaration 尾段与 checkModuleDeclaration 跨文件分支；
  另补 getMergedSymbol 语义（经 globals 取跨文件合并符号）。
- complexRecursiveCollections：本地多报 4 条 TS2300（namespace Set 导出函数 vs interface Set
  同名成员），根因是 Rust binder declareSymbol 的 existing 查重在成员路径回退查 exports 表
  （Go 分表）。修复 2 消除；但该用例仍有 TS2395 多报×4、TS2430 措辞差异、TS2344/TS2320 缺失
  等其他域差异，预计仍红（TS2300 域内部分已消）。

## 验证关注点（主 agent 复跑时）
1. duplicateIdentifierChecks 是否全绿——尤其依赖：
   - check_computed_property_name_type 对 `const foo = "foo"` 给出字面量类型、
     对 `const sym = Symbol()` 给出 unique symbol 类型（is_type_usable_as_property_name）；
   - I12（158-160）三条带 related 的 '[foo]' 报错来自新增 check_member_late_merge_conflicts。
2. 修复 2 的回归面：
   - members-only 查重改动影响所有合并符号（ns+interface/ns+enum）；
   - mixed_exportness 过滤改动影响 TS2395 域（export var/var 混合）——已按语句层上溯修正；
   - clodule 族（cloduleWithDuplicateMember1）应由「TS2393 误报」转「TS2300×5 正确」。
3. 修复 3 的回归面：check_function_or_constructor_symbol 现在经 globals 解析合并符号
   （声明集交集守卫），影响所有函数符号检查入口；ambient 判定从 current_file 改为声明自身文件。

## 交接事项
- 族内仍未覆盖的子根因：
  - multipleExportAssignments(.InAmbientDeclaration)：缺 TS2300 'export=' ×2。
    declare_symbol_into 的冲突路径看起来应报（excludes=All），需实跑定位是 existing 未命中
    还是 binder_diagnostics 收集问题。
  - moduleSharesNameWithImportDeclarationInsideIt5：缺 TS2300 'M' ×2（namespace 内
    import M = ... 与 export namespace M 的别名冲突域）。
  - genericClassesRedeclaration：缺 TS2374（重复索引签名）×2，m1c_2.rs 索引签名去重域。
  - jsdocInTypeScript：多报 TS2552/TS2300 'T'（JSDoc typedef 在 TS 文件的解析域）。
  - errorElaboration/gettersAndSettersErrors/interfaceDeclaration1/functionExpressionShadowedByParams/
    extension：残余差异为其他错误码（TS2538/TS2808/TS2717/TS2310/TS2339），非 TS2300 域。
  - noSymbolForMergeCrash：缺 TS2649（augment 非 module 实体），mergeSymbol 的
    NamespaceModule 分支域。
- 仓库规范遗留：checker_class_dup_declarations 拆分后主文件 223 行合规；但
  checker_function_symbol_checks.rs（531，改动前已 431）与
  checker_statements_module_declaration_checks.rs（364，改动前已超）改动前即超 300 行，
  本轮为控制与并发片的冲突面未做拆分重构，建议后续独立结构性拆分。
- symbols_symbol_conflicts.rs 的 member_flags 空吞分支（member 间冲突不报，交 checker 循环）
  本轮未动：class/interface 成员冲突由 checker 侧循环覆盖且输出经 dedup 等价；若后续发现
  module 级成员冲突被吞（如 export=），可按「双侧均为 class-like 容器成员才吞」细化。
