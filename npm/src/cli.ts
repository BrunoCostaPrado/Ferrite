#!/usr/bin/env node

import { compile } from "./compiler.js"

const entry = process.argv[2]
const result = await compile(entry)

if (result.ok) {
	for (const o of result.outputs) console.log(o.path)
} else {
	for (const e of result.errors) console.error(e)
	process.exit(1)
}
