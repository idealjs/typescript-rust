# 构建矩阵与发布流程

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
FROM ghcr.io/idealjs/tsox:latest
COPY . /app
RUN tsox -p tsconfig.json --noEmit
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
          name: tsox-${{ matrix.target }}
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
            tsox-*/tsox
```

### 步骤 4：npm 发布

```bash
# 1. 构建所有平台二进制
./npm/scripts/build-all-platforms.sh

# 2. 测试
npm pack
npm install -g ./typescript-native-preview-0.1.0.tgz
tsox --version

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
