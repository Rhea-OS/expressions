import fs from "node:fs/promises";
import cp from "node:child_process";
import {esbuild} from 'builder';

export default {
	async "build:main.wasm"(config) {
		const out = config.out.join("mod");
		const wasmPack = cp.spawn('wasm-pack', [
			'build',
			'--out-dir',
			out.path,
		], {
			stdio: "inherit"
		});
		await new Promise((ok, err) => wasmPack.once("exit", code => code === 0 ? ok() : err(code)));
	},
	async "build:main.js"(config) {
		await esbuild.build({
			entryPoints: ["lib.ts"],
			bundle: true,
			sourcemap: true,
			format: 'esm',
			loader: {
				".wasm": "binary"
			},
			outdir: config.out.join("pkg").path
		});
	},
	async "build:lib.d.ts"(config) {
		const lib = await fs.readFile('./lib.d.ts', {encoding: 'utf8'});
		const mod = await fs.readFile(config.out.join("mod/expression_js.d.ts").path, {encoding: 'utf8'});

		await fs.writeFile(config.out.join("pkg/expr.d.ts").path, mod);

		await fs.writeFile(config.out.join("pkg/lib.d.ts").path, lib
			.replaceAll(lib.match(/( {2,})/)[0], '\t')
			.replaceAll(/\n{2,}/g, '\n'));
	},
	async "build:package.json"(config) {
		await fs.writeFile(config.out.join("pkg/package.json").path, JSON.stringify({
			name: "expression",
			type: "module",
			main: "./lib.js",
			typings: "./lib.d.ts"
		}, null, 4));
	},
	async "build:readme.md"(config) {
		await fs.writeFile(config.out.join('pkg/README.md'), await fs.readFile('./dist.md', {encoding: 'utf8'}));
	}
}