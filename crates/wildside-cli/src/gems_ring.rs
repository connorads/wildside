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
}

impl GemsRingFrameDecode {
    pub(crate) fn is_clean(&self) -> bool {
        !self.queue_overflow && self.unknown_opcode.is_none()
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
            let wiped = outside_changed_positions
                .iter()
                .all(|&position| ring[position] == 0);
            if !wiped {
                decode.queue_overflow = true;
                self.pending.clear();
                self.anchor = index;
                self.prev_ring = *ring;
                return decode;
            }
            // sub_E13DA zero-fill: reinit. The repost (constant bytes from
            // ring position 0) may land in the same snapshot or later.
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
            // Reinit index-reset with a stale ring (wipe and repost arrive
            // in later snapshots).
            decode.ring_reinit = true;
            self.pending.clear();
            self.anchor = 0;
            self.prev_ring = *ring;
            return decode;
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
