# dji-log-parser-js

This crate is used to build the javascript bindings.

Packages are published to npm: https://www.npmjs.com/package/dji-log-parser-js

## Build

```bash
wasm-pack build --target web && node ./patch.mjs
```

## Test

After the build, checks the package against the snapshots of the Rust library:

```bash
node --test
```

## Publish

```bash
wasm-pack login
npm publish --access=public
```
