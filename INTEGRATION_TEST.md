# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：`tsconfig.app.json`（target es2023, lib ES2023+DOM, jsx react-jsx, bundler mode）
日期：2026-08-02

## 结果摘要

| 编译器 | 诊断行数 | 诊断码分布 |
|--------|---------|-----------|
| **TSGO (Go oracle)** | 3 | TS2688(1): Cannot find type 'vite/client' |
| **TSOX (Rust)** | 104 | TS2602(52), TS7026(49), TS2604(2), TS2304(1) |

Go oracle 仅报 1 个有效错误（vite/client 类型定义缺失，因 node_modules 未安装）。
Rust 产出 104 个错误，全部为 false positive。

## 差异分析

### 差异 1：JSX 全局命名空间未加载（TS2602 + TS7026）— 101 个

**现象**：每个 JSX 元素报 2 个错误：
- TS2602: JSX element implicitly has type 'any' because the global type 'JSX.Element' does not exist
- TS7026: JSX element implicitly has type 'any' because no interface 'JSX.IntrinsicElements' exists

**根因**：`jsx: "react-jsx"` 模式下，JSX 全局命名空间由 `@types/react` 的
`react/jsx-runtime` 或 `react` 包中的 `global.d.ts` 提供。Rust checker 未从
node_modules 解析 `@types/react` 的 JSX 全局声明。

**影响文件**：`src/App.tsx`、`src/main.tsx`

**修复方向**：
1. 确保 `@types/react` 被正确发现并加载（node_modules 类型解析）
2. 加载 `@types/react` 的 `global.d.ts` 中的 `declare global { namespace JSX { ... } }`
3. 或者：加载内置的 JSX 辅助类型（当 `jsx: "react-jsx"` 时不依赖显式 import）

### 差异 2：JSX 组件类型解析失败（TS2604）— 2 个

**现象**：
- `main.tsx(7,4): TS2604: JSX element type 'StrictMode' does not have any construct or call signatures`
- `main.tsx(8,6): TS2604: JSX element type 'App' does not have any construct or call signatures`

**根因**：`StrictMode` 来自 `react` 包，`App` 是本地组件。两者都因 JSX 全局命名空间
缺失而无法正确解析组件类型。这是差异 1 的连锁反应。

**修复方向**：修复差异 1 后此问题自动解决。

### 差异 3：DOM 全局 'document' 未找到（TS2304）— 1 个

**现象**：`main.tsx(6,12): TS2304: Cannot find name 'document'`

**根因**：`lib: ["ES2023", "DOM"]` 配置了 DOM 库，但 Rust 可能未正确加载
`lib.dom.d.ts` 中的全局 `document` 变量声明。

**修复方向**：
1. 检查 lib 解析是否正确包含 `lib.dom.d.ts`
2. 检查 `declare var document: Document;` 是否在全局作用域中可用

## Go oracle 的诊断（基准）

```
error TS2688: Cannot find type definition file for 'vite/client'.
  The file is in the program because:
    Entry point of type library 'vite/client' specified in compilerOptions
```

这是唯一的有效错误——node_modules 中缺少 `vite/client` 类型定义（需要 `npm install`）。

## 修复优先级

1. **高**：JSX 全局命名空间加载（解决 101/104 个 false positive）
2. **高**：DOM lib 全局变量加载（解决 document/window 等）
3. **中**：@types/react 从 node_modules 解析

## 测试命令

```sh
# Rust
/Users/cqh/workspace/typescript-rust/target/release/tsox -p /Users/cqh/workspace/ai-Color-toner/tsconfig.app.json

# Go oracle
/Users/cqh/workspace/typescript-go/built/local/tsgo -p /Users/cqh/workspace/ai-Color-toner/tsconfig.app.json
```
