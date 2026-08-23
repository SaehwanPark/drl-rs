import assert from "node:assert/strict";
import fs from "node:fs";

const bootstrap = fs.readFileSync("web/bootstrap.js", "utf8");
const index = fs.readFileSync("web/index.html", "utf8");
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
assert.match(bootstrap, /closeClearSaveDialog\(clear_save\(\)\)/);
assert.match(
  index,
  /<section id="clear-save-dialog"[^>]*role="dialog"/,
  "the page must expose the confirmation dialog",
);

console.log("Browser control contract: PASS (Clear Save dialog, focus cycle, and cancel status)");
