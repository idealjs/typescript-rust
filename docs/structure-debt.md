# 结构债务清单（AGENTS.md 规则 2/3 执行现状）

> 参考数据（2026-09-05 全仓审计）。规则本身见根目录 [`AGENTS.md`](../AGENTS.md)。

## 规则 2：单文件 ≤300 行

- 现状：**82 个非测试 .rs 文件超限**。最大：checker/checker.rs 26,517 行、
  diagnostics/messages_generated.rs 24,069（生成文件）、parser/mod.rs 9,851、
  checker/relater.rs 8,632、tsoptions/mod.rs 6,471、checker/typenode.rs 6,150。
- **已完成（第一期，checker.rs 26,517 -> 12,992 + 10 个族文件）**：
  `checker/checker/{calls,element_access,literals,statements,classes,enums,
  operators,expressions,modules,resolve}.rs`（140–1,981 行/个），按 rustfmt
  函数边界切分，跨族调用提升 pub(crate)，可见性自 HEAD 逐一恢复。
  方法：利用格式约定（方法起始于 4 空格 `fn` 行、结束于恰好 `    }` 行），
  不做括号匹配。
- 说明：`messages_generated.rs` 与 `ast/node_data_generated.rs` 为**构建生成物**
  （build.rs 产出），按惯例不拆分。
- 分期：
  1. 生成物豁免登记（已豁免）。
  2. 中型文件（300–1,000 行，约 30 个）逐个按职责拆分为 `mod.rs` 子目录，
     `pub use` 保持对外接口不变。
  3. 巨型文件（checker.rs 等）按既有内聚区域（grammar checks / assertion /
     assignment / narrowing / services …）先拆出叶子模块，再递归。
- 每步验收：`cargo test` 全门禁 + 12,466 用例 sweep 抽样。

### 已完成（2026-09-05）

| 文件 | 拆分前 | 拆分后 |
|---|---|---|
| checker/checker.rs | 26,517 | 12,992 + 10 族文件 |
| checker/relater.rs | 7,052 | 538 + 6 族文件 |
| checker/typenode.rs | 5,020 | 650 + 6 族文件 |
| checker/flow.rs | 3,840 | 546 + 5 族文件 |
| parser/mod.rs | 9,132 | 3,321 + 6 族文件 |
| binder/mod.rs | 4,159 | 979 + 3 族文件 |
| emitter/mod.rs | 4,883 | 2,361 + 6 族文件 |

方法：rustfmt 格式约定定位方法 span（4 空格 `fn` 行起、恰好 `    }` 行止），
按职责族切块，跨族调用提升 `pub(crate)`，可见性自 HEAD 恢复。

### 待拆（下一期）

parser 3,321 / binder 977 / emitter 2,361 的 mod.rs 二期收尾；
tsoptions 5,905；nodebuilder 2,609；scanner；printer；module/resolver；
execute；checker/checker.rs 12,992 二期。

## 规则 3：测试只在 tests/

- 现状：**71 个 `#[cfg(test)]` 内联模块、1,306 个内联 `#[test]`**
  （scanner 131、tsoptions 128、emitter 121、parser 111、printer 105…）。
- 分期：
  1. 被测私有逻辑改经 `#[doc(hidden)] pub` 暴露；测试逐模块迁移至
     `tests/`（按被测模块一对一同名文件）。
  2. 迁移顺序按模块体量从大到小：scanner → tsoptions → emitter → parser →
     printer → execute → checker → …
  3. 每迁移一个模块，删除对应内联模块并跑门禁。
- 新增测试一律直接放 `tests/`。
