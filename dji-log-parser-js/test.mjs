// Tests the package built in ./pkg: run `node --test` after the build.

import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { DJILog } from "./pkg/dji_log_parser_js.js";

const fixtures = "../dji-log-parser/tests/fixtures";
const snapshots = "../dji-log-parser/tests/snapshots";

test("the package has the wasm inlined and no Node API", () => {
  assert.deepEqual(readdirSync("./pkg").filter((file) => file.endsWith(".wasm")), []);
  const js = readFileSync("./pkg/dji_log_parser_js.js", "utf8");
  for (const api of ["node:", "require(", "__dirname", "import.meta"]) {
    assert.ok(!js.includes(api), `the package uses ${api}`);
  }
});

test("errors are Error objects, not strings or panics", () => {
  // A panic of the wasm is a RuntimeError, which is an Error too
  const isError = (error) => error instanceof Error && !(error instanceof WebAssembly.RuntimeError);
  assert.throws(() => new DJILog(new Uint8Array([1, 2, 3])), isError);

  const parser = new DJILog(readFileSync(`${fixtures}/v14_mini4pro.txt`));
  assert.throws(() => parser.records([[{ featurePoint: 1 }]]), isError);
  assert.throws(() => parser.frames("not keychains"), isError);
});

// Each snapshot of the Rust library summarizes the fixture of the same name
for (const file of readdirSync(snapshots).filter((file) => file.endsWith(".snap"))) {
  const name = file.replace(/^snapshots__/, "").replace(/\.snap$/, "");

  test(`${name} is parsed like the Rust library`, () => {
    const snapshot = JSON.parse(readFileSync(`${snapshots}/${file}`, "utf8").split(/^---$/m)[2]);
    const keychainsPath = `${fixtures}/${name}.keychains.json`;
    const keychains = existsSync(keychainsPath)
      ? JSON.parse(readFileSync(keychainsPath, "utf8"))
      : undefined;

    const parser = new DJILog(readFileSync(`${fixtures}/${name}.txt`));
    const recordCounts = {};
    for (const record of parser.records(keychains)) {
      recordCounts[record.type] = (recordCounts[record.type] ?? 0) + 1;
    }
    const frames = parser.frames(keychains);

    assertSame(parser.version, snapshot.version, "version");
    assertSame(parser.details, snapshot.details, "details");
    assertSame(recordCounts, snapshot.record_counts, "record_counts");
    assertSame(frames.length, snapshot.frame_count, "frame_count");
    for (const { index, frame } of snapshot.sample_frames) {
      assertSame(frames[index], frame, `frames[${index}]`);
    }
  });
}

// Compares a value of the package with the JSON of the snapshot, where the `f32` are rounded
// and a missing value is `null`.
function assertSame(actual, expected, path) {
  if (expected === null) {
    assert.equal(actual ?? null, null, path);
  } else if (typeof expected === "number" && typeof actual === "number") {
    assert.ok(actual === expected || actual === Math.fround(expected), `${path}: ${actual} != ${expected}`);
  } else if (typeof expected === "object") {
    assert.equal(typeof actual, "object", path);
    const keys = new Set([...Object.keys(actual ?? {}), ...Object.keys(expected)]);
    for (const key of keys) {
      assertSame(actual?.[key], expected[key] ?? null, `${path}.${key}`);
    }
  } else {
    assert.equal(actual, expected, path);
  }
}
