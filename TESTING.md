# 测试流程

1. 每次测试 1000 个测试用例,不需要考虑回归。指令如下

```
TSOX_SUBMODULE_START=5100 TSOX_SUBMODULE_END=6099 TSOX_SUBMODULE_JOBS=4 cargo test --test submodule_compiler
```

2. 测试完成后，检查测试日志，对比 go 版，是否符合测试预期。
  - 如果不符合预期，检查原逻辑，尝试修复，重复测试步骤
  - 如果符合测试预期，记录到`当前批次`后，执行后100个测试用例，重复测试步骤

3. 严格 1000 个一批、只向前、不做回归
4. 不允许原逻辑未跳过的情况下，进行跳过测试用例

# 分诊（triage）规则

## 什么是分诊

某失败用例的背后是**一整个未移植的子系统**（如 .d.ts 声明产生、泛型实例化、
contextual signature 推理），无法在一批内逐例修复时，把该用例的基线路径登记到
`tests/baselines/reference/triaged.txt`。harness 再跑它时：

- 用例**照常运行、照常对比输出**（不是 skip；skip 是因不支持的编译选项直接不运行）
- 输出与官方基线不一致时，因路径在 triaged.txt 中，记为 `accepted-diff` 而非 fail

批次结果三类：`passed`（与官方基线完全一致）/ `accepted-diff`（分诊的已知差异）/
`skipped`（选项不支持未运行）。0 fail = passed + accepted-diff 覆盖全部。

## 判断标准

- 能在合理范围内对齐 Go 行为的**必须修**，不许用分诊掩盖实现错误
- 分诊的必须是"缺子系统"，不是"实现错了但懒得改"
- 每个分诊组必须带注明根因的日期头，说明缺什么

## triaged.txt 条目格式

- 路径相对 `tests/baselines/reference/`，即 `compiler/<stem>.errors.txt`
- `<stem>` 是**去掉 .ts/.tsx 扩展名**的用例名；多配置基线（如 `// @strict: true, false`
  产生的两份基线）写成 `compiler/<stem>(key=val).errors.txt`，
  例：`compiler/deleteExpressionMustBeOptional(strict=true).errors.txt`
- 组头两行 `##` 包裹：`## 日期 批次范围: 根因描述 ##`
- 同一根因的用例聚成一组，修复子系统后整组删除

## 人工查看分诊条目

```bash
# 总览：按组头列出所有分诊组及其根因
grep '^##' tests/baselines/reference/triaged.txt

# 条目总数
grep -c '^compiler/' tests/baselines/reference/triaged.txt

# 查看某个根因组下的全部条目（例如 const-enum 组；组头是多行 ##，
# 终止条件用下一组的日期头 /^## 2026-/，并过滤掉 ## 行）
sed -n '/const-enum checker family/,/^## 2026-/{/^##/!p}' tests/baselines/reference/triaged.txt | grep '^compiler/'

# 复核单个用例的真实差异：先跑它，再对比参考与实际输出
TSOX_SUBMODULE_START=5100 TSOX_SUBMODULE_END=6099 TSOX_SUBMODULE_FILTER=<用例名> \
  TSOX_SUBMODULE_JOBS=1 cargo test --test submodule_compiler
diff "tests/baselines/reference/compiler/<stem>.errors.txt" \
     "tests/baselines/local/compiler/<stem>.errors.txt"
# （无差异时 local 下是对应的 .errors.txt.delete 标记：官方有基线、我们无错误输出）

# 修复某子系统后，从台账删除整组条目，并重跑受影响批次验证转绿
```

# 当前批次

- start: 5100
- end: 6099

# 测试流程修改

1. 如果存在 rust 测试套件问题
  - 完全停止测试，与测试代办
  - 集中精力修复测试套件产生的问题
