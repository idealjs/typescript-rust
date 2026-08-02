# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：自建 `tsconfig.emit-{rust,go}.json`（target es2023, jsx react-jsx, emit + declaration）
日期：2026-08-02

## 测试 A：诊断对比（noEmit 模式）

| 编译器 | 诊断行数 | 诊断码分布 |
|--------|---------|-----------|
| **TSGO (Go oracle)** | 0 | （零错误，yarn install 后） |
| **TSOX (Rust)** | 127 | TS1003(20), TS1005(20), TS2304(87) |

### 差异 A1：JSX 全局命名空间未加载（TS2602 + TS7026）— 101 个

**根因**：`jsx: "react-jsx"` 模式下，JSX 全局命名空间由 `@types/react` 的
`global.d.ts` 提供。Rust checker 未从 node_modules 解析 `@types/react`。

### 差异 A2：DOM 全局变量未找到（TS2304）— 1 个

**根因**：`lib.dom.d.ts` 中的 `declare var document` 全局声明未加载。

### 差异 A3：node_modules .js 文件解析错误（TS1003/1005）— 40 个

**根因**：React 的 `index.js` 使用 CommonJS `module.exports = require(...)`，
Rust parser 将其当作 TypeScript 解析而非 JavaScript，产生语法错误。
Go oracle 在 `allowJs: false`（默认）下不解析 node_modules 中的 .js 文件。

## 测试 B：Emit 产物对比

### 产物文件集

| Go oracle (6 files) | Rust (9 files) | 说明 |
|---------------------|----------------|------|
| App.js ✅ | App.js ✅ | |
| App.d.ts ✅ | App.d.ts ✅ | |
| App.js.map ✅ | App.js.map ✅ | |
| main.js ✅ | main.js ✅ | |
| main.d.ts ✅ | main.d.ts ✅ | |
| main.js.map ✅ | main.js.map ✅ | |
| — | **index.js/d.ts/map** | ❌ Rust 多 emit 了 node_modules/react/index.js |
| — | **client.js/d.ts/map** | ❌ Rust 多 emit 了 node_modules/react-dom/client.js |

### 差异 B1：JSX transform 未执行（最关键）

**Go oracle**：将 JSX 转换为 React 运行时调用：
```js
// Go 产出
import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
return _jsxs("section", { id: "center", children: [...] });
```

**Rust**：原样输出 JSX 语法（未 transform）：
```js
// Rust 产出
return (
  <>
    <section id="center">
      <div className="hero">...
```

**根因**：Rust emitter 使用 text-slice 模式（直接从源码切片），不执行 JSX→jsx-runtime
transform。需要实现 JSX transformer（`react-jsx` 模式的 automatic runtime）。

### 差异 B2：Import 路径重写未执行

**Go oracle**：`.tsx` 扩展名重写为 `.js`：
```js
// Go: import App from './App.js'
```

**Rust**：保留原样：
```js
// Rust: import App from './App.tsx'
```

**根因**：`rewriteRelativeImportExtensions: true` 未实现。

### 差异 B3：Declaration emit 差异

**Go oracle**：只 emit 类型声明，过滤掉值 import：
```ts
// Go d.ts
import './App.css';
declare function App(): import("react").JSX.Element;
export default App;
```

**Rust**：原样复制源码（包含所有 import 和函数体）：
```ts
// Rust d.ts
import { useState } from 'react'
import reactLogo from './assets/react.svg'
...
declare function App();
export default App
```

**根因**：Rust declaration emitter 未实现类型提取/值过滤。

### 差异 B4：非空断言未擦除

**Rust** 产出中保留了 `!` 操作符：
```js
createRoot(document.getElementById('root')!).render(
```

**Go oracle** 正确擦除：
```js
createRoot(document.getElementById('root')).render(
```

### 差异 B5：分号格式差异

Rust 产出不加行尾分号，Go oracle 自动添加。这是 text-slice vs AST emit 的
本质差异。

### 差异 B6：node_modules 文件被 emit

Rust 错误地将 node_modules 中的 `react/index.js` 和 `react-dom/client.js`
也 emit 到 outDir。Go oracle 只 emit 项目源码文件。

**根因**：Rust 编译器未正确区分项目文件和外部依赖文件。

## 修复优先级

### Emit 层（影响产物正确性）

1. **P0**：JSX transform（react-jsx automatic runtime）
2. **P0**：Import 路径重写（`.tsx` → `.js`）
3. **P0**：非空断言 `!` 擦除
4. **P1**：Declaration emit 类型提取
5. **P1**：node_modules 文件排除 emit

### Checker 层（影响诊断准确性）

6. **P1**：JSX 全局命名空间加载（@types/react）
7. **P1**：DOM lib 全局变量加载
8. **P2**：node_modules .js 文件不解析（allowJs 默认 false）

## 测试命令

```sh
# 诊断对比
TSGO=/Users/cqh/workspace/typescript-go/built/local/tsgo
TSOX=/Users/cqh/workspace/typescript-rust/target/release/tsox
PROJ=/Users/cqh/workspace/ai-Color-toner

$TSGO -p $PROJ/tsconfig.emit-go.json
$TSOX -p $PROJ/tsconfig.emit-rust.json

# 产物对比
diff -r $PROJ/dist-go $PROJ/dist-rust
```

## 结论

诊断层面：Rust 产出 127 个 false positive，Go 零错误。
Emit 层面：JSX transform 是最大差距——Rust 不转换 JSX，Go 正确产出 jsx-runtime 调用。
这是当前阻碍 Rust 编译器在真实 React 项目中可用的核心问题。
