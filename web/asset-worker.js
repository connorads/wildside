import init, {
  extract_runtime_bundle_from_rom,
} from "./pkg/wildside_web.js";

const CANONICAL_SHA1 = "28b904000b2863b6760531807760b571f1a5fc1d";
const REMEMBERED_ROM = `kid-${CANONICAL_SHA1}.bin`;

let wasmReady;

function workerError(kind, message) {
  return { kind, message };
}

function parseConfig(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw workerError("config", "config must be a JSON object");
  }
  if (value.schemaVersion !== 1) {
    throw workerError("config", "config.schemaVersion must be 1");
  }
  const source = value.source;
  if (!source || typeof source !== "object" || Array.isArray(source)) {
    throw workerError("config", "config.source must be an object");
  }
  if (source.kind === "user-rom") {
    return { schemaVersion: 1, source: { kind: "user-rom" } };
  }
  if (source.kind !== "rom-url" && source.kind !== "bundle-url") {
    throw workerError(
      "config",
      "config.source.kind must be user-rom, rom-url, or bundle-url",
    );
  }
  if (typeof source.url !== "string" || source.url.trim() === "") {
    throw workerError("config", "URL sources require a non-empty url");
  }
  const url = new URL(source.url, self.location.href);
  if (
    url.origin !== self.location.origin &&
    url.protocol !== "https:"
  ) {
    throw workerError(
      "config",
      "cross-origin source URLs must use HTTPS",
    );
  }
  if (source.fallback !== "user-rom" && source.fallback !== "none") {
    throw workerError(
      "config",
      "URL sources require fallback set to user-rom or none",
    );
  }
  return {
    schemaVersion: 1,
    source: {
      kind: source.kind,
      url: url.href,
      fallback: source.fallback,
    },
  };
}

async function fetchBytes(url, label) {
  let response;
  try {
    response = await fetch(url);
  } catch (error) {
    throw workerError(
      "network-cors",
      `${label} could not be fetched: ${String(error)}`,
    );
  }
  if (!response.ok) {
    throw workerError("network-cors", `${label}: HTTP ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function opfsRoot() {
  if (!navigator.storage?.getDirectory) return null;
  return navigator.storage.getDirectory();
}

async function loadRememberedRom() {
  try {
    const root = await opfsRoot();
    if (!root) return null;
    const handle = await root.getFileHandle(REMEMBERED_ROM);
    return new Uint8Array(await (await handle.getFile()).arrayBuffer());
  } catch (error) {
    if (error?.name === "NotFoundError") return null;
    throw workerError("storage", `remembered ROM could not be read: ${error}`);
  }
}

async function rememberRom(bytes) {
  try {
    await navigator.storage?.persist?.();
    const root = await opfsRoot();
    if (!root) {
      throw new Error("Origin Private File System is unavailable");
    }
    const handle = await root.getFileHandle(REMEMBERED_ROM, { create: true });
    const writable = await handle.createWritable();
    await writable.write(bytes);
    await writable.close();
  } catch (error) {
    postMessage({
      type: "progress",
      message: `Game is ready, but the ROM was not remembered: ${error}`,
    });
  }
}

async function forgetRom() {
  try {
    const root = await opfsRoot();
    if (root) {
      await root.removeEntry(REMEMBERED_ROM);
    }
  } catch (error) {
    if (error?.name !== "NotFoundError") {
      throw workerError("storage", `remembered ROM could not be removed: ${error}`);
    }
  }
  postMessage({ type: "forgot" });
}

async function extractRom(bytes, remember) {
  postMessage({ type: "progress", message: "Verifying ROM…" });
  try {
    wasmReady ??= init();
    await wasmReady;
    postMessage({ type: "progress", message: "Extracting runtime assets…" });
    const bundle = extract_runtime_bundle_from_rom(bytes);
    if (remember) await rememberRom(bytes);
    postMessage({ type: "ready", bundle }, [bundle.buffer]);
  } catch (error) {
    throw workerError("unsupported-rom", String(error));
  }
}

async function useRememberedOrPicker(previousError) {
  const remembered = await loadRememberedRom();
  if (remembered) {
    try {
      await extractRom(remembered, false);
      return;
    } catch (error) {
      await forgetRom();
      previousError ??= error;
    }
  }
  postMessage({ type: "needs-rom", previousError });
}

async function bootstrap(configUrl) {
  const configBytes = await fetchBytes(configUrl, "config.json");
  let config;
  try {
    config = parseConfig(JSON.parse(new TextDecoder().decode(configBytes)));
  } catch (error) {
    if (error?.kind) throw error;
    throw workerError("config", `config.json is invalid JSON: ${error}`);
  }

  if (config.source.kind === "user-rom") {
    await useRememberedOrPicker();
    return;
  }

  try {
    postMessage({ type: "progress", message: "Fetching configured game data…" });
    const bytes = await fetchBytes(config.source.url, config.source.kind);
    if (config.source.kind === "bundle-url") {
      postMessage({ type: "ready", bundle: bytes }, [bytes.buffer]);
    } else {
      await extractRom(bytes, false);
    }
  } catch (error) {
    if (config.source.fallback === "user-rom") {
      await useRememberedOrPicker(error);
      return;
    }
    throw error;
  }
}

self.addEventListener("message", (event) => {
  const request = event.data;
  const operation = (() => {
    switch (request?.type) {
      case "bootstrap":
        return bootstrap(request.configUrl);
      case "import-rom":
        return extractRom(new Uint8Array(request.bytes), request.remember === true);
      case "forget-rom":
        return forgetRom();
      default:
        return Promise.reject(workerError("protocol", "unknown asset worker request"));
    }
  })();
  operation.catch((error) => {
    if (request?.type === "import-rom") {
      postMessage({
        type: "needs-rom",
        previousError: error?.kind
          ? error
          : workerError("extraction", String(error)),
      });
      return;
    }
    postMessage({
      type: "error",
      error: error?.kind
        ? error
        : workerError("extraction", String(error)),
    });
  });
});
