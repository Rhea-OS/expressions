import mod from '#mod/main.wasm';
import * as util from '#mod/util.js';

const wasm = new WebAssembly.Instance(new WebAssembly.Module(mod), {
    "./expression_js_bg.js": util as any,
});

util.__wbg_set_wasm(wasm.exports);
util.__wbindgen_init_externref_table();

export * from '#mod/util.js';