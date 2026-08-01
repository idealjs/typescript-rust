# @typescript/native-preview (Rust)

Native TypeScript compiler preview - Rust port.

This package contains the Rust implementation of the TypeScript compiler (`tsgo`).

## Installation

```bash
npm install @typescript/native-preview
```

## Usage

### CLI

```bash
npx tsgo --version
npx tsgo file.ts
npx tsgo -p tsconfig.json
```

### Programmatic API

```typescript
import { createSyncApi } from "@typescript/native-preview/unstable/sync";

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

# The binary will be at target/release/tsox
# Copy it to npm/bin/tsgo or use the bin/tsgo shim
```
