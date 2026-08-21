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

function writeStatus(message) {
  status.textContent = message;
  log.textContent = message;
}

start.addEventListener("click", async () => {
  if (started) return;
  try {
    await init();
    const result = await boot();
    started = true;
    start.disabled = true;
    canvas.focus();
    writeStatus(`Ready (${result}). Audio unlocked from this gesture.`);
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
  unlock_audio().then(writeStatus);
  const muted = mute.getAttribute("aria-pressed") !== "true";
  mute.setAttribute("aria-pressed", String(muted));
  mute.textContent = muted ? "Unmute" : "Mute";
  writeStatus(set_muted(muted));
});

volume.addEventListener("input", () => {
  if (started) {
    unlock_audio().then(() => writeStatus(set_volume(Number(volume.value))));
  }
});

window.addEventListener("resize", () => {
  if (started) resize(canvas.clientWidth, canvas.clientHeight, window.devicePixelRatio || 1);
});

document.addEventListener("visibilitychange", () => {
  if (started && document.hidden) writeStatus("Paused presentation while the tab is hidden; simulation is unchanged.");
});
