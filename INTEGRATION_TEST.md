# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：`tsconfig.emit-{rust,go}.json`（target es2023, jsx react-jsx, emit + declaration）
更新日期：2026-08-02（F/G 修复后）

## 测试 A：诊断对比

| 编译器 | 诊断数 | 说明 |
|--------|--------|------|
| **TSGO (Go oracle)** | 0 | 零错误 |
| **TSOX (Rust)** | 2 | TS2604: JSX 组件类型（App/StrictMode） |

**改善**：从 127 个 → 2 个。剩余 2 个是 checker 无法正确解析 React 组件类型
（需要加载 `@types/react` 的完整类型声明）。

## 测试 B：Emit 产物对比

### 产物文件集

| Go (6 files) | Rust (6 files) | 说明 |
|--------------|----------------|------|
| App.js ✅ | App.js ✅ | JSX transform 正确 |
| App.d.ts ✅ | App.d.ts ✅ | 差异见 B3 |
| App.js.map ✅ | App.js.map ✅ | |
| main.js ✅ | main.js ✅ | import 重写正确 |
| main.d.ts ✅ | main.d.ts ✅ | 差异见 B3 |
| main.js.map ✅ | main.js.map ✅ | |

**已修复**：node_modules 文件不再被 emit（从 9 → 6 文件）。

### 差异 B1：JSX Transform ✅ 已修复

Go 和 Rust 现在都输出 `_jsx()`/`_jsxs()` 调用，调用字符串完全一致。

### 差异 B2：Import 路径重写 ✅ 已修复

`import App from './App.tsx'` 正确重写为 `import App from './App.js'`。

### 差异 B3：Declaration Emit — 部分对齐

**Go**：从 checker 符号生成类型声明，过滤值 import：
```ts
import './App.css';
declare function App(): import("react").JSX.Element;
export default App;
```

**Rust**：基于源码切片，保留值 import 但擦除函数体：
```ts
import { useState } from 'react';
import './App.css';
declare function App();
export default App;
```

**根因**：需要实现完整 declaration transformer（Go ~3000 行）。

### 差异 B4：格式差异（cosmetic）

Go printer 将多行表达式折叠为单行（`return (_jsxs(...))`）。
Rust 保留源码缩进和换行。功能等价但格式不同。

### 差异 B5：非空断言 `!` ✅ 已修复

`document.getElementById('root')!` 正确擦除为 `document.getElementById('root')`。

## 修复状态总览

| 差异 | 状态 | 说明 |
|------|------|------|
| B1: JSX Transform | ✅ | `_jsx`/`_jsxs`/`_Fragment` 完全对齐 |
| B2: Import 重写 | ✅ | `.tsx`→`.js`，仅限 import/export |
| B3: Declaration emit | ⏳ | 需完整 declaration transformer |
| B4: 格式（分号/缩进） | ⏳ | text-slice vs AST printer 固有差异 |
| B5: 非空断言 | ✅ | 正确擦除 |
| B6: node_modules emit | ✅ | 排除外部库 |
| A1: JSX 诊断 | ✅ | 合成全局命名空间 |
| A2: DOM document | ✅ | fallback 全局变量 |
| A3: .js 不解析 | ✅ | allowJs 默认跳过 |
| TS2604 组件类型 | ⏳ | 需加载 @types/react 完整类型 |

## 测试命令

```sh
TSGO=/Users/cqh/workspace/typescript-go/built/local/tsgo
TSOX=/Users/cqh/workspace/typescript-rust/target/release/tsox
PROJ=/Users/cqh/workspace/ai-Color-toner

$TSGO -p $PROJ/tsconfig.emit-go.json
$TSOX -p $PROJ/tsconfig.emit-rust.json
diff -r $PROJ/dist-go $PROJ/dist-rust
```
