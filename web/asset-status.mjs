/**
 * Describes optional browser asset packs without making them part of the
 * deterministic simulation or release-rights boundary.
 */
export const ASSET_PACKS = Object.freeze([
  Object.freeze({
    id: "graphics",
    label: "graphics",
    required: true,
    path: "./assets/legacy/drl/graphics/background.png",
    fallback: "none",
  }),
  Object.freeze({
    id: "audio-hq",
    label: "HQ audio",
    required: false,
    path: "./assets/legacy-drlhq/sound/dssgtatk.wav",
    fallback: "procedural Web Audio",
  }),
  Object.freeze({
    id: "audio-lq",
    label: "LQ audio",
    required: false,
    path: "./assets/legacy-drl/sound/dssgtatk.wav",
    fallback: "procedural Web Audio",
  }),
  Object.freeze({
    id: "fonts",
    label: "bitmap fonts",
    required: false,
    path: "./assets/legacy/drl/fonts/font10x19.png",
    fallback: "DOM/browser text",
  }),
]);

/**
 * Checks one representative file for each declared pack.
 *
 * The fetch function is injectable so the contract can be tested without a
 * browser or network. Optional packs may be absent from a release bundle;
 * their fallbacks remain the supported presentation policy.
 */
export async function inspectAssetPacks({
  fetchImpl = globalThis.fetch,
  packs = ASSET_PACKS,
} = {}) {
  if (typeof fetchImpl !== "function") {
    return packs.map((pack) => ({ ...pack, available: false, reason: "fetch-unavailable" }));
  }

  return Promise.all(
    packs.map(async (pack) => {
      try {
        const response = await fetchImpl(pack.path, {
          method: "HEAD",
          cache: "no-store",
        });
        return { ...pack, available: response?.ok === true, reason: response?.ok ? "present" : "missing" };
      } catch {
        return { ...pack, available: false, reason: "unreachable" };
      }
    }),
  );
}

/**
 * Converts probe results into a short, accessible startup status message.
 */
export function formatAssetPackStatus(packs) {
  const requiredMissing = packs.filter((pack) => pack.required && !pack.available);
  const optionalMissing = packs.filter((pack) => !pack.required && !pack.available);
  const labels = packs.map((pack) => `${pack.label}: ${pack.available ? "ready" : "missing"}`);
  let message = `Asset packs — ${labels.join(", ")}.`;
  if (requiredMissing.length > 0) {
    message += " Required graphics are missing; gameplay may not render.";
  }
  if (optionalMissing.length > 0) {
    const fallbacks = [...new Set(optionalMissing.map((pack) => pack.fallback))].join(" and ");
    message += ` Optional packs use ${fallbacks} fallback${fallbacks.includes(" and ") ? "s" : ""}.`;
  }
  return message;
}
