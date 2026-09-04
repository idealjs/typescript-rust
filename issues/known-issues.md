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
