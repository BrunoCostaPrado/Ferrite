import {
	type CompileOutput,
	type CompileResult,
	compile,
	loadConfig,
} from "./compiler"
import type { CompilerConfig } from "./types"

export type { CompileOutput, CompileResult, CompilerConfig }
export { compile, loadConfig }

/** Define a compiler configuration with type safety. */
export function defineConfig(config: CompilerConfig): CompilerConfig {
	return config
}

/** Compile a TypeScript file. Loads ferrite.config.ts if no config passed. */
export async function ferrite(
	entry: string,
	config?: CompilerConfig,
): Promise<CompileResult> {
	return compile(entry, config)
}
