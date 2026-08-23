import assert from "node:assert/strict";
import fs from "node:fs";
import { registerOfflineCache } from "../web/offline-cache.mjs";

const diagnostics = [];
const writeDiagnostic = (...message) => diagnostics.push(message);

const unavailable = await registerOfflineCache({}, writeDiagnostic);
assert.equal(unavailable, " Offline cache unavailable in this browser.");
assert.equal(diagnostics.at(-1)[0], "Offline cache unavailable");

let registrationCall;
const installing = await registerOfflineCache(
  {
    serviceWorker: {
      register: async (...args) => {
        registrationCall = args;
        return { active: false };
      },
    },
  },
  writeDiagnostic,
);
assert.equal(installing, " Offline cache installation started for the next reload.");
assert.deepEqual(registrationCall, ["./service-worker.js", { scope: "./" }]);

const ready = await registerOfflineCache(
  { serviceWorker: { register: async () => ({ active: true }) } },
  writeDiagnostic,
);
assert.equal(ready, " Offline cache ready for the next reload.");

const failure = await registerOfflineCache(
  {
    serviceWorker: {
      register: async () => {
        throw new Error("registration blocked");
      },
    },
  },
  writeDiagnostic,
);
assert.match(failure, /Offline cache unavailable \(Error: registration blocked\)\./);
assert.match(diagnostics.at(-1)[1], /registration blocked/);

const bootstrap = fs.readFileSync("web/bootstrap.js", "utf8");
assert.ok(
  bootstrap.indexOf("const offlineCacheReady") < bootstrap.indexOf("start.addEventListener"),
  "offline registration must begin before the start handler",
);
assert.match(bootstrap, /await offlineCacheReady/);
assert.match(bootstrap, /offlineMessage\.includes\("Offline cache unavailable"\)/);

console.log("Offline-cache bootstrap contract: PASS (capability, install, ready, failure, ordering)");
