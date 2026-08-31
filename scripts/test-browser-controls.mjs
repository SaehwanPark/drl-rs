import assert from "node:assert/strict";
import fs from "node:fs";

const bootstrap = fs.readFileSync("web/bootstrap.js", "utf8");
const index = fs.readFileSync("web/index.html", "utf8");
const wasm = fs.readFileSync("crates/drl-web/src/lib.rs", "utf8");
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
assert.match(wasm, /Saved session incompatible/);
assert.match(
  wasm,
  /Use Clear save to remove it, then save a new session from this build\./,
  "incompatible saves need an actionable recovery instruction",
);
assert.match(persistence, /SNAPSHOT_V3/);
assert.match(persistence, /CURRENT_FIXED_CONTENT_ID/);

console.log(
  "Browser control contract: PASS (Clear Save dialog, focus cycle, persistence recovery, and V3 codec)",
);
