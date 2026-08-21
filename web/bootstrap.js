import init, { boot, dispatch_inventory, resize, restart as restart_game, set_muted, set_volume, unlock_audio } from "./pkg/drl_web.js";

const status = document.querySelector("#game-status");
const log = document.querySelector("#game-log");
const canvas = document.querySelector("#game-canvas");
const start = document.querySelector("#start-button");
const restart = document.querySelector("#restart-button");
const inventory = document.querySelector("#inventory");
const mute = document.querySelector("#mute-button");
const volume = document.querySelector("#volume-control");
let started = false;
let audioTask = Promise.resolve();

function writeStatus(message) {
  status.textContent = message;
  log.textContent = message;
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
    .catch(() => writeStatus("Audio unavailable; gameplay continues."));
}

start.addEventListener("click", async () => {
  if (started) return;
  try {
    await init();
    const result = await boot();
    started = true;
    start.disabled = true;
    canvas.focus();
    // `boot()` writes the accurate ready/suspended/unavailable audio state.
    // Keep that message and mirror it to the log instead of assuming success.
    log.textContent = status.textContent || `Ready (${result}).`;
  } catch (error) {
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
