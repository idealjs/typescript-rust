# 打包与分发计划

## 现状概览

### 当前产物

| 产物 | 路径 | 说明 |
|------|------|------|
| Rust 二进制 `tsox` | `target/release/tsox` | CLI 编译器 + `--lsp` LSP 服务器 + `--api` JSON-RPC API |
| Rust 库 crate `tsox` | `src/lib.rs` | 可作为 Rust crate 依赖 |
| npm 包 `@typescript/native-preview` | `npm/` | 通过 `npx tsgo` 调用 Rust 二进制 |
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
- `npm/package.json` — 包定义，名称 `@typescript/native-preview`
- `npm/bin/tsgo` — shell 入口
- `npm/lib/tsgo.js` — Node.js shim，调用 Rust 二进制
- `npm/lib/getExePath.js` — 自动查找二进制路径（开发/安装环境）
- `npm/scripts/build.sh` — 构建脚本

#### 需要改进的项

1. **多平台二进制分发** — 当前只有一个通用 binary，需要按平台分发：
   ```
   npm/
   ├── bin/
   │   ├── tsgo                          # 统一入口 shim
   │   ├── tsgo-darwin-arm64             # macOS Apple Silicon
   │   ├── tsgo-darwin-x64              # macOS Intel
   │   ├── tsgo-linux-x64               # Linux x86_64
   │   ├── tsgo-linux-arm64             # Linux ARM64
   │   └── tsgo-win32-x64.exe           # Windows x86_64
   ├── lib/
   │   ├── tsgo.js
   │   └── getExePath.js
   └── package.json
   ```

2. **`getExePath.js` 改进** — 按平台选择正确的二进制：
   ```js
   const platform = `${process.platform}-${process.arch}`;
   const binName = `tsgo-${platform}`;
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
     "bin": { "tsgo": "./bin/tsgo" },
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
             artifact: tsgo-linux-x64
           - os: ubuntu-latest
             target: aarch64-unknown-linux-gnu
             artifact: tsgo-linux-arm64
           - os: macos-latest
             target: aarch64-apple-darwin
             artifact: tsgo-darwin-arm64
           - os: macos-13
             target: x86_64-apple-darwin
             artifact: tsgo-darwin-x64
           - os: windows-latest
             target: x86_64-pc-windows-msvc
             artifact: tsgo-win32-x64.exe
     steps:
       - cargo build --release --target ${{ matrix.target }}
       - 上传 artifact 到 GitHub Release
   ```

2. **发布物**：
   - `tsgo-{platform}-{arch}.tar.gz`（Linux/macOS）
   - `tsgo-{platform}-{arch}.zip`（Windows）
   - `tsgo-{version}-checksums.txt`（SHA256 校验和）

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
   keywords = ["typescript", "compiler", "lsp", "tsgo"]
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
COPY --from=builder /app/target/release/tsox /usr/local/bin/tsgo
ENTRYPOINT ["tsgo"]
```

---

## 使用场景

### 场景 A：终端用户（通过 npm）

```bash
# 安装
npm install -g @typescript/native-preview

# 使用
tsgo --version
tsgo -p tsconfig.json
tsgo src/index.ts --outDir dist

# LSP 模式（配合编辑器）
tsgo --lsp
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
require'lspconfig'.tsgo.setup{
  cmd = { "tsgo", "--lsp" },
  filetypes = { "typescript", "typescriptreact", "javascript", "javascriptreact" },
}
```

### 场景 C：Rust 项目依赖

```toml
# Cargo.toml
[dependencies]
tsox = "0.1"
```

```rust
use tsox::compiler::{Program, ProgramOptions, CompilerHostImpl};
use tsox::bundled::lib_path;

let host = Arc::new(CompilerHostImpl::new(fs, cwd, lib_path()));
let program = Program::new(ProgramOptions { config, host });
let diagnostics = program.get_semantic_diagnostics();

// 使用 LSP 功能
use tsox::ls::language_service::LanguageService;
let ls = LanguageService::new(/* ... */);
let hover = ls.provide_hover(&uri, position);
```

### 场景 D：CI/CD 流水线

```yaml
# GitHub Actions
- name: Type Check
  run: npx @typescript/native-preview -p tsconfig.json --noEmit
```

```dockerfile
# Dockerfile
FROM ghcr.io/idealjs/tsgo:latest
COPY . /app
RUN tsgo -p tsconfig.json --noEmit
```

---

## 构建与发布流程

### 步骤 1：本地构建

```bash
# Debug 构建（快速验证）
cargo build

# Release 构建（优化）
cargo build --release

# 构建 npm 包
cd npm && ./scripts/build.sh
```

### 步骤 2：跨平台交叉编译

```bash
# macOS Apple Silicon
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Linux x64 (从 macOS 交叉编译)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Windows x64
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

### 步骤 3：CI 自动构建

扩展 `.github/workflows/rust.yml`，新增 `release` job：

```yaml
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: ubuntu-latest, target: aarch64-unknown-linux-gnu }
          - { os: macos-latest,  target: aarch64-apple-darwin }
          - { os: macos-13,      target: x86_64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: tsgo-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/tsox

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            tsgo-*/tsox
```

### 步骤 4：npm 发布

```bash
# 1. 构建所有平台二进制
./npm/scripts/build-all-platforms.sh

# 2. 测试
npm pack
npm install -g ./typescript-native-preview-0.1.0.tgz
tsgo --version

# 3. 发布
npm publish
```

---

## 版本管理

| 版本 | 含义 | 示例 |
|------|------|------|
| `0.x.y` | 预览版（API 可能变化） | `0.1.0` |
| `1.0.0` | 首个稳定版 | — |
| `1.x.0` | 功能更新 | — |
| `1.0.x` | Bug 修复 | — |

版本号同步：`Cargo.toml` 版本 = `npm/package.json` 版本 = git tag 版本。

---

## 优先级排序

| 优先级 | 任务 | 前置条件 |
|--------|------|---------|
| P0 | 完善 npm 包多平台支持 | 无 |
| P0 | 编写用户使用文档 | 无 |
| P1 | CI 自动构建 + GitHub Release | GitHub Actions 配置 |
| P1 | `cargo publish` 到 crates.io | Cargo.toml 完善 |
| P2 | Docker 镜像 | CI 构建稳定 |
| P2 | VS Code 扩展集成 | LSP 稳定测试 |
| P3 | Homebrew formula | 稳定版发布后 |
