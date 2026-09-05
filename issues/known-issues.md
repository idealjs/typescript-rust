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

