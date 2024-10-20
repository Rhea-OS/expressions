import mod from '#mod/main.wasm';
import * as util from '#mod/util.js';

const wasm = new WebAssembly.Instance(new WebAssembly.Module(mod), {
    "./expression_js_bg.js": util
});

util.__wbg_set_wasm(wasm.exports);

export * from '#mod/util.js';