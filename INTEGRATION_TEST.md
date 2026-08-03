# 集成测试差异记录

测试项目：`ai-Color-toner`（React + Vite + TypeScript）
配置：`tsconfig.emit-{rust,go}.json`（target es2023, jsx react-jsx, emit + declaration + sourceMap）
更新日期：2026-08-03

## 当前状态：JS/d.ts 字节一致，诊断一致

### 诊断

| 编译器 | 诊断数 |
|--------|--------|
| **TSGO (Go oracle)** | **0** |
| **TSOX (Rust)** | **0** |

### Emit 产物（6 文件 × 2 编译器）

| 文件 | 状态 |
|------|------|
| **App.js** | ✅ 字节一致 |
| **App.d.ts** | ✅ 字节一致 |
| **main.js** | ✅ 字节一致 |
| **main.d.ts** | ✅ 字节一致 |
| App.js.map | 结构正确，精度差距 |
| main.js.map | 结构正确，精度差距 |

### Source Map 精度

| 文件 | Go | Rust | 说明 |
|------|-----|------|------|
| App.js.map | 2,291 B (415 段) | 162 B (11 段) | JSX 行：Go 363 段 vs Rust 1 段 |
| main.js.map | 348 B (48 段) | 153 B (23 段) | |

Source map 精度差距源于架构差异：
- Go 使用 **AST printer**，遍历每个 AST 节点时记录映射
- Rust 使用 **text-slice**，在文本切片级别记录映射
- 达到字节一致需要：AST 级 JSX 变换 + AST printer 替代 text-slice

## 改善历程

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

## Emit 产物一致性改善历程

| 修复 | 差异消除 | 说明 |
|------|---------|------|
| JS 格式（空行/缩进/分号/折叠） | 部分→完全 | `fold_expression_newlines` + `reindent_and_dedup` |
| .d.ts 空行 | 消除 | `reindent_and_dedup` 应用到声明输出 |
| .d.ts 返回类型 | 消除 | JSX 返回检测 → 推断 `import("react").JSX.Element` |
| Source map 字符级追踪 | 实现 | `src_offsets` 数组 + 位置感知规范化 |

## 测试命令

```sh
TSGO=/Users/cqh/workspace/typescript-go/built/local/tsgo
TSOX=/Users/cqh/workspace/typescript-rust/target/release/tsox
PROJ=/Users/cqh/workspace/ai-Color-toner

$TSGO -p $PROJ/tsconfig.emit-go.json    # 0 errors
$TSOX -p $PROJ/tsconfig.emit-rust.json  # 0 errors

# 对比产物
diff $PROJ/dist-go/App.js    $PROJ/dist-rust/App.js     # identical
diff $PROJ/dist-go/App.d.ts  $PROJ/dist-rust/App.d.ts   # identical
diff $PROJ/dist-go/main.js   $PROJ/dist-rust/main.js    # identical
diff $PROJ/dist-go/main.d.ts $PROJ/dist-rust/main.d.ts  # identical
```

## 单元测试基线

**1,282 lib 通过**，0 failed，0 ignored
