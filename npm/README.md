# @idealjs/tsox

Native TypeScript compiler — Rust port of [typescript-go](https://github.com/microsoft/typescript-go).

## Installation

```bash
npm install @idealjs/tsox
```

## Usage

### CLI

```bash
# Compile a project
npx tsox -p tsconfig.json

# Compile a single file
npx tsox file.ts --outDir dist

# Type-check only (no emit)
npx tsox -p tsconfig.json --noEmit

# Watch mode
npx tsox -p tsconfig.json --watch

# LSP server mode (for editor integration)
npx tsox --lsp

# API server mode (JSON-RPC over stdio)
npx tsox --api
```

### Programmatic API

```typescript
import { createSyncApi } from "@idealjs/tsox/unstable/sync";

const api = createSyncApi();
const program = api.createProgram({
  rootNames: ["file.ts"],
  options: {}
});
const diagnostics = program.getSemanticDiagnostics();
```

## Building from source

```bash
# Build the Rust binary
cargo build --release

# Build the npm package
cd npm && ./scripts/build.sh
```

The binary will be at `target/release/tsox`.
