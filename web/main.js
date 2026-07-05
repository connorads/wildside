// Browser shell glue for the wasm engine (#19): fetch bundle, fixed-60Hz step
// loop, keyboard/gamepad -> controller mask, RGBA blit to a 320x224 canvas.
import init, { WildsideGame } from "./pkg/wildside_web.js";

const STATUS = document.getElementById("status");
const CANVAS = document.getElementById("screen");
const CTX = CANVAS.getContext("2d");

// Mirrors FrameInput::from_mask (wildside-core types.rs): Up/Down/Left/Right/
// B/C/A/Start = bits 0..7, and the native SDL shell's key bindings
// (native_player keyboard_buttons).
const KEY_BITS = new Map([
  ["ArrowUp", 0x01],
  ["KeyW", 0x01],
  ["ArrowDown", 0x02],
  ["KeyS", 0x02],
  ["ArrowLeft", 0x04],
  ["KeyA", 0x04],
  ["ArrowRight", 0x08],
  ["KeyD", 0x08],
  ["KeyZ", 0x10],
  ["Space", 0x10],
  ["KeyX", 0x20],
  ["KeyC", 0x40],
  ["ShiftLeft", 0x40],
  ["ShiftRight", 0x40],
  ["Enter", 0x80],
]);

let heldMask = 0;
window.addEventListener("keydown", (event) => {
  const bit = KEY_BITS.get(event.code);
  if (bit !== undefined) {
    heldMask |= bit;
    event.preventDefault();
  }
});
window.addEventListener("keyup", (event) => {
  const bit = KEY_BITS.get(event.code);
  if (bit !== undefined) {
    heldMask &= ~bit;
    event.preventDefault();
  }
});
// Keys released while the tab is hidden never fire keyup.
window.addEventListener("blur", () => {
  heldMask = 0;
});

// Standard-mapping gamepad -> same mask (dpad 12-15, face buttons, start 9).
function gamepadMask() {
  let mask = 0;
  for (const pad of navigator.getGamepads()) {
    if (!pad || pad.mapping !== "standard") continue;
    const pressed = (index) => pad.buttons[index]?.pressed;
    if (pressed(12)) mask |= 0x01;
    if (pressed(13)) mask |= 0x02;
    if (pressed(14)) mask |= 0x04;
    if (pressed(15)) mask |= 0x08;
    if (pressed(0)) mask |= 0x10; // B (jump)
    if (pressed(1)) mask |= 0x20; // C
    if (pressed(2)) mask |= 0x40; // A
    if (pressed(9)) mask |= 0x80; // Start
  }
  return mask;
}

async function fetchBytes(url, label) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${label}: HTTP ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function main() {
  const wasm = await init();
  STATUS.textContent = "fetching asset bundle…";
  const bundle = await fetchBytes("bundle.wsbndl", "bundle.wsbndl");

  STATUS.textContent = "booting…";
  const level = Number(new URLSearchParams(location.search).get("level") ?? 0);
  const game = new WildsideGame(bundle, level);
  const width = WildsideGame.width();
  const height = WildsideGame.height();
  const image = CTX.createImageData(width, height);
  STATUS.textContent = "";

  // Console parity probe: hash of the current RGB24 frame, comparable against
  // native capture-frames pixels for the same scenario frame (#19 verification).
  window.wildside = game;

  const blit = () => {
    // Re-acquire the memory view every frame: linear memory may have grown,
    // which detaches previous views.
    const frame = new Uint8ClampedArray(
      wasm.memory.buffer,
      game.frame_ptr(),
      game.frame_len(),
    );
    image.data.set(frame);
    CTX.putImageData(image, 0, 0);
  };

  // Fixed 60Hz accumulator over rAF. AudioContext.currentTime becomes the
  // clock once audio lands (#20); performance.now drift is fine without it.
  const FRAME_MS = 1000 / 60;
  const MAX_STEPS_PER_TICK = 4; // background-tab catch-up guard
  let previous = performance.now();
  let accumulator = 0;

  const tick = (now) => {
    accumulator = Math.min(
      accumulator + (now - previous),
      FRAME_MS * MAX_STEPS_PER_TICK,
    );
    previous = now;
    let stepped = false;
    while (accumulator >= FRAME_MS) {
      game.step(heldMask | gamepadMask());
      accumulator -= FRAME_MS;
      stepped = true;
    }
    if (stepped) {
      blit();
      // VDP shell screens (Sega/title/level card) render black until the
      // web-audio phase brings the VDP renderer along (#20); the ROM holds
      // the level card until a button edge, so say so.
      STATUS.textContent = game.on_shell_screen() ? "· press any key ·" : "";
    }
    requestAnimationFrame(tick);
  };
  blit();
  requestAnimationFrame(tick);
}

main().catch((error) => {
  STATUS.textContent = String(error);
  throw error;
});
