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

## 非确定性（高优先）

- **run-to-run 结果翻转**：`allowSyntheticDefaultImports9.ts` 等用例在同一二进制上
  多次运行结果不一致（通过/不一致翻转）。头号嫌疑：checker 中以 `Arc` 指针值
  为键的缓存（interface_instantiation_cache / attached_type_args_cache 等）
  存在 ABA 问题——进程间分配布局不同导致偶发错命中。
  部分缓存已做"值钉住"防护（见 git history「三缓存 ABA 钉住」），未覆盖全部。
  影响：任何单次运行的结果都不可作为最终判定；sweep 的 FAIL/通过都可能翻转。
  修复方向：①为所有指针键缓存补充"值等价"校验或改用内容键
  ②排查无锁单线程下的迭代顺序依赖（HashMap 随机种子逐进程不同）。

