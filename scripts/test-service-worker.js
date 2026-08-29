"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");

const origin = "https://game.test";
const scope = `${origin}/app/`;
const cacheVersion = "v1-contract";
const template = fs.readFileSync("web/service-worker.js", "utf8");
const source = template
  .replace('/* __CACHE_VERSION__ */ "v1"', JSON.stringify(cacheVersion))
  .replace(
    "/* __PRECACHE_URLS__ */ []",
    JSON.stringify(["./index.html", "./service-worker.js"]),
  );

const listeners = new Map();
const cachesByName = new Map();
const fetchResponses = new Map();
const fetchCalls = [];

function keyFor(request) {
  const url = typeof request === "string" ? request : request.url;
  return new URL(url, scope).toString();
}

class MockResponse {
  constructor(body, { ok = true, status = 200 } = {}) {
    this.body = body;
    this.ok = ok;
    this.status = status;
  }

  clone() {
    return new MockResponse(this.body, { ok: this.ok, status: this.status });
  }

  static error() {
    return new MockResponse("", { ok: false, status: 0 });
  }
}

class MockCache {
  constructor(name) {
    this.name = name;
    this.entries = new Map();
  }

  async addAll(urls) {
    for (const url of urls) {
      this.entries.set(keyFor(url), new MockResponse(`precache:${url}`));
    }
  }

  async put(request, response) {
    this.entries.set(keyFor(request), response.clone());
  }

  async match(request) {
    return this.entries.get(keyFor(request))?.clone();
  }
}

const cacheStorage = {
  async open(name) {
    let cache = cachesByName.get(name);
    if (!cache) {
      cache = new MockCache(name);
      cachesByName.set(name, cache);
    }
    return cache;
  },

  async keys() {
    return [...cachesByName.keys()];
  },

  async delete(name) {
    return cachesByName.delete(name);
  },

  async match(request) {
    for (const cache of cachesByName.values()) {
      const response = await cache.match(request);
      if (response) return response;
    }
    return undefined;
  },
};

async function mockFetch(request) {
  const url = keyFor(request);
  fetchCalls.push(url);
  const response = fetchResponses.get(url);
  if (response instanceof Error) throw response;
  if (!response) throw new Error(`unexpected fetch: ${url}`);
  return response.clone();
}

const worker = {
  location: { origin },
  registration: { scope },
  addEventListener(type, listener) {
    listeners.set(type, listener);
  },
};

vm.runInNewContext(source, {
  URL,
  Response: MockResponse,
  caches: cacheStorage,
  console,
  fetch: mockFetch,
  self: worker,
});

async function dispatch(type, event) {
  let waitUntil = Promise.resolve();
  let responsePromise;
  const dispatchEvent = {
    ...event,
    waitUntil(promise) {
      waitUntil = Promise.resolve(promise);
    },
    respondWith(promise) {
      responsePromise = Promise.resolve(promise);
    },
  };
  listeners.get(type)(dispatchEvent);
  await waitUntil;
  return responsePromise ? responsePromise : undefined;
}

async function main() {
  await dispatch("install", {});
  const currentName = `drl-rs-m10-${cacheVersion}`;
  const current = cachesByName.get(currentName);
  assert.ok(current, "install opens the current cache");
  assert.equal(
    (await current.match(`${scope}index.html`)).body,
    "precache:./index.html",
  );

  cachesByName.set("drl-rs-m10-old", new MockCache("drl-rs-m10-old"));
  cachesByName.set("unrelated-cache", new MockCache("unrelated-cache"));
  await dispatch("activate", {});
  assert.equal(cachesByName.has("drl-rs-m10-old"), false);
  assert.equal(cachesByName.has("unrelated-cache"), true);
  assert.equal(cachesByName.has(currentName), true);

  const staleShellUrl = `${scope}stale.html`;
  const staleAssetUrl = `${scope}stale.js`;
  const unrelated = cachesByName.get("unrelated-cache");
  unrelated.entries.set(keyFor(`${scope}index.html`), new MockResponse("stale shell"));
  unrelated.entries.set(keyFor(staleAssetUrl), new MockResponse("stale asset"));
  current.entries.delete(keyFor(staleShellUrl));
  current.entries.delete(keyFor(staleAssetUrl));

  const navigationUrl = `${origin}/app/level`;
  fetchResponses.set(navigationUrl, new MockResponse("online shell"));
  const navigation = {
    method: "GET",
    mode: "navigate",
    url: navigationUrl,
  };
  let response = await dispatch("fetch", { request: navigation });
  assert.equal(response.body, "online shell");
  assert.equal(fetchCalls.at(-1), navigationUrl);
  assert.equal((await current.match(navigationUrl)).body, "online shell");

  fetchResponses.set(navigationUrl, new Error("offline"));
  response = await dispatch("fetch", { request: navigation });
  assert.equal(response.body, "precache:./index.html");

  current.entries.delete(keyFor(`${scope}index.html`));
  fetchResponses.set(staleShellUrl, new Error("offline stale shell"));
  response = await dispatch("fetch", {
    request: { method: "GET", mode: "navigate", url: staleShellUrl },
  });
  assert.equal(response.status, 0, "navigation must fail closed without current-cache shell");
  assert.equal(response.body, "");

  fetchResponses.set(staleAssetUrl, new Error("offline stale asset"));
  await assert.rejects(
    dispatch("fetch", {
      request: { method: "GET", mode: "same-origin", url: staleAssetUrl },
    }),
    /offline stale asset/,
    "asset fetch must not use an unrelated cache entry",
  );

  const cachedAsset = {
    method: "GET",
    mode: "same-origin",
    url: `${origin}/app/service-worker.js`,
  };
  const callsBeforeCacheHit = fetchCalls.length;
  response = await dispatch("fetch", { request: cachedAsset });
  assert.equal(response.body, "precache:./service-worker.js");
  assert.equal(fetchCalls.length, callsBeforeCacheHit);

  const uncachedAssetUrl = `${origin}/app/new.js`;
  fetchResponses.set(uncachedAssetUrl, new MockResponse("new asset"));
  response = await dispatch("fetch", {
    request: { method: "GET", mode: "same-origin", url: uncachedAssetUrl },
  });
  assert.equal(response.body, "new asset");
  assert.equal((await current.match(uncachedAssetUrl)).body, "new asset");

  const callsBeforeRejectedRequests = fetchCalls.length;
  assert.equal(
    await dispatch("fetch", {
      request: {
        method: "GET",
        mode: "cors",
        url: "https://cdn.example.test/app.js",
      },
    }),
    undefined,
  );
  assert.equal(
    await dispatch("fetch", {
      request: { method: "POST", mode: "same-origin", url: navigationUrl },
    }),
    undefined,
  );
  assert.equal(fetchCalls.length, callsBeforeRejectedRequests);

  console.log("Service-worker contract: PASS (install, activate, fetch, and request gating)");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
