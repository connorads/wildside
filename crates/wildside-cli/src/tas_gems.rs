//! Route-wide per-frame GEMS/Z80 audio-timing parity
//! (`gems_frame_signature_v1`).
//!
//! The oracle helper samples a [`GemsFrameSignature`] at every
//! `FrameRendered` boundary; this module produces the native stream from the
//! persistent GEMS timing engine driven by the same trace→command replayer as
//! audio rendering, then compares the two streams like the per-frame RNG
//! pass. v1 deliberately measures divergence rather than claiming parity: see
//! [`GEMS_MODELLED_ROUTE_FRAMES`].
//!
//! Sampling instant (see [`GemsFrameSignature`] for the derivation): the
//! oracle's `FrameRendered` fires when the VDP enters scanline 232 — VBlank
//! start (224) plus the pinned jgenesis `NTSC_BOTTOM_BORDER` (8) — so the
//! native probe samples its engine 8 scanlines past each VBlank frame
//! boundary. Both offsets come from the pinned VDP raster geometry; nothing
//! is fitted. The oracle additionally quantises to the end of the 68000
//! instruction quantum that crossed the line; that residual jitter is why
//! `z80_mclk_counter` and `ym2612_mclk_counter` are advisory.

use anyhow::Result;
use sha1::{Digest, Sha1};
use wildside_core::{
    GemsFrameSignature, GemsStopMusicBusAckPolls, M68kRefreshPhase, NativeTraceFrame,
    gems_stop_music_elapsed_timing,
};
use wildside_data::AssetManifest;
use wildside_frame::{
    NativeAudioDirector, NativeAudioSink, TimedGemsStopMusicCursor, TimedGemsTitleLoc700Cursor,
    apply_gems_power_on_post_clear_reset, apply_gems_power_on_z80_startup,
};
use wildside_md::{GemsMasterClockTimingEngine, GemsNtscTimelineOrigin, GemsTimingCheckpoint};

const MASTER_CLOCKS_PER_SCANLINE: u64 = 3_420;
const NTSC_SCANLINES_PER_FRAME: u64 = 262;
/// jgenesis `NTSC_BOTTOM_BORDER`: `FrameRendered` fires when the VDP enters
/// scanline 232, eight scanlines after the scanline-224 VBlank frame boundary
/// the native timeline is aligned to.
const FRAME_RENDERED_SKEW_SCANLINES: u64 = 8;

/// Frames of the route whose native per-frame GEMS timeline is defended as
/// faithful. Frames 0..24 replay ASM `EntryPoint`'s power-on Z80 bootstrap,
/// post-clear reset, and the progressive first `StopMusic` driver copy
/// through frame 15. The tail remains the
/// coarse continuous route replayer (trace-delta commands at frame boundaries
/// plus whole-frame advances) and is measured debt, never claimed parity.
/// Raise only with per-frame evidence from a seeded run.
pub(crate) const GEMS_MODELLED_ROUTE_FRAMES: usize = 25;

/// Signature fields compared strictly (a difference is a mismatch).
const STRICT_FIELDS: &[&str] = &[
    "audio_ram_crc32",
    "bank_address",
    "z80_pc",
    "z80_wait_mclk_cycles",
    "pending_m68k_wait_cycles",
    "status_bits",
];

/// Divider-phase remainders re-sampled by the oracle at a 68000 instruction
/// boundary; they carry that boundary's jitter on every frame, so they are
/// captured and reported but never gate.
const ADVISORY_FIELDS: &[&str] = &["z80_mclk_counter", "ym2612_mclk_counter"];

fn signature_from_checkpoint(checkpoint: &GemsTimingCheckpoint) -> GemsFrameSignature {
    let mut ram_crc = crc32fast::Hasher::new();
    ram_crc.update(&checkpoint.audio_ram);
    GemsFrameSignature {
        audio_ram_crc32: ram_crc.finalize(),
        bank_address: checkpoint.bank_address,
        z80_pc: checkpoint.z80_pc,
        z80_mclk_counter: checkpoint.z80_mclk_counter,
        z80_wait_mclk_cycles: checkpoint.z80_wait_mclk_cycles,
        pending_m68k_wait_cycles: checkpoint.pending_m68k_wait_cycles,
        ym2612_mclk_counter: checkpoint.ym2612_mclk_counter,
        status_bits: GemsFrameSignature::pack_status_bits(
            checkpoint.z80_halt,
            checkpoint.busreq,
            checkpoint.reset,
            checkpoint.vdp_owns_bus,
            checkpoint.z80_iff1,
            checkpoint.z80_halted,
        ),
    }
}

/// Persistent GEMS engine wrapped so the per-frame sample lands on the
/// oracle's `FrameRendered` raster line while the timeline still advances
/// exactly 262 scanlines per frame.
///
/// Protocol per frame: the [`NativeAudioDirector`] applies that frame's
/// commands and calls [`NativeAudioSink::step_frame`], which advances to the
/// next VBlank frame boundary; the extractor then calls
/// [`GemsSignatureProbe::sample_at_frame_rendered`], which advances the
/// 8-scanline skew and snapshots. The following `step_frame` advances only
/// the remaining 254 scanlines, so commands are applied 8 scanlines into
/// VBlank — the same region where the real V_Int handler writes the GEMS
/// queue.
struct GemsSignatureProbe {
    engine: GemsMasterClockTimingEngine,
    skew_outstanding: bool,
    power_on_frame: usize,
    power_on_stop_music: Option<TimedGemsStopMusicCursor>,
    power_on_vram_clear: Option<TimedGemsTitleLoc700Cursor>,
}

impl GemsSignatureProbe {
    fn from_kid_rom_power_on(rom: &[u8]) -> Result<Self> {
        let mut engine = GemsMasterClockTimingEngine::from_kid_rom_with_ntsc_timeline_origin(
            rom,
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        )?;
        apply_gems_power_on_z80_startup(&mut engine, rom)
            .expect("fresh power-on GEMS engine accepts EntryPoint startup schedule");
        Ok(Self {
            engine,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
        })
    }

    fn from_kid_rom_coarse(rom: &[u8]) -> Result<Self> {
        Ok(Self {
            engine: GemsMasterClockTimingEngine::from_kid_rom(rom)?,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
        })
    }

    fn sample_power_on_frame_rendered(&mut self) -> GemsFrameSignature {
        // jgenesis FrameRendered fires at scanline 232. Absolute power-on
        // clocks therefore sample frame N at N*262+232 scanlines.
        let target = (self.power_on_frame as u64 * NTSC_SCANLINES_PER_FRAME
            + (NTSC_SCANLINES_PER_FRAME - 30))
            * MASTER_CLOCKS_PER_SCANLINE;
        if self.power_on_frame == 3 {
            apply_gems_power_on_post_clear_reset(&mut self.engine)
                .expect("frame 3 crosses EntryPoint's post-clear reset write");
        }
        if self.power_on_frame == 14 {
            // ASM StopMusic $E14C8 enters at the selected-PC cycle below with
            // SR=$2708, so VINT stays masked throughout this first call.
            const ENTRY_CYCLES: u32 = 1_886_313;
            let entry_master_clock = u64::from(ENTRY_CYCLES) * 7;
            self.engine
                .advance_ntsc_master_clocks(entry_master_clock - self.engine.master_clock());
            let timing = gems_stop_music_elapsed_timing(
                GemsStopMusicBusAckPolls {
                    driver_copy_sub_e13da: 1,
                    queue_transactions_sub_e143a: [1; 6],
                },
                M68kRefreshPhase::new(46).expect("selected-PC refresh phase"),
                &[],
            )
            .expect("masked first StopMusic source schedule");
            self.power_on_stop_music =
                Some(TimedGemsStopMusicCursor::new(timing, entry_master_clock));
        }
        if self.power_on_frame == 16 {
            // ASM loc_65C -> sub_6E24 -> sub_914/Stop_z80 $E13BC and
            // sub_924/Start_z80 $E13D0 briefly arbitrate the driver before
            // loc_700 begins the full-VRAM clear. Selected-PC tracing pins
            // BUSREQ after `$E13BC` completes and its release after `$E13D0`
            // completes.
            const SUB_E13BC_ENTRY_CYCLES: u64 = 2_117_935;
            const SUB_E13BC_BUS_REQUEST_CYCLES: u64 = 2_117_957;
            const SUB_E13D0_ENTRY_CYCLES: u64 = 2_118_213;
            const SUB_E13D0_BUS_RELEASE_CYCLES: u64 = 2_118_233;
            const LOC_700_ENTRY_CYCLES: u64 = 2_118_345;
            let entry_master_clock = LOC_700_ENTRY_CYCLES * 7;
            let stop_music = self
                .power_on_stop_music
                .as_mut()
                .expect("first StopMusic precedes loc_700");
            stop_music
                .advance_to_master_clock(&mut self.engine, 2_112_337 * 7)
                .expect("frame 15 reaches first sub_E143A monotonically");
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (1_902, 4, 2),
                "oracle first sub_E143A Z80 timing"
            );
            stop_music
                .advance_to_master_clock(&mut self.engine, 2_115_739 * 7)
                .expect("frame 15 reaches StopMusic $E150C monotonically");
            let checkpoint = self.engine.checkpoint();
            assert_eq!(
                (
                    checkpoint.z80_pc,
                    checkpoint.z80_mclk_counter,
                    checkpoint.z80_t_cycles_wait,
                ),
                (65, 13, 13),
                "oracle StopMusic $E150C Z80 timing"
            );
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13BC_ENTRY_CYCLES * 7)
                .expect("first StopMusic reaches sub_E13BC monotonically");
            assert_eq!(self.engine.z80_pc(), 469, "oracle sub_E13BC Z80 PC");
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13BC_BUS_REQUEST_CYCLES * 7)
                .expect("sub_E13BC reaches its BUSREQ completion monotonically");
            self.engine.set_z80_control(true, false);
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13D0_ENTRY_CYCLES * 7)
                .expect("sub_E13BC reaches sub_E13D0 monotonically");
            assert_eq!(self.engine.z80_pc(), 677, "oracle sub_E13D0 Z80 PC");
            stop_music
                .advance_to_master_clock(&mut self.engine, SUB_E13D0_BUS_RELEASE_CYCLES * 7)
                .expect("sub_E13D0 reaches its BUSREQ release monotonically");
            self.engine.set_z80_control(false, false);
            stop_music
                .advance_to_master_clock(&mut self.engine, entry_master_clock)
                .expect("sub_E13D0 reaches loc_700 monotonically");
            assert_eq!(self.engine.z80_pc(), 479, "oracle loc_700 Z80 PC");
            self.power_on_stop_music = None;
            self.power_on_vram_clear = Some(
                TimedGemsTitleLoc700Cursor::new(
                    wildside_core::vdp::LoadInGameCpuRasterCursor {
                        beam: wildside_core::vdp::H40BeamPosition::new(143, 2_715)
                            .expect("oracle loc_700 beam"),
                        refresh_phase: M68kRefreshPhase::new(14)
                            .expect("oracle loc_700 refresh phase"),
                    },
                    entry_master_clock,
                )
                .expect("ASM loc_700 hardware schedule"),
            );
        }
        if let Some(cursor) = &mut self.power_on_vram_clear {
            cursor
                .advance_to_master_clock(&mut self.engine, target)
                .expect("power-on loc_700 cursor advances monotonically");
        } else if let Some(cursor) = &mut self.power_on_stop_music {
            cursor
                .advance_to_master_clock(&mut self.engine, target)
                .expect("power-on StopMusic cursor advances monotonically");
        } else {
            self.engine
                .advance_ntsc_master_clocks(target - self.engine.master_clock());
        }
        self.power_on_frame += 1;
        signature_from_checkpoint(&self.engine.checkpoint())
    }

    fn sample_at_frame_rendered(&mut self) -> GemsFrameSignature {
        assert!(
            !self.skew_outstanding,
            "sample_at_frame_rendered must follow a step_frame"
        );
        self.engine
            .advance_ntsc_master_clocks(FRAME_RENDERED_SKEW_SCANLINES * MASTER_CLOCKS_PER_SCANLINE);
        self.skew_outstanding = true;
        signature_from_checkpoint(&self.engine.checkpoint())
    }
}

impl NativeAudioSink for GemsSignatureProbe {
    fn start_sequence(&mut self, sound_id: u8) {
        self.engine.start_sequence(sound_id);
    }

    fn stop_sequence(&mut self, sound_id: u8) {
        self.engine.stop_sequence(sound_id);
    }

    fn stop_music(&mut self) {
        self.engine.stop_music();
    }

    fn set_music_speed(&mut self, speed: u8) {
        self.engine.set_music_speed(speed);
    }

    fn step_frame(&mut self) {
        let scanlines = if std::mem::take(&mut self.skew_outstanding) {
            NTSC_SCANLINES_PER_FRAME - FRAME_RENDERED_SKEW_SCANLINES
        } else {
            NTSC_SCANLINES_PER_FRAME
        };
        self.engine
            .advance_ntsc_master_clocks(scanlines * MASTER_CLOCKS_PER_SCANLINE);
    }

    fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        vec![0.0; sample_count * 2]
    }

    fn requires_continuous_clock(&self) -> bool {
        true
    }
}

/// Drive the audio replayer over the whole trace with the timing engine as
/// the sink, sampling one signature per frame at the oracle's
/// `FrameRendered` raster line.
pub(crate) fn extract_native_gems_frame_signatures(
    rom: &[u8],
    manifest: &AssetManifest,
    trace: &[NativeTraceFrame],
) -> Result<Vec<Option<GemsFrameSignature>>> {
    let mut power_on_probe = GemsSignatureProbe::from_kid_rom_power_on(rom)?;
    let mut coarse_probe = GemsSignatureProbe::from_kid_rom_coarse(rom)?;
    let mut director = NativeAudioDirector::new();
    Ok(trace
        .iter()
        .enumerate()
        .map(|(frame_index, frame)| {
            director.step(manifest, frame, &mut coarse_probe);
            let coarse_signature = coarse_probe.sample_at_frame_rendered();
            if frame_index < GEMS_MODELLED_ROUTE_FRAMES {
                Some(power_on_probe.sample_power_on_frame_rendered())
            } else {
                Some(coarse_signature)
            }
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct GemsFrameMismatch {
    pub(crate) frame_index: usize,
    /// Strict fields that diverge ("presence" when one side has no sample).
    pub(crate) diverging_fields: Vec<&'static str>,
    pub(crate) native: Option<GemsFrameSignature>,
    pub(crate) oracle: Option<GemsFrameSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GemsFrameComparison {
    pub(crate) start_frame: Option<usize>,
    pub(crate) compared_frames: usize,
    pub(crate) mismatch_count: usize,
    /// Frames whose only strict residual is the documented jgenesis
    /// undocumented-X/Y-flag `SUB` Z80 PC divergence.
    pub(crate) oracle_limitation_count: usize,
    /// Frames where only the advisory divider phases differ.
    pub(crate) advisory_only_frames: usize,
    pub(crate) first_mismatch: Option<GemsFrameMismatch>,
    pub(crate) mismatch_digest: Option<String>,
}

fn strict_diverging_fields(
    native: &GemsFrameSignature,
    oracle: &GemsFrameSignature,
) -> Vec<&'static str> {
    let differs = [
        native.audio_ram_crc32 != oracle.audio_ram_crc32,
        native.bank_address != oracle.bank_address,
        native.z80_pc != oracle.z80_pc,
        native.z80_wait_mclk_cycles != oracle.z80_wait_mclk_cycles,
        native.pending_m68k_wait_cycles != oracle.pending_m68k_wait_cycles,
        native.status_bits != oracle.status_bits,
    ];
    STRICT_FIELDS
        .iter()
        .zip(differs)
        .filter_map(|(field, differs)| differs.then_some(*field))
        .collect()
}

fn advisory_fields_diverge(native: &GemsFrameSignature, oracle: &GemsFrameSignature) -> bool {
    native.z80_mclk_counter != oracle.z80_mclk_counter
        || native.ym2612_mclk_counter != oracle.ym2612_mclk_counter
}

/// ORACLE LIMITATION: jgenesis preserves stale undocumented X/Y bits for Z80
/// subtract-family instructions, while hardware and the native core copy
/// bits 5/3 from the result. Two exact consequences are classified here:
/// GEMS `CP B` at $01E2 shifts IRQ boundaries and their `$003E` saved-SP byte
/// on frames 16, 19, 20, and 22; later `SUB 1` at $07B3 leaves the oracle at
/// PC $02AA where hardware reaches $02AE. Every other strict field must be
/// byte-equal.
fn gems_divergence_matches_documented_z80_xy_flag_case(
    frame_index: usize,
    native: &GemsFrameSignature,
    oracle: &GemsFrameSignature,
    diverging: &[&'static str],
) -> bool {
    let irq_saved_sp = diverging == ["audio_ram_crc32"]
        && matches!(
            (
                frame_index,
                native.audio_ram_crc32,
                oracle.audio_ram_crc32,
                native.z80_pc,
                oracle.z80_pc,
            ),
            (16, 580_638_908, 502_849_585, 0x07AA, 0x07AA)
                | (19, 1_005_120_388, 76_272_393, 0x07A9, 0x07A9)
                | (20, 3_170_182_089, 2_207_652_676, 0x0C2B, 0x0C2B)
                | (22, 2_567_830_830, 2_792_277_411, 0x01D2, 0x01D2)
        );
    let later_pc = diverging == ["z80_pc"] && native.z80_pc == 0x02AE && oracle.z80_pc == 0x02AA;
    irq_saved_sp || later_pc
}

fn update_gems_frame_digest(hasher: &mut Sha1, signature: Option<&GemsFrameSignature>) {
    match signature {
        Some(signature) => {
            hasher.update([1]);
            hasher.update(signature.audio_ram_crc32.to_be_bytes());
            hasher.update(signature.bank_address.to_be_bytes());
            hasher.update(signature.z80_pc.to_be_bytes());
            hasher.update(signature.z80_wait_mclk_cycles.to_be_bytes());
            hasher.update(signature.pending_m68k_wait_cycles.to_be_bytes());
            hasher.update([signature.status_bits]);
            // Advisory divider phases are deliberately excluded: they carry
            // the oracle's instruction-boundary jitter, not model fidelity.
        }
        None => hasher.update([0]),
    }
}

pub(crate) fn compare_gems_frame_signatures(
    native: &[Option<GemsFrameSignature>],
    oracle: &[Option<GemsFrameSignature>],
    range_start: usize,
) -> GemsFrameComparison {
    let shared_len = native.len().min(oracle.len());
    let start_frame = (range_start.min(shared_len)..shared_len)
        .find(|&index| native[index].is_some() && oracle[index].is_some());
    let Some(start) = start_frame else {
        return GemsFrameComparison {
            start_frame: None,
            compared_frames: 0,
            mismatch_count: 0,
            oracle_limitation_count: 0,
            advisory_only_frames: 0,
            first_mismatch: None,
            mismatch_digest: None,
        };
    };

    let mut hasher = Sha1::new();
    let mut mismatch_count = 0usize;
    let mut oracle_limitation_count = 0usize;
    let mut advisory_only_frames = 0usize;
    let mut first_mismatch = None;
    for frame_index in start..shared_len {
        let native_signature = native[frame_index].as_ref();
        let oracle_signature = oracle[frame_index].as_ref();
        let diverging: Vec<&'static str> = match (native_signature, oracle_signature) {
            (Some(native), Some(oracle)) => {
                let diverging = strict_diverging_fields(native, oracle);
                if diverging.is_empty() {
                    if advisory_fields_diverge(native, oracle) {
                        advisory_only_frames += 1;
                    }
                    continue;
                }
                if gems_divergence_matches_documented_z80_xy_flag_case(
                    frame_index,
                    native,
                    oracle,
                    &diverging,
                ) {
                    oracle_limitation_count += 1;
                    continue;
                }
                diverging
            }
            (None, None) => continue,
            _ => vec!["presence"],
        };

        mismatch_count += 1;
        hasher.update((frame_index as u64).to_be_bytes());
        update_gems_frame_digest(&mut hasher, native_signature);
        update_gems_frame_digest(&mut hasher, oracle_signature);
        first_mismatch.get_or_insert_with(|| GemsFrameMismatch {
            frame_index,
            diverging_fields: diverging,
            native: native_signature.copied(),
            oracle: oracle_signature.copied(),
        });
    }

    GemsFrameComparison {
        start_frame,
        compared_frames: shared_len - start,
        mismatch_count,
        oracle_limitation_count,
        advisory_only_frames,
        first_mismatch,
        mismatch_digest: (mismatch_count != 0).then(|| format!("{:x}", hasher.finalize())),
    }
}

impl GemsFrameComparison {
    /// Report block mirroring `rng_frame_comparison`, with explicit projection
    /// honesty metadata and coverage.
    pub(crate) fn report_json(
        &self,
        route_frames: usize,
        native_frames: usize,
        oracle_frames: usize,
    ) -> serde_json::Value {
        serde_json::json!({
            "projection": wildside_core::GEMS_FRAME_SIGNATURE_PROJECTION_NAME,
            "proves_ym_writes": false,
            "proves_sample_audio": false,
            "proves_intra_frame_ordering": false,
            "advisory_fields": ADVISORY_FIELDS,
            "coverage": {
                "route_frames": route_frames,
                "modelled_frames": GEMS_MODELLED_ROUTE_FRAMES,
            },
            "frame_counts": {
                "requested": route_frames,
                "native": native_frames,
                "oracle": oracle_frames,
            },
            "start_frame": self.start_frame,
            "compared_frames": self.compared_frames,
            "mismatch_count": self.mismatch_count,
            "oracle_limitation_count": self.oracle_limitation_count,
            "advisory_only_frames": self.advisory_only_frames,
            "mismatch_digest": self.mismatch_digest,
            "first_mismatch": self.first_mismatch,
        })
    }

    pub(crate) fn console_summary(&self) -> String {
        match &self.first_mismatch {
            Some(first) => format!(
                "{} mismatches across {} compared frames ({} X/Y-flag oracle limitations, {} advisory-only); first at frame {} ({})",
                self.mismatch_count,
                self.compared_frames,
                self.oracle_limitation_count,
                self.advisory_only_frames,
                first.frame_index,
                first.diverging_fields.join(","),
            ),
            None => format!(
                "all {} compared frames match ({} X/Y-flag oracle limitations, {} advisory-only)",
                self.compared_frames, self.oracle_limitation_count, self.advisory_only_frames,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(z80_pc: u16) -> GemsFrameSignature {
        GemsFrameSignature {
            audio_ram_crc32: 0x1234_5678,
            bank_address: 0x000E_0000,
            z80_pc,
            z80_mclk_counter: 8,
            z80_wait_mclk_cycles: 0,
            pending_m68k_wait_cycles: 0,
            ym2612_mclk_counter: 18,
            status_bits: GemsFrameSignature::STATUS_IFF1,
        }
    }

    #[test]
    fn comparison_starts_when_both_sides_have_samples() {
        let native = vec![None, Some(signature(0x0793)), Some(signature(0x0793))];
        let oracle = vec![Some(signature(0x0100)), None, Some(signature(0x0793))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.start_frame, Some(2));
        assert_eq!(comparison.compared_frames, 1);
        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.first_mismatch, None);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn injected_strict_divergence_is_caught_with_field_evidence() {
        let mut diverged = signature(0x0793);
        diverged.audio_ram_crc32 ^= 1;
        diverged.status_bits |= GemsFrameSignature::STATUS_HALTED;
        let native = vec![Some(signature(0x0793)), Some(signature(0x0793))];
        let oracle = vec![Some(signature(0x0793)), Some(diverged)];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 1);
        let first = comparison.first_mismatch.expect("mismatch evidence");
        assert_eq!(first.frame_index, 1);
        assert_eq!(
            first.diverging_fields,
            vec!["audio_ram_crc32", "status_bits"]
        );
        assert!(comparison.mismatch_digest.is_some());
    }

    #[test]
    fn documented_xy_flag_pc_residual_reclassifies_as_oracle_limitation() {
        // ORACLE LIMITATION: jgenesis SUB X/Y-flag preservation leaves the
        // oracle at $02AA where hardware-faithful execution reaches $02AE.
        let native = vec![Some(signature(0x02AE))];
        let oracle = vec![Some(signature(0x02AA))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
        assert_eq!(comparison.mismatch_digest, None);

        // The reversed pair is NOT the documented case and stays a mismatch.
        let reversed = compare_gems_frame_signatures(
            &[Some(signature(0x02AA))],
            &[Some(signature(0x02AE))],
            0,
        );
        assert_eq!(reversed.mismatch_count, 1);
    }

    #[test]
    fn xy_flag_predicate_requires_every_other_strict_field_equal() {
        let mut oracle_signature = signature(0x02AA);
        oracle_signature.bank_address ^= 0x8000;

        let comparison =
            compare_gems_frame_signatures(&[Some(signature(0x02AE))], &[Some(oracle_signature)], 0);

        assert_eq!(comparison.oracle_limitation_count, 0);
        assert_eq!(comparison.mismatch_count, 1);
        assert_eq!(
            comparison
                .first_mismatch
                .expect("mismatch")
                .diverging_fields,
            vec!["bank_address", "z80_pc"],
        );
    }

    #[test]
    fn frame_16_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x07AA);
        native.audio_ram_crc32 = 580_638_908;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 502_849_585;
        let mut native_frames = vec![None; 16];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 16];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_19_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x07A9);
        native.audio_ram_crc32 = 1_005_120_388;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 76_272_393;
        let mut native_frames = vec![None; 19];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 19];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_20_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x0C2B);
        native.audio_ram_crc32 = 3_170_182_089;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 2_207_652_676;
        let mut native_frames = vec![None; 20];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 20];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn frame_22_saved_sp_crc_is_the_documented_xy_flag_oracle_limitation() {
        let mut native = signature(0x01D2);
        native.audio_ram_crc32 = 2_567_830_830;
        let mut oracle = native;
        oracle.audio_ram_crc32 = 2_792_277_411;
        let mut native_frames = vec![None; 22];
        native_frames.push(Some(native));
        let mut oracle_frames = vec![None; 22];
        oracle_frames.push(Some(oracle));

        let comparison = compare_gems_frame_signatures(&native_frames, &oracle_frames, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.oracle_limitation_count, 1);
        assert_eq!(comparison.first_mismatch, None);
    }

    #[test]
    fn advisory_divider_phase_jitter_never_gates_but_is_counted() {
        let mut jittered = signature(0x0793);
        jittered.z80_mclk_counter = 11;
        jittered.ym2612_mclk_counter = 3;
        let native = vec![Some(signature(0x0793))];
        let oracle = vec![Some(jittered)];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 0);
        assert_eq!(comparison.advisory_only_frames, 1);
        assert_eq!(comparison.mismatch_digest, None);
    }

    #[test]
    fn missing_sample_on_one_side_is_a_presence_mismatch() {
        let native = vec![Some(signature(0x0793)), None];
        let oracle = vec![Some(signature(0x0793)), Some(signature(0x0793))];

        let comparison = compare_gems_frame_signatures(&native, &oracle, 0);

        assert_eq!(comparison.mismatch_count, 1);
        assert_eq!(
            comparison
                .first_mismatch
                .expect("mismatch")
                .diverging_fields,
            vec!["presence"],
        );
    }

    #[test]
    fn digest_covers_every_mismatch_not_only_the_first() {
        let mut diverged = signature(0x0793);
        diverged.audio_ram_crc32 ^= 1;
        let native = vec![Some(signature(0x0793)); 3];
        let baseline = compare_gems_frame_signatures(
            &native,
            &[Some(signature(0x0793)), Some(diverged), Some(diverged)],
            0,
        );
        let mut moved = diverged;
        moved.audio_ram_crc32 ^= 2;
        let changed_tail = compare_gems_frame_signatures(
            &native,
            &[Some(signature(0x0793)), Some(diverged), Some(moved)],
            0,
        );

        assert_eq!(baseline.mismatch_count, 2);
        assert_eq!(changed_tail.mismatch_count, 2);
        assert_ne!(
            baseline.mismatch_digest, changed_tail.mismatch_digest,
            "digest must include every mismatched frame's evidence",
        );
    }

    #[test]
    fn probe_samples_on_the_frame_rendered_raster_line() {
        // The oracle samples at VDP scanline 232 = VBlank start + 8 bottom
        // border lines; the probe must land each sample exactly there while
        // the timeline still advances 262 scanlines per frame.
        let mut probe = GemsSignatureProbe {
            engine: GemsMasterClockTimingEngine::new(&[0x76], vec![]),
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
        };

        probe.step_frame();
        let first = probe.engine.master_clock();
        probe.sample_at_frame_rendered();
        assert_eq!(first, NTSC_SCANLINES_PER_FRAME * MASTER_CLOCKS_PER_SCANLINE);
        assert_eq!(
            probe.engine.master_clock(),
            (NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SKEW_SCANLINES) * MASTER_CLOCKS_PER_SCANLINE
        );

        probe.step_frame();
        probe.sample_at_frame_rendered();
        assert_eq!(
            probe.engine.master_clock(),
            (2 * NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SKEW_SCANLINES)
                * MASTER_CLOCKS_PER_SCANLINE,
            "the skew is a sampling offset, not extra timeline",
        );
    }

    #[test]
    fn probe_signature_reports_halted_driver_status() {
        // Driver [0x76] executes HALT immediately; by the first sample the
        // signature must carry STATUS_HALTED with interrupts disabled.
        let mut probe = GemsSignatureProbe {
            engine: GemsMasterClockTimingEngine::new(&[0x76], vec![]),
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
        };
        probe.step_frame();
        let signature = probe.sample_at_frame_rendered();

        assert_eq!(
            signature.status_bits & GemsFrameSignature::STATUS_HALTED,
            GemsFrameSignature::STATUS_HALTED
        );
        assert_eq!(signature.status_bits & GemsFrameSignature::STATUS_IFF1, 0);
        assert_eq!(signature.z80_pc, 1);
    }

    #[test]
    fn power_on_probe_matches_entrypoint_z80_bootstrap_at_frames_zero_and_three() {
        // ASM EntryPoint Z80InitLoop $0500 copies Z80StartupCodeBegin; $0532
        // reasserts reset after ClrRAMLoop. These strict fields come from the
        // published-route oracle's FrameRendered samples, not fitted phases.
        const STARTUP: [u8; 38] = [
            0xAF, 0x01, 0xD9, 0x1F, 0x11, 0x27, 0x00, 0x21, 0x26, 0x00, 0xF9, 0x77, 0xED, 0xB0,
            0xDD, 0xE1, 0xFD, 0xE1, 0xED, 0x47, 0xED, 0x4F, 0xD1, 0xE1, 0xF1, 0x08, 0xD9, 0xC1,
            0xD1, 0xE1, 0xF1, 0xF9, 0xF3, 0xED, 0x56, 0x36, 0xE9, 0xE9,
        ];
        let mut rom = vec![0; 0x059A];
        rom[0x0574..0x059A].copy_from_slice(&STARTUP);
        let mut engine = GemsMasterClockTimingEngine::new_with_ntsc_timeline_origin(
            &[0x76],
            vec![],
            GemsNtscTimelineOrigin::PowerOnScanlineZero,
        );
        apply_gems_power_on_z80_startup(&mut engine, &rom).expect("source startup schedule");
        let mut probe = GemsSignatureProbe {
            engine,
            skew_outstanding: false,
            power_on_frame: 0,
            power_on_stop_music: None,
            power_on_vram_clear: None,
        };

        let frame_zero = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_zero.audio_ram_crc32, 2_155_335_881);
        assert_eq!(frame_zero.z80_pc, 0x000C);
        assert_eq!(frame_zero.status_bits, 0);

        probe.sample_power_on_frame_rendered();
        probe.sample_power_on_frame_rendered();
        let frame_three = probe.sample_power_on_frame_rendered();
        assert_eq!(frame_three.audio_ram_crc32, 3_532_775_288);
        assert_eq!(frame_three.z80_pc, 0);
        assert_eq!(frame_three.status_bits, GemsFrameSignature::STATUS_RESET);
    }

    #[test]
    fn power_on_probe_matches_stop_music_and_loc_700_through_frame_24() {
        // ASM StopMusic -> sub_E13DA -> loc_E1400/loc_E1406, then loc_700.
        // Expected signatures are oracle FrameRendered samples except for the
        // documented hardware-correct CP X/Y flag consequences.
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.cache/upstream/kid-chameleon-disasm/kid.bin"),
        )
        .expect("bootstrapped Kid ROM");
        let mut probe = GemsSignatureProbe::from_kid_rom_power_on(&rom).expect("power-on probe");
        let signatures: Vec<_> = (0..25)
            .map(|_| probe.sample_power_on_frame_rendered())
            .collect();
        assert_eq!(signatures[14].audio_ram_crc32, 687_583_812);
        assert_eq!(signatures[14].z80_pc, 0);
        assert_eq!(signatures[14].pending_m68k_wait_cycles, 1);
        assert_eq!(
            signatures[14].status_bits,
            GemsFrameSignature::STATUS_BUSREQ
        );

        assert_eq!(signatures[15].audio_ram_crc32, 2_882_818_897);
        assert_eq!(signatures[15].z80_pc, 0);
        assert_eq!(signatures[15].pending_m68k_wait_cycles, 0);
        assert_eq!(
            signatures[15].status_bits,
            GemsFrameSignature::STATUS_BUSREQ
        );

        assert_eq!(signatures[16].z80_pc, 1_962);
        assert_eq!(signatures[16].audio_ram_crc32, 580_638_908);
        assert_eq!(signatures[16].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[16].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[17].z80_pc, 677);
        assert_eq!(signatures[17].audio_ram_crc32, 3_802_706_905);
        assert_eq!(signatures[17].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[17].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[18].z80_pc, 2_891);
        assert_eq!(signatures[18].audio_ram_crc32, 3_729_737_286);
        assert_eq!(signatures[18].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[18].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[19].z80_pc, 1_961);
        assert_eq!(signatures[19].audio_ram_crc32, 1_005_120_388);
        assert_eq!(signatures[19].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[19].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[20].z80_pc, 3_115);
        assert_eq!(signatures[20].audio_ram_crc32, 3_170_182_089);
        assert_eq!(signatures[20].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[20].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[21].z80_pc, 677);
        assert_eq!(signatures[21].audio_ram_crc32, 1_434_734_151);
        assert_eq!(signatures[21].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[21].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[22].z80_pc, 466);
        assert_eq!(signatures[22].audio_ram_crc32, 2_567_830_830);
        assert_eq!(signatures[22].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[22].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[23].z80_pc, 2_884);
        assert_eq!(signatures[23].audio_ram_crc32, 1_929_709_730);
        assert_eq!(signatures[23].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[23].status_bits, GemsFrameSignature::STATUS_IFF1);

        assert_eq!(signatures[24].z80_pc, 2_888);
        assert_eq!(signatures[24].audio_ram_crc32, 1_700_834_292);
        assert_eq!(signatures[24].pending_m68k_wait_cycles, 0);
        assert_eq!(signatures[24].status_bits, GemsFrameSignature::STATUS_IFF1);
    }
}
