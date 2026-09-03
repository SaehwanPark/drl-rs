import assert from "node:assert/strict";
import fs from "node:fs";

const bootstrap = fs.readFileSync("web/bootstrap.js", "utf8");
const index = fs.readFileSync("web/index.html", "utf8");
// The browser shell is a module map plus focused modules; contracts are checked
// across the whole shell rather than one monolithic source file.
const wasmShell = [
  "lib.rs",
  "session.rs",
  "input.rs",
  "dom.rs",
  "gpu.rs",
  "wasm/mod.rs",
  "wasm/storage.rs",
  "wasm/renderer.rs",
  "wasm/scene.rs",
  "wasm/app.rs",
  "wasm/shell_dom.rs",
  "wasm/animation_loop.rs",
  "wasm/exports.rs",
].map((name) => [name, fs.readFileSync(`crates/drl-web/src/${name}`, "utf8")]);
function read(name) {
  const found = wasmShell.find(([candidate]) => candidate === name);
  assert.ok(found, `the browser shell must keep crates/drl-web/src/${name}`);
  return found[1];
}
// Contract strings are asserted on the module that owns them rather than on the
// concatenated shell, so losing an intended owner fails the check.
const wasmExports = read("wasm/exports.rs");
const wasmShellDom = read("wasm/shell_dom.rs");
const persistence = fs.readFileSync("crates/drl-web/src/persistence.rs", "utf8");
const clearSaveHandler = bootstrap.match(
  /clearSaveButton\.addEventListener\("click", \(\) => \{([\s\S]*?)\n\}\);/,
);

assert.ok(clearSaveHandler, "Clear Save must have a click handler");
assert.match(
  clearSaveHandler[1],
  /started\) return;[\s\S]*clearSaveDialog\.hidden = false/,
  "Clear Save must open an explicit dialog",
);
assert.match(bootstrap, /cancelClearSaveButton\.addEventListener\("click"/);
assert.match(bootstrap, /confirmClearSaveButton\.addEventListener\("click"/);
assert.match(bootstrap, /event\.key === "Tab"/);
assert.match(bootstrap, /focusables\.indexOf\(document\.activeElement\)/);
assert.match(bootstrap, /Saved session kept\./);
assert.match(bootstrap, /const message = clear_save\(\)/);
assert.match(bootstrap, /writePersistenceStatus\(message\)/);
assert.match(bootstrap, /function writePersistenceStatus/);
assert.match(bootstrap, /clearDiagnostic\("persistence"\)/);
assert.match(bootstrap, /data-diagnostic-source/);
assert.match(bootstrap, /message === "Restarted deterministic M4 session\."/);
assert.match(
  index,
  /<section id="clear-save-dialog"[^>]*role="dialog"/,
  "the page must expose the confirmation dialog",
);
assert.match(bootstrap, /loadButton\.addEventListener\("click", \(\) => \{/);
// The incompatible-save title is a producer/consumer pair: wasm/exports.rs writes the
// diagnostics title and wasm/shell_dom.rs compares against it, so both must keep it.
assert.match(wasmExports, /Saved session incompatible/);
assert.match(wasmShellDom, /Some\("Saved session incompatible"\)/);
assert.match(
  wasmExports,
  /Use Clear save to remove it, then save a new session from this build\./,
  "incompatible saves need an actionable recovery instruction",
);
assert.match(persistence, /SNAPSHOT_V3/);
assert.match(persistence, /CURRENT_FIXED_CONTENT_ID/);

console.log(
  "Browser control contract: PASS (Clear Save dialog, focus cycle, persistence recovery, and V3 codec)",
);
