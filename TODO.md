# TODO

当前状态见 [`TESTING.md`](./TESTING.md) 顶部「当前状态」；遗留正确性差异台账在
[`tests/baselines/reference/triaged.txt`](./tests/baselines/reference/triaged.txt)。

## 性能（对应 idealjs/ts-go-rust-bench 的根因报告，按收益排序）

- [ ] 跨配置复用已解析库（SourceFile/程序级缓存）：多配置用例省 N-1 次全库解析，
      24 配置用例 12.8s → ~1.6s，全库墙钟约 -55%
- [ ] 扫描器注释/空白批量跳过 + ASCII 快路径：lib.dom.d.ts 1.35s → ~0.4s，底板 -60%
- [ ] 解析器分配架构：节点 arena、标识符 interning、text 零拷贝（去注释后仍 ~5×）
- [ ] 多线程解析/检查可行性评估

## 正确性（日期根因组，随分诊台账在案）

- [ ] 字符串映射 intrinsic 子系统（`Uppercase/Lowercase/Capitalize/Uncapitalize`）
      ——templateLiteralTypes1 同族
- [ ] 裸 Array 惰性成员表 vs 实例化 ReadonlyArray 的泛型签名规范化
      ——arrayToLocaleStringES2020
- [ ] 默认再导出链 TYPE 意义传播——reexportDefaultIsCallable
- [ ] 选项门家族补齐：allowJs/checkJs、ES5 downlevel emit、moduleResolution=Classic、
      noemit helpers、outFile 等

## 工程化

- [ ] CI 接入：四套门禁 + 分页 sweep 抽样
- [ ] bench 仓 runner 常态化（每次优化后 `bench/compare.py` 复测）
- [ ] `-next` 声明产出与 transpile 22 例 accepted-diff 逐项清账

## 已完成（历史摘要，详情见 TESTING.md 各轮记录）

- 全量 12,466 用例（compiler/conformance/transpile）基线对齐，双轮 sweep 全 0 FAIL
- 早期会话：对象字面量方法/访问器简写、构造器参数属性、`#prop` 私有标识符、
  `static {}` 块、CommonJS 全局、类访问器+生成器、逻辑赋值、for-of 定赋值等
  （彼时 conformance 30%→后经分页修复轮全量对齐）
