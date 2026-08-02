# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：`tsconfig.emit-{rust,go}.json`（target es2023, jsx react-jsx, emit + declaration）
更新日期：2026-08-02

## 里程碑：0 诊断对齐

| 编译器 | 诊断数 | 说明 |
|--------|--------|------|
| **TSGO (Go oracle)** | **0** | 零错误 |
| **TSOX (Rust)** | **0** | 零错误 ✅ |

**改善历程**：127 → 28 → 8 → 2 → **0**

## Emit 产物

### 产物文件集

| Go (6 files) | Rust (6 files) | 说明 |
|--------------|----------------|------|
| App.js | App.js | JSX transform 正确 |
| App.d.ts | App.d.ts | 值 import 过滤 ✅ |
| App.js.map | App.js.map | |
| main.js | main.js | import 重写正确 |
| main.d.ts | main.d.ts | 完全匹配 ✅ |
| main.js.map | main.js.map | |

### 剩余 cosmetic 差异

- `App.d.ts`：Go 生成返回类型 `import("react").JSX.Element`，Rust 暂为空（需 checker node-builder）
- `App.js`：格式差异（分号/缩进），text-slice vs AST printer 固有差异

## 修复历程

| 修复 | 诊断减少 | 说明 |
|------|---------|------|
| F1-F6 + G1-G3 | 127→2 | JSX transform + import 重写 + 类型擦除 + DOM 全局 |
| @types/react resolver | 2→137 | 移除过早 try_file，允许 @types fallback |
| +54 内置全局类型 | 137→28 | Promise/Partial/Readonly/Pick 等 |
| type predicate parser | 28→22 | `object is T` 语法支持 |
| TS2302 static flag leak | 22→15 | in_static_member_type 跨声明体重置 |
| TS2300 UMD namespace | 15→14 | export as namespace 不触发重复检查 |
| TS2304 overload scope | 14→8 | 泛型参数作用域推入 |
| 方法泛型参数 parser | 8→2 | `<K extends keyof S>` 方法签名 |
| TS2604 any 抑制 | 2→0 | 未解析类型不报 false positive |

## 测试命令

```sh
TSGO=/Users/cqh/workspace/typescript-go/built/local/tsgo
TSOX=/Users/cqh/workspace/typescript-rust/target/release/tsox
PROJ=/Users/cqh/workspace/ai-Color-toner

$TSGO -p $PROJ/tsconfig.emit-go.json  # 0 errors
$TSOX -p $PROJ/tsconfig.emit-rust.json  # 0 errors
```
