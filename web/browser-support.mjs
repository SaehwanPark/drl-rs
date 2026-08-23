/**
 * Classifies the browser capabilities required by the first playable slice.
 * The returned strings are user-facing recovery guidance, not a support
 * claim for any browser or graphics backend.
 */
export function browserSupportDiagnostic({ secureContext, webgpu }) {
  if (secureContext === false) {
    return {
      title: "Secure context required",
      detail: "WebGPU startup is blocked because this page is not running in a secure context.",
      action: "Serve the static bundle over HTTPS or use localhost, then retry.",
      status: "Browser graphics unavailable: secure context required.",
    };
  }
  if (!webgpu) {
    return {
      title: "WebGPU unavailable",
      detail: "This build requires the WebGPU browser API for graphics initialization.",
      action: "Use a desktop Chromium browser with WebGPU enabled; other backends are not claimed.",
      status: "Browser graphics unavailable: WebGPU is not exposed.",
    };
  }
  return null;
}
