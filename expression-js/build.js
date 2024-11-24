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
    //     await esbuild.build({
    //         entryPoints: ["./lib.ts"],
    //         bundle: true,
    //         sourcemap: true,
    //         format: "esm",
    //         outdir: config.out.join("tmp").path,
    //         loader: {
    //             ".wasm": "empty"
    //         },
    //         plugins: [dtsPlugin({
    //             experimentalBundling: true,
    //             outDir: config.out.join("pkg")
    //         })]
    //     });
    //
        await fs.copyFile(config.out.join("mod/expression_js.d.ts").path, config.out.join("pkg/lib.d.ts").path);
    },
    async "build:package.json"(config) {
        await fs.writeFile(config.out.join("pkg/package.json").path, JSON.stringify({
            name: "expression",
            type: "module",
            main: "./lib.js",
            typings: "./lib.d.ts"
        }, null, 4));
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