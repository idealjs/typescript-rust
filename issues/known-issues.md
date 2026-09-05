# 已知问题索引

问题与决策记录入口（决策背景见各链接文档；本文件只做索引）。

## 正确性

- **与 tsgo 基线的剩余差距**：715 用例 / 736 配置（Go 通过、我们 diff）。
  逐例语料、聚类与工具：[`../scripts/gostd/`](../scripts/gostd/)；
  分型：664 我们多报（检查器长尾缺口）、44 漏报（选项/声明诊断族）、
  11+8 弃用与 @types 发现多报、9 同码文本差。
- **upstream 口径分诊台账**：`tests/baselines/reference/triaged.txt`
  （128 个日期根因组 / 5,670 条登记；仅 upstream 口径使用）。

## 性能

- 解析管线为最大单点（默认库解析 0.96–1.4s/次构建）；根因与修复优先级：
  bench 仓 idealjs/ts-go-rust-bench `results/2026-09-04-root-causes.md`。

## 风险

- [`known-risks.md`](./known-risks.md) — 四项已识别风险与缓解状态。

## 非确定性（已修复 2026-09-05）

- **run-to-run 结果翻转**：`allowSyntheticDefaultImports9.ts` 等用例在同一二进制上
  多次运行结果不一致。根因：checker 多处缓存以 `Arc` 指针值为键——
  `Arc` 释放后地址被新对象复用（ABA），错命中导致诊断翻转。
  无防护的键：`relation_cache`/`relation_in_progress`（值为 bool，不钉任何一侧）、
  `degraded_type_ptrs`（degraded 未 accepted 的类型不进任何缓存，Arc 即释放）、
  `probe_cache_permissive/restrictive` 与 `subst_object_in_progress`（值是派生类型，
  输入类型不被钉住）、`type_argument_stack_hash` 与 `flow_cache_key`（地址混入哈希做缓存键）。
  修复：`Type` 启用全局 `AtomicU32` 唯一 id（`types.rs::next_type_id`，26 处构造点全部接入），
  上述缓存全部改按 id 键；id 单调递增不复用，ABA 通道消灭。
  附带修复：`compare_types` 此前因 id 全 0 恒等短路（永远返回 Equal），
  类型排序/去重随 id 启用恢复语义（对齐 tsc compareTypeIds）。
  验证：单用例 20 连跑输出哈希一致；page-1 三连跑失败集一致；
  300 例双跑失败集一致且与改动前逐用例相同（30 FAIL 集合零漂移）；
  门禁 1362/2/1010/15 全绿。
  遗留：Symbol/Node/Program 指针键因对象存活期等于进程存活期而安全，保持不变；
  `Signature` id 仍全 0（独立隐患，未启用签名同一性比较）。

## skipDefaultLibCheck：已实现、刻意未作为 harness 默认开启（2026-09-06）

- tsgo 的 test harness 对每条测试默认 `SkipDefaultLibCheck=true`（harnessutil.go，
  TSUnknown 时强制置 true），程序内全部 lib 文件（libFiles 全集）不参与类型检查。
- 我们的移植已实现同语义（compiler/mod.rs `build_checker_internal` 按
  `default_library_file_names` 全集跳过 `check_source_file`，get_diagnostics_to_report
  同步过滤），但 runner 从不设置该选项——测试口径当前为"检查 lib"。
- **不能直接开启**：开启后 `abstractClassUnionInstantiation.ts` 丢失 3 条 TS2511
  （140 例区间 105 pass→104）。丢失的恰是 `.map(cls => new cls())` 行——`Array.map`
  的回调上下文类型来自 lib.es5；我们跳过 lib 文件检查后，经 lib 泛型回调的
  上下文类型化失效。tsgo 同样跳过检查却不丢（其签名按需解析，不依赖文件级检查）。
  前置修复=让 lib 签名的上下文类型化与"该文件是否被 check"解耦。
- 开启也无性能收益：140 例 JOBS=8 实测 89s→90s（0），因为每例成本主体是 lib 的
  **解析**（worker-per-case 下每例冷解析）而非检查；20 线程骤降的杠杆不在这一项。
- Go 参照（同批 140 文件，tsgo runner `-parallel 20` + GOMAXPROCS=20）：**6s**；
  Rust debug：JOBS=8=89s / JOBS=20=149s（50 例撞 30s 超时记 SKIP，骤降复现）。


## 剩余 ~2× 差距的定位（2026-09-06，分配架构修复 ec405ae 之后）

- 分配架构修复后：lib.dom 解析 1.47s→158ms（9.3×），单例分配流量 51GB→29MB，
  140 例 JOBS=8 89s→22s、JOBS=20 149s→14s（0 超时，并行恢复扩展），
  全量 compiler 套件 66.6min→约 11min（20 worker 折算，跑到 99% 手动中止）。
- **新瓶颈=检查器固定地板**：worker 单例相位计时（TSOX_PROBE_PHASES）
  program_build=292ms / check=990ms / 其他≈20ms，与用例难度无关——check 的
  99.7% 花在检查整个 lib bundle 上（harness 未开 skipDefaultLibCheck）；
  临时开启实测 check 990ms→2.9ms。
- **开启 skipDefaultLibCheck 的真正前置**：绑定器没有为接口声明预填充成员
  符号——globals 合并后的 `Array` 符号 members 仅 `{T, ""}`，完整实例侧成员
  （map/filter/…）在"检查该 lib 文件"时才经 type_alias_links.declared_type
  解析并缓存。跳过检查后 `[...].map` 属性解析失败→回调参数上下文类型退化为
  any→TS2511 丢失（abstractClassUnionInstantiation）。tsgo 的 binder 在绑定
  阶段即解析接口成员，跳过检查无此损失。修复方向：绑定器预填充接口成员
  符号，或检查器对未解析符号做按需 resolveTypeMembers（get_property_of_type
  已可拆 cached/lazy 两段，lazy 段待接口类型解析器就位后接入）。
