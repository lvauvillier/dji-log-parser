# dji-log-parser-js

This crate is used to build the javascript bindings.

Packages are published to npm: https://www.npmjs.com/package/dji-log-parser-js

## Build

```bash
wasm-pack build --target nodejs  && node ./patch.mjs
```

## Publish

```bash
wasm-pack login
npm publish --access=public
```
