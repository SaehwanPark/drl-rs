import assert from "node:assert/strict";
import { browserSupportDiagnostic } from "../web/browser-support.mjs";

const secureContext = browserSupportDiagnostic({ secureContext: false, webgpu: true });
assert.deepEqual(secureContext, {
  title: "Secure context required",
  detail: "WebGPU startup is blocked because this page is not running in a secure context.",
  action: "Serve the static bundle over HTTPS or use localhost, then retry.",
  status: "Browser graphics unavailable: secure context required.",
});

const missingWebGpu = browserSupportDiagnostic({ secureContext: true, webgpu: false });
assert.deepEqual(missingWebGpu, {
  title: "WebGPU unavailable",
  detail: "This build requires the WebGPU browser API for graphics initialization.",
  action: "Use a desktop Chromium browser with WebGPU enabled; other backends are not claimed.",
  status: "Browser graphics unavailable: WebGPU is not exposed.",
});

assert.equal(browserSupportDiagnostic({ secureContext: true, webgpu: true }), null);
assert.equal(
  browserSupportDiagnostic({ secureContext: undefined, webgpu: true }),
  null,
  "older browsers without isSecureContext must still reach the WebGPU check",
);

console.log("Browser support classifier contract: PASS (secure context, WebGPU, supported startup)");
