# Ferrite

Node.js API and CLI for the [Ferrite](../) TypeScript compiler.

## Install

```bash
npm install ferrite
```

This ships the Rust compiler binary for your platform. No separate Rust toolchain needed.

## CLI

```bash
# Compile a file
ferrite src/index.ts

# Use ferrite.config.ts (auto-detected)
ferrite build
```

## API

```ts
import { compile, ferrite, defineConfig } from "ferrite"
```

### `compile(entry?, config?)`

Compile a TypeScript file. Loads `ferrite.config.ts` from cwd if no config passed.

```ts
const result = await compile("src/index.ts")
// result.ok: boolean
// result.outputs: [{ path: string, content: string }]
// result.errors: string[]
```

### `ferrite(entry, config?)`

Alias for `compile`. Same signature, same return.

```ts
const result = await ferrite("src/index.ts", {
  target: "es2020",
  strict: true,
  dts: false,
})
```

### `defineConfig(config)`

Type-safe config helper for `ferrite.config.ts`.

```ts
import { defineConfig } from "ferrite"

export default defineConfig({
  entry: ["src/index.ts"],
  outDir: "dist",
  target: "es2020",
})
```

### `loadConfig()`

Load `ferrite.config.ts` / `.js` / `.mjs` from cwd. Returns `CompilerConfig | null`.

## Config

`ferrite.config.ts` (or `.js` / `.json`):

```ts
export default {
  entry: ["src/index.ts"],
  outDir: "dist",
  target: "es2020",
  strict: true,
  dts: true,
  sourcemap: true,
  // tsconfig overrides
  module: "nodenext",
  jsx: "react-jsx",
  paths: { "@/*": ["src/*"] },
  baseUrl: ".",
}
```

All fields optional. See `CompilerConfig` in `types.ts` for the full list.

## Types

```ts
import type {
  CompilerConfig,
  CompileOutput,
  CompileResult,
  Platform,
  Format,
  Target,
  Entry,
} from "ferrite"
```

## License

ISC
