// Browser shell glue for the wasm engine (#19): fetch bundle, fixed-60Hz step
// loop, keyboard/gamepad -> controller mask, RGBA blit to a 320x224 canvas.
import init, { WildsideGame } from "./pkg/wildside_web.js";

const STATUS = document.getElementById("status");
const CANVAS = document.getElementById("screen");
const CTX = CANVAS.getContext("2d");
const ROM_PANEL = document.getElementById("rom-panel");
const ROM_INPUT = document.getElementById("rom-file");
const REMEMBER_ROM = document.getElementById("remember-rom");
const FORGET_ROM = document.getElementById("forget-rom");

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

function loadAssetBundle() {
  return new Promise((resolve, reject) => {
    const worker = new Worker("./asset-worker.js", { type: "module" });
    let gameReady = false;
    const showPicker = (previousError) => {
      ROM_PANEL.hidden = false;
      STATUS.textContent = previousError
        ? `${previousError.message} Choose your ROM to continue.`
        : "Choose your legally obtained Kid Chameleon ROM.";
    };

    worker.addEventListener("message", (event) => {
      const response = event.data;
      switch (response?.type) {
        case "progress":
          STATUS.textContent = response.message;
          break;
        case "needs-rom":
          showPicker(response.previousError);
          break;
        case "ready":
          gameReady = true;
          ROM_PANEL.hidden = true;
          resolve(response.bundle);
          break;
        case "forgot":
          STATUS.textContent = "Remembered ROM removed.";
          if (!gameReady) showPicker();
          break;
        case "error":
          reject(new Error(`${response.error.kind}: ${response.error.message}`));
          worker.terminate();
          break;
      }
    });

    ROM_INPUT.addEventListener("change", async () => {
      const file = ROM_INPUT.files?.[0];
      if (!file) return;
      ROM_INPUT.disabled = true;
      try {
        const bytes = await file.arrayBuffer();
        worker.postMessage(
          {
            type: "import-rom",
            bytes,
            remember: REMEMBER_ROM.checked,
          },
          [bytes],
        );
      } finally {
        ROM_INPUT.disabled = false;
        ROM_INPUT.value = "";
      }
    });
    FORGET_ROM.addEventListener("click", () => {
      worker.postMessage({ type: "forget-rom" });
    });
    worker.postMessage({
      type: "bootstrap",
      configUrl: new URL("./config.json", location.href).href,
    });
    window.wildsideAssetWorker = worker;
  });
}

// --- Audio (#20): wasm GEMS samples → SAB ring → AudioWorklet. ---
// The engine emits each stepped frame's interleaved-stereo f32 at the
// AudioContext rate (fixed 60Hz remainder division on the Rust side); the
// worklet drains the ring at the device rate. ~0.74s of headroom at 44.1kHz —
// occupancy stays near zero because both sides run off the same audio clock.
const AUDIO_RING_F32 = 1 << 16;
let audio = null; // { ctx, ring, state } once running
let audioUnavailable = false; // failed or impossible; stop retrying

async function enableAudio(game) {
  if (audio || audioUnavailable) return;
  audioUnavailable = true; // latched off during async setup; cleared on success
  if (!crossOriginIsolated) {
    console.warn(
      "audio disabled: page is not cross-origin isolated - serve with " +
        "COOP/COEP headers (mise run web-worker-dev)",
    );
    return;
  }
  try {
    const ctx = new AudioContext();
    game.enable_audio(ctx.sampleRate);
    await ctx.audioWorklet.addModule("./audio-worklet.js");
    const ringBuffer = new SharedArrayBuffer(AUDIO_RING_F32 * 4);
    const stateBuffer = new SharedArrayBuffer(8); // Int32 [read, write]
    const node = new AudioWorkletNode(ctx, "wildside-audio", {
      outputChannelCount: [2],
      processorOptions: { ringBuffer, stateBuffer },
    });
    node.connect(ctx.destination);
    await ctx.resume();
    audio = {
      ctx,
      ring: new Float32Array(ringBuffer),
      state: new Int32Array(stateBuffer),
    };
    audioUnavailable = false;
    // Console probe for verification: ring indices + context state.
    window.wildsideAudio = audio;
  } catch (error) {
    console.warn("audio unavailable:", error);
  }
}

// Copy the last step's samples from wasm linear memory into the ring. Skips
// the frame when the ring is full (drains at device rate, so only a stalled
// context could fill it) rather than overwriting unread audio.
function pushAudioFrame(wasm, game) {
  if (!audio) return;
  const samples = new Float32Array(
    wasm.memory.buffer,
    game.audio_ptr(),
    game.audio_len(),
  );
  const capacity = audio.ring.length;
  const read = Atomics.load(audio.state, 0);
  const write = Atomics.load(audio.state, 1);
  const free = capacity - ((write - read + capacity) % capacity) - 2;
  if (samples.length > free) return;
  const untilWrap = Math.min(samples.length, capacity - write);
  audio.ring.set(samples.subarray(0, untilWrap), write);
  if (untilWrap < samples.length) {
    audio.ring.set(samples.subarray(untilWrap), 0);
  }
  Atomics.store(audio.state, 1, (write + samples.length) % capacity);
}

async function main() {
  STATUS.textContent = "Loading game data…";
  const [wasm, bundle] = await Promise.all([init(), loadAssetBundle()]);

  STATUS.textContent = "booting…";
  // No ?level= boots the real attract flow (Sega logo → intro → title menu);
  // ?level=N warps straight into that level.
  const levelParam = new URLSearchParams(location.search).get("level");
  const game = new WildsideGame(
    bundle,
    levelParam === null ? undefined : Number(levelParam),
  );
  const width = WildsideGame.width();
  const height = WildsideGame.height();
  const image = CTX.createImageData(width, height);
  STATUS.textContent = "";

  // Console parity probe: hash of the current RGB24 frame, comparable against
  // native capture-frames pixels for the same scenario frame (#19 verification).
  window.wildside = game;

  // Autoplay policy: an AudioContext only runs after a user gesture, so hook
  // audio start to the first input rather than boot.
  const gestureAudio = () => {
    enableAudio(game);
  };
  window.addEventListener("keydown", gestureAudio);
  window.addEventListener("pointerdown", gestureAudio);

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

  // Fixed 60Hz accumulator over rAF. Once audio runs, the accumulator clocks
  // off AudioContext.currentTime so sample production and playback share one
  // clock and the ring occupancy cannot drift; performance.now otherwise.
  const FRAME_MS = 1000 / 60;
  const MAX_STEPS_PER_TICK = 4; // background-tab catch-up guard
  let previous = performance.now();
  let previousClockWasAudio = false;
  let accumulator = 0;

  const tick = (now) => {
    let clockNow = now;
    if (audio) {
      clockNow = audio.ctx.currentTime * 1000;
      if (!previousClockWasAudio) {
        // Clock source changed: restart the delta from the new timeline.
        previous = clockNow;
        previousClockWasAudio = true;
      }
    }
    accumulator = Math.min(
      accumulator + (clockNow - previous),
      FRAME_MS * MAX_STEPS_PER_TICK,
    );
    previous = clockNow;
    let stepped = false;
    while (accumulator >= FRAME_MS) {
      game.step(heldMask | gamepadMask());
      pushAudioFrame(wasm, game);
      accumulator -= FRAME_MS;
      stepped = true;
    }
    if (stepped) {
      blit();
      // The ROM holds shell screens (title menu, level cards) on a button
      // edge; without a hint an idle card reads as a hang.
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
