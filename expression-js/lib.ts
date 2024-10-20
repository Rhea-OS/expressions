import mod from '#mod/main.wasm';

export default async function init(): Promise<typeof import("#mod/def.d.ts")> {
    const util = await import("#mod/util.js");

    return await WebAssembly.compile(mod)
        .then(mod => new WebAssembly.Instance(mod, {
            "./expression_js_bg.js": util
        }))
        .then(mod => {
            util.__wbg_set_wasm(mod.exports);
            return util;
        });
}
