//! Root-cause clustering for TAS visual divergence.
//!
//! `visual-triage` replays the published route, re-renders each frame the
//! pixel mismatch report lists as divergent, samples per-pixel draw
//! provenance on the exact scene that produced the diffed RGB, and clusters
//! frames by root-cause signature (divergence class + renderer layer + draw
//! command kind + palette line). The spatial clusterer in `tas_pixels`
//! answers "where do frames diverge"; this answers "which native draw path
//! is responsible", collapsing thousands of grid buckets into a short ranked
//! histogram of named causes. Pure tooling — no gameplay behaviour.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::diff::{BBox, changed_pixel_coords, generate_diff_composite, subsample_deterministic};
use crate::native_player::{
    FinalScenePixelDiagnostic, LayerSpan, NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, SceneLayer,
    layer_for_command_index, load_session_assets, sample_scene_pixel_provenance,
};
use crate::scenarios::{
    DynamicScenario, NativeDynamicRunOptions, try_for_each_native_dynamic_frame,
};
use crate::tas_pixels::{
    TAS_PIXEL_PROJECTION_NAME, collapse_frame_ranges, render_source_port_frame_with_scene,
};
use wildside_data::AssetManifest;

/// Changed pixels sampled for provenance per divergent frame.
const PROVENANCE_SAMPLES_PER_FRAME: usize = 512;

/// Samples kept per cluster: {first, worst, last} member frames.
const SAMPLES_PER_CLUSTER: usize = 3;

/// Grid cell edge (pixels) for the coarse VDP-path signature, mirroring the
/// spatial clusterer's bucket size.
const VDP_GRID_CELL: usize = 8;

/// Exact palette lookups retained across the first sampled divergent scene
/// frame. This stays bounded while exposing enough information to distinguish
/// compact, local-line, and expanded CRAM palette routes.
const PALETTE_RESOLUTION_SAMPLES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DivergenceClass {
    /// No accepted native colour write: native shows the scene background
    /// where the oracle has content — native is missing a draw.
    OracleExtra,
    /// Native accepted a colour write where the oracle shows the scene
    /// background — native draws content the oracle does not.
    NativeExtra,
    /// Both sides drew content but the colours differ.
    Recolour,
}

impl DivergenceClass {
    fn as_str(self) -> &'static str {
        match self {
            DivergenceClass::OracleExtra => "oracle_extra",
            DivergenceClass::NativeExtra => "native_extra",
            DivergenceClass::Recolour => "recolour",
        }
    }
}

/// Root-cause signature of one divergent pixel. `tile_index` deliberately
/// stays out of the cluster key: its cardinality would re-fragment the
/// histogram back into per-sprite buckets; it is kept as a descriptive
/// histogram inside each cluster instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PixelSignature {
    class: DivergenceClass,
    layer: SceneLayer,
    kind: String,
    palette_line: Option<u8>,
}

impl PixelSignature {
    fn key(&self) -> String {
        let palette = self
            .palette_line
            .map_or_else(|| "-".to_owned(), |line| format!("pal{line}"));
        format!(
            "{}|{}|{}|{palette}",
            self.class.as_str(),
            self.layer.as_str(),
            self.kind
        )
    }

    fn describe(&self) -> String {
        let palette = self
            .palette_line
            .map(|line| format!(" (palette {line})"))
            .unwrap_or_default();
        match self.class {
            DivergenceClass::OracleExtra => {
                "oracle-only content where native draws nothing (native missing a draw)".to_owned()
            }
            DivergenceClass::NativeExtra => format!(
                "native {} {}{palette} drawn where oracle shows background",
                self.layer.as_str(),
                self.kind
            ),
            DivergenceClass::Recolour => format!(
                "native {} {}{palette} colour differs from oracle",
                self.layer.as_str(),
                self.kind
            ),
        }
    }
}

/// Classify one sampled divergent pixel from its provenance diagnostic.
/// Returns the signature plus the final write's tile index for histograms.
pub(crate) fn classify_pixel(
    diagnostic: &FinalScenePixelDiagnostic,
    spans: &[LayerSpan],
    oracle_rgb: [u8; 3],
    scene_background: [u8; 3],
) -> (PixelSignature, Option<u16>) {
    // The last accepted colour write is what the rasteriser left on screen;
    // rgb-less accepted writes (clear_priority_rect, palette fade) do not
    // deposit content, so a pixel without any colour write is scene
    // background — classified structurally, not by RGB coincidence.
    let final_colour_write = diagnostic
        .writes
        .iter()
        .rev()
        .find(|write| write.accepted && write.rgb.is_some());
    match final_colour_write {
        None => (
            PixelSignature {
                class: DivergenceClass::OracleExtra,
                layer: SceneLayer::Oracle,
                kind: "none".to_owned(),
                palette_line: None,
            },
            None,
        ),
        Some(write) => {
            let class = if oracle_rgb == scene_background {
                DivergenceClass::NativeExtra
            } else {
                DivergenceClass::Recolour
            };
            (
                PixelSignature {
                    class,
                    layer: layer_for_command_index(spans, write.command_index),
                    kind: write.kind.clone(),
                    palette_line: write.palette_line,
                },
                write.tile_index,
            )
        }
    }
}

/// One triaged divergent frame: its dominant signature plus descriptive
/// histograms over the sampled pixels that carry that signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrameTriage {
    frame_index: usize,
    changed_pixels: usize,
    bbox: BBox,
    key: String,
    description: String,
    tile_histogram: BTreeMap<u16, usize>,
    palette_histogram: BTreeMap<u8, usize>,
}

/// Reduce a frame's sampled pixel signatures to its dominant signature
/// (highest sample count, ties broken by key ascending) and histogram the
/// dominant pixels' tile indices and palette lines.
pub(crate) fn frame_signature(
    samples: &[(PixelSignature, Option<u16>)],
) -> Option<(String, String, BTreeMap<u16, usize>, BTreeMap<u8, usize>)> {
    let mut counts: BTreeMap<&PixelSignature, usize> = BTreeMap::new();
    for (signature, _) in samples {
        *counts.entry(signature).or_default() += 1;
    }
    // BTreeMap iterates in key order, so `>` keeps the first (smallest) of
    // equal counts — deterministic tie-break by signature ordering.
    let dominant = counts
        .iter()
        .fold(
            None::<(&PixelSignature, usize)>,
            |best, (sig, count)| match best {
                Some((_, best_count)) if *count <= best_count => best,
                _ => Some((sig, *count)),
            },
        )?
        .0;
    let mut tiles: BTreeMap<u16, usize> = BTreeMap::new();
    let mut palettes: BTreeMap<u8, usize> = BTreeMap::new();
    for (signature, tile_index) in samples {
        if signature != dominant {
            continue;
        }
        if let Some(tile_index) = tile_index {
            *tiles.entry(*tile_index).or_default() += 1;
        }
        if let Some(palette_line) = signature.palette_line {
            *palettes.entry(palette_line).or_default() += 1;
        }
    }
    Some((dominant.key(), dominant.describe(), tiles, palettes))
}

/// Coarse signature for frames rendered through the VDP path (Sega screen /
/// intro / title), which have no `Scene` to sample. Name-table attribution is
/// out of scope; the quantised bbox mirrors the spatial clusterer.
pub(crate) fn vdp_frame_key(bbox: &BBox) -> String {
    format!(
        "vdp:grid:{},{}-{},{}",
        bbox.min_x / VDP_GRID_CELL,
        bbox.min_y / VDP_GRID_CELL,
        bbox.max_x / VDP_GRID_CELL,
        bbox.max_y / VDP_GRID_CELL,
    )
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SignatureCluster {
    key: String,
    description: String,
    frame_count: usize,
    /// Sum of member frames' FULL changed-pixel counts (not just samples).
    total_changed_pixels: u64,
    /// Consecutive member frame indices collapsed to inclusive [start, end].
    frame_ranges: Vec<[usize; 2]>,
    /// {first, worst, last} member frames (deduplicated, ascending).
    sample_frame_indices: Vec<usize>,
    /// Tile indices (hex) seen among sampled dominant-signature pixels.
    tile_index_histogram: BTreeMap<String, usize>,
    /// Palette lines seen among sampled dominant-signature pixels.
    palette_line_histogram: BTreeMap<String, usize>,
    /// The worst member frame's mismatch bbox.
    example_bbox: BBox,
}

impl SignatureCluster {
    fn console_summary(&self) -> String {
        format!(
            "{}: {} frames, {} px, {} range(s), samples {:?} — {}",
            self.key,
            self.frame_count,
            self.total_changed_pixels,
            self.frame_ranges.len(),
            self.sample_frame_indices,
            self.description,
        )
    }
}

/// Group triaged frames by signature key, sum full pixel impact, collapse
/// ranges, merge histograms; sort by total changed pixels descending then key
/// ascending (mirrors the spatial clusterer's ranking).
pub(crate) fn cluster_signatures(frames: &[FrameTriage]) -> Vec<SignatureCluster> {
    let mut groups: BTreeMap<&str, Vec<&FrameTriage>> = BTreeMap::new();
    for frame in frames {
        groups.entry(&frame.key).or_default().push(frame);
    }
    let mut clusters: Vec<SignatureCluster> = groups
        .into_values()
        .map(|mut members| {
            members.sort_by_key(|member| member.frame_index);
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
            samples.truncate(SAMPLES_PER_CLUSTER);
            let mut tile_index_histogram: BTreeMap<String, usize> = BTreeMap::new();
            let mut palette_line_histogram: BTreeMap<String, usize> = BTreeMap::new();
            for member in &members {
                for (tile, count) in &member.tile_histogram {
                    *tile_index_histogram
                        .entry(format!("{tile:#06x}"))
                        .or_default() += count;
                }
                for (palette, count) in &member.palette_histogram {
                    *palette_line_histogram
                        .entry(palette.to_string())
                        .or_default() += count;
                }
            }
            SignatureCluster {
                key: members[0].key.clone(),
                description: members[0].description.clone(),
                frame_count: members.len(),
                total_changed_pixels: members
                    .iter()
                    .map(|member| member.changed_pixels as u64)
                    .sum(),
                frame_ranges: collapse_frame_ranges(&sorted_indices),
                sample_frame_indices: samples,
                tile_index_histogram,
                palette_line_histogram,
                example_bbox: worst.bbox,
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
pub(crate) struct VisualTriageReport {
    schema_version: u32,
    projection: String,
    frame_range: [usize; 2],
    mismatch_frames: usize,
    total_changed_pixels: u64,
    vdp_frames: usize,
    scene_frames: usize,
    sampled_pixels: u64,
    /// Sampled pixels where the provenance re-walk's final RGB disagreed with
    /// the actually-rendered pixel (tint blends, SAT first-opaque arbitration
    /// across entries). Attribution still uses the final accepted write; a
    /// large count here means the re-walk is drifting from `render_scene`.
    provenance_disagreement_pixels: u64,
    /// Frames whose recomputed changed-pixel count differs from the mismatch
    /// report's — non-zero means the renderer changed since the report ran.
    recount_drift_frames: usize,
    palette_resolution_samples: Vec<PaletteResolutionSample>,
    clusters: Vec<SignatureCluster>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct PaletteResolutionSample {
    frame_index: usize,
    x: usize,
    y: usize,
    layer: String,
    kind: String,
    tile_index: u16,
    hardware_palette_line: u8,
    pixel_colour_index: u8,
    hardware_cram_slot: usize,
    palette_colour_count: usize,
    resolved_palette_slot: usize,
    raw_palette_word: u16,
    background_row_index: Option<usize>,
    background_layer_speed: Option<u8>,
    background_scroll_x_px: Option<i32>,
    background_ripple_row_offset: Option<i32>,
    native_rgb: [u8; 3],
    oracle_rgb: [u8; 3],
}

/// Subset of the `tas_pixels` mismatch report this command consumes.
#[derive(Debug, Deserialize)]
struct MismatchReportInput {
    projection: String,
    mismatch_frames: usize,
    total_changed_pixels: u64,
    per_frame: Vec<MismatchFrameInput>,
}

#[derive(Debug, Deserialize)]
struct MismatchFrameInput {
    frame_index: usize,
    changed_pixels: usize,
}

/// Frame index encoded in an oracle PNG file name, keyed off the `frame_N`
/// label suffix — the numeric prefix is a zero-padded sequence number whose
/// width varies, so it is not trusted. The second field is true for a plain
/// periodic capture (`<seq>_frame_N.png`); transition checkpoints re-capture
/// the same frame under labels like `transition_00_to_1b_post_frame_N`.
pub(crate) fn oracle_png_frame_index(file_name: &str) -> Option<(usize, bool)> {
    let stem = file_name.strip_suffix(".png")?;
    let (label_prefix, label_index) = stem.rsplit_once("_frame_")?;
    let frame_index = label_index.parse().ok()?;
    let plain = !label_prefix.is_empty() && label_prefix.bytes().all(|byte| byte.is_ascii_digit());
    Some((frame_index, plain))
}

/// Reconstruct `frame_index -> PNG path` from the cached oracle directory;
/// no manifest JSON survives there, only the labelled frame captures. When a
/// frame was captured under several labels (periodic + transition
/// checkpoint), prefer the plain `frame_N` capture, then the
/// lexicographically smallest name, so the choice is deterministic.
fn oracle_png_index(oracle_dir: &Path) -> Result<BTreeMap<usize, PathBuf>> {
    let mut by_index: BTreeMap<usize, (bool, PathBuf)> = BTreeMap::new();
    for entry in std::fs::read_dir(oracle_dir)
        .with_context(|| format!("failed to read oracle dir {}", oracle_dir.display()))?
    {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((frame_index, plain)) = oracle_png_frame_index(name) else {
            continue;
        };
        match by_index.entry(frame_index) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert((plain, path));
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let (existing_plain, existing_path) = slot.get();
                let wins = (plain && !*existing_plain)
                    || (plain == *existing_plain && path < *existing_path);
                if wins {
                    slot.insert((plain, path));
                }
            }
        }
    }
    Ok(by_index
        .into_iter()
        .map(|(frame_index, (_, path))| (frame_index, path))
        .collect())
}

fn load_oracle_rgb(path: &Path) -> Result<Vec<u8>> {
    let image = image::open(path)
        .with_context(|| format!("failed to read oracle frame {}", path.display()))?
        .to_rgb8();
    if image.dimensions() != (NATIVE_FRAME_WIDTH, NATIVE_FRAME_HEIGHT) {
        bail!(
            "oracle frame {} has dimensions {:?}, expected {}x{}",
            path.display(),
            image.dimensions(),
            NATIVE_FRAME_WIDTH,
            NATIVE_FRAME_HEIGHT
        );
    }
    Ok(image.into_raw())
}

fn bbox_of_coords(coords: &[(usize, usize)]) -> Option<BBox> {
    let mut coords = coords.iter();
    let &(x, y) = coords.next()?;
    let mut bbox = BBox {
        min_x: x,
        min_y: y,
        max_x: x,
        max_y: y,
    };
    for &(x, y) in coords {
        bbox.min_x = bbox.min_x.min(x);
        bbox.min_y = bbox.min_y.min(y);
        bbox.max_x = bbox.max_x.max(x);
        bbox.max_y = bbox.max_y.max(y);
    }
    Some(bbox)
}

/// Native/oracle RGB retained for a cluster's first and worst member frames
/// so composites can be written after ranking without a second replay.
struct ClusterArt {
    first: (usize, Vec<u8>, Vec<u8>),
    worst: (usize, usize, Vec<u8>, Vec<u8>),
}

fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

pub(crate) fn run_visual_triage(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario_path: &Path,
    mismatch_report: Option<&Path>,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    top: usize,
    rerun_oracle: bool,
) -> Result<()> {
    let scenario_json = std::fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read {}", scenario_path.display()))?;
    let mut scenario: DynamicScenario =
        serde_json::from_str(&scenario_json).context("invalid scenario JSON")?;

    let run_dir = repo_root.join(".cache/tas-verify").join(&scenario.slug);
    let mismatch_report_path =
        mismatch_report.map_or_else(|| run_dir.join("mismatch-report.json"), Path::to_path_buf);
    let mismatch_json = std::fs::read_to_string(&mismatch_report_path).with_context(|| {
        format!(
            "failed to read mismatch report {} (run verify-tas --compare-pixels \
             --pixel-mismatch-report first)",
            mismatch_report_path.display()
        )
    })?;
    let mismatch: MismatchReportInput =
        serde_json::from_str(&mismatch_json).context("invalid mismatch report JSON")?;
    if mismatch.projection != TAS_PIXEL_PROJECTION_NAME {
        bail!(
            "mismatch report projection {} is not {TAS_PIXEL_PROJECTION_NAME}",
            mismatch.projection
        );
    }

    let start = frame_start.unwrap_or(0);
    let end = frame_end
        .unwrap_or(scenario.frame_count)
        .min(scenario.frame_count);
    // The run-dir scenario.json is volatile: frontier gates re-slice it in
    // place, which would silently shrink the triaged corpus. Refuse to
    // proceed unless the caller explicitly asked for a narrower window.
    let beyond_scenario = mismatch
        .per_frame
        .iter()
        .filter(|frame| frame.frame_index >= scenario.frame_count)
        .count();
    if beyond_scenario > 0 && frame_end.is_none() {
        bail!(
            "mismatch report lists {beyond_scenario} divergent frames at or beyond the \
             scenario's {} frames — the scenario is likely a frontier slice; pass the \
             canonical scenario (e.g. tests/data/tas/{}/scenario.json) or an explicit \
             --frame-end",
            scenario.frame_count,
            scenario.slug
        );
    }
    if end < scenario.frame_count {
        // Replay always starts at frame 0 for determinism; truncating the
        // tail merely stops the walk early.
        scenario.script = scenario.script[..end.min(scenario.script.len())].to_vec();
        scenario.frame_count = end;
    }
    let divergent: BTreeMap<usize, usize> = mismatch
        .per_frame
        .iter()
        .filter(|frame| frame.frame_index >= start && frame.frame_index < end)
        .map(|frame| (frame.frame_index, frame.changed_pixels))
        .collect();
    if divergent.is_empty() {
        bail!("mismatch report lists no divergent frames in {start}..{end}");
    }

    let oracle_dir = run_dir.join("oracle");
    if rerun_oracle {
        println!("visual-triage: regenerating oracle captures ({end} frames)...");
        let env = crate::oracle::ensure_oracles(repo_root)?;
        std::fs::create_dir_all(&oracle_dir)?;
        let scenario_file = run_dir.join("scenario.json");
        crate::tas::run_tas_oracle(
            &env,
            rom_path,
            &scenario_file,
            &oracle_dir,
            None,
            false,
            false,
        )?;
    }
    let oracle_pngs = oracle_png_index(&oracle_dir)?;
    let mut missing_oracle = divergent
        .keys()
        .filter(|index| !oracle_pngs.contains_key(index))
        .copied()
        .collect::<Vec<_>>();
    if !missing_oracle.is_empty() {
        missing_oracle.truncate(8);
        bail!(
            "oracle dir {} is missing captures for divergent frames {missing_oracle:?} \
             (re-run with --rerun-oracle)",
            oracle_dir.display()
        );
    }

    println!(
        "visual-triage: {} — {} divergent frames in {start}..{end}",
        scenario.slug,
        divergent.len()
    );

    let mut current_level = scenario.warp_to_level.unwrap_or(0);
    let (mut level_visuals, mut render_level_block_grid) =
        load_session_assets(manifest, current_level)?;
    let mut frames: Vec<FrameTriage> = Vec::new();
    let mut cluster_art: BTreeMap<String, ClusterArt> = BTreeMap::new();
    let mut visited_frames = 0usize;
    let mut vdp_frames = 0usize;
    let mut scene_frames = 0usize;
    let mut sampled_pixels = 0u64;
    let mut provenance_disagreement_pixels = 0u64;
    let mut recount_drift_frames = 0usize;
    let mut palette_resolution_samples = Vec::new();
    try_for_each_native_dynamic_frame(
        manifest.clone(),
        &scenario,
        NativeDynamicRunOptions::default(),
        |frame_index, runtime, previous_trace, trace| {
            let Some(&reported_changed_pixels) = divergent.get(&frame_index) else {
                return Ok(());
            };
            visited_frames += 1;
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
            // mutations remain visible (same latch as compare_tas_pixels).
            let block_grid = runtime
                .source_port_render_block_grid()
                .or(Some(&render_level_block_grid));
            let (native_rgb, scene_info) = render_source_port_frame_with_scene(
                runtime,
                previous_trace,
                trace,
                &level_visuals,
                block_grid,
            );
            let oracle_rgb = load_oracle_rgb(&oracle_pngs[&frame_index])?;
            let coords =
                changed_pixel_coords(&native_rgb, &oracle_rgb, NATIVE_FRAME_WIDTH as usize);
            if coords.len() != reported_changed_pixels {
                recount_drift_frames += 1;
            }
            let Some(bbox) = bbox_of_coords(&coords) else {
                // Frame matches now; counted as drift above, nothing to triage.
                return Ok(());
            };
            let changed_pixels = coords.len();
            let (key, description, tile_histogram, palette_histogram) = match &scene_info {
                None => {
                    vdp_frames += 1;
                    let key = vdp_frame_key(&bbox);
                    (
                        key,
                        "VDP-path frame (Sega/intro/title) — coarse spatial signature only"
                            .to_owned(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                    )
                }
                Some((scene, spans)) => {
                    scene_frames += 1;
                    let pixels = subsample_deterministic(&coords, PROVENANCE_SAMPLES_PER_FRAME);
                    let diagnostics = sample_scene_pixel_provenance(scene, &pixels);
                    sampled_pixels += diagnostics.len() as u64;
                    if palette_resolution_samples.len() < PALETTE_RESOLUTION_SAMPLES {
                        for diagnostic in &diagnostics {
                            let Some(write) = diagnostic
                                .writes
                                .iter()
                                .rev()
                                .find(|write| write.accepted && write.rgb.is_some())
                            else {
                                continue;
                            };
                            let (
                                Some(tile_index),
                                Some(palette_line),
                                Some(palette_index),
                                Some(palette_colour_count),
                                Some(resolved_palette_slot),
                                Some(raw_palette_word),
                            ) = (
                                write.tile_index,
                                write.palette_line,
                                write.palette_index,
                                write.palette_colour_count,
                                write.resolved_palette_slot,
                                write.raw_palette_word,
                            )
                            else {
                                continue;
                            };
                            let offset =
                                (diagnostic.y * NATIVE_FRAME_WIDTH as usize + diagnostic.x) * 3;
                            let background_scroll =
                                wildside_frame::background_scroll_pixel_diagnostic_loc_3050a(
                                    trace,
                                    &level_visuals,
                                    diagnostic.x as i32,
                                    diagnostic.y as i32,
                                );
                            let background_pixel = wildside_frame::sample_background_pixel_loc_c070(
                                trace,
                                &level_visuals,
                                diagnostic.x as i32,
                                diagnostic.y as i32,
                            );
                            palette_resolution_samples.push(PaletteResolutionSample {
                                frame_index,
                                x: diagnostic.x,
                                y: diagnostic.y,
                                layer: layer_for_command_index(spans, write.command_index)
                                    .as_str()
                                    .to_owned(),
                                kind: write.kind.clone(),
                                tile_index,
                                hardware_palette_line: palette_line,
                                pixel_colour_index: palette_index,
                                hardware_cram_slot: usize::from(palette_line) * 16
                                    + usize::from(palette_index),
                                palette_colour_count,
                                resolved_palette_slot,
                                raw_palette_word,
                                background_row_index: background_scroll
                                    .as_ref()
                                    .map(|sample| sample.sample_row),
                                background_layer_speed: background_scroll
                                    .as_ref()
                                    .map(|sample| sample.layer_speed_loc_3050a),
                                background_scroll_x_px: background_scroll
                                    .map(|sample| sample.scroll_x_px_loc_3050a),
                                background_ripple_row_offset: background_pixel
                                    .and_then(|sample| sample.row_offset),
                                native_rgb: diagnostic.final_rgb,
                                oracle_rgb: [
                                    oracle_rgb[offset],
                                    oracle_rgb[offset + 1],
                                    oracle_rgb[offset + 2],
                                ],
                            });
                            if palette_resolution_samples.len() >= PALETTE_RESOLUTION_SAMPLES {
                                break;
                            }
                        }
                    }
                    let samples: Vec<(PixelSignature, Option<u16>)> = diagnostics
                        .iter()
                        .map(|diagnostic| {
                            let offset =
                                (diagnostic.y * NATIVE_FRAME_WIDTH as usize + diagnostic.x) * 3;
                            // Guard the provenance re-walk against drifting
                            // from render_scene; pause darken post-processes
                            // the framebuffer, so skip the check there.
                            if !scene.pause_palette_darken
                                && native_rgb[offset..offset + 3] != diagnostic.final_rgb
                            {
                                provenance_disagreement_pixels += 1;
                            }
                            let oracle_pixel = [
                                oracle_rgb[offset],
                                oracle_rgb[offset + 1],
                                oracle_rgb[offset + 2],
                            ];
                            classify_pixel(diagnostic, spans, oracle_pixel, scene.background)
                        })
                        .collect();
                    let (key, description, tiles, palettes) =
                        frame_signature(&samples).expect("divergent frame has sampled pixels");
                    (key, description, tiles, palettes)
                }
            };
            let art = cluster_art
                .entry(key.clone())
                .or_insert_with(|| ClusterArt {
                    first: (frame_index, native_rgb.clone(), oracle_rgb.clone()),
                    worst: (
                        frame_index,
                        changed_pixels,
                        native_rgb.clone(),
                        oracle_rgb.clone(),
                    ),
                });
            if changed_pixels > art.worst.1 {
                art.worst = (
                    frame_index,
                    changed_pixels,
                    native_rgb.clone(),
                    oracle_rgb.clone(),
                );
            }
            frames.push(FrameTriage {
                frame_index,
                changed_pixels,
                bbox,
                key,
                description,
                tile_histogram,
                palette_histogram,
            });
            Ok(())
        },
    )?;

    if visited_frames != divergent.len() {
        bail!(
            "replay visited only {visited_frames} of {} divergent frames — scenario/report \
             mismatch",
            divergent.len()
        );
    }

    let clusters = cluster_signatures(&frames);
    let total_changed_pixels: u64 = frames.iter().map(|frame| frame.changed_pixels as u64).sum();

    // Invariants: the clusters must account for exactly the triaged corpus.
    let cluster_pixels: u64 = clusters
        .iter()
        .map(|cluster| cluster.total_changed_pixels)
        .sum();
    let cluster_frames: usize = clusters.iter().map(|cluster| cluster.frame_count).sum();
    assert_eq!(cluster_pixels, total_changed_pixels);
    assert_eq!(cluster_frames, frames.len());
    assert_eq!(vdp_frames + scene_frames, frames.len());

    let report = VisualTriageReport {
        schema_version: 1,
        projection: mismatch.projection,
        frame_range: [start, end],
        mismatch_frames: frames.len(),
        total_changed_pixels,
        vdp_frames,
        scene_frames,
        sampled_pixels,
        provenance_disagreement_pixels,
        recount_drift_frames,
        palette_resolution_samples,
        clusters,
    };

    let out_dir = repo_root
        .join(".cache/visual-triage-runs")
        .join(chrono_timestamp());
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let report_path = out_dir.join("report.json");
    std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
        .with_context(|| format!("failed to write {}", report_path.display()))?;
    let latest_link = repo_root.join(".cache/visual-triage-runs/latest");
    let _ = std::fs::remove_file(&latest_link);
    std::os::unix::fs::symlink(&out_dir, &latest_link).ok();

    for cluster in report.clusters.iter().take(top) {
        let Some(art) = cluster_art.get(&cluster.key) else {
            continue;
        };
        for (label, frame_index, native_rgb, oracle_rgb) in [
            ("first", art.first.0, &art.first.1, &art.first.2),
            ("worst", art.worst.0, &art.worst.2, &art.worst.3),
        ] {
            let (cw, ch, composite) = generate_diff_composite(
                native_rgb,
                oracle_rgb,
                NATIVE_FRAME_WIDTH,
                NATIVE_FRAME_HEIGHT,
            );
            let file_name = format!(
                "{}-{label}-{frame_index}.png",
                cluster.key.replace(['|', ':', ','], "_")
            );
            image::save_buffer(
                out_dir.join(&file_name),
                &composite,
                cw,
                ch,
                image::ColorType::Rgb8,
            )
            .with_context(|| format!("failed to write composite {file_name}"))?;
        }
    }

    println!(
        "visual-triage: {} frames ({} scene, {} VDP), {} px, {} clusters (spatial clusterer: \
         {} report frames / {} px)",
        report.mismatch_frames,
        report.scene_frames,
        report.vdp_frames,
        report.total_changed_pixels,
        report.clusters.len(),
        mismatch.mismatch_frames,
        mismatch.total_changed_pixels,
    );
    if report.recount_drift_frames > 0 {
        println!(
            "  WARNING: {} frames recount differently from the mismatch report — regenerate it",
            report.recount_drift_frames
        );
    }
    if report.provenance_disagreement_pixels > 0 {
        println!(
            "  note: {}/{} sampled pixels disagree between the provenance re-walk and \
             render_scene (tint blends / SAT arbitration)",
            report.provenance_disagreement_pixels, report.sampled_pixels
        );
    }
    for (rank, cluster) in report.clusters.iter().take(top).enumerate() {
        println!("  {:>2}. {}", rank + 1, cluster.console_summary());
    }
    println!("  report: {}", report_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_player::FinalScenePixelWrite;

    fn write(
        command_index: usize,
        kind: &str,
        rgb: Option<[u8; 3]>,
        accepted: bool,
        tile_index: Option<u16>,
        palette_line: Option<u8>,
    ) -> FinalScenePixelWrite {
        FinalScenePixelWrite {
            command_index,
            kind: kind.to_owned(),
            origin_x: None,
            origin_y: None,
            width: None,
            height: None,
            tile_index,
            palette_line,
            palette_index: None,
            palette_colour_count: None,
            resolved_palette_slot: None,
            raw_palette_word: None,
            source_x: None,
            source_y: None,
            rgb,
            priority: None,
            prior_high_priority: false,
            accepted,
        }
    }

    fn diagnostic(writes: Vec<FinalScenePixelWrite>) -> FinalScenePixelDiagnostic {
        FinalScenePixelDiagnostic {
            x: 5,
            y: 7,
            final_rgb: [0, 0, 0],
            final_high_priority: false,
            writes,
        }
    }

    fn enemy_spans() -> Vec<LayerSpan> {
        vec![
            LayerSpan {
                layer: SceneLayer::Terrain,
                range: 0..2,
            },
            LayerSpan {
                layer: SceneLayer::Enemies,
                range: 2..4,
            },
        ]
    }

    const BACKGROUND: [u8; 3] = [28, 42, 56];

    #[test]
    fn classify_pixel_without_colour_writes_is_oracle_extra() {
        // No writes at all, and rejected / rgb-less writes, both classify
        // structurally as native-missing-content.
        for writes in [
            Vec::new(),
            vec![
                write(
                    2,
                    "sprite_tile",
                    Some([1, 2, 3]),
                    false,
                    Some(0x10),
                    Some(3),
                ),
                write(3, "clear_priority_rect", None, true, None, None),
            ],
        ] {
            let (signature, tile) =
                classify_pixel(&diagnostic(writes), &enemy_spans(), [9, 9, 9], BACKGROUND);
            assert_eq!(signature.class, DivergenceClass::OracleExtra);
            assert_eq!(signature.layer, SceneLayer::Oracle);
            assert_eq!(signature.key(), "oracle_extra|oracle|none|-");
            assert_eq!(tile, None);
        }
    }

    #[test]
    fn classify_pixel_native_extra_when_oracle_shows_background() {
        let (signature, tile) = classify_pixel(
            &diagnostic(vec![write(
                2,
                "sprite_tile",
                Some([200, 10, 10]),
                true,
                Some(0x1A2),
                Some(3),
            )]),
            &enemy_spans(),
            BACKGROUND,
            BACKGROUND,
        );
        assert_eq!(signature.class, DivergenceClass::NativeExtra);
        assert_eq!(signature.layer, SceneLayer::Enemies);
        assert_eq!(signature.key(), "native_extra|enemies|sprite_tile|pal3");
        assert_eq!(tile, Some(0x1A2));
        assert_eq!(
            signature.describe(),
            "native enemies sprite_tile (palette 3) drawn where oracle shows background"
        );
    }

    #[test]
    fn classify_pixel_recolour_uses_last_accepted_colour_write() {
        // A later rejected write and an rgb-less accepted write must not
        // steal attribution from the last accepted colour write.
        let (signature, tile) = classify_pixel(
            &diagnostic(vec![
                write(
                    0,
                    "indexed_tile",
                    Some([1, 1, 1]),
                    true,
                    Some(0x50),
                    Some(0),
                ),
                write(2, "sprite_tile", Some([2, 2, 2]), true, Some(0x60), Some(1)),
                write(
                    3,
                    "sprite_tile",
                    Some([3, 3, 3]),
                    false,
                    Some(0x70),
                    Some(2),
                ),
                write(3, "clear_priority_rect", None, true, None, None),
            ]),
            &enemy_spans(),
            [9, 9, 9],
            BACKGROUND,
        );
        assert_eq!(signature.class, DivergenceClass::Recolour);
        assert_eq!(signature.layer, SceneLayer::Enemies);
        assert_eq!(signature.key(), "recolour|enemies|sprite_tile|pal1");
        assert_eq!(tile, Some(0x60));
    }

    fn sig(
        class: DivergenceClass,
        layer: SceneLayer,
        kind: &str,
        pal: Option<u8>,
    ) -> PixelSignature {
        PixelSignature {
            class,
            layer,
            kind: kind.to_owned(),
            palette_line: pal,
        }
    }

    #[test]
    fn frame_signature_picks_dominant_and_histograms_its_pixels() {
        let enemy = sig(
            DivergenceClass::NativeExtra,
            SceneLayer::Enemies,
            "sprite_tile",
            Some(3),
        );
        let hud = sig(DivergenceClass::Recolour, SceneLayer::Hud, "text", None);
        let samples = vec![
            (enemy.clone(), Some(0x10)),
            (enemy.clone(), Some(0x10)),
            (enemy.clone(), Some(0x11)),
            (hud.clone(), None),
        ];
        let (key, _, tiles, palettes) = frame_signature(&samples).expect("samples present");
        assert_eq!(key, "native_extra|enemies|sprite_tile|pal3");
        assert_eq!(
            tiles,
            BTreeMap::from([(0x10u16, 2usize), (0x11u16, 1usize)])
        );
        assert_eq!(palettes, BTreeMap::from([(3u8, 3usize)]));
        assert_eq!(frame_signature(&[]), None);
    }

    #[test]
    fn frame_signature_ties_break_by_signature_order() {
        let a = sig(DivergenceClass::NativeExtra, SceneLayer::Enemies, "a", None);
        let b = sig(DivergenceClass::NativeExtra, SceneLayer::Enemies, "b", None);
        let (key, ..) =
            frame_signature(&[(b.clone(), None), (a.clone(), None)]).expect("samples present");
        assert_eq!(
            key,
            a.key(),
            "equal counts must pick the smallest signature"
        );
    }

    fn triage(frame_index: usize, changed_pixels: usize, key: &str) -> FrameTriage {
        FrameTriage {
            frame_index,
            changed_pixels,
            bbox: BBox {
                min_x: frame_index,
                min_y: 0,
                max_x: frame_index + 1,
                max_y: 1,
            },
            key: key.to_owned(),
            description: format!("desc {key}"),
            tile_histogram: BTreeMap::from([(0x10u16, 1usize)]),
            palette_histogram: BTreeMap::from([(3u8, 1usize)]),
        }
    }

    #[test]
    fn cluster_signatures_groups_sums_ranges_and_sorts_by_pixel_impact() {
        let frames = vec![
            triage(10, 5, "recolour|hud|text|-"),
            triage(11, 40, "recolour|hud|text|-"),
            triage(12, 8, "recolour|hud|text|-"),
            triage(20, 3, "recolour|hud|text|-"),
            triage(15, 60000, "native_extra|enemies|sprite_tile|pal3"),
        ];
        let clusters = cluster_signatures(&frames);
        assert_eq!(clusters.len(), 2);

        assert_eq!(clusters[0].key, "native_extra|enemies|sprite_tile|pal3");
        assert_eq!(clusters[0].total_changed_pixels, 60000);
        assert_eq!(clusters[0].frame_ranges, vec![[15, 15]]);
        assert_eq!(clusters[0].sample_frame_indices, vec![15]);

        let hud = &clusters[1];
        assert_eq!(hud.frame_count, 4);
        assert_eq!(hud.total_changed_pixels, 56);
        assert_eq!(hud.frame_ranges, vec![[10, 12], [20, 20]]);
        // Samples: first (10), worst (11), last (20).
        assert_eq!(hud.sample_frame_indices, vec![10, 11, 20]);
        // Worst member's bbox is the example.
        assert_eq!(hud.example_bbox.min_x, 11);
        // Histograms merge across members.
        assert_eq!(
            hud.tile_index_histogram,
            BTreeMap::from([("0x0010".to_owned(), 4usize)])
        );
        assert_eq!(
            hud.palette_line_histogram,
            BTreeMap::from([("3".to_owned(), 4usize)])
        );
    }

    #[test]
    fn oracle_png_names_parse_by_label_suffix_not_prefix() {
        assert_eq!(oracle_png_frame_index("0000_frame_0.png"), Some((0, true)));
        assert_eq!(
            oracle_png_frame_index("0042_frame_1697.png"),
            Some((1697, true))
        );
        assert_eq!(
            oracle_png_frame_index("18159_frame_18159.png"),
            Some((18159, true))
        );
        // Transition checkpoints re-capture the same frame under a longer
        // label; they parse but are not "plain" periodic captures.
        assert_eq!(
            oracle_png_frame_index("11017_transition_00_to_1b_post_frame_11017.png"),
            Some((11017, false))
        );
        assert_eq!(oracle_png_frame_index("audio.wav"), None);
        assert_eq!(oracle_png_frame_index("frame_12.png"), None);
        assert_eq!(oracle_png_frame_index("0001_frame_x.png"), None);
    }

    #[test]
    fn vdp_frame_key_quantises_bbox_to_grid_cells() {
        assert_eq!(
            vdp_frame_key(&BBox {
                min_x: 0,
                min_y: 0,
                max_x: 319,
                max_y: 223,
            }),
            "vdp:grid:0,0-39,27"
        );
        assert_eq!(
            vdp_frame_key(&BBox {
                min_x: 289,
                min_y: 13,
                max_x: 293,
                max_y: 16,
            }),
            "vdp:grid:36,1-36,2"
        );
    }

    #[test]
    fn bbox_of_coords_is_tight_and_none_when_empty() {
        assert_eq!(bbox_of_coords(&[]), None);
        assert_eq!(
            bbox_of_coords(&[(5, 2), (1, 9), (7, 3)]),
            Some(BBox {
                min_x: 1,
                min_y: 2,
                max_x: 7,
                max_y: 9,
            })
        );
    }
}
