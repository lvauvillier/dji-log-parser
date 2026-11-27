// Inlines the wasm in the package built with `--target web` and instantiates it on import, so that
// the same package works in Node, in browsers and in bundlers without any loader.

import { copyFileSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";

const name = "dji_log_parser_js";
const dir = process.argv[2] ?? "./pkg";
const js = readFileSync(`${dir}/${name}.js`, "utf8");
const wasm = readFileSync(`${dir}/${name}_bg.wasm`, "base64");

// Bundlers would look for the wasm file at its URL
const wasmUrl = `new URL('${name}_bg.wasm', import.meta.url)`;
const exports = "export { initSync, __wbg_init as default };";
if (!js.includes(wasmUrl) || !js.includes(exports)) {
  throw new Error("Unexpected wasm-bindgen output, the patch needs an update");
}
writeFileSync(
  `${dir}/${name}.js`,
  js.replace(wasmUrl, "wasmBytes").replace(
    exports,
    `const wasmBytes = Uint8Array.from(atob(${JSON.stringify(wasm)}), (c) => c.charCodeAt(0));
initSync({ module: wasmBytes });`
  )
);

// The init functions aren't exported anymore
const types = readFileSync(`${dir}/${name}.d.ts`, "utf8");
const init = types.indexOf("export type InitInput");
if (init < 0) {
  throw new Error("Unexpected wasm-bindgen types, the patch needs an update");
}
writeFileSync(`${dir}/${name}.d.ts`, types.slice(0, init).trimEnd() + "\n");

const pkg = JSON.parse(readFileSync(`${dir}/package.json`, "utf8"));
pkg.files = [`${name}.js`, `${name}.d.ts`];
pkg.engines = { node: ">=18.0.0" };
delete pkg.sideEffects;
writeFileSync(`${dir}/package.json`, JSON.stringify(pkg, null, 2));

copyFileSync("./README.pkg.md", `${dir}/README.md`);
unlinkSync(`${dir}/${name}_bg.wasm`);
unlinkSync(`${dir}/${name}_bg.wasm.d.ts`);

console.log("Wasm inlined");
