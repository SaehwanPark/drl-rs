const CACHE_NAMESPACE = "drl-rs-m10-";
const CACHE_VERSION = /* __CACHE_VERSION__ */ "v1";
const CACHE_NAME = `${CACHE_NAMESPACE}${CACHE_VERSION}`;
const PRECACHE_URLS = /* __PRECACHE_URLS__ */ [];

function sameOriginGet(request) {
  return request.method === "GET" && new URL(request.url).origin === self.location.origin;
}

function shellUrl() {
  return new URL("./index.html", self.registration.scope).toString();
}

async function cacheSuccessfulResponse(request, response) {
  if (response.ok) {
    const cache = await caches.open(CACHE_NAME);
    await cache.put(request, response.clone());
  }
  return response;
}

async function matchCurrentCache(request) {
  const cache = await caches.open(CACHE_NAME);
  return cache.match(request);
}

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_URLS)),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith(CACHE_NAMESPACE) && key !== CACHE_NAME)
            .map((key) => caches.delete(key)),
        ),
      ),
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (!sameOriginGet(request)) return;

  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => cacheSuccessfulResponse(request, response))
        .catch(() => matchCurrentCache(shellUrl()).then((response) => response || Response.error())),
    );
    return;
  }

  event.respondWith(
    matchCurrentCache(request).then((cached) =>
      cached || fetch(request).then((response) => cacheSuccessfulResponse(request, response)),
    ),
  );
});
