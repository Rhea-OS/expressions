import fss from "node:fs";
import fs from "node:fs/promises";
import cp from "node:child_process";
import {chalk, esbuild, is_source, log, Path, util} from 'builder';

export default {
    async "build:main.wasm"(config) {
        if (!await util.has_changed({
            glob: path => is_source(path),
            dependents: [config.out.join("build/mod/expression_js_bg.wasm")]
        }))
            return log.verbose("Skipping Rebuild");

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
        if (!await util.has_changed({
            glob: path => is_source(path),
            dependents: [config.out.join("main.js")]
        }))
            return log.verbose("Skipping Rebuild");

        await esbuild.build({
            entryPoints: ["lib.ts"],
            bundle: true,
            sourcemap: true,
            platform: 'node',
            format: 'esm',
            loader: {
                ".ttf": "copy",
                ".wasm": "binary"
            },
            external: ['electron', 'obsidian'],
            outdir: config.out.path
        });
    },
    async "test:lib.js"(config) {
        const file = config.out.join("lib.js");
        /**
         * @type {typeof import('./build/mod/expression_js.d.ts')}
         */
        const mod = await import(file.path).then(mod => mod.default());

        const cx = new mod.Context(new mod.DataSource({
            listRows: () => [],
            listColumns: () => [],
            countRows: () => 0,
            getRow: row => void 0
        }));

        cx.pushGlobal("x", "Hello")

        console.log(cx.evaluate(`x+x`));
    }
}