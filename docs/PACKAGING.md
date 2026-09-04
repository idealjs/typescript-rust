# 打包与分发计划

## 现状概览

### 当前产物

| 产物 | 路径 | 说明 |
|------|------|------|
| Rust 二进制 `tsox` | `target/release/tsox` | CLI 编译器 + `--lsp` LSP 服务器 + `--api` JSON-RPC API |
| Rust 库 crate `tsox` | `src/lib.rs` | 可作为 Rust crate 依赖 |
| npm 包 `@idealjs/tsox` | `npm/` | 通过 `npx tsox` 调用 Rust 二进制 |
| 内置 lib.d.ts | `bundled/libs/` | 120+ 个 .d.ts 文件，编译时 `include_str!` 嵌入 |
| 本地化消息包 | `diagnostics/loc/*.json.gz` | 13 种语言，编译时嵌入 |

### 三种运行模式

```
tsox [file.ts | -p tsconfig.json]     # 编译器模式（默认）
tsox --lsp                             # LSP 服务器模式（stdio JSON-RPC）
tsox --api                             # API 服务器模式（stdio JSON-RPC）
```

---

## 分发渠道

### 渠道 1：npm 包（主要渠道）

**目标**：让 JS/TS 开发者通过 `npm install` 一键获得可执行的编译器。

#### 当前状态

已有完整的 npm 包结构：
- `npm/package.json` — 包定义，名称 `@idealjs/tsox`
- `npm/bin/tsox` — shell 入口
- `npm/lib/tsgo.js` — Node.js shim，调用 Rust 二进制（`npx tsox` 命令名）
- `npm/lib/getExePath.js` — 自动查找二进制路径（开发/安装环境）
- `npm/scripts/build.sh` — 构建脚本

#### 需要改进的项

1. **多平台二进制分发** — 当前只有一个通用 binary，需要按平台分发：
   ```
   npm/
   ├── bin/
   │   ├── tsox                          # 统一入口 shim
   │   ├── tsox-darwin-arm64             # macOS Apple Silicon
   │   ├── tsox-darwin-x64              # macOS Intel
   │   ├── tsox-linux-x64               # Linux x86_64
   │   ├── tsox-linux-arm64             # Linux ARM64
   │   └── tsox-win32-x64.exe           # Windows x86_64
   ├── lib/
   │   ├── tsgo.js
   │   └── getExePath.js
   └── package.json
   ```

2. **`getExePath.js` 改进** — 按平台选择正确的二进制：
   ```js
   const platform = `${process.platform}-${process.arch}`;
   const binName = `tsox-${platform}`;
   ```

3. **可选：optionalDependencies 方案** — 拆分为平台子包：
   ```json
   {
     "optionalDependencies": {
       "@typescript/native-preview-darwin-arm64": "0.1.0",
       "@typescript/native-preview-linux-x64": "0.1.0"
     }
   }
   ```
   类似 `@esbuild/linux-x64`、`@swc/core-linux-x64` 的模式。

4. **package.json 更新**：
   ```json
   {
     "name": "@typescript/native-preview",
     "version": "0.1.0",
     "bin": { "tsox": "./bin/tsox" },
     "engines": { "node": ">=18.0.0" },
     "os": ["darwin", "linux", "win32"],
     "cpu": ["x64", "arm64"],
     "files": ["bin", "lib", "README.md"]
   }
   ```

### 渠道 2：GitHub Releases

**目标**：提供独立二进制下载，不需要 Node.js。

#### 计划

1. **CI 构建**（扩展 `.github/workflows/rust.yml`）：
   ```yaml
   # 新增 release job
   build-release:
     strategy:
       matrix:
         include:
           - os: ubuntu-latest
             target: x86_64-unknown-linux-gnu
             artifact: tsox-linux-x64
           - os: ubuntu-latest
             target: aarch64-unknown-linux-gnu
             artifact: tsox-linux-arm64
           - os: macos-latest
             target: aarch64-apple-darwin
             artifact: tsox-darwin-arm64
           - os: macos-13
             target: x86_64-apple-darwin
             artifact: tsox-darwin-x64
           - os: windows-latest
             target: x86_64-pc-windows-msvc
             artifact: tsox-win32-x64.exe
     steps:
       - cargo build --release --target ${{ matrix.target }}
       - 上传 artifact 到 GitHub Release
   ```

2. **发布物**：
   - `tsox-{platform}-{arch}.tar.gz`（Linux/macOS）
   - `tsox-{platform}-{arch}.zip`（Windows）
   - `tsox-{version}-checksums.txt`（SHA256 校验和）

3. **触发条件**：git tag `v0.1.0` 触发自动发布

### 渠道 3：Rust crate（crates.io）

**目标**：让其他 Rust 项目可以直接依赖 `tsox` 库。

#### 计划

1. **发布前检查**：
   - 确认 `Cargo.toml` 的 `description`、`repository`、`homepage`、`keywords` 字段完整
   - 确认 `LICENSE` 文件存在（已有 Apache-2.0）
   - `cargo publish --dry-run` 验证打包内容

2. **更新 Cargo.toml**：
   ```toml
   [package]
   name = "tsox"
   version = "0.1.0"
   edition = "2024"
   description = "TypeScript compiler ported from Go to Rust"
   license = "Apache-2.0"
   repository = "https://github.com/idealjs/typescript-rust"
   homepage = "https://github.com/idealjs/typescript-rust"
   keywords = ["typescript", "compiler", "lsp", "tsox"]
   categories = ["compilers", "development-tools"]

   [features]
   default = ["compiler"]
   compiler = []
   lsp = []
   api = []
   ```

3. **发布命令**：
   ```sh
   cargo publish
   ```

### 渠道 4：Docker 镜像

**目标**：提供容器化使用方式（CI/CD 场景）。

#### 计划

```dockerfile
FROM rust:1.96-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tsox /usr/local/bin/tsox
ENTRYPOINT ["tsox"]
```

---

## 使用场景

### 场景 A：终端用户（通过 npm）

```bash
# 安装
npm install -g @typescript/native-preview

# 使用
tsox --version
tsox -p tsconfig.json
tsox src/index.ts --outDir dist

# LSP 模式（配合编辑器）
tsox --lsp
```

### 场景 B：编辑器集成（LSP）

在 VS Code / Neovim / Helix 中配置：

```json
// VS Code settings.json
{
  "typescript.tsdk": "node_modules/@typescript/native-preview",
  "tsserver.useNativePreview": true
}
```

或直接指向二进制：

```lua
-- Neovim lspconfig
require'lspconfig'.tsox.setup{
  cmd = { "tsox", "--lsp" },
  filetypes = { "typescript", "typescriptreact", "javascript", "javascriptreact" },
}
```

### 场景 C：Rust 项目依赖

```toml
