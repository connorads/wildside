//! #437 bounded-span timing replay: feed the ORACLE's captured 68k→Z80
//! stimulus stream (`oracle-probe --gems-stimulus-frames`, #437 capture) into
//! the native GEMS timing engine, seeded only with reinit-surviving phase
//! scalars, and measure how the timing-emergent signature fields evolve under
//! identical stimuli.
//!
//! The output is a measured-debt report, never a CI gate: a native-vs-oracle
//! span comparison over the frozen-edge debt is expected to show divergence —
//! that IS the measurement. If spans match, the route's 36,836-frame timing
//! debt is proven to be pure 68k-schedule unknowns rather than engine
//! defects; if they diverge, the field census names the defect classes.
//!
//! Modelling choices (documented residue classes, not bugs):
//! - Stimuli are applied at the stamped instruction-OPENING cycle, while the
//!   oracle lands pending writes at instruction close — an intra-instruction
//!   skew of a few 68k cycles per write.
//! - The engine's `pending_m68k_wait_cycles` ledger is drained (discarded)
//!   after every advance because the oracle's cycle stamps already include
//!   the 68000 wait time jgenesis inserted; the ledger is left undrained
//!   only across the final advance to each `FrameRendered` sample so a wait
//!   queued inside the crossing quantum stays visible like the oracle's.
//! - Between the seed anchor (StopMusic entry) and the reinit's BUSREQ
//!   window, up to a couple of Z80 instructions execute divergent code; the
//!   reinit erases everything except the wait/parity/bank ledgers those
//!   instructions could touch.

use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
use std::path::Path;

use crate::oracle::{OracleGemsStimulusRow, OracleManifest};
use crate::tas_gems::{
    advisory_fields_diverge, signature_from_checkpoint, strict_diverging_fields,
};
use wildside_md::{GemsMasterClockTimingEngine, GemsNtscTimelineOrigin, GemsReplaySeed};

const MASTER_CLOCKS_PER_SCANLINE: u64 = 3_420;
const NTSC_SCANLINES_PER_FRAME: u64 = 262;
/// jgenesis `FrameRendered` fires when the VDP enters scanline 232 (VBlank
/// start 224 + 8 bottom-border lines) — same instant the signature streams
/// sample (see `tas_gems.rs` module docs).
const FRAME_RENDERED_SCANLINE: u64 = 232;
const MCLK_PER_M68K_CYCLE: u64 = 7;

fn frame_sample_mclk(frame: usize, captured: Option<u64>) -> Result<(u64, bool)> {
    let nominal = (frame as u64 * NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SCANLINE)
        * MASTER_CLOCKS_PER_SCANLINE;
    let Some(captured) = captured else {
        return Ok((nominal, false));
    };
    ensure!(
        captured >= nominal,
        "oracle GEMS sample clock for frame {frame} precedes FrameRendered: {captured} < {nominal}"
    );
    Ok((captured, true))
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct GemsSpanFrameDiff {
    pub(crate) frame: usize,
    pub(crate) strict_fields: Vec<&'static str>,
    pub(crate) advisory_diverges: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct GemsSpanReport {
    pub(crate) manifest: String,
    /// Sensitivity-probe offset added to every stimulus stamp (68k cycles),
    /// approximating instruction-close application; 0 = raw opening stamps.
    pub(crate) stamp_skew_cycles: u64,
    pub(crate) anchor_frame: usize,
    pub(crate) anchor_pc: u32,
    pub(crate) anchor_m68k_cycles: u64,
    pub(crate) seed: GemsReplaySeed,
    pub(crate) start_frame: usize,
    pub(crate) end_frame: usize,
    pub(crate) stimulus_count: usize,
    /// Number of frames sampled at the oracle's captured instruction-close
    /// instant rather than the legacy nominal scanline-232 boundary.
    pub(crate) oracle_sample_clock_frames: usize,
    pub(crate) frames_compared: usize,
    pub(crate) exact_frames: usize,
    pub(crate) advisory_only_frames: usize,
    pub(crate) strict_mismatch_frames: usize,
    pub(crate) first_strict_divergence: Option<GemsSpanFrameDiff>,
    pub(crate) first_divergence_native: Option<wildside_core::GemsFrameSignature>,
    pub(crate) first_divergence_oracle: Option<wildside_core::GemsFrameSignature>,
    /// strict field name → number of diverging frames.
    pub(crate) field_census: BTreeMap<&'static str, usize>,
    pub(crate) frame_diffs: Vec<GemsSpanFrameDiff>,
}

fn apply_stimulus_row(engine: &mut GemsMasterClockTimingEngine, row: &OracleGemsStimulusRow) {
    match row.kind.as_str() {
        "ram_write" => engine.apply_68k_z80_ram_write(row.addr, row.value),
        "busreq" => engine.apply_68k_busreq_bit(row.value != 0),
        "reset" => engine.apply_68k_reset_bit(row.value != 0),
        other => panic!("unknown captured stimulus kind {other:?}"),
    }
}

/// Replay `[start_frame, end_frame]` from the anchor's seed and stimuli,
/// comparing the projected signature against the oracle's per-frame stream.
pub(crate) fn replay_gems_span(
    manifest: &OracleManifest,
    manifest_label: &str,
    rom: &[u8],
    anchor_frame: usize,
    start_frame: usize,
    end_frame: usize,
    stamp_skew_cycles: u64,
) -> Result<GemsSpanReport> {
    ensure!(
        anchor_frame < start_frame && start_frame <= end_frame,
        "span must start after the anchor frame ({anchor_frame} !< {start_frame} <= {end_frame})"
    );
    let anchor = manifest
        .m68k_pc_trace
        .iter()
        .find(|row| row.frame == anchor_frame && row.gems_checkpoint.is_some())
        .with_context(|| {
            format!("no pc-trace row with a GEMS checkpoint at anchor frame {anchor_frame}")
        })?;
    let checkpoint = anchor
        .gems_checkpoint
        .as_ref()
        .expect("anchor row filtered on checkpoint presence");
    ensure!(
        !checkpoint.z80_halt && !checkpoint.vdp_owns_bus && !checkpoint.z80_stalled,
        "anchor checkpoint must be a plain running boundary (no VDP halt/stall)"
    );
    ensure!(
        checkpoint.bank_current_bit == 0,
        "anchor falls mid-way through a Z80 bank-register write sequence"
    );

    let anchor_m68k_cycles = anchor.m68k_cycles_before_instruction;
    let seed = GemsReplaySeed {
        master_clock: anchor_m68k_cycles * MCLK_PER_M68K_CYCLE,
        bank_address: checkpoint.bank_number << 15,
        z80_mclk_counter: checkpoint.z80_mclk_counter,
        z80_t_cycles_wait: checkpoint.z80_t_cycles_wait,
        z80_wait_mclk_cycles: checkpoint.z80_wait_mclk_cycles,
        z80_odd_access: checkpoint.z80_odd_access,
        ym2612_mclk_counter: checkpoint.ym2612_mclk_counter,
        ym2612_input_mclk_counter: checkpoint.ym2612_mclk_counter % 7,
        pending_m68k_wait_cycles: checkpoint.m68k_wait_cpu_cycles,
        busreq: checkpoint.z80_busreq,
        reset: checkpoint.z80_reset,
    };

    let mut engine = GemsMasterClockTimingEngine::from_kid_rom_with_ntsc_timeline_origin(
        rom,
        GemsNtscTimelineOrigin::PowerOnScanlineZero,
    )?;
    engine.seed_replay_phase(&seed);

    let stimuli: Vec<&OracleGemsStimulusRow> = manifest
        .gems_stimuli
        .iter()
        .filter(|row| {
            row.frame >= anchor_frame
                && row.frame <= end_frame
                && row.m68k_cycle_counter >= anchor_m68k_cycles
        })
        .collect();
    ensure!(
        !stimuli.is_empty(),
        "no captured stimuli in frames {anchor_frame}..={end_frame}; \
         re-run oracle-probe with --gems-stimulus-frames over the span"
    );

    let mut stimuli_iter = stimuli.iter().peekable();
    let mut frame_diffs = Vec::new();
    let mut field_census: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut first_strict_divergence = None;
    let mut first_divergence_native = None;
    let mut first_divergence_oracle = None;
    let mut frames_compared = 0usize;
    let mut exact_frames = 0usize;
    let mut advisory_only_frames = 0usize;
    let mut strict_mismatch_frames = 0usize;
    let mut oracle_sample_clock_frames = 0usize;

    for frame in start_frame..=end_frame {
        let captured_sample_mclk = manifest
            .gems_frame_sample_mclk
            .get(frame)
            .copied()
            .flatten();
        let (sample_mclk, used_oracle_sample_clock) =
            frame_sample_mclk(frame, captured_sample_mclk)?;
        oracle_sample_clock_frames += usize::from(used_oracle_sample_clock);
        while let Some(row) = stimuli_iter.peek() {
            let stimulus_mclk = (row.m68k_cycle_counter + stamp_skew_cycles) * MCLK_PER_M68K_CYCLE;
            if stimulus_mclk >= sample_mclk {
                break;
            }
            if stimulus_mclk > engine.master_clock() {
                engine.advance_ntsc_master_clocks(stimulus_mclk - engine.master_clock());
                let _ = engine.take_pending_m68k_wait_cycles();
            }
            apply_stimulus_row(&mut engine, row);
            stimuli_iter.next();
        }
        if engine.master_clock() < sample_mclk {
            // No drain across the sampling advance: a wait queued inside the
            // crossing quantum stays visible, matching the oracle sampler.
            engine.advance_ntsc_master_clocks(sample_mclk - engine.master_clock());
        }
        let native = signature_from_checkpoint(&engine.checkpoint());
        let _ = engine.take_pending_m68k_wait_cycles();

        let Some(Some(oracle)) = manifest.gems_frame_signatures.get(frame) else {
            continue;
        };
        frames_compared += 1;
        let strict_fields = strict_diverging_fields(&native, oracle);
        let advisory_diverges = advisory_fields_diverge(&native, oracle);
        if strict_fields.is_empty() {
            if advisory_diverges {
                advisory_only_frames += 1;
            } else {
                exact_frames += 1;
            }
        } else {
            strict_mismatch_frames += 1;
            for field in &strict_fields {
                *field_census.entry(field).or_default() += 1;
            }
            let diff = GemsSpanFrameDiff {
                frame,
                strict_fields,
                advisory_diverges,
            };
            if first_strict_divergence.is_none() {
                first_strict_divergence = Some(diff.clone());
                first_divergence_native = Some(native);
                first_divergence_oracle = Some(*oracle);
            }
            frame_diffs.push(diff);
        }
    }
    ensure!(
        frames_compared != 0,
        "no oracle signatures over frames {start_frame}..={end_frame}; \
         capture with --capture-gems-state"
    );

    Ok(GemsSpanReport {
        manifest: manifest_label.to_string(),
        stamp_skew_cycles,
        anchor_frame,
        anchor_pc: anchor.pc,
        anchor_m68k_cycles,
        seed,
        start_frame,
        end_frame,
        stimulus_count: stimuli.len(),
        oracle_sample_clock_frames,
        frames_compared,
        exact_frames,
        advisory_only_frames,
        strict_mismatch_frames,
        first_strict_divergence,
        first_divergence_native,
        first_divergence_oracle,
        field_census,
        frame_diffs,
    })
}

/// `gems-span-replay` subcommand body (#437 deliverable).
pub(crate) fn run_gems_span_replay(
    repo_root: &Path,
    rom_path: &Path,
    manifest_path: &Path,
    anchor_frame: usize,
    start_frame: usize,
    end_frame: usize,
    stamp_skew_cycles: u64,
    output: Option<&Path>,
) -> Result<()> {
    let manifest: OracleManifest = serde_json::from_slice(
        &std::fs::read(manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?,
    )
    .context("invalid oracle manifest JSON")?;
    let rom = std::fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;

    println!(
        "gems-span-replay: anchor {anchor_frame}, span {start_frame}..={end_frame} ({})",
        manifest_path.display()
    );
    let report = replay_gems_span(
        &manifest,
        &manifest_path.display().to_string(),
        &rom,
        anchor_frame,
        start_frame,
        end_frame,
        stamp_skew_cycles,
    )?;

    let output_path = output.map_or_else(
        || {
            repo_root.join(format!(
                ".cache/audit/gems-span-replay-{start_frame}-{end_frame}.json"
            ))
        },
        Path::to_path_buf,
    );
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, serde_json::to_vec_pretty(&report)?)?;

    println!(
        "  {} stimuli, {} frames compared: {} exact, {} advisory-only, {} strict-mismatch",
        report.stimulus_count,
        report.frames_compared,
        report.exact_frames,
        report.advisory_only_frames,
        report.strict_mismatch_frames,
    );
    match &report.first_strict_divergence {
        Some(diff) => println!(
            "  first strict divergence: frame {} in [{}]",
            diff.frame,
            diff.strict_fields.join(", ")
        ),
        None => println!("  no strict divergence over the span"),
    }
    for (field, count) in &report.field_census {
        println!("    {field}: {count} frames");
    }
    println!("  report: {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_instruction_close_clock_replaces_the_nominal_frame_boundary() {
        let nominal =
            (871 * NTSC_SCANLINES_PER_FRAME + FRAME_RENDERED_SCANLINE) * MASTER_CLOCKS_PER_SCANLINE;
        assert_eq!(frame_sample_mclk(871, None).unwrap(), (nominal, false));
        assert_eq!(
            frame_sample_mclk(871, Some(nominal + 2)).unwrap(),
            (nominal + 2, true)
        );
        assert!(frame_sample_mclk(871, Some(nominal - 1)).is_err());
    }
}
