use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};

use crate::diff::BBox;
use crate::native_player::{
    LayerSpan, NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, Scene, build_scene_with_layers,
    load_session_assets, render_native_vdp_rgb, render_scene,
    render_trace_with_vdp_latched_state_and_resident, render_vdp_trace_overlay,
};
use crate::oracle::{OracleFrame, OracleManifest};
use crate::scenarios::{
    DynamicScenario, NativeDynamicRunOptions, try_for_each_native_dynamic_frame,
};
use wildside_core::{NativeRuntime, NativeTraceFrame, terrain::BlockGrid};
use wildside_data::AssetManifest;
use wildside_render::NativeLevelVisuals;

pub(crate) const TAS_PIXEL_PROJECTION_NAME: &str = "source_port_visible_rgb24_v1";

/// Samples kept per cluster in the mismatch report: {first, worst, last}.
const SAMPLES_PER_CLUSTER: usize = 3;

/// Grid cell edge (pixels) for bucketing non-HUD mismatch bboxes.
const CLUSTER_GRID_CELL: usize = 8;

struct NamedRegion {
    name: &'static str,
    bbox: BBox,
}

/// HUD rects verified against oracle frames during the #406 frame-963
/// investigation. A frame's mismatch bbox must be FULLY contained by a rect to
/// take its name; everything else buckets by quantised-grid key, so wide
/// transform-flash bboxes cluster naturally without a full-screen catch-all.
/// TODO(#406): add ankh/lives/diamond-count/hitpoints rects once verified.
const HUD_REGIONS: &[NamedRegion] = &[
    NamedRegion {
        name: "timer_digits",
        bbox: BBox {
            min_x: 16,
            min_y: 16,
            max_x: 41,
            max_y: 23,
        },
    },
    NamedRegion {
        name: "hud_diamond",
        bbox: BBox {
            min_x: 288,
            min_y: 34,
            max_x: 299,
            max_y: 45,
        },
    },
];

/// First HUD rect that fully contains `bbox`, if any.
fn match_region(bbox: &BBox) -> Option<&'static str> {
    HUD_REGIONS
        .iter()
        .find(|region| region.bbox.contains(bbox))
        .map(|region| region.name)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelFrameMismatch {
    frame_index: usize,
    label: String,
    changed_pixels: usize,
    bbox: BBox,
    region: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelCluster {
    key: String,
    region: Option<&'static str>,
    /// Union of every member frame's mismatch bbox.
    bbox: BBox,
    frame_count: usize,
    total_changed_pixels: u64,
    min_changed_pixels: usize,
    max_changed_pixels: usize,
    /// Consecutive member frame indices collapsed to inclusive [start, end].
    frame_ranges: Vec<[usize; 2]>,
    /// {first, worst, last} member frames (deduplicated, ascending).
    sample_frame_indices: Vec<usize>,
}

impl TasPixelCluster {
    pub(crate) fn console_summary(&self) -> String {
        format!(
            "{}: {} frames, {} px (per-frame {}..{}), bbox ({},{})-({},{}), {} range(s), samples {:?}",
            self.key,
            self.frame_count,
            self.total_changed_pixels,
            self.min_changed_pixels,
            self.max_changed_pixels,
            self.bbox.min_x,
            self.bbox.min_y,
            self.bbox.max_x,
            self.bbox.max_y,
            self.frame_ranges.len(),
            self.sample_frame_indices,
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct TasPixelMismatchReport {
    schema_version: u32,
    projection: &'static str,
    frame_range: [usize; 2],
    frame_width: usize,
    frame_height: usize,
    mismatch_frames: usize,
    total_changed_pixels: u64,
    pub(crate) clusters: Vec<TasPixelCluster>,
    per_frame: Vec<TasPixelFrameMismatch>,
}

fn cluster_key(mismatch: &TasPixelFrameMismatch) -> String {
    if let Some(region) = mismatch.region {
        return region.to_owned();
    }
    let bbox = &mismatch.bbox;
    format!(
        "grid:{},{}-{},{}",
        bbox.min_x / CLUSTER_GRID_CELL,
        bbox.min_y / CLUSTER_GRID_CELL,
        bbox.max_x / CLUSTER_GRID_CELL,
        bbox.max_y / CLUSTER_GRID_CELL,
    )
}

/// Collapse sorted frame indices into inclusive consecutive ranges.
pub(crate) fn collapse_frame_ranges(sorted_indices: &[usize]) -> Vec<[usize; 2]> {
    let mut ranges: Vec<[usize; 2]> = Vec::new();
    for &index in sorted_indices {
        match ranges.last_mut() {
            Some(range) if index == range[1] + 1 => range[1] = index,
            _ => ranges.push([index, index]),
        }
    }
    ranges
}

/// Group mismatch frames into a deterministic, prioritised cluster worklist:
/// by HUD region name where the bbox is contained, else by quantised-grid key;
/// sorted by total changed pixels descending, then key ascending.
pub(crate) fn cluster_mismatches(
    frames: &[TasPixelFrameMismatch],
    samples_per_cluster: usize,
) -> Vec<TasPixelCluster> {
    let mut groups: BTreeMap<String, Vec<&TasPixelFrameMismatch>> = BTreeMap::new();
    for frame in frames {
        groups.entry(cluster_key(frame)).or_default().push(frame);
    }
    let mut clusters: Vec<TasPixelCluster> = groups
        .into_iter()
        .map(|(key, mut members)| {
            members.sort_by_key(|member| member.frame_index);
            let bbox = members
                .iter()
                .skip(1)
                .fold(members[0].bbox, |acc, member| acc.union(&member.bbox));
            let sorted_indices: Vec<usize> =
                members.iter().map(|member| member.frame_index).collect();
            // First member with the maximum count, so "worst" is deterministic.
            let worst = members
                .iter()
                .copied()
                .reduce(|best, member| {
                    if member.changed_pixels > best.changed_pixels {
                        member
                    } else {
                        best
                    }
                })
                .expect("groups only exist for non-empty member lists");
            let mut samples = vec![
                sorted_indices[0],
                worst.frame_index,
                *sorted_indices.last().expect("non-empty"),
            ];
            samples.sort_unstable();
            samples.dedup();
            samples.truncate(samples_per_cluster);
            TasPixelCluster {
                region: members[0].region,
                bbox,
                frame_count: members.len(),
                total_changed_pixels: members
                    .iter()
                    .map(|member| member.changed_pixels as u64)
                    .sum(),
                min_changed_pixels: members
                    .iter()
                    .map(|member| member.changed_pixels)
                    .min()
                    .expect("non-empty"),
                max_changed_pixels: worst.changed_pixels,
                frame_ranges: collapse_frame_ranges(&sorted_indices),
                sample_frame_indices: samples,
                key,
            }
        })
        .collect();
    clusters.sort_by(|a, b| {
        b.total_changed_pixels
            .cmp(&a.total_changed_pixels)
            .then_with(|| a.key.cmp(&b.key))
    });
    clusters
}

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
    frame_width: usize,
    compared_frames: usize,
    exact_match_frames: usize,
    mismatch_frames: usize,
    total_changed_pixels: u64,
    digest: Sha1,
    first_mismatch: Option<TasPixelFrameEvidence>,
    worst_mismatch: Option<TasPixelFrameEvidence>,
    detail: Option<Vec<TasPixelFrameMismatch>>,
}

impl PixelComparisonAccumulator {
    /// `frame_width` derives per-pixel x/y for bbox detail; only read when
    /// `collect_detail` is true, so the summary-only path never allocates.
    fn with_detail(frame_width: usize, collect_detail: bool) -> Self {
        Self {
            frame_width,
            detail: collect_detail.then(Vec::new),
            ..Self::default()
        }
    }

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

        // Detail mode derives the mismatch bbox in the same single pass that
        // produces the count, so summary fields (and mismatch_digest) stay
        // byte-identical either way.
        let (changed_pixels, bbox) = if self.detail.is_some() {
            match crate::diff::pixel_mismatch_bbox(native_rgb, oracle_rgb, self.frame_width) {
                Some((count, bbox)) => (count, Some(bbox)),
                None => (0, None),
            }
        } else {
            (
                crate::diff::pixel_mismatch_count(native_rgb, oracle_rgb),
                None,
            )
        };
        self.compared_frames += 1;
        if changed_pixels == 0 {
            self.exact_match_frames += 1;
            return Ok(());
        }

        if let (Some(detail), Some(bbox)) = (self.detail.as_mut(), bbox) {
            detail.push(TasPixelFrameMismatch {
                frame_index,
                label: label.to_owned(),
                changed_pixels,
                region: match_region(&bbox),
                bbox,
            });
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

/// Build the standalone mismatch report from the per-frame detail collected
/// alongside a summary comparison. Pure aggregation, no I/O.
fn build_mismatch_report(
    summary: &TasPixelComparisonReport,
    per_frame: Vec<TasPixelFrameMismatch>,
) -> TasPixelMismatchReport {
    TasPixelMismatchReport {
        schema_version: 1,
        projection: TAS_PIXEL_PROJECTION_NAME,
        frame_range: summary.frame_range,
        frame_width: NATIVE_FRAME_WIDTH as usize,
        frame_height: NATIVE_FRAME_HEIGHT as usize,
        mismatch_frames: summary.mismatch_frames,
        total_changed_pixels: summary.total_changed_pixels,
        clusters: cluster_mismatches(&per_frame, SAMPLES_PER_CLUSTER),
        per_frame,
    }
}

pub(crate) fn compare_tas_pixels(
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    oracle: &OracleManifest,
    start: usize,
    end: usize,
    collect_mismatch_detail: bool,
) -> Result<(TasPixelComparisonReport, Option<TasPixelMismatchReport>)> {
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
    let mut accumulator = PixelComparisonAccumulator::with_detail(
        NATIVE_FRAME_WIDTH as usize,
        collect_mismatch_detail,
    );
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
            let block_grid = runtime
                .source_port_render_block_grid()
                .or(Some(&render_level_block_grid));
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

    let detail = accumulator.detail.take();
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
    let mismatch_report = detail.map(|per_frame| build_mismatch_report(&report, per_frame));
    Ok((report, mismatch_report))
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
    render_source_port_frame_with_scene(runtime, previous_trace, trace, level_visuals, block_grid).0
}

/// [`render_source_port_frame`] plus the scene and layer spans the RGB came
/// from. `None` on the VDP path (Sega/intro/title screens render real VDP
/// pixels, not a `Scene`). Provenance sampling must use this scene — not a
/// rebuilt one — so it attributes exactly the pixels that were diffed.
pub(crate) fn render_source_port_frame_with_scene(
    runtime: &NativeRuntime,
    previous_trace: &NativeTraceFrame,
    trace: &NativeTraceFrame,
    level_visuals: &NativeLevelVisuals,
    block_grid: Option<&BlockGrid>,
) -> (Vec<u8>, Option<(Scene, Vec<LayerSpan>)>) {
    let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];
    let mut resolved_trace = trace.clone();
    if previous_trace.mode_level_work_after_hud_sub_44dc.is_some()
        && previous_trace.current_level == trace.current_level
        && let Some(work) = resolved_trace
            .mode_level_vblank_work_before_sub_e12
            .as_mut()
        && work.dynamic_art_tiles_loc_ad4.is_none()
    {
        // ASM Mode_Level loc_87E -> sub_1492 -> sub_13C0 loc_141A writes the
        // player mapping's tile lengths after the HUD pass. The following
        // VBlank consumes that retained loc_AD4 queue before sub_E12.
        let player_frame = crate::native_player::select_player_dynamic_art_frame_loc_141a(
            previous_trace,
            &level_visuals.player,
        );
        work.dynamic_art_tiles_loc_ad4 =
            Some(crate::native_player::player_frame_dynamic_art_tiles_loc_141a(player_frame));
        // A preceding complete Mode_Level pass proves this is not the initial
        // Make_SpriteAttr_HUD counter build.
        work.hud_path_sub_bdc0 =
            wildside_core::ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters;
    }
    let render_trace = render_trace_with_vdp_latched_state_and_resident(
        &resolved_trace,
        Some(previous_trace),
        runtime.source_port_render_trace_snapshot(),
    );
    if let Some(vdp) = runtime.vdp_state() {
        render_native_vdp_rgb(vdp, &mut rgb);
        render_vdp_trace_overlay(&render_trace, level_visuals, &mut rgb);
        (rgb, None)
    } else {
        let (scene, spans) =
            build_scene_with_layers(&render_trace, false, Some(level_visuals), block_grid);
        render_scene(&scene, &mut rgb);
        (rgb, Some((scene, spans)))
    }
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

    fn mismatch(frame_index: usize, changed_pixels: usize, bbox: BBox) -> TasPixelFrameMismatch {
        TasPixelFrameMismatch {
            frame_index,
            label: format!("frame_{frame_index}"),
            changed_pixels,
            region: match_region(&bbox),
            bbox,
        }
    }

    fn bbox(min_x: usize, min_y: usize, max_x: usize, max_y: usize) -> BBox {
        BBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    #[test]
    fn match_region_requires_full_containment() {
        // Inside timer_digits (16,16)-(41,23) and hud_diamond (288,34)-(299,45).
        assert_eq!(match_region(&bbox(16, 16, 41, 23)), Some("timer_digits"));
        assert_eq!(match_region(&bbox(20, 18, 30, 22)), Some("timer_digits"));
        assert_eq!(match_region(&bbox(288, 34, 299, 45)), Some("hud_diamond"));
        assert_eq!(match_region(&bbox(290, 40, 295, 44)), Some("hud_diamond"));
        // Overlapping but not contained, and fully outside.
        assert_eq!(match_region(&bbox(15, 16, 41, 23)), None);
        assert_eq!(match_region(&bbox(288, 34, 300, 45)), None);
        assert_eq!(match_region(&bbox(0, 100, 319, 150)), None);
    }

    #[test]
    fn cluster_mismatches_groups_unions_and_sorts() {
        let frames = vec![
            // timer_digits region: frames 10, 11, 12, 20 (range collapse: 10-12, 20)
            mismatch(10, 5, bbox(16, 16, 24, 23)),
            mismatch(11, 40, bbox(20, 16, 41, 23)),
            mismatch(12, 8, bbox(16, 16, 30, 23)),
            mismatch(20, 3, bbox(30, 17, 41, 22)),
            // full-width flash: grid key, dominates by total pixels
            mismatch(15, 60000, bbox(0, 0, 319, 223)),
        ];
        let clusters = cluster_mismatches(&frames, SAMPLES_PER_CLUSTER);
        assert_eq!(clusters.len(), 2);

        // Sorted by total pixels descending: the flash cluster first.
        assert_eq!(clusters[0].key, "grid:0,0-39,27");
        assert_eq!(clusters[0].region, None);
        assert_eq!(clusters[0].frame_count, 1);
        assert_eq!(clusters[0].total_changed_pixels, 60000);
        assert_eq!(clusters[0].frame_ranges, vec![[15, 15]]);
        assert_eq!(clusters[0].sample_frame_indices, vec![15]);

        let timer = &clusters[1];
        assert_eq!(timer.key, "timer_digits");
        assert_eq!(timer.region, Some("timer_digits"));
        assert_eq!(timer.bbox, bbox(16, 16, 41, 23));
        assert_eq!(timer.frame_count, 4);
        assert_eq!(timer.total_changed_pixels, 56);
        assert_eq!(timer.min_changed_pixels, 3);
        assert_eq!(timer.max_changed_pixels, 40);
        assert_eq!(timer.frame_ranges, vec![[10, 12], [20, 20]]);
        // Samples: first (10), worst (11), last (20).
        assert_eq!(timer.sample_frame_indices, vec![10, 11, 20]);
    }

    #[test]
    fn cluster_sort_ties_break_by_key_ascending() {
        let frames = vec![
            mismatch(1, 7, bbox(100, 100, 101, 101)),
            mismatch(2, 7, bbox(200, 200, 201, 201)),
        ];
        let clusters = cluster_mismatches(&frames, SAMPLES_PER_CLUSTER);
        assert_eq!(clusters.len(), 2);
        assert!(clusters[0].key < clusters[1].key);
    }

    #[test]
    fn summary_only_accumulator_collects_no_detail() {
        let mut accumulator = PixelComparisonAccumulator::with_detail(2, false);
        accumulator
            .observe(5, "changed", &[0, 1, 2, 3, 4, 5], &[0, 1, 9, 8, 4, 5])
            .expect("changed frame");
        assert!(accumulator.detail.is_none());
    }

    #[test]
    fn detail_mode_keeps_summary_identical_and_report_totals_consistent() {
        let native = [0u8, 1, 2, 3, 4, 5];
        let changed = [0u8, 1, 9, 8, 4, 5];
        let mut summary_only = PixelComparisonAccumulator::with_detail(2, false);
        let mut detailed = PixelComparisonAccumulator::with_detail(2, true);
        for accumulator in [&mut summary_only, &mut detailed] {
            accumulator
                .observe(4, "exact", &native, &native)
                .expect("exact frame");
            accumulator
                .observe(5, "changed", &native, &changed)
                .expect("changed frame");
        }

        let detail = detailed.detail.take().expect("detail enabled");
        assert_eq!(detail.len(), 1);
        assert_eq!(detail[0].frame_index, 5);
        assert_eq!(detail[0].changed_pixels, 2);
        assert_eq!(detail[0].bbox, bbox(0, 0, 1, 0));

        let summary = summary_only.finish([4, 6], 2, 2, 2);
        let detailed_summary = detailed.finish([4, 6], 2, 2, 2);
        assert_eq!(summary, detailed_summary);

        let report = build_mismatch_report(&detailed_summary, detail);
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.projection, TAS_PIXEL_PROJECTION_NAME);
        assert_eq!(report.frame_range, detailed_summary.frame_range);
        assert_eq!(report.mismatch_frames, detailed_summary.mismatch_frames);
        assert_eq!(
            report.total_changed_pixels,
            detailed_summary.total_changed_pixels
        );
        assert_eq!(
            report
                .clusters
                .iter()
                .map(|cluster| cluster.total_changed_pixels)
                .sum::<u64>(),
            detailed_summary.total_changed_pixels
        );
        assert_eq!(
            report
                .per_frame
                .iter()
                .map(|frame| frame.changed_pixels as u64)
                .sum::<u64>(),
            detailed_summary.total_changed_pixels
        );
    }
}
