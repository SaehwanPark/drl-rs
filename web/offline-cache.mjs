/**
 * Starts browser-owned offline cache registration independently from game
 * graphics. The injected browser capability and diagnostic sink keep this
 * boundary deterministic and easy to exercise outside a browser.
 */
export function offlineRegistrationStatus(registration) {
  if (registration.waiting) {
    return " Offline cache update is ready; reload when convenient.";
  }
  if (registration.active) {
    return " Offline cache ready for the next reload.";
  }
  return " Offline cache installation started for the next reload.";
}

export async function registerOfflineCache(browserNavigator, writeDiagnostic) {
  if (!("serviceWorker" in browserNavigator)) {
    writeDiagnostic(
      "Offline cache unavailable",
      "This browser does not expose service workers for this deployment.",
      "Gameplay can continue online; no data is sent by this diagnostic.",
    );
    return " Offline cache unavailable in this browser.";
  }
  try {
    const registration = await browserNavigator.serviceWorker.register(
      "./service-worker.js",
      { scope: "./", updateViaCache: "none" },
    );
    return offlineRegistrationStatus(registration);
  } catch (error) {
    writeDiagnostic(
      "Offline cache unavailable",
      `The service worker could not be registered locally (${error}).`,
      "Gameplay can continue online; retry after checking the HTTPS deployment.",
    );
    return ` Offline cache unavailable (${error}).`;
  }
}
