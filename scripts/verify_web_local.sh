#!/bin/sh
set -eu

ROM=${WILDSIDE_ROM:-}
if [ -z "$ROM" ]; then
  for candidate in roms/kid.bin kid.bin kc.md .cache/upstream/kid-chameleon-disasm/kid.bin; do
    if [ -f "$candidate" ]; then
      ROM=$candidate
      break
    fi
  done
fi
if [ -z "$ROM" ]; then
  echo "verify-web-local needs WILDSIDE_ROM or a canonical repo-local ROM"
  exit 1
fi
ROM=$(cd "$(dirname "$ROM")" && pwd)/$(basename "$ROM")

PORT=${WILDSIDE_WEB_PORT:-8791}
BASE_URL=http://127.0.0.1:$PORT
SESSION=wildside-local
BUNDLE=${TMPDIR:-/tmp}/wildside-runtime-local.wsbndl
LOG=${TMPDIR:-/tmp}/wildside-wrangler-local.log

cargo run -q -p wildside-cli -- pack-web-bundle --output "$BUNDLE"
python3 scripts/package_web.py --config web/config.json --output web/dist
wrangler dev --port "$PORT" >"$LOG" 2>&1 &
WRANGLER_PID=$!
cleanup() {
  playwright-cli -s="$SESSION" close >/dev/null 2>&1 || true
  kill "$WRANGLER_PID" >/dev/null 2>&1 || true
  python3 scripts/package_web.py --config web/config.json --output web/dist
}
trap cleanup EXIT INT TERM

attempt=0
until curl -fsS "$BASE_URL/" >/dev/null; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 50 ]; then
    cat "$LOG"
    exit 1
  fi
  sleep 0.2
done

HEADERS=$(curl -fsSI "$BASE_URL/" | tr '[:upper:]' '[:lower:]')
echo "$HEADERS" | grep -q 'cross-origin-opener-policy: same-origin'
echo "$HEADERS" | grep -q 'cross-origin-embedder-policy: require-corp'
echo "$HEADERS" | grep -q 'x-content-type-options: nosniff'
curl -fsSI "$BASE_URL/pkg/wildside_web_bg.wasm" | grep -qi 'content-type: application/wasm'

playwright-cli -s="$SESSION" open --browser=chromium "$BASE_URL/" >/dev/null
playwright-cli -s="$SESSION" run-code "async page => {
  await page.waitForFunction(() => document.querySelector('#rom-panel')?.hidden === false);
  if (!await page.evaluate(() => crossOriginIsolated)) throw new Error('page is not cross-origin isolated');
  await page.getByRole('checkbox', { name: 'Remember on this device' }).check();
  await page.locator('#rom-file').setInputFiles('$ROM');
  await page.waitForFunction(() => Boolean(window.wildside));
  await page.reload();
  await page.waitForFunction(() => Boolean(window.wildside));
  await page.getByRole('button', { name: 'Forget saved ROM' }).click();
  await page.reload();
  await page.waitForFunction(() => document.querySelector('#rom-panel')?.hidden === false);
}" >/dev/null

python3 scripts/package_web.py \
  --config web/configs/bundle-url.json \
  --output web/dist \
  --copy-source "$BUNDLE"
sleep 0.5
playwright-cli -s="$SESSION" goto "$BASE_URL/" >/dev/null
playwright-cli -s="$SESSION" run-code \
  "async page => await page.waitForFunction(() => Boolean(window.wildside))" >/dev/null

python3 scripts/package_web.py \
  --config web/configs/rom-url.json \
  --output web/dist \
  --copy-source "$ROM"
sleep 0.5
playwright-cli -s="$SESSION" goto "$BASE_URL/" >/dev/null
playwright-cli -s="$SESSION" run-code \
  "async page => await page.waitForFunction(() => Boolean(window.wildside))" >/dev/null

python3 scripts/package_web.py --config web/configs/rom-url.json --output web/dist
sleep 0.5
playwright-cli -s="$SESSION" goto "$BASE_URL/" >/dev/null
playwright-cli -s="$SESSION" run-code "async page => {
  await page.waitForFunction(() => document.querySelector('#rom-panel')?.hidden === false);
  if (!document.querySelector('#status').textContent.includes('Choose your ROM')) {
    throw new Error('failed URL did not fall back to picker');
  }
}" >/dev/null

echo "web local verification passed"
