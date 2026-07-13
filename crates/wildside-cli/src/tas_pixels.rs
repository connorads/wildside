use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};

use crate::native_player::{
    NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene, load_session_assets,
    render_native_vdp_rgb, render_scene, render_trace_with_vdp_latched_state,
    render_vdp_trace_overlay,
};
use crate::oracle::{OracleFrame, OracleManifest};
use crate::scenarios::{
    DynamicScenario, NativeDynamicRunOptions, try_for_each_native_dynamic_frame,
};
use wildside_core::{NativeRuntime, NativeTraceFrame, terrain::BlockGrid};
use wildside_data::AssetManifest;
use wildside_render::NativeLevelVisuals;

pub(crate) const TAS_PIXEL_PROJECTION_NAME: &str = "source_port_visible_rgb24_v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelFrameEvidence {
    frame_index: usize,
    label: String,
    changed_pixels: usize,
    native_sha1: String,
    oracle_sha1: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelFrameCounts {
    requested: usize,
    native: usize,
    oracle: usize,
    compared: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelComparisonReport {
    projection: &'static str,
    frame_range: [usize; 2],
    frame_counts: TasPixelFrameCounts,
    proves_requested_frame_pixels: bool,
    proves_every_frame_in_range: bool,
    proves_whole_state: bool,
    exact_match_frames: usize,
    mismatch_frames: usize,
    total_changed_pixels: u64,
    mismatch_digest: String,
    first_mismatch: Option<TasPixelFrameEvidence>,
    worst_mismatch: Option<TasPixelFrameEvidence>,
}

impl TasPixelComparisonReport {
    pub(crate) fn console_summary(&self) -> String {
        let coverage = if self.proves_every_frame_in_range {
            "every frame in range"
        } else {
            "requested checkpoints"
        };
        match &self.first_mismatch {
            Some(first) => format!(
                "{} compared ({coverage}); {} exact, {} divergent; first mismatch frame {} ({} changed pixels)",
                self.frame_counts.compared,
                self.exact_match_frames,
                self.mismatch_frames,
                first.frame_index,
                first.changed_pixels,
            ),
            None => format!(
                "{} compared ({coverage}); all pixels exact",
                self.frame_counts.compared
            ),
        }
    }
}

#[derive(Default)]
struct PixelComparisonAccumulator {
    compared_frames: usize,
    exact_match_frames: usize,
    mismatch_frames: usize,
    total_changed_pixels: u64,
    digest: Sha1,
    first_mismatch: Option<TasPixelFrameEvidence>,
    worst_mismatch: Option<TasPixelFrameEvidence>,
}

impl PixelComparisonAccumulator {
    fn observe(
        &mut self,
        frame_index: usize,
        label: &str,
        native_rgb: &[u8],
        oracle_rgb: &[u8],
    ) -> Result<()> {
        if native_rgb.len() != oracle_rgb.len() || !native_rgb.len().is_multiple_of(3) {
            bail!(
                "TAS pixel frame {frame_index} has incompatible RGB lengths: native={} oracle={}",
                native_rgb.len(),
                oracle_rgb.len()
            );
        }

        let changed_pixels = crate::diff::pixel_mismatch_count(native_rgb, oracle_rgb);
        self.compared_frames += 1;
        if changed_pixels == 0 {
            self.exact_match_frames += 1;
            return Ok(());
        }

        self.mismatch_frames += 1;
        self.total_changed_pixels = self
            .total_changed_pixels
            .saturating_add(changed_pixels as u64);
        self.digest.update(frame_index.to_le_bytes());
        self.digest.update(changed_pixels.to_le_bytes());
        let evidence = TasPixelFrameEvidence {
            frame_index,
            label: label.to_owned(),
            changed_pixels,
            native_sha1: sha1_hex(native_rgb),
            oracle_sha1: sha1_hex(oracle_rgb),
        };
        self.first_mismatch.get_or_insert_with(|| evidence.clone());
        if self
            .worst_mismatch
            .as_ref()
            .is_none_or(|worst| changed_pixels > worst.changed_pixels)
        {
            self.worst_mismatch = Some(evidence);
        }
        Ok(())
    }

    fn finish(
        self,
        frame_range: [usize; 2],
        requested_frames: usize,
        native_frames: usize,
        oracle_frames: usize,
    ) -> TasPixelComparisonReport {
        let proves_requested_frame_pixels = requested_frames == native_frames
            && requested_frames == oracle_frames
            && requested_frames == self.compared_frames;
        TasPixelComparisonReport {
            projection: TAS_PIXEL_PROJECTION_NAME,
            frame_range,
            frame_counts: TasPixelFrameCounts {
                requested: requested_frames,
                native: native_frames,
                oracle: oracle_frames,
                compared: self.compared_frames,
            },
            proves_requested_frame_pixels,
            proves_every_frame_in_range: proves_requested_frame_pixels
                && requested_frames == frame_range[1].saturating_sub(frame_range[0]),
            proves_whole_state: false,
            exact_match_frames: self.exact_match_frames,
            mismatch_frames: self.mismatch_frames,
            total_changed_pixels: self.total_changed_pixels,
            mismatch_digest: format!("{:x}", self.digest.finalize()),
            first_mismatch: self.first_mismatch,
            worst_mismatch: self.worst_mismatch,
        }
    }
}

pub(crate) fn compare_tas_pixels(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    oracle: &OracleManifest,
    start: usize,
    end: usize,
) -> Result<TasPixelComparisonReport> {
    let requested: BTreeMap<usize, &str> = scenario
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.frame_index >= start && checkpoint.frame_index < end)
        .map(|checkpoint| (checkpoint.frame_index, checkpoint.label.as_str()))
        .collect();
    let oracle_frames = oracle_frames_by_index(&oracle.frames, start, end)?;
    let mut missing_oracle = requested
        .keys()
        .filter(|index| !oracle_frames.contains_key(index))
        .copied()
        .collect::<Vec<_>>();
    if !missing_oracle.is_empty() {
        missing_oracle.truncate(8);
        bail!("oracle pixel evidence is missing requested frames {missing_oracle:?}");
    }

    let mut current_level = scenario.warp_to_level.unwrap_or(0);
    let (mut level_visuals, mut render_level_block_grid) =
        load_session_assets(manifest, current_level)?;
    let mut accumulator = PixelComparisonAccumulator::default();
    let mut native_frames = 0usize;
    try_for_each_native_dynamic_frame(
        manifest.clone(),
        scenario,
        NativeDynamicRunOptions::default(),
        |frame_index, runtime, previous_trace, trace| {
            let Some(label) = requested.get(&frame_index) else {
                return Ok(());
            };
            let render_level = runtime.source_port_render_level().0;
            if render_level != current_level {
                current_level = render_level;
                (level_visuals, render_level_block_grid) =
                    load_session_assets(manifest, current_level)?;
            }
            // ASM loc_119D8 / ReadJoypad loc_C0A reset Current_LevelID before
            // loc_6E2 finishes fading the outgoing demo planes. In that
            // window, pair the latched render level's art with its own layout;
            // normal gameplay still uses runtime.block_grid() so live block
            // mutations remain visible to the verifier.
            let block_grid = if render_level == trace.current_level.0 {
                runtime.block_grid()
            } else {
                Some(&render_level_block_grid)
            };
            let native_rgb = render_source_port_frame(
                runtime,
                previous_trace,
                trace,
                &level_visuals,
                block_grid,
            );
            let oracle_frame = oracle_frames
                .get(&frame_index)
                .expect("missing oracle frames rejected before replay");
            let oracle_rgb = load_oracle_rgb(oracle_frame)?;
            accumulator.observe(frame_index, label, &native_rgb, &oracle_rgb)?;
            native_frames += 1;
            Ok(())
        },
    )?;

    let report = accumulator.finish(
        [start, end],
        requested.len(),
        native_frames,
        oracle_frames.len(),
    );
    if !report.proves_requested_frame_pixels {
        bail!(
            "incomplete TAS pixel coverage: requested={} native={} oracle={} compared={}",
            report.frame_counts.requested,
            report.frame_counts.native,
            report.frame_counts.oracle,
            report.frame_counts.compared
        );
    }
    Ok(report)
}

fn oracle_frames_by_index(
    frames: &[OracleFrame],
    start: usize,
    end: usize,
) -> Result<BTreeMap<usize, &OracleFrame>> {
    let requested_range = start..end;
    let mut by_index = BTreeMap::new();
    let mut duplicate_indices = BTreeSet::new();
    for frame in frames
        .iter()
        .filter(|frame| requested_range.contains(&frame.frame_index))
    {
        if by_index.insert(frame.frame_index, frame).is_some() {
            duplicate_indices.insert(frame.frame_index);
        }
    }
    if !duplicate_indices.is_empty() {
        bail!("oracle pixel evidence has duplicate frames {duplicate_indices:?}");
    }
    Ok(by_index)
}

fn render_source_port_frame(
    runtime: &NativeRuntime,
    previous_trace: &NativeTraceFrame,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) -> Vec<u8> {
    let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
    let render_trace = render_trace_with_vdp_latched_state(trace, Some(previous_trace));
    if let Some(vdp) = runtime.vdp_state() {
        render_native_vdp_rgb(vdp, &mut rgb);
        render_vdp_trace_overlay(&render_trace, level_visuals, &mut rgb);
    } else {
        let scene = build_scene(&render_trace, false, Some(level_visuals), block_grid);
        render_scene(&scene, &mut rgb);
    }
    rgb
}

fn load_oracle_rgb(frame: &OracleFrame) -> Result<Vec<u8>> {
    let image = image::open(&frame.png_path)
        .with_context(|| format!("failed to read oracle frame {}", frame.png_path.display()))?
        .to_rgb8();
    if image.dimensions() != (NATIVE_FRAME_WIDTH, NATIVE_FRAME_HEIGHT) {
        bail!(
            "oracle frame {} has dimensions {:?}, expected {}x{}",
            frame.frame_index,
            image.dimensions(),
            NATIVE_FRAME_WIDTH,
            NATIVE_FRAME_HEIGHT
        );
    }
    Ok(image.into_raw())
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut digest = Sha1::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_report_distinguishes_exact_and_changed_frames() {
        let mut accumulator = PixelComparisonAccumulator::default();
        accumulator
            .observe(4, "exact", &[0, 1, 2, 3, 4, 5], &[0, 1, 2, 3, 4, 5])
            .expect("exact frame");
        accumulator
            .observe(5, "changed", &[0, 1, 2, 3, 4, 5], &[0, 1, 9, 8, 4, 5])
            .expect("changed frame");

        let report = accumulator.finish([4, 6], 2, 2, 2);

        assert_eq!(report.exact_match_frames, 1);
        assert_eq!(report.mismatch_frames, 1);
        assert_eq!(report.total_changed_pixels, 2);
        assert!(report.proves_requested_frame_pixels);
        assert!(report.proves_every_frame_in_range);
        assert_eq!(report.first_mismatch, report.worst_mismatch);
        assert_eq!(report.first_mismatch.expect("mismatch").frame_index, 5);
    }

    #[test]
    fn pixel_report_does_not_overclaim_sampled_or_incomplete_coverage() {
        let mut sampled = PixelComparisonAccumulator::default();
        sampled
            .observe(5, "sample", &[0, 0, 0], &[0, 0, 0])
            .expect("sample");
        let sampled = sampled.finish([0, 10], 1, 1, 1);
        assert!(sampled.proves_requested_frame_pixels);
        assert!(!sampled.proves_every_frame_in_range);

        let incomplete = PixelComparisonAccumulator::default().finish([0, 1], 1, 0, 1);
        assert!(!incomplete.proves_requested_frame_pixels);
        assert!(!incomplete.proves_every_frame_in_range);
    }

    #[test]
    fn pixel_comparison_rejects_non_rgb_or_different_sized_buffers() {
        let mut accumulator = PixelComparisonAccumulator::default();
        let error = accumulator
            .observe(7, "bad", &[0, 0], &[0, 0])
            .expect_err("two-byte buffers are not RGB24");
        assert!(error.to_string().contains("incompatible RGB lengths"));
    }
}
