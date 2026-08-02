# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：`tsconfig.emit-{rust,go}.json`（target es2023, jsx react-jsx, emit + declaration）
更新日期：2026-08-02

## 测试 A：诊断对比

| 编译器 | 诊断数 | 说明 |
|--------|--------|------|
| **TSGO (Go oracle)** | 0 | 零错误 |
| **TSOX (Rust)** | 28 | @types/react 内部 + 2 个 TS2604 |

**改善历程**：127 → 28（修复 resolver/parser/globals 后）

### 剩余 28 个错误分类

| 诊断码 | 数量 | 根因 |
|--------|------|------|
| TS2304 | 10 | 泛型参数解析（State/Action/Payload/is 在 @types/react 内部） |
| TS1005 | 8 | parser 对复杂泛型约束的语法支持 |
| TS2302 | 7 | static members 引用 class type params |
| TS2604 | 2 | React 组件类型（依赖 @types/react 完整解析） |
| TS2300 | 1 | `export as namespace React` 全局注册重复 |

**根因**：这些都是 checker 深度问题——泛型参数作用域、static 成员类型参数访问、
`export as namespace` 全局合并。需要 checker 深层改进。

## 测试 B：Emit 产物对比

### 产物文件集

| Go (6 files) | Rust (6 files) | 说明 |
|--------------|----------------|------|
| App.js ✅ | App.js ✅ | JSX transform 正确 |
| App.d.ts ✅ | App.d.ts ✅ | 值 import 过滤 ✅，返回类型推断 ⏳ |
| App.js.map ✅ | App.js.map ✅ | |
| main.js ✅ | main.js ✅ | import 重写正确 |
| main.d.ts ✅ | main.d.ts ✅ | 完全匹配 ✅ |
| main.js.map ✅ | main.js.map ✅ | |

## 修复状态总览

| 差异 | 状态 | 说明 |
|------|------|------|
| B1: JSX Transform | ✅ | `_jsx`/`_jsxs`/`_Fragment` 完全对齐 |
| B2: Import 重写 | ✅ | `.tsx`→`.js`，仅限 import/export |
| B3: Declaration emit | ✅ 部分 | 值 import 过滤 ✅，返回类型推断 ⏳ |
| B5: 非空断言 | ✅ | 正确擦除 |
| B6: node_modules emit | ✅ | 排除外部库 |
| A1: JSX 诊断 | ✅ | 合成全局命名空间 |
| A2: DOM document | ✅ | fallback 全局变量 |
| A3: .js 不解析 | ✅ | allowJs 默认跳过 |
| @types/react 解析 | ✅ | resolver 修复（移除过早 try_file） |
| @types/react 类型 | ✅ 部分 | 54 个内置全局已添加 |
| TS2604 组件类型 | ⏳ | 需 checker 深层修复 |

## 测试命令

```sh
TSGO=/Users/cqh/workspace/typescript-go/built/local/tsgo
TSOX=/Users/cqh/workspace/typescript-rust/target/release/tsox
PROJ=/Users/cqh/workspace/ai-Color-toner

$TSGO -p $PROJ/tsconfig.emit-go.json
$TSOX -p $PROJ/tsconfig.emit-rust.json
```
