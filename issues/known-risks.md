# 风险分析:死循环 / 内存异常(2026-08-21)

针对最近一轮修复(round 9–13,提交 `4dfb191b9`…`d1a996988`)的改动审查,
定位到以下可能导致**长命令运行下线程/进程崩溃**(表现为挂死、超时风暴、OOM)的风险点。
按严重程度排序。

---

## 处理记录(2026-08-22,四个 Issue 全部落地,单测见 convergence_tests + resolver tests)

- **Issue 1**:
  - any 误判——`typenode.rs` 基类型降级判定改用 `is_type_error`(intrinsic
    name == "error",error_type 是 OnceLock 单例,判定精确);合法解析成 `any`
    的基类型不再标记 degraded(单测 `any_base_is_not_degradation`)。
  - 收敛性——新增 `heritage_retry_counts`(`HERITAGE_RETRY_LIMIT = 2`):同一
    接口符号的降级结果最多重试 2 次不落缓存,第 3 次起**接受**——强制写入
    declared-type / instantiation 缓存,并把 `heritage_degraded_events` 回滚到
    本次解析入口值,使外围 node-memo 帧正常缓存。真环(`A extends B; B extends
    A`)与稠密图因此有界收敛(每符号最多 3 次全量解析),对应 Go 惰性成员解析
    下"环基类型 = 不完整但稳定的共享类型对象"语义(单测
    `cyclic_base_interfaces_converge`)。epoch 子树作用域维持现状(入口/出口
    比较,已是正确作用域)。
- **Issue 2**:
  - 两个 memo 缓存加容量上限(各 300_000 条,超限整体 clear——纯函数缓存,
    清空不损正确性):`type_node_subst_cache`、`instantiated_member_type_cache`
    (单测 `subst_cache_respects_capacity`)。
  - 预算重置点对齐 Go(checker.go ~L2251 checkSourceElement):`check_statement`
    与 `check_source_file` 入口均重置 `type_instantiation_count`,emitter/LSP
    等非 check_expression 入口不再继承耗尽的预算静默退化。
- **Issue 3**:`attach_class_statics`(栈深即链深,上限 200 帧)与
  `resolve_base_class_instance_type`(`type_resolution_stack.len() >= 200`)
  补深度守卫,深(非环)继承链不再无界递归(单测 `deep_class_chain_bounded`,
  260 级链在默认栈上通过)。API 嵌入方的大栈要求仍建议在文档标注(CLI/harness
  已是 256MB 线程)。
- **Issue 4**:`ResolutionState.export_target_depth`(上限 16)守卫
  `load_module_from_target_export_or_import` 的条件对象/数组递归(单测
  `exports_target_nesting_bounded`)。

---

## Issue 1(高,死循环/挂死):heritage-degradation epoch 机制使缓存大面积失效 → 重复全量重解析

- **引入提交**:round 9(`4dfb191b9` 附近的 epoch 机制)+ D1 补丁 `d1a996988`
- **位置**:
  - `src/checker/typenode.rs:845` — 接口 cycle-guard 失败时 `heritage_degraded_events += 1`
  - `src/checker/typenode.rs:961-966` — 基类型解析结果含 `TypeFlags::Any` 即标记 degraded 并再次递增计数
  - `src/checker/typenode.rs:1080-1090` — degraded 时跳过 declared-type 缓存
  - `src/checker/typenode.rs:142-144` — epoch 不一致时跳过 `type_node_subst_cache` 写入
  - `src/checker/typenode.rs:304-314`(`cache_type`)— epoch 不一致时跳过 per-node resolved_type 缓存
- **机理**:
  1. 接口继承基类时若基类解析命中 cycle guard(互相递归接口、lib.dom 稠密继承图的首轮解析顺序),
     返回 error type → 标记 degraded → **三层缓存全部跳过**(declared-type、node-memo、subst-cache)。
  2. 对真正循环继承(`A extends B; B extends A`,TS 中合法)或稠密图,**每次重解析都会再次命中
     cycle guard → 永远无法收敛缓存** → 每个引用点都从头走完整 merge → O(引用数 × 继承图大小)。
  3. 长命令(全量 sweep、watch、LSP 反复 re-check)下表现为**无限循环式挂死/超时风暴**——
     代码注释自己承认了 "the r9 timeout storm"(`typenode.rs:93`)。
- **附带误判 bug**:`typenode.rs:961` 用 `bt.flags.contains(TypeFlags::Any)` 判定"基类走了
  cycle guard",但 `error_type()` 本身就是 `TypeFlags::Any`(`checker.rs:1768`),而**合法解析成
  `any` 的基类型同样是 Any flag**——`interface I extends SomeAnyAlias` 会被误标 degraded,
  永久禁用该接口的全部缓存,放大重解析风暴。
- **建议**:
  - 区分 error type 与 any(如独立 `TypeFlags::Error` 或比对 `Arc::ptr_eq(bt, &self.error_type())`);
  - degraded 跳过缓存应有**次数上限**(重试 N 次后强制缓存,接受不完整结果),保证收敛;
  - epoch 只在"本次查询子树内"生效,避免全局计数器把无关查询的缓存一并禁用。

---

## Issue 2(高,内存异常/OOM):两个 memo 缓存无界增长且永不清理

- **引入提交**:round 9/10(`4dfb191b9`、`162caa92a`)
- **位置**:
  - `src/checker/checker.rs:427` — `type_node_subst_cache: HashMap<(usize, u64), Arc<Type>>`
    (键 = node.id × 替换栈哈希,`typenode.rs:85-144` 写入)
  - `src/checker/checker.rs:641` — `instantiated_member_type_cache: HashMap<(usize, usize), Arc<Type>>`
    (`typenode.rs:3586-3610` 写入)
- **机理**:
  1. subst-cache 的键含**替换栈哈希**——泛型嵌套实例化下,不同栈组合数是组合级增长;
     每个条目还永久持有 `Arc<Type>`(结构化 object type 含完整成员 symbol 表,数百字节到 KB)。
  2. 5M 实例化预算(`typenode.rs:112-115`)在 `check_expression` 里**每个表达式重置**
     (`checker.rs:13081`),但**缓存本身跨表达式、跨文件、整个 Checker 生命周期累积,从不清理**
     (全仓库无 `.clear()` 调用)。
  3. 大型项目/长命令下条目数可达百万级 → 数 GB 常驻 → **OOM 进程崩溃**。注释自述
     "deeplyNestedConditionalTypes aborted on OOM before this"(`typenode.rs:84`)——memo
     解决了重解析爆炸,但引入了无界驻留。
- **建议**:
  - 按文件/按 check 轮次清理,或加 LRU/容量上限;
  - 预算计数器改为全局(或同时全局+每表达式),与缓存容量联动;
  - 注意 `type_instantiation_count` 只在 `check_expression` 重置——从 emitter/LS 等其它入口
    驱动的查询一旦累计 ≥5M,所有带替换的查询**静默退化为 error type**(正确性问题,且会掩盖 Issue 1)。

---

## Issue 3(中,崩溃):500 深度守卫只覆盖 `get_type_from_type_node` 帧,其它递归路径无守卫 → 栈溢出

- **位置**:
  - 深度守卫仅存在于 `get_type_from_type_node`(`src/checker/typenode.rs:116`,`type_resolution_depth >= 500`);
  - 但递归还流经**不经过该函数**的路径:
    - `attach_class_statics` → `get_type_of_class_declaration`(基类链递归,`checker.rs:4506-4565`,
      仅有 node_id 循环守卫,无深度限制);
    - `merge_interface_type_with_base` 逐基合并链;
    - relater / nodebuilder 的比较与实例化递归。
- **机理**:lib.dom 规模的深继承链下,这些未计数帧 × 每帧较大本地变量(排序 Vec、merged 成员表)
  可耗尽栈 → SIGSEGV 线程/进程崩溃。`main.rs:19-21` 已用 256MB 大栈线程缓解,CLI 主路径 OK;
  但 **API 直接嵌入方在默认 8MB 栈上调用 Checker 仍会崩**。
- **建议**:为 class-statics / merge 路径补统一深度守卫;在库层文档标注大栈要求,或由
  `Checker::new` 自行 spawn 大栈线程。

---

## Issue 4(低,死循环风险):self-name 模块解析无重入/深度保护

- **引入提交**:`923b175b4`(`load_module_from_self_name_reference`,`src/module/resolver.rs:1614-1648`)
- **机理**:包内裸 specifier 命中自身 package.json `name` 后走自身 exports 解析。
  `load_module_from_exports` 链路**没有 visited-set / 深度上限**;配合 `getPackageScopeForPath`
  向上找包作用域,若 exports 目标经符号链接/目录重入指回待解析模块本身,理论上可构造循环。
  正常文件目标会终止,置信度较低,但防御缺失。
- **建议**:解析链增加 (specifier, containing_directory) 访问集合或深度上限(对齐 Go 端
  `types.Separator`/解析深度保护)。

---

## 复现线索

- `TESTING.md` / 提交 `139a7ea59` 记录的 "r9 timeout storm"(Issue 1 直接对应);
- `deeplyNestedConditionalTypes` 曾 OOM(提交注释,Issue 2);
- 长时间 sweep 脚本(`run_full_sweep_*.sh`)与 `submodule_*_run_*.log` 中若出现超时/被杀(SIGKILL/OOM),
  优先核对 Issue 1/2。
