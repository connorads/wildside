# Audio SFX Spike — Findings and Entry Points

**Status**: paused after audit, no code landed.
**Written**: 2026-04-24
**Relates to**: multi-lane recovery plan Lane 3 ("3-slice SFX spike — jump, stomp-enemy, take-damage").

## Intent of this document

The SFX spike is the only lane of the recovery plan that did not land in the current session. The plan budgeted 2 weeks for it and explicitly allowed bailing out with a "what was learned" writeup. This is that writeup. A future session should start here rather than re-auditing the audio pipeline from scratch.

## What is already real (no work needed)

- **ymfm-sys FFI** (`crates/ymfm-sys/build.rs:18-29`): compiles 5 C++ sources from `third_party/ymfm/src/` — `ymfm_adpcm.cpp`, `ymfm_misc.cpp`, `ymfm_opn.cpp` (YM2612 core), `ymfm_pcm.cpp`, `ymfm_ssg.cpp` — plus `bridge.cpp`. Full emulator, not a stub.
- **kid-md `AudioMixer`** (`crates/kid-md/src/lib.rs:178-184, 458-472, 1859-1928`): consumes `AudioEvent::Ym2612Write` / `AudioEvent::PsgWrite` events from a `VecDeque`, dispatches each directly to `ym2612.write_port(port, value)` / `psg_registers.apply_write(value)`, and those feed the real synthesisers during sample generation. Live audio synthesis, wired end-to-end.
- **`kidcli verify-audio-output`** (`crates/kidcli/src/verify.rs:513, 598`): active subcommand. Envelope RMS similarity threshold is 60% (`const MIN_AUDIO_ENVELOPE_SIMILARITY: f32 = 0.60`). Current scenarios cover title-card idle, level-1 entry, walking, enemy window, and a single jump — no jump/stomp/damage combo yet.

## What is scaffold-only (the actual gap)

- **`sound_driver_write_sub_E1330`** (`crates/kid-core/src/lib.rs:555-563`) returns a `SoundDriverWriteSubE1330` metadata struct describing `{ queued_bytes: [0xFF, 0x05, sound_id], preserves_a0, preserves_d0 }`. **The function has no side effect** — it does not push anything onto the mixer's event queue. Every other `sound_driver_write_sub_*` in that file is the same shape: structural ASM parity, zero runtime effect.
- **No SFX tables exist** in `crates/kid-core/`. No `Ym2612RegAddr` usage, no `sfx_*` constants, no `crates/kid-core/src/audio/` module.
- **No call-site wiring**: `crates/kid-core/src/physics.rs:748` has a comment `// ... moveq #sfx_Jump,d0; jsr PlaySound` describing the ASM but does not emit any audio event in native. Same shape at `blocks.rs:524`, `projectiles.rs:364`, and at least 12 other locations.

## What blocks progress (the hard bit)

**GEMS sequence byte format is undocumented publicly.** The ROM holds sound data in `.cache/upstream/kid-chameleon-disasm/sound/*.bin`:
- `sequences.bin` — per-sound-ID event streams
- `instruments.bin` — YM2612 register presets
- `envelopes.bin`
- `GEMS_Sounddriver.bin`

`doc/sound.txt` in the disasm has **external tool links only** (gemsplay, gems) — no byte-format spec. Converting `sfx_Jump` (whatever ID number that is) into a list of YM2612 `(port, register, value)` writes requires reverse engineering the command byte format of the GEMS sequence interpreter.

`.cache/references/gems/` and `.cache/references/gemsplay/` are available as research repos and are the obvious entry points. They are community-maintained GEMS decoders; whether they are complete and correct enough to extract a sequence by sound ID without manual validation is unknown and should be the first thing a future session checks.

## ASM call-site inventory

`jsr (j_PlaySound).l` appears **84 times** in `kid.asm`. Heavy concentration in level setup, enemy defeat, player damage, and movement routines. For the spike, the three targeted call sites are:

| SFX | ASM routine | Native analogue |
|-----|-------------|-----------------|
| jump | near `sub_8F26` (player jump start) | `crates/kid-core/src/physics.rs:748` (has the comment already) |
| stomp-enemy | enemy defeat handler | TBD — grep `// ... jsr PlaySound` in `combat.rs` and `enemies.rs` |
| take-damage | `Kid_Hurt` routine | TBD — grep in `combat.rs` |

## Recommended next-session entry points

1. **First commit — research only.** Open `.cache/references/gemsplay/` and decide: can we call it as a library, or do we need a Rust port of its sequence parser? 90 minutes, no code in kid-core/ yet.
2. **Second commit — one SFX hand-decoded.** Pick `sfx_Jump` (simplest to test, single well-isolated call site). Read the sequence bytes out of `sequences.bin`, hand-convert one frame's worth into a `&'static [(u8, u8, u8)]` table (port, register, value), add an `audio` module, and write a unit test that checks the expected register sequence for a single jump. No mixer wiring yet.
3. **Third commit — mixer wiring.** Turn `sound_driver_write_sub_E1330` (or a new `play_sound_sub_E1330`) into a real emitter that pushes `AudioEvent::Ym2612Write` onto the mixer queue when called. Update the physics-jump call site.
4. **Fourth commit — verify-audio-output scenario.** Add a scenario that jumps once on level 1 and assert the envelope RMS similarity ≥ 0.60 against the oracle capture.
5. **Fifth and sixth commits — stomp and damage SFX.** Repeat steps 2-4 for each remaining SFX.

## Known risks and pitfalls

- **Oracle envelope RMS at 60% is a loose match.** A wrong-but-close sequence may pass. Cross-check by capturing `kidcli trace-audio` (if it exists; if not, add it) and comparing the register-write count and ordering frame-by-frame against the oracle's YM2612 writes. **Caveat**: jgenesis does not expose per-frame YM2612 register writes today. The oracle only emits PCM frames and checkpoint images, so register-level cross-checking requires instrumenting jgenesis separately (out of scope for a single spike).
- **Do not attempt to emulate the Z80 sound driver.** That is the full-GEMS project and is parked. The spike must go ROM data → static Rust table → mixer directly.
- **Budget**: plan allocated 2 weeks for the 3-slice. If reverse-engineering `sequences.bin` stalls past 3 days on the first SFX, stop and write another ADR rather than widening the guess.

## Do not do (rejected approaches)

- **Do not hand-author register writes from listening to the game.** That is a guess; it will pass the 60% threshold by accident and fail silently on any subsequent reverse-engineering session.
- **Do not widen `sound_driver_write_sub_E1330` to accept a synthetic byte stream**. The ASM thunk's contract is `sound_id → queued bytes`; keep the structural parity clean and put the new emitter alongside it as a separate function with a distinct name.
- **Do not chase per-call-site audio flags through all 84 `jsr j_PlaySound` sites** before the pipeline is proven on one SFX. The plan's correction is explicit: prove end-to-end with one SFX, then scale.

## Referenced files at time of writing

- Plan: (none checked in — see conversation history of the multi-lane recovery session)
- `crates/ymfm-sys/build.rs`
- `crates/kid-md/src/lib.rs`
- `crates/kid-core/src/lib.rs` (sound_driver_write_sub_* family)
- `crates/kidcli/src/verify.rs` (verify-audio-output)
- `.cache/upstream/kid-chameleon-disasm/sound/*.bin`
- `.cache/upstream/kid-chameleon-disasm/doc/sound.txt`
- `.cache/references/gems/`, `.cache/references/gemsplay/`
