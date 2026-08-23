import init, { boot, clear_save, dispatch_inventory, load, resize, restart as restart_game, save, set_muted, set_volume, unlock_audio } from "./pkg/drl_web.js";
import { browserSupportDiagnostic } from "./browser-support.mjs";
import { registerOfflineCache } from "./offline-cache.mjs";

const status = document.querySelector("#game-status");
const diagnostics = document.querySelector("#game-diagnostics");
const diagnosticTitle = document.querySelector("#diagnostics-title");
const diagnosticDetail = document.querySelector("#diagnostics-detail");
const diagnosticAction = document.querySelector("#diagnostics-action");
const canvas = document.querySelector("#game-canvas");
const start = document.querySelector("#start-button");
const restart = document.querySelector("#restart-button");
const saveButton = document.querySelector("#save-button");
const loadButton = document.querySelector("#load-button");
const clearSaveButton = document.querySelector("#clear-save-button");
const clearSaveDialog = document.querySelector("#clear-save-dialog");
const cancelClearSaveButton = document.querySelector("#cancel-clear-save");
const confirmClearSaveButton = document.querySelector("#confirm-clear-save");
const inventory = document.querySelector("#inventory");
const mute = document.querySelector("#mute-button");
const volume = document.querySelector("#volume-control");
let started = false;
let audioTask = Promise.resolve();

function writeStatus(message) {
  status.textContent = message;
}

function writeDiagnostic(title, detail, action) {
  diagnosticTitle.textContent = title;
  diagnosticDetail.textContent = detail;
  diagnosticAction.textContent = action;
  diagnostics.hidden = false;
  diagnostics.focus({ preventScroll: true });
}

function clearDiagnostic() {
  diagnostics.hidden = true;
  diagnosticTitle.textContent = "Browser support diagnostic";
  diagnosticDetail.textContent = "";
  diagnosticAction.textContent = "";
}

function queueAudioSetting(setting) {
  // Web Audio unlock temporarily takes the mixer out of WASM storage. Queue
  // all control changes so rapid UI events cannot observe a missing mixer or
  // apply an older setting after a newer one.
  audioTask = audioTask
    .catch(() => {})
    .then(async () => {
      await unlock_audio();
      writeStatus(setting());
    })
    .catch(() => {
      writeDiagnostic(
        "Audio unavailable",
        "The browser did not unlock Web Audio for this presentation session.",
        "Gameplay continues without audio; check browser site permissions and retry."
      );
      writeStatus("Audio unavailable; gameplay continues.");
    });
}

const offlineCacheReady = registerOfflineCache(navigator, writeDiagnostic);

start.addEventListener("click", async () => {
  if (started) return;
  // Keep capability classification pure so unsupported environments cannot
  // initialize WASM or accidentally submit a gameplay command.
  const environmentDiagnostic = browserSupportDiagnostic({
    secureContext: window.isSecureContext,
    webgpu: Boolean(navigator.gpu),
  });
  if (environmentDiagnostic) {
    writeDiagnostic(
      environmentDiagnostic.title,
      environmentDiagnostic.detail,
      environmentDiagnostic.action,
    );
    writeStatus(environmentDiagnostic.status);
    return;
  }
  try {
    await init();
    clearDiagnostic();
    const result = await boot();
    started = true;
    start.disabled = true;
    if (diagnostics.hidden) {
      canvas.focus();
    }
    // `boot()` writes the accurate ready/suspended/unavailable audio state.
    // Keep that message instead of assuming audio or graphics success.
    const readyMessage = status.textContent || `Ready (${result}).`;
    const offlineMessage = await offlineCacheReady;
    if (offlineMessage.includes("Offline cache unavailable")) {
      writeDiagnostic(
        "Offline cache unavailable",
        offlineMessage.trim(),
        "Gameplay can continue online; retry after checking the HTTPS deployment.",
      );
    }
    writeStatus(`${readyMessage}${offlineMessage}`);
  } catch (error) {
    writeDiagnostic(
      "Browser graphics unavailable",
      `WebGPU startup failed locally (${error}).`,
      "Use a supported desktop Chromium WebGPU environment; gameplay state was not started."
    );
    writeStatus(`Browser graphics unavailable: ${error}`);
  }
});

inventory.addEventListener("click", (event) => {
  const button = event.target.closest("button[data-action][data-item-id]");
  if (!button || !started) return;
  writeStatus(dispatch_inventory(button.dataset.action, Number(button.dataset.itemId)));
});

restart.addEventListener("click", () => {
  if (started) writeStatus(restart_game());
});

saveButton.addEventListener("click", () => {
  if (started) writeStatus(save());
});

loadButton.addEventListener("click", () => {
  if (started) writeStatus(load());
});

function closeClearSaveDialog(statusMessage) {
  clearSaveDialog.hidden = true;
  clearSaveButton.focus();
  writeStatus(statusMessage);
}

clearSaveButton.addEventListener("click", () => {
  if (!started) return;
  clearSaveDialog.hidden = false;
  cancelClearSaveButton.focus();
});

cancelClearSaveButton.addEventListener("click", () => {
  closeClearSaveDialog("Saved session kept.");
});

confirmClearSaveButton.addEventListener("click", () => {
  closeClearSaveDialog(clear_save());
});

document.addEventListener("keydown", (event) => {
  if (clearSaveDialog.hidden) return;
  if (event.key === "Tab") {
    event.preventDefault();
    const focusables = [cancelClearSaveButton, confirmClearSaveButton];
    const currentIndex = focusables.indexOf(document.activeElement);
    const direction = event.shiftKey ? -1 : 1;
    focusables[(currentIndex + direction + focusables.length) % focusables.length].focus();
  } else if (event.key === "Escape") {
    event.preventDefault();
    closeClearSaveDialog("Saved session kept.");
  }
});

mute.addEventListener("click", () => {
  if (!started) return;
  const muted = mute.getAttribute("aria-pressed") !== "true";
  mute.setAttribute("aria-pressed", String(muted));
  mute.textContent = muted ? "Unmute" : "Mute";
  queueAudioSetting(() => set_muted(muted));
});

volume.addEventListener("input", () => {
  if (started) {
    queueAudioSetting(() => set_volume(Number(volume.value)));
  }
});

window.addEventListener("resize", () => {
  if (started) resize(canvas.clientWidth, canvas.clientHeight, window.devicePixelRatio || 1);
});

document.addEventListener("visibilitychange", () => {
  if (started && document.hidden) writeStatus("Paused presentation while the tab is hidden; simulation is unchanged.");
});
