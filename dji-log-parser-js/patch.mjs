// Create isomorphic code from nodejs target
// https://github.com/rustwasm/wasm-pack/issues/1334

import { copyFileSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";

// --- Get lib name from Cargo.toml

const cargoTomlContent = readFileSync("./Cargo.toml", "utf8");
const cargoPackageName = /\[package\]\nname = "(.*?)"/.exec(
  cargoTomlContent
)[1];
const name = cargoPackageName.replace(/-/g, "_");

// --- Patch to convert CJS code to ESM module and inline wasm

const jsContent = readFileSync(`./pkg/${name}.js`, "utf8");
const patched = jsContent
  // use global TextDecoder TextEncoder
  .replace("require(`util`)", "globalThis")
  // attach to `imports` instead of module.exports
  .replace("= module.exports", "= imports")
  .replace(/$/, "export default imports")
  .replace(/\nclass (.*?) \{/g, "\nclass $1Class {")
  .replace(
    /\nmodule\.exports\.(.*?) = \1;/g,
    "\nexport const $1 = imports.$1 = $1Class"
  )
  .replace(/\nmodule\.exports\.(.*?)\s+/g, "\nexport const $1 = imports.$1 ")

  // inline bytes Uint8Array
  .replace(
    /\nconst path.*\nconst bytes.*\n/,
    `
var __toBinary = /* @__PURE__ */ (() => {
  var table = new Uint8Array(128);
  for (var i = 0; i < 64; i++)
    table[i < 26 ? i + 65 : i < 52 ? i + 71 : i < 62 ? i - 4 : i * 4 - 205] = i;
  return (base64) => {
    var n = base64.length, bytes = new Uint8Array((n - (base64[n - 1] == "=") - (base64[n - 2] == "=")) * 3 / 4 | 0);
    for (var i2 = 0, j = 0; i2 < n; ) {
      var c0 = table[base64.charCodeAt(i2++)], c1 = table[base64.charCodeAt(i2++)];
      var c2 = table[base64.charCodeAt(i2++)], c3 = table[base64.charCodeAt(i2++)];
      bytes[j++] = c0 << 2 | c1 >> 4;
      bytes[j++] = c1 << 4 | c2 >> 2;
      bytes[j++] = c2 << 6 | c3;
    }
    return bytes;
  };
})();

const bytes = __toBinary(${JSON.stringify(
      readFileSync(`./pkg/${name}_bg.wasm`, "base64")
    )});
`
  );
writeFileSync(`./pkg/${name}.mjs`, patched);

// --- Update package.json

const pkgContent = JSON.parse(readFileSync(`./pkg/package.json`, "utf8"));
pkgContent.files = [`${name}.mjs`, `${name}.d.ts`];
pkgContent.main = `${name}.mjs`;
pkgContent.types = `${name}.d.ts`;
pkgContent.type = "module";
pkgContent.engines = {
  node: ">=18.0.0",
};
writeFileSync(`./pkg/package.json`, JSON.stringify(pkgContent, null, 2));

// --- Copy README

copyFileSync("./README.pkg.md", "./pkg/README.md");

// --- Remove artifacts

unlinkSync(`./pkg/${name}.js`);
unlinkSync(`./pkg/${name}_bg.wasm`);
unlinkSync(`./pkg/${name}_bg.wasm.d.ts`);

console.log("✨ Isomorphic patch applied");
