import assert from "node:assert/strict";
import {
  ASSET_PACKS,
  formatAssetPackStatus,
  inspectAssetPacks,
} from "../web/asset-status.mjs";

function mockFetch(present) {
  const requests = [];
  const fetchImpl = async (path, options) => {
    requests.push({ path, options });
    return { ok: present.has(path) };
  };
  return { fetchImpl, requests };
}

const allPaths = new Set(ASSET_PACKS.map((pack) => pack.path));
const all = mockFetch(allPaths);
const allReport = await inspectAssetPacks({ fetchImpl: all.fetchImpl });
assert.ok(allReport.every((pack) => pack.available));
assert.match(formatAssetPackStatus(allReport), /graphics: ready/);
assert.equal(all.requests.length, ASSET_PACKS.length);
assert.ok(all.requests.every(({ options }) => options.method === "HEAD"));

const optional = mockFetch(new Set([ASSET_PACKS[0].path]));
const optionalReport = await inspectAssetPacks({ fetchImpl: optional.fetchImpl });
assert.equal(optionalReport.find((pack) => pack.id === "graphics").available, true);
assert.ok(optionalReport.filter((pack) => !pack.required).every((pack) => !pack.available));
assert.match(formatAssetPackStatus(optionalReport), /procedural Web Audio/);
assert.match(formatAssetPackStatus(optionalReport), /DOM\/browser text/);

const required = mockFetch(new Set());
const requiredReport = await inspectAssetPacks({ fetchImpl: required.fetchImpl });
assert.ok(requiredReport.every((pack) => !pack.available));
assert.match(formatAssetPackStatus(requiredReport), /Required graphics are missing/);

const failed = await inspectAssetPacks({
  fetchImpl: async () => {
    throw new Error("network unavailable");
  },
});
assert.ok(failed.every((pack) => pack.reason === "unreachable"));
assert.match(formatAssetPackStatus(failed), /Optional packs use/);

const noFetch = await inspectAssetPacks({ fetchImpl: null });
assert.ok(noFetch.every((pack) => pack.reason === "fetch-unavailable"));

console.log("Asset-pack diagnostics contract: PASS (present, optional, required, and failure states)");
