//! GEMS mailbox-ring decoder (#435): recover the ordered 68k→Z80 command
//! stream from per-frame Z80 audio-RAM snapshots.
//!
//! ASM ground truth (kid.asm): the 68k posts commands into a 64-byte ring at
//! Z80 `$1A40` (`sub_E143A` loads `$A01A40`), indexed by the single write
//! byte at `$0036`, advanced per byte and wrapped `& $3F` (`sub_E148E`).
//! Every command is framed `[$FF][opcode][params...]` (`sub_E1482` falls
//! through into `sub_E148E`, writing marker + opcode). StopMusic
//! (`loc_E14C8`, opcode $0B) first reloads the whole driver image and
//! zero-fills Z80 RAM to `$2000` (`sub_E13DA`), resets the Z80
//! (`sub_E1416`), then posts 14 ring bytes: `$FF $0B` + four 3-byte
//! little-endian data pointers (`sub_E14AC`).
//!
//! Snapshot realities this decoder handles (validated on the 924-frame
//! `.cache/oracle-probe` capture):
//! - consumed bytes are never cleared (the Z80 read pointer lives
//!   elsewhere), so only the `[anchor, index)` window is new content;
//! - `sub_E148E` commits the index after EVERY byte, so a snapshot can
//!   catch a partial command — pending bytes carry to the next frame and
//!   the command attributes to the frame where its last byte lands;
//! - a StopMusic re-initialisation spans multiple snapshots: the reloaded
//!   driver image resets `$0036` to 0 first, the zero-fill wipes the ring
//!   on a later sample, and the 14-byte repost lands after that. Stale
//!   pre-reset ring bytes can be byte-identical to the repost because the
//!   four pointers are constants.

pub(crate) use wildside_core::{
    GEMS_COMMAND_RING_BASE as GEMS_RING_BASE, GEMS_COMMAND_RING_LEN as GEMS_RING_LEN,
    GEMS_COMMAND_RING_WRITE_INDEX_ADDR as GEMS_RING_WRITE_INDEX_ADDR,
};

/// Total posted bytes per opcode, marker `$FF` included.
///
/// Poster-side lengths from the kid.asm sound subs, cross-checked against
/// the GEMS driver's consume-side dispatch (gemsplay `GemsPlay.c:1648-1845`):
/// $10/$12 = 3 (sub_E151C/sub_E153E), $05 = 3 (sub_E1552; the 5-byte
/// "loc_E15AC" shape is really `[$FF $05 speed]` + `[$FF $0D]`),
/// $0C/$0D = 2 (sub_E155A/sub_E1568), $00/$01/$02/$06/$07/$0E = 4
/// (loc_E1584 family), $17 = 5 (loc_E15E8), $0B = 14 (loc_E14C8).
/// Opcodes the Kid ROM never posts but the GEMS dispatch consumes are
/// included so an unexpected posting decodes instead of derailing the
/// stream: $03/$1C/$1D/$20 = 3, $04 = 5, $14/$1A/$1B/$1F = 4, $16 = 2.
/// $1E (virtual-channel pitch bend) is variable-length in the driver and
/// is deliberately absent: seeing it fails loudly as an unknown opcode.
pub(crate) fn gems_command_posted_len(opcode: u8) -> Option<usize> {
    match opcode {
        0x0C | 0x0D | 0x16 => Some(2),
        0x03 | 0x05 | 0x10 | 0x12 | 0x1C | 0x1D | 0x20 => Some(3),
        0x00 | 0x01 | 0x02 | 0x06 | 0x07 | 0x0E | 0x14 | 0x1A | 0x1B | 0x1F => Some(4),
        0x04 | 0x17 => Some(5),
        0x0B => Some(14),
        _ => None,
    }
}

/// One decoded command: opcode plus its parameter bytes (marker stripped).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GemsRingCommand {
    pub(crate) opcode: u8,
    pub(crate) args: Vec<u8>,
}

/// Decode outcome for one frame snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub(crate) struct GemsRingFrameDecode {
    /// Commands whose final byte landed in this frame's window.
    pub(crate) commands: Vec<GemsRingCommand>,
    /// A StopMusic re-initialisation was observed at this snapshot (index
    /// reset and/or ring wipe from `sub_E13DA`).
    pub(crate) ring_reinit: bool,
    /// Bytes outside the posted window changed without a recognisable
    /// re-initialisation: the window may have lapped the 64-byte ring in
    /// one frame, so posted commands were lost. Fail loudly downstream.
    pub(crate) queue_overflow: bool,
    /// An opcode outside [`gems_command_posted_len`]'s table: the stream
    /// cannot be framed past it. Fail loudly downstream.
    pub(crate) unknown_opcode: Option<u8>,
    /// An index reset was hypothesised (backwards move to 0 with unchanged
    /// window bytes) but the following content was neither a wipe nor a
    /// StopMusic repost — the "reset" may really have been a wrap posting
    /// byte-identical content ending exactly at position 0, whose command
    /// would otherwise vanish silently. Fail loudly downstream.
    pub(crate) reset_unverified: bool,
}

impl GemsRingFrameDecode {
    pub(crate) fn is_clean(&self) -> bool {
        !self.queue_overflow && self.unknown_opcode.is_none() && !self.reset_unverified
    }
}

/// Streaming ring decoder: feed one full-RAM (or ring-window) snapshot per
/// frame in order; frame decodes come back with commands attributed to the
/// frame in which they completed.
#[derive(Debug, Clone)]
pub(crate) struct GemsRingDecoder {
    anchor: usize,
    prev_ring: [u8; GEMS_RING_LEN],
    /// Marker-led bytes of a command still incomplete at the last window
    /// end (`sub_E148E` commits the index per byte, so snapshots can catch
    /// partial commands).
    pending: Vec<u8>,
    /// An index reset was seen with unchanged window bytes; the next
    /// contentful snapshot must be a wipe or a StopMusic repost to confirm
    /// it was a genuine `sub_E13DA` reinit and not a byte-identical wrap.
    reset_awaiting_verification: bool,
}

impl Default for GemsRingDecoder {
    fn default() -> Self {
        Self {
            // Anchor 0 against an all-zero power-on ring, so a posting that
            // completes before the first snapshot (e.g. the boot StopMusic)
            // still decodes.
            anchor: 0,
            prev_ring: [0; GEMS_RING_LEN],
            pending: Vec::new(),
            reset_awaiting_verification: false,
        }
    }
}

impl GemsRingDecoder {
    /// Decode one snapshot given a manifest ring capture (#435).
    pub(crate) fn decode_manifest_ring(
        &mut self,
        capture: &wildside_core::GemsFrameRing,
    ) -> GemsRingFrameDecode {
        let mut ring = [0u8; GEMS_RING_LEN];
        ring.copy_from_slice(&capture.bytes);
        self.decode_frame(&ring, (capture.write_index as usize) % GEMS_RING_LEN)
    }

    /// Decode one snapshot given the full 8 KiB Z80 RAM image.
    pub(crate) fn decode_ram_frame(&mut self, ram: &[u8]) -> GemsRingFrameDecode {
        let mut ring = [0u8; GEMS_RING_LEN];
        ring.copy_from_slice(&ram[GEMS_RING_BASE..GEMS_RING_BASE + GEMS_RING_LEN]);
        let index = (ram[GEMS_RING_WRITE_INDEX_ADDR] as usize) % GEMS_RING_LEN;
        self.decode_frame(&ring, index)
    }

    /// Decode one snapshot given the 64-byte ring window and write index.
    ///
    /// Disambiguation rules, validated against the real capture:
    /// - New content lives only in `[anchor, index)` (mod 64); bytes
    ///   outside that window change only when `sub_E13DA` wipes the ring
    ///   (all zeroes) — anything else is a same-frame lap (overflow).
    /// - `index` moving backwards to 0 with the window bytes unchanged is
    ///   the reloaded driver image resetting `$0036` while the ring is
    ///   still stale (observed mid-reinit shape); a genuine wrap always
    ///   carries changed window bytes.
    pub(crate) fn decode_frame(
        &mut self,
        ring: &[u8; GEMS_RING_LEN],
        index: usize,
    ) -> GemsRingFrameDecode {
        let mut decode = GemsRingFrameDecode::default();
        let window: Vec<usize> = if index >= self.anchor {
            (self.anchor..index).collect()
        } else {
            (self.anchor..GEMS_RING_LEN).chain(0..index).collect()
        };
        let outside_changed_positions: Vec<usize> = (0..GEMS_RING_LEN)
            .filter(|position| !window.contains(position))
            .filter(|&position| ring[position] != self.prev_ring[position])
            .collect();
        let window_changed = window
            .iter()
            .any(|&position| ring[position] != self.prev_ring[position]);

        if !outside_changed_positions.is_empty() {
            // Reinit hypothesis: sub_E13DA wiped the ring (changed bytes
            // outside the fresh content are zero) and the repost rewrote
            // `[0, index)` — which overlaps the OLD window's complement
            // whenever the previous anchor sat inside the fresh content, so
            // nonzero changes below `index` are the repost, not a lap. A
            // repost always begins with the StopMusic posting, so demand
            // `$FF $0B` at position 0 (or an empty fresh window mid-wipe).
            let reinit_consistent = outside_changed_positions
                .iter()
                .all(|&position| ring[position] == 0 || position < index)
                && (index == 0 || (ring[0] == 0xFF && ring[1] == 0x0B));
            if !reinit_consistent {
                decode.queue_overflow = true;
                self.pending.clear();
                self.anchor = index;
                self.prev_ring = *ring;
                return decode;
            }
            // sub_E13DA zero-fill: reinit. The repost (constant bytes from
            // ring position 0) may land in the same snapshot or later. A
            // wipe or repost also verifies any pending index-reset.
            self.reset_awaiting_verification = false;
            decode.ring_reinit = true;
            self.pending.clear();
            self.anchor = 0;
            if index > 0 {
                let bytes: Vec<u8> = (0..index).map(|position| ring[position]).collect();
                self.consume(&bytes, &mut decode);
                self.anchor = index;
            }
            self.prev_ring = *ring;
            return decode;
        }

        if index == self.anchor {
            self.prev_ring = *ring;
            return decode;
        }

        if index < self.anchor && index == 0 && !window_changed {
            // Hypothesised reinit index-reset with a stale ring. From the
            // snapshot alone this is indistinguishable from a wrap that
            // posted byte-identical content ending exactly at position 0
            // (which would silently drop that command), so the next
            // contentful snapshot must confirm it: a wipe or a `$FF $0B`
            // repost verifies; anything else flags `reset_unverified`.
            decode.ring_reinit = true;
            self.reset_awaiting_verification = true;
            self.pending.clear();
            self.anchor = 0;
            self.prev_ring = *ring;
            return decode;
        }

        if self.reset_awaiting_verification {
            self.reset_awaiting_verification = false;
            let is_stop_music_repost =
                self.anchor == 0 && index >= 2 && ring[0] == 0xFF && ring[1] == 0x0B;
            if !is_stop_music_repost {
                decode.reset_unverified = true;
                self.pending.clear();
                self.anchor = index;
                self.prev_ring = *ring;
                return decode;
            }
        }

        let bytes: Vec<u8> = window.iter().map(|&position| ring[position]).collect();
        self.consume(&bytes, &mut decode);
        self.anchor = index;
        self.prev_ring = *ring;
        decode
    }

    /// Feed newly posted bytes through the `[$FF][opcode][params]` framer.
    fn consume(&mut self, bytes: &[u8], decode: &mut GemsRingFrameDecode) {
        for &byte in bytes {
            if self.pending.is_empty() {
                if byte == 0xFF {
                    self.pending.push(byte);
                } else {
                    // A non-marker byte where a command must start: the
                    // stream is unframeable.
                    decode.unknown_opcode = Some(byte);
                    return;
                }
                continue;
            }
            self.pending.push(byte);
            let opcode = self.pending[1];
            let Some(total) = gems_command_posted_len(opcode) else {
                decode.unknown_opcode = Some(opcode);
                self.pending.clear();
                return;
            };
            if self.pending.len() == total {
                decode.commands.push(GemsRingCommand {
                    opcode,
                    args: self.pending[2..].to_vec(),
                });
                self.pending.clear();
            }
        }
    }
}

/// Projection name for the command-parity comparison: per-frame ordered
/// equality of the decoded 68k→Z80 mailbox command stream. Authoritative
/// audio gate per the #416 supersession decision.
pub(crate) const GEMS_COMMAND_PARITY_PROJECTION_NAME: &str = "gems_command_parity_v1";

/// One command with the frame its final posted byte landed in.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct GemsTimedCommand {
    pub(crate) frame_index: usize,
    pub(crate) command: GemsRingCommand,
}

/// One genuine command-stream divergence. `native`/`oracle` is `None` for
/// an insertion on the other side.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct GemsCommandMismatch {
    pub(crate) native: Option<GemsTimedCommand>,
    pub(crate) oracle: Option<GemsTimedCommand>,
}

/// Result of [`compare_gems_command_streams`].
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct GemsCommandComparison {
    pub(crate) compared_frames: usize,
    pub(crate) native_command_count: usize,
    pub(crate) oracle_command_count: usize,
    /// Commands identical on both sides at the identical frame.
    pub(crate) matched_exact: usize,
    /// Commands identical on both sides but attributed to adjacent frames:
    /// both sides sample at the same raster line, but a command posted at
    /// the boundary instant can quantise to either neighbour. Split by
    /// direction because the two counts must be pinned EXACTLY by the
    /// baseline — a real one-frame native scheduling shift converts
    /// reconciled pairs into exact pairs (or flips their direction), so a
    /// mere downward-only ratchet would read that regression as an
    /// improvement.
    pub(crate) reconciled_skew_commands: usize,
    /// Reconciled pairs where the native attribution is the earlier frame.
    pub(crate) reconciled_native_earlier: usize,
    /// Reconciled pairs where the oracle attribution is the earlier frame.
    pub(crate) reconciled_oracle_earlier: usize,
    /// Genuine divergences (value, ordering, presence, or frame drift > 1).
    pub(crate) mismatch_count: usize,
    pub(crate) first_mismatch: Option<GemsCommandMismatch>,
    pub(crate) mismatch_digest: Option<String>,
    /// StopMusic re-initialisations observed per side (evidence).
    pub(crate) native_reinit_frames: usize,
    pub(crate) oracle_reinit_frames: usize,
}

/// Decode one side's ring snapshots into a timed command sequence.
/// Fails on any unclean frame (unknown opcode / ring overflow): the
/// comparison must never run over a stream the decoder could not frame.
pub(crate) fn decode_ring_stream(
    rings: &[Option<wildside_core::GemsFrameRing>],
    side: &str,
) -> anyhow::Result<(Vec<GemsTimedCommand>, usize)> {
    let mut decoder = GemsRingDecoder::default();
    let mut commands = Vec::new();
    let mut reinit_frames = 0usize;
    for (frame_index, capture) in rings.iter().enumerate() {
        let Some(capture) = capture else { continue };
        let decode = decoder.decode_manifest_ring(capture);
        if !decode.is_clean() {
            anyhow::bail!(
                "{side} GEMS ring stream failed to decode at frame {frame_index} \
                 (unknown_opcode={:?}, queue_overflow={}, reset_unverified={})",
                decode.unknown_opcode,
                decode.queue_overflow,
                decode.reset_unverified
            );
        }
        if decode.ring_reinit {
            reinit_frames += 1;
        }
        for command in decode.commands {
            commands.push(GemsTimedCommand {
                frame_index,
                command,
            });
        }
    }
    Ok((commands, reinit_frames))
}

/// Strict ordered comparison of the two decoded command sequences with a
/// ±1-frame reconciliation window for the 8-scanline sampling skew.
///
/// Alignment is a greedy two-pointer walk with single-step lookahead for
/// insertions/deletions: equal commands pair (frame drift 0 = exact, 1 =
/// reconciled, >1 = genuine mismatch); unequal commands consult one
/// command of lookahead to classify one side as an insertion before
/// falling back to a paired value mismatch.
pub(crate) fn compare_gems_command_streams(
    native_rings: &[Option<wildside_core::GemsFrameRing>],
    oracle_rings: &[Option<wildside_core::GemsFrameRing>],
) -> anyhow::Result<GemsCommandComparison> {
    use sha1::{Digest, Sha1};
    let compared_frames = native_rings.len().min(oracle_rings.len());
    let (native, native_reinit_frames) =
        decode_ring_stream(&native_rings[..compared_frames], "native")?;
    let (oracle, oracle_reinit_frames) =
        decode_ring_stream(&oracle_rings[..compared_frames], "oracle")?;

    let mut matched_exact = 0usize;
    let mut reconciled_skew_commands = 0usize;
    let mut reconciled_native_earlier = 0usize;
    let mut reconciled_oracle_earlier = 0usize;
    let mut mismatches: Vec<GemsCommandMismatch> = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < native.len() && j < oracle.len() {
        let n = &native[i];
        let o = &oracle[j];
        if n.command == o.command {
            let drift = n.frame_index.abs_diff(o.frame_index);
            match drift {
                0 => matched_exact += 1,
                1 => {
                    reconciled_skew_commands += 1;
                    if n.frame_index < o.frame_index {
                        reconciled_native_earlier += 1;
                    } else {
                        reconciled_oracle_earlier += 1;
                    }
                }
                _ => mismatches.push(GemsCommandMismatch {
                    native: Some(n.clone()),
                    oracle: Some(o.clone()),
                }),
            }
            i += 1;
            j += 1;
            continue;
        }
        let native_is_insertion = native
            .get(i + 1)
            .is_some_and(|next| next.command == o.command);
        let oracle_is_insertion = oracle
            .get(j + 1)
            .is_some_and(|next| next.command == n.command);
        if native_is_insertion && !oracle_is_insertion {
            mismatches.push(GemsCommandMismatch {
                native: Some(n.clone()),
                oracle: None,
            });
            i += 1;
        } else if oracle_is_insertion && !native_is_insertion {
            mismatches.push(GemsCommandMismatch {
                native: None,
                oracle: Some(o.clone()),
            });
            j += 1;
        } else {
            mismatches.push(GemsCommandMismatch {
                native: Some(n.clone()),
                oracle: Some(o.clone()),
            });
            i += 1;
            j += 1;
        }
    }
    for n in &native[i..] {
        mismatches.push(GemsCommandMismatch {
            native: Some(n.clone()),
            oracle: None,
        });
    }
    for o in &oracle[j..] {
        mismatches.push(GemsCommandMismatch {
            native: None,
            oracle: Some(o.clone()),
        });
    }

    let mismatch_digest = (!mismatches.is_empty()).then(|| {
        let mut hasher = Sha1::new();
        for mismatch in &mismatches {
            for side in [&mismatch.native, &mismatch.oracle] {
                match side {
                    Some(timed) => {
                        hasher.update([1u8, timed.command.opcode]);
                        hasher.update((timed.frame_index as u64).to_be_bytes());
                        hasher.update(&timed.command.args);
                    }
                    None => hasher.update([0u8]),
                }
            }
        }
        format!("{:x}", hasher.finalize())
    });

    Ok(GemsCommandComparison {
        compared_frames,
        native_command_count: native.len(),
        oracle_command_count: oracle.len(),
        matched_exact,
        reconciled_skew_commands,
        reconciled_native_earlier,
        reconciled_oracle_earlier,
        mismatch_count: mismatches.len(),
        first_mismatch: mismatches.first().cloned(),
        mismatch_digest,
        native_reinit_frames,
        oracle_reinit_frames,
    })
}

/// Minimal manifest view: only the per-frame audio-RAM dumps and ring
/// snapshots from `oracle-probe --capture-gems-state` (serde skips the rest
/// of the manifest, including the heavyweight `m68k_pc_trace`).
#[derive(serde::Deserialize)]
struct GemsRamManifest {
    #[serde(default)]
    gems_frame_audio_ram: Vec<Option<Vec<u8>>>,
    #[serde(default)]
    gems_frame_rings: Vec<Option<wildside_core::GemsFrameRing>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct GemsRingFrameEvents {
    pub(crate) frame_index: usize,
    #[serde(flatten)]
    pub(crate) decode: GemsRingFrameDecode,
}

/// Decode every captured frame of an oracle manifest and print the decoded
/// event stream plus an opcode census. Returns an error when any frame is
/// unclean (unknown opcode or overflow) — the vocabulary must be fixed from
/// data before any comparison work trusts the decoder.
pub(crate) fn run_gems_ring_decode(
    manifest_path: &std::path::Path,
    output_path: Option<&std::path::Path>,
) -> anyhow::Result<()> {
    use anyhow::Context;
    let content = std::fs::read(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: GemsRamManifest = serde_json::from_slice(&content)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    if manifest.gems_frame_audio_ram.is_empty() {
        anyhow::bail!(
            "{} has no gems_frame_audio_ram frames; capture with oracle-probe --capture-gems-state",
            manifest_path.display()
        );
    }

    let mut decoder = GemsRingDecoder::default();
    let mut events: Vec<GemsRingFrameEvents> = Vec::new();
    let mut opcode_census: std::collections::BTreeMap<u8, usize> =
        std::collections::BTreeMap::new();
    let mut unclean = 0usize;
    let mut captured = 0usize;
    for (frame_index, ram) in manifest.gems_frame_audio_ram.iter().enumerate() {
        let Some(ram) = ram else { continue };
        captured += 1;
        let decode = decoder.decode_ram_frame(ram);
        if decode.commands.is_empty() && decode.is_clean() && !decode.ring_reinit {
            continue;
        }
        for command in &decode.commands {
            *opcode_census.entry(command.opcode).or_default() += 1;
        }
        if !decode.is_clean() {
            unclean += 1;
        }
        events.push(GemsRingFrameEvents {
            frame_index,
            decode,
        });
    }

    // Cross-check (#435): when the manifest also carries the compact ring
    // snapshots, decoding them must reproduce the full-RAM decode exactly —
    // the rings are what full-route verify-tas runs ship instead of 8 KiB
    // per frame, so any divergence invalidates the compact path.
    if !manifest.gems_frame_rings.is_empty() && !manifest.gems_frame_audio_ram.is_empty() {
        let mut ring_decoder = GemsRingDecoder::default();
        let mut ring_events: Vec<GemsRingFrameEvents> = Vec::new();
        for (frame_index, capture) in manifest.gems_frame_rings.iter().enumerate() {
            let Some(capture) = capture else { continue };
            let decode = ring_decoder.decode_manifest_ring(capture);
            if decode.commands.is_empty() && decode.is_clean() && !decode.ring_reinit {
                continue;
            }
            ring_events.push(GemsRingFrameEvents {
                frame_index,
                decode,
            });
        }
        let matches = ring_events.len() == events.len()
            && ring_events.iter().zip(events.iter()).all(|(ring, ram)| {
                ring.frame_index == ram.frame_index && ring.decode == ram.decode
            });
        if !matches {
            anyhow::bail!(
                "ring-snapshot decode diverges from full-RAM decode ({} vs {} event frames)",
                ring_events.len(),
                events.len()
            );
        }
        println!(
            "  ring-snapshot cross-check: {} event frames match the full-RAM decode",
            ring_events.len()
        );
    }

    println!(
        "gems-ring-decode: {} captured frames, {} event frames, {} unclean",
        captured,
        events.len(),
        unclean
    );
    for event in &events {
        let mut parts: Vec<String> = event
            .decode
            .commands
            .iter()
            .map(|command| {
                format!(
                    "{:02X}[{}]",
                    command.opcode,
                    command
                        .args
                        .iter()
                        .map(|byte| format!("{byte:02X}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            })
            .collect();
        if event.decode.ring_reinit {
            parts.insert(0, "REINIT".to_owned());
        }
        if event.decode.queue_overflow {
            parts.push("OVERFLOW".to_owned());
        }
        if let Some(opcode) = event.decode.unknown_opcode {
            parts.push(format!("UNKNOWN({opcode:02X})"));
        }
        println!("  frame {:>6}: {}", event.frame_index, parts.join(" "));
    }
    println!("  opcode census:");
    for (opcode, count) in &opcode_census {
        println!("    {opcode:02X}: {count}");
    }

    if let Some(path) = output_path {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(&events)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
        println!("  events: {}", path.display());
    }

    if unclean > 0 {
        anyhow::bail!("{unclean} frame(s) failed to decode cleanly (unknown opcode or overflow)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_with(bytes: &[(usize, u8)]) -> [u8; GEMS_RING_LEN] {
        let mut ring = [0u8; GEMS_RING_LEN];
        for &(position, byte) in bytes {
            ring[position % GEMS_RING_LEN] = byte;
        }
        ring
    }

    fn write(ring: &mut [u8; GEMS_RING_LEN], start: usize, bytes: &[u8]) -> usize {
        for (offset, &byte) in bytes.iter().enumerate() {
            ring[(start + offset) % GEMS_RING_LEN] = byte;
        }
        (start + bytes.len()) % GEMS_RING_LEN
    }

    /// The 14-byte StopMusic posting with the ROM's constant pointers
    /// (unk_E160E, unk_E25EA, unk_E2766, unk_E8716; kid.asm loc_E14C8).
    const STOP_MUSIC: [u8; 14] = [
        0xFF, 0x0B, 0x0E, 0x16, 0x0E, 0xEA, 0x25, 0x0E, 0x66, 0x27, 0x0E, 0x16, 0x87, 0x0E,
    ];

    #[test]
    fn decodes_boot_stop_music_posting_from_zero_ring() {
        // ASM loc_E14C8 via sub_E13DA: observed at capture frame 16.
        let mut decoder = GemsRingDecoder::default();
        let empty = ring_with(&[]);
        assert_eq!(
            decoder.decode_frame(&empty, 0),
            GemsRingFrameDecode::default()
        );

        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &STOP_MUSIC);
        let decode = decoder.decode_frame(&ring, index);
        assert!(decode.is_clean());
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
        assert_eq!(decode.commands[0].args.len(), 12);
        assert_eq!(&decode.commands[0].args[..3], &[0x0E, 0x16, 0x0E]);
    }

    #[test]
    fn multi_frame_reinit_matches_observed_capture_shape() {
        // Observed capture frames 30..32: index resets with a stale ring,
        // the zero-fill lands next frame, then the identical repost.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &STOP_MUSIC);
        decoder.decode_frame(&ring, index);

        // Frame 30 shape: index 14 -> 0, ring unchanged.
        let decode = decoder.decode_frame(&ring, 0);
        assert!(
            decode.ring_reinit,
            "index reset with stale ring is a reinit"
        );
        assert!(decode.commands.is_empty());

        // Frame 31 shape: ring zero-filled, index still 0.
        let wiped = ring_with(&[]);
        let decode = decoder.decode_frame(&wiped, 0);
        assert!(decode.commands.is_empty());
        assert!(decode.is_clean());

        // Frame 32 shape: repost of the identical 14 bytes.
        let mut repost = ring_with(&[]);
        let index = write(&mut repost, 0, &STOP_MUSIC);
        let decode = decoder.decode_frame(&repost, index);
        assert!(decode.is_clean());
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
    }

    #[test]
    fn same_frame_reinit_and_start_sequence_decode_together() {
        // Observed capture frames 68..70: after the index reset the repost
        // AND a PlaySound land inside one frame window ([0,17)).
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &STOP_MUSIC);
        decoder.decode_frame(&ring, index);
        let decode = decoder.decode_frame(&ring, 0);
        assert!(decode.ring_reinit);

        let mut repost = ring_with(&[]);
        let mut index = write(&mut repost, 0, &STOP_MUSIC);
        index = write(&mut repost, index, &[0xFF, 0x10, 0x03]);
        let decode = decoder.decode_frame(&repost, index);
        assert!(decode.is_clean());
        assert_eq!(
            decode.commands,
            vec![
                GemsRingCommand {
                    opcode: 0x0B,
                    args: STOP_MUSIC[2..].to_vec()
                },
                GemsRingCommand {
                    opcode: 0x10,
                    args: vec![0x03]
                },
            ]
        );
    }

    #[test]
    fn reinit_zero_fill_erasing_old_tail_is_not_overflow() {
        // Observed capture frames 139..140: the wipe zeroes bytes past the
        // repost window (the old FF 10 03 tail) in the same snapshot.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let mut index = write(&mut ring, 0, &STOP_MUSIC);
        index = write(&mut ring, index, &[0xFF, 0x10, 0x03]);
        decoder.decode_frame(&ring, index);
        let decode = decoder.decode_frame(&ring, 0);
        assert!(decode.ring_reinit);

        let mut repost = ring_with(&[]);
        let index = write(&mut repost, 0, &STOP_MUSIC);
        let decode = decoder.decode_frame(&repost, index);
        assert!(
            decode.is_clean(),
            "zero-filled tail is the reinit wipe, not a lap"
        );
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
    }

    #[test]
    fn wraps_around_the_ring_boundary() {
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        // Advance the anchor near the end of the ring with filler posts.
        let mut index = 0;
        for _ in 0..20 {
            index = write(&mut ring, index, &[0xFF, 0x10, 0x01]);
            decoder.decode_frame(&ring, index);
        }
        assert_eq!(index, 60);
        // A 3-byte command crossing 63 -> 0.
        index = write(&mut ring, index, &[0xFF, 0x12, 0x07]);
        assert_eq!(index, 63);
        index = write(&mut ring, index, &[0xFF, 0x10, 0x02]);
        assert_eq!(index, 2);
        let decode = decoder.decode_frame(&ring, index);
        assert!(decode.is_clean());
        assert_eq!(
            decode.commands,
            vec![
                GemsRingCommand {
                    opcode: 0x12,
                    args: vec![0x07]
                },
                GemsRingCommand {
                    opcode: 0x10,
                    args: vec![0x02]
                },
            ]
        );
    }

    #[test]
    fn partial_command_carries_to_the_completing_frame() {
        // sub_E148E commits the index per byte, so a snapshot can catch a
        // half-posted command; it must attribute to the completing frame.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &[0xFF, 0x10]);
        let decode = decoder.decode_frame(&ring, index);
        assert!(decode.commands.is_empty());
        assert!(decode.is_clean());

        let index = write(&mut ring, index, &[0x04]);
        let decode = decoder.decode_frame(&ring, index);
        assert_eq!(
            decode.commands,
            vec![GemsRingCommand {
                opcode: 0x10,
                args: vec![0x04]
            }]
        );
    }

    #[test]
    fn unknown_opcode_fails_loudly() {
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &[0xFF, 0x1E, 0x00, 0x00]);
        let decode = decoder.decode_frame(&ring, index);
        assert_eq!(decode.unknown_opcode, Some(0x1E));
        assert!(!decode.is_clean());
    }

    #[test]
    fn ring_lap_is_flagged_as_overflow() {
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &[0xFF, 0x10, 0x01]);
        decoder.decode_frame(&ring, index);
        // Overwrite the whole ring with different non-zero content and a
        // short forward index move: outside-window bytes changed without a
        // wipe or a StopMusic repost at 0.
        let mut lapped = [0x21u8; GEMS_RING_LEN];
        lapped[0] = 0x33;
        let decode = decoder.decode_frame(&lapped, 6);
        assert!(decode.queue_overflow);
        assert!(!decode.is_clean());
    }

    fn rings_from_events(
        total_frames: usize,
        events: &[(usize, &[u8])],
    ) -> Vec<Option<wildside_core::GemsFrameRing>> {
        let mut ring = [0u8; GEMS_RING_LEN];
        let mut index = 0usize;
        let mut out = Vec::with_capacity(total_frames);
        for frame in 0..total_frames {
            for &(event_frame, bytes) in events {
                if event_frame == frame {
                    index = write(&mut ring, index, bytes);
                }
            }
            out.push(Some(wildside_core::GemsFrameRing {
                bytes: ring.to_vec(),
                write_index: index as u8,
            }));
        }
        out
    }

    #[test]
    fn reinit_repost_overlapping_small_anchor_is_not_overflow() {
        // Full-route shape that first surfaced at frame 4967: the previous
        // anchor sits INSIDE the fresh repost content, so nonzero repost
        // bytes land "outside" the old window. Must classify as reinit.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &[0xFF, 0x10, 0x01]);
        let decode = decoder.decode_frame(&ring, index);
        assert_eq!(decode.commands.len(), 1);

        let mut repost = ring_with(&[]);
        let index = write(&mut repost, 0, &STOP_MUSIC);
        let decode = decoder.decode_frame(&repost, index);
        assert!(
            decode.is_clean(),
            "repost over a small anchor is a reinit, not a lap"
        );
        assert!(decode.ring_reinit);
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
    }

    #[test]
    fn byte_identical_wrap_to_zero_fails_loudly_without_repost() {
        // A wrap posting byte-identical content that ends exactly at ring
        // position 0 is indistinguishable from a reinit index-reset at the
        // snapshot where it happens; the decoder hypothesises a reset and
        // demands the next content be a wipe or FF 0B repost. Anything
        // else must fail loudly instead of silently dropping the command.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let mut index = 0usize;
        // First lap: 32 pause commands fill the 64-byte ring exactly.
        for _ in 0..32 {
            index = write(&mut ring, index, &[0xFF, 0x0C]);
            let decode = decoder.decode_frame(&ring, index);
            assert!(decode.is_clean());
        }
        assert_eq!(index, 0);
        // Second lap up to position 62: bytes are identical to the stale
        // first-lap content, so window_changed stays false throughout.
        for _ in 0..31 {
            index = write(&mut ring, index, &[0xFF, 0x0C]);
            decoder.decode_frame(&ring, index);
        }
        assert_eq!(index, 62);
        // The ambiguous posting: identical bytes ending exactly at 0.
        index = write(&mut ring, index, &[0xFF, 0x0C]);
        assert_eq!(index, 0);
        let decode = decoder.decode_frame(&ring, index);
        assert!(
            decode.ring_reinit,
            "reset hypothesised at the ambiguous snapshot"
        );
        // Next content is a plain command, not a wipe/repost: loud failure.
        index = write(&mut ring, index, &[0xFF, 0x0C]);
        let decode = decoder.decode_frame(&ring, index);
        assert!(decode.reset_unverified, "unverified reset must fail loudly");
        assert!(!decode.is_clean());
    }

    #[test]
    fn verified_reset_followed_by_repost_stays_clean() {
        // The genuine counterpart of the ambiguous case above: after the
        // index reset the repost arrives, so the hypothesis verifies.
        let mut decoder = GemsRingDecoder::default();
        let mut ring = ring_with(&[]);
        let index = write(&mut ring, 0, &STOP_MUSIC);
        decoder.decode_frame(&ring, index);
        let decode = decoder.decode_frame(&ring, 0);
        assert!(decode.ring_reinit);
        // Repost identical constants without an intervening wipe snapshot.
        let decode = decoder.decode_frame(&ring, STOP_MUSIC.len());
        assert!(decode.is_clean(), "FF 0B repost verifies the reset");
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
    }

    #[test]
    fn identical_streams_match_exactly() {
        let events: &[(usize, &[u8])] = &[(3, &[0xFF, 0x10, 0x02]), (7, &[0xFF, 0x12, 0x02])];
        let native = rings_from_events(10, events);
        let oracle = rings_from_events(10, events);
        let comparison = compare_gems_command_streams(&native, &oracle).expect("clean streams");
        assert_eq!(comparison.matched_exact, 2);
        assert_eq!(comparison.reconciled_skew_commands, 0);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn one_frame_drift_reconciles_but_two_frames_is_genuine() {
        let native = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02])]);
        let skewed = rings_from_events(10, &[(4, &[0xFF, 0x10, 0x02])]);
        let comparison = compare_gems_command_streams(&native, &skewed).expect("clean streams");
        assert_eq!(comparison.reconciled_skew_commands, 1);
        assert_eq!(comparison.reconciled_native_earlier, 1);
        assert_eq!(comparison.reconciled_oracle_earlier, 0);
        assert_eq!(comparison.mismatch_count, 0);

        let reversed = compare_gems_command_streams(&skewed, &native).expect("clean streams");
        assert_eq!(reversed.reconciled_oracle_earlier, 1);
        assert_eq!(reversed.reconciled_native_earlier, 0);

        let far = rings_from_events(10, &[(6, &[0xFF, 0x10, 0x02])]);
        let comparison = compare_gems_command_streams(&native, &far).expect("clean streams");
        assert_eq!(comparison.reconciled_skew_commands, 0);
        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert_eq!(first.native.expect("native side").frame_index, 3);
        assert_eq!(first.oracle.expect("oracle side").frame_index, 6);
    }

    #[test]
    fn value_and_ordering_divergence_are_genuine_mismatches() {
        let native = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02])]);
        let value = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x05])]);
        let comparison = compare_gems_command_streams(&native, &value).expect("clean streams");
        assert_eq!(comparison.mismatch_count, 1, "sound-id value mismatch");

        let ordered: &[(usize, &[u8])] = &[(3, &[0xFF, 0x10, 0x02]), (3, &[0xFF, 0x12, 0x07])];
        let swapped: &[(usize, &[u8])] = &[(3, &[0xFF, 0x12, 0x07]), (3, &[0xFF, 0x10, 0x02])];
        let comparison = compare_gems_command_streams(
            &rings_from_events(10, ordered),
            &rings_from_events(10, swapped),
        )
        .expect("clean streams");
        assert!(
            comparison.mismatch_count > 0,
            "same-frame command reordering must not silently pass"
        );
    }

    #[test]
    fn one_sided_command_is_a_presence_mismatch() {
        let native = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02]), (5, &[0xFF, 0x12, 0x02])]);
        let oracle = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02])]);
        let comparison = compare_gems_command_streams(&native, &oracle).expect("clean streams");
        assert_eq!(comparison.matched_exact, 1);
        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert!(first.oracle.is_none(), "native-only command");
    }

    #[test]
    fn mid_stream_insertions_classify_one_sided_on_both_sides() {
        // Native has an extra leading command: the lookahead must classify
        // it native-only and still pair the shared command exactly.
        let native = rings_from_events(10, &[(2, &[0xFF, 0x10, 0x09]), (5, &[0xFF, 0x12, 0x02])]);
        let oracle = rings_from_events(10, &[(5, &[0xFF, 0x12, 0x02])]);
        let comparison = compare_gems_command_streams(&native, &oracle).expect("clean streams");
        assert_eq!(comparison.matched_exact, 1);
        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert_eq!(first.native.expect("native side").frame_index, 2);
        assert!(first.oracle.is_none());

        // Mirrored: oracle-only insertion mid-stream.
        let comparison = compare_gems_command_streams(&oracle, &native).expect("clean streams");
        assert_eq!(comparison.matched_exact, 1);
        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert!(first.native.is_none());
        assert_eq!(first.oracle.expect("oracle side").frame_index, 2);
    }

    #[test]
    fn oracle_tail_commands_drain_as_presence_mismatches() {
        let native = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02])]);
        let oracle = rings_from_events(10, &[(3, &[0xFF, 0x10, 0x02]), (7, &[0xFF, 0x12, 0x02])]);
        let comparison = compare_gems_command_streams(&native, &oracle).expect("clean streams");
        assert_eq!(comparison.matched_exact, 1);
        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert!(first.native.is_none(), "oracle-only tail command");
        assert_eq!(first.oracle.expect("oracle side").frame_index, 7);
    }

    #[test]
    fn reinit_frames_are_counted_per_side() {
        // Native initialises once; oracle reinits twice (boot shape).
        let mut oracle = Vec::new();
        let mut ring = [0u8; GEMS_RING_LEN];
        let index = write(&mut ring, 0, &STOP_MUSIC);
        oracle.push(Some(wildside_core::GemsFrameRing {
            bytes: ring.to_vec(),
            write_index: index as u8,
        }));
        // Index reset with stale ring, then wipe, then repost.
        oracle.push(Some(wildside_core::GemsFrameRing {
            bytes: ring.to_vec(),
            write_index: 0,
        }));
        oracle.push(Some(wildside_core::GemsFrameRing {
            bytes: vec![0; GEMS_RING_LEN],
            write_index: 0,
        }));
        oracle.push(Some(wildside_core::GemsFrameRing {
            bytes: ring.to_vec(),
            write_index: index as u8,
        }));
        let (commands, reinits) = decode_ring_stream(&oracle, "oracle").expect("clean");
        assert_eq!(commands.len(), 2, "both StopMusic postings decode");
        assert_eq!(reinits, 2, "index-reset and wipe frames count as reinits");
    }

    #[test]
    fn unclean_stream_is_a_hard_error() {
        let rings = rings_from_events(4, &[(1, &[0xFF, 0x1E, 0x00, 0x00])]);
        let error = compare_gems_command_streams(&rings, &rings_from_events(4, &[]))
            .expect_err("unknown opcode must not compare");
        assert!(error.to_string().contains("failed to decode"));
    }

    #[test]
    fn decode_ram_frame_reads_ring_and_index_from_full_ram() {
        let mut decoder = GemsRingDecoder::default();
        let mut ram = vec![0u8; 0x2000];
        for (offset, &byte) in STOP_MUSIC.iter().enumerate() {
            ram[GEMS_RING_BASE + offset] = byte;
        }
        ram[GEMS_RING_WRITE_INDEX_ADDR] = STOP_MUSIC.len() as u8;
        let decode = decoder.decode_ram_frame(&ram);
        assert!(decode.is_clean());
        assert_eq!(decode.commands.len(), 1);
        assert_eq!(decode.commands[0].opcode, 0x0B);
    }
}
