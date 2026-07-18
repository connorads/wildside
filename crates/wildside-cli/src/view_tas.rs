use crate::diff::{annotate_diff, draw_text, generate_diff_composite};
use crate::inspect::describe_frame_input;
use crate::native_player::{
    NATIVE_FRAME_HEIGHT, NATIVE_FRAME_WIDTH, build_scene, load_session_assets,
    render_native_vdp_rgb, render_scene,
};
use crate::oracle::{OracleManifest, ensure_oracles, run_jgenesis_capture_dynamic};
use crate::scenarios::{
    DynamicCheckpoint, DynamicScenario, NativeDynamicRunOptions,
    run_native_dynamic_scenario_with_vdp,
};
use crate::sweep::{
    ADVISORY_FIELDS, FrontierKind, StateDiff, compare_sweep_states_for_frontier,
    extract_checkpoint_states, find_first_crc_mismatch,
};
use crate::verify::{FrameDiff, compare_rgb_frames, load_png_rgb};
use anyhow::{Context, Result, anyhow, bail};
use image::{ColorType, ImageFormat};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use wildside_core::terrain::BlockGrid;
use wildside_core::{FrameInput, NativeTraceFrame};
use wildside_data::AssetManifest;
use wildside_render::NativeLevelVisuals;

const VIEW_TAS_MAX_DEFAULT_FRAMES: usize = 2_000;
const VIEW_TAS_FOOTER_HEIGHT: u32 = 56;
const VIEW_TAS_LABEL_PREFIX: &str = "frame_";
const VIEW_TAS_COMPOSITE_CACHE_SIZE: usize = 180;

pub(crate) struct TasComparisonRun {
    pub(crate) scenario: DynamicScenario,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) native_trace: Vec<NativeTraceFrame>,
    /// Per-frame VDP snapshot: `Some` on non-gameplay screens (title/Sega/intro)
    /// where we render real VDP pixels, `None` during gameplay where the
    /// object-level `build_scene` path renders. See `vdp_state` (lib.rs:2491).
    pub(crate) vdp_snapshots: Vec<Option<wildside_core::vdp::VdpState>>,
    pub(crate) oracle: OracleManifest,
    pub(crate) diff_by_frame: BTreeMap<usize, Vec<StateDiff>>,
    pub(crate) divergent_frames: Vec<usize>,
    pub(crate) first_divergence: Option<usize>,
    pub(crate) run_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameStatus {
    Match,
    Advisory,
    Diverge,
}

impl FrameStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Match => "MATCH",
            Self::Advisory => "ADVISORY",
            Self::Diverge => "DIVERGE",
        }
    }
}

#[derive(Debug)]
struct CachedDisplayFrame {
    composite_rgb: Vec<u8>,
    frame_diff: FrameDiff,
}

#[derive(Debug, Serialize)]
struct ViewTasSummary {
    slug: String,
    start: usize,
    end: usize,
    frame_count: usize,
    first_divergence: Option<usize>,
    divergent_frames: Vec<usize>,
    oracle_checkpoint_count: usize,
}

pub(crate) fn load_dynamic_scenario(scenario_path: &Path) -> Result<DynamicScenario> {
    let scenario_json = fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read {}", scenario_path.display()))?;
    serde_json::from_str(&scenario_json).context("invalid scenario JSON")
}

pub(crate) fn resolve_view_tas_range(
    frame_count: usize,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    whole_run: bool,
) -> Result<(usize, usize)> {
    if whole_run && (frame_start.is_some() || frame_end.is_some()) {
        bail!("pass either --whole-run or an explicit frame range, not both");
    }

    let start = frame_start.unwrap_or(0);
    if start > frame_count {
        bail!("frame_start {start} exceeds scenario length {frame_count}");
    }

    let end = frame_end.unwrap_or(frame_count).min(frame_count);
    if start >= end {
        bail!("resolved frame range is empty ({start}..{end})");
    }

    if frame_start.is_some() || frame_end.is_some() {
        return Ok((start, end));
    }

    if whole_run || frame_count <= VIEW_TAS_MAX_DEFAULT_FRAMES {
        return Ok((0, frame_count));
    }

    bail!(
        "scenario has {frame_count} frames; pass --frame-start/--frame-end or --whole-run to load it"
    )
}

pub(crate) fn prepare_tas_comparison_run(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: &DynamicScenario,
    start: usize,
    end: usize,
    cache_namespace: &str,
    frontier_kind: FrontierKind,
) -> Result<TasComparisonRun> {
    if start >= end {
        bail!("invalid empty frame range {start}..{end}");
    }
    if end > scenario.frame_count {
        bail!(
            "frame_end {end} exceeds scenario length {}",
            scenario.frame_count
        );
    }

    let mut sliced = scenario.clone();
    sliced.script = sliced.script[..end.min(sliced.script.len())].to_vec();
    sliced.frame_count = end;
    sliced.checkpoints = (start..end)
        .map(|frame_index| DynamicCheckpoint {
            label: format!("{VIEW_TAS_LABEL_PREFIX}{frame_index}"),
            frame_index,
        })
        .collect();

    let run_dir = repo_root
        .join(".cache")
        .join(cache_namespace)
        .join(&scenario.slug)
        .join(format!("{start:06}-{end:06}"));
    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create {}", run_dir.display()))?;

    let scenario_file = run_dir.join("scenario.json");
    fs::write(&scenario_file, serde_json::to_vec_pretty(&sliced)?)
        .with_context(|| format!("failed to write {}", scenario_file.display()))?;

    println!(
        "{cache_namespace}: {} (frames {start}..{end}, frontier {:?})",
        sliced.slug, frontier_kind
    );
    println!("  running native ({end} frames)...");
    let (native_trace, vdp_snapshots) = run_native_dynamic_scenario_with_vdp(
        manifest.clone(),
        &sliced,
        NativeDynamicRunOptions::default(),
    )?;

    println!("  running oracle ({end} frames)...");
    let env = ensure_oracles(repo_root)?;
    let oracle_dir = run_dir.join("oracle");
    fs::create_dir_all(&oracle_dir)
        .with_context(|| format!("failed to create {}", oracle_dir.display()))?;
    let oracle = run_jgenesis_capture_dynamic(&env, rom_path, &scenario_file, &oracle_dir, None)?;

    let native_states = extract_checkpoint_states(&native_trace, &sliced.checkpoints);
    let state_diffs =
        compare_sweep_states_for_frontier(&native_states, &oracle.states, frontier_kind);
    let diff_by_frame = index_state_diffs_by_frame(&state_diffs)?;

    let native_crcs: Vec<u32> = native_trace
        .iter()
        .map(NativeTraceFrame::authoritative_crc32)
        .collect();
    let first_divergence = find_first_crc_mismatch(
        native_crcs
            .get(start..end)
            .ok_or_else(|| anyhow!("native CRC window {start}..{end} out of bounds"))?,
        oracle
            .frame_crcs
            .get(start..end)
            .ok_or_else(|| anyhow!("oracle CRC window {start}..{end} out of bounds"))?,
    )
    .map(|offset| start + offset);

    let divergent_frames = collect_divergent_frames(&native_crcs, &oracle.frame_crcs, start, end)
        .into_iter()
        .filter(|frame| diff_by_frame.contains_key(frame) || first_divergence == Some(*frame))
        .collect::<Vec<_>>();

    let summary = ViewTasSummary {
        slug: sliced.slug.clone(),
        start,
        end,
        frame_count: end - start,
        first_divergence,
        divergent_frames: divergent_frames.clone(),
        oracle_checkpoint_count: oracle.states.len(),
    };
    let summary_file = run_dir.join("summary.json");
    fs::write(&summary_file, serde_json::to_vec_pretty(&summary)?)
        .with_context(|| format!("failed to write {}", summary_file.display()))?;

    Ok(TasComparisonRun {
        scenario: sliced,
        start,
        end,
        native_trace,
        vdp_snapshots,
        oracle,
        diff_by_frame,
        divergent_frames,
        first_divergence,
        run_dir,
    })
}

pub(crate) fn run_view_tas(
    repo_root: &Path,
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario_path: &Path,
    frame_start: Option<usize>,
    frame_end: Option<usize>,
    whole_run: bool,
    frontier_kind: FrontierKind,
) -> Result<()> {
    let scenario = load_dynamic_scenario(scenario_path)?;
    let (start, end) =
        resolve_view_tas_range(scenario.frame_count, frame_start, frame_end, whole_run)?;
    let comparison = prepare_tas_comparison_run(
        repo_root,
        rom_path,
        manifest,
        &scenario,
        start,
        end,
        "view-tas",
        frontier_kind,
    )?;
    ViewTasApp::new(manifest.clone(), comparison).run()
}

fn index_state_diffs_by_frame(
    state_diffs: &[StateDiff],
) -> Result<BTreeMap<usize, Vec<StateDiff>>> {
    let mut diff_by_frame = BTreeMap::new();
    for diff in state_diffs {
        let frame_index = frame_index_from_checkpoint_label(&diff.checkpoint)?;
        diff_by_frame
            .entry(frame_index)
            .or_insert_with(Vec::new)
            .push(diff.clone());
    }
    Ok(diff_by_frame)
}

fn collect_divergent_frames(
    native_crcs: &[u32],
    oracle_crcs: &[u32],
    start: usize,
    end: usize,
) -> Vec<usize> {
    native_crcs[start..end]
        .iter()
        .zip(&oracle_crcs[start..end])
        .enumerate()
        .filter_map(|(offset, (native_crc, oracle_crc))| {
            (native_crc != oracle_crc).then_some(start + offset)
        })
        .collect()
}

fn frame_index_from_checkpoint_label(label: &str) -> Result<usize> {
    label
        .strip_prefix(VIEW_TAS_LABEL_PREFIX)
        .ok_or_else(|| anyhow!("unsupported checkpoint label format: {label}"))?
        .parse::<usize>()
        .with_context(|| format!("invalid checkpoint frame index in label {label}"))
}

struct ViewTasApp {
    manifest: AssetManifest,
    comparison: TasComparisonRun,
    asset_cache: HashMap<u8, (NativeLevelVisuals, BlockGrid)>,
    display_cache: HashMap<usize, CachedDisplayFrame>,
    current_frame: usize,
    show_details: bool,
    autoplay: bool,
}

impl ViewTasApp {
    fn new(manifest: AssetManifest, comparison: TasComparisonRun) -> Self {
        Self {
            manifest,
            current_frame: comparison.start,
            comparison,
            asset_cache: HashMap::new(),
            display_cache: HashMap::new(),
            show_details: true,
            autoplay: false,
        }
    }

    fn run(mut self) -> Result<()> {
        let sdl = sdl2::init().map_err(|error| anyhow!("failed to init SDL: {error}"))?;
        let video = sdl
            .video()
            .map_err(|error| anyhow!("failed to init SDL video: {error}"))?;

        let display_width = NATIVE_FRAME_WIDTH * 3;
        let display_height = NATIVE_FRAME_HEIGHT + VIEW_TAS_FOOTER_HEIGHT;

        let window = video
            .window("Kid TAS Viewer", display_width, display_height)
            .position_centered()
            .resizable()
            .build()
            .map_err(|error| anyhow!("failed to create SDL window: {error}"))?;
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|error| anyhow!("failed to create SDL canvas: {error}"))?;
        canvas
            .set_logical_size(display_width, display_height)
            .map_err(|error| anyhow!("failed to set SDL logical size: {error}"))?;

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, display_width, display_height)
            .map_err(|error| anyhow!("failed to create SDL texture: {error}"))?;
        let mut events = sdl
            .event_pump()
            .map_err(|error| anyhow!("failed to create SDL event pump: {error}"))?;

        let mut last_tick = Instant::now();
        let mut needs_redraw = true;

        loop {
            for event in events.poll_iter() {
                if self.handle_event(&event) {
                    return Ok(());
                }
                needs_redraw = true;
            }

            if self.autoplay && last_tick.elapsed() >= Duration::from_millis(16) {
                if self.current_frame + 1 < self.comparison.end {
                    self.current_frame += 1;
                    needs_redraw = true;
                } else {
                    self.autoplay = false;
                }
                last_tick = Instant::now();
            }

            if needs_redraw {
                let rgb = self.render_current_frame()?;
                texture
                    .update(None, &rgb, display_width as usize * 3)
                    .map_err(|error| anyhow!("failed to update SDL texture: {error}"))?;
                canvas.clear();
                canvas
                    .copy(&texture, None, None)
                    .map_err(|error| anyhow!("failed to copy SDL texture: {error}"))?;
                canvas.present();
                let title = format!(
                    "Kid TAS Viewer | {} | frame {} | {}",
                    self.comparison.scenario.slug,
                    self.current_frame,
                    self.frame_status(self.current_frame).label()
                );
                let _ = canvas.window_mut().set_title(&title);
                needs_redraw = false;
            }

            thread::sleep(Duration::from_millis(5));
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Quit { .. } => true,
            Event::KeyDown {
                keycode: Some(Keycode::Escape | Keycode::Q),
                repeat: false,
                ..
            } => true,
            Event::KeyDown {
                keycode: Some(Keycode::Right | Keycode::L),
                repeat: false,
                ..
            } => {
                self.current_frame = (self.current_frame + 1).min(self.comparison.end - 1);
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left | Keycode::H),
                repeat: false,
                ..
            } => {
                self.current_frame = self
                    .current_frame
                    .saturating_sub(1)
                    .max(self.comparison.start);
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::PageDown),
                repeat: false,
                ..
            } => {
                self.current_frame = (self.current_frame + 60).min(self.comparison.end - 1);
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::PageUp),
                repeat: false,
                ..
            } => {
                self.current_frame = self
                    .current_frame
                    .saturating_sub(60)
                    .max(self.comparison.start);
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::Home),
                repeat: false,
                ..
            } => {
                self.current_frame = self.comparison.start;
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::End),
                repeat: false,
                ..
            } => {
                self.current_frame = self.comparison.end - 1;
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                repeat: false,
                ..
            } => {
                self.autoplay = !self.autoplay;
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::I),
                repeat: false,
                ..
            } => {
                self.show_details = !self.show_details;
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                repeat: false,
                ..
            } => {
                if let Some(frame) = self.comparison.first_divergence {
                    self.current_frame = frame;
                }
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::N),
                repeat: false,
                ..
            } => {
                if let Some(frame) = self.next_divergent_frame() {
                    self.current_frame = frame;
                }
                false
            }
            Event::KeyDown {
                keycode: Some(Keycode::P),
                repeat: false,
                ..
            } => {
                if let Some(frame) = self.previous_divergent_frame() {
                    self.current_frame = frame;
                }
                false
            }
            _ => false,
        }
    }

    fn next_divergent_frame(&self) -> Option<usize> {
        self.comparison
            .divergent_frames
            .iter()
            .copied()
            .find(|frame| *frame > self.current_frame)
    }

    fn previous_divergent_frame(&self) -> Option<usize> {
        self.comparison
            .divergent_frames
            .iter()
            .rev()
            .copied()
            .find(|frame| *frame < self.current_frame)
    }

    fn frame_status(&self, frame_index: usize) -> FrameStatus {
        let diffs = self.comparison.diff_by_frame.get(&frame_index);
        if let Some(frame_diffs) = diffs {
            if frame_diffs
                .iter()
                .any(|diff| !ADVISORY_FIELDS.contains(&diff.field.as_str()))
            {
                return FrameStatus::Diverge;
            }
            if !frame_diffs.is_empty() {
                return FrameStatus::Advisory;
            }
        }

        let native_crc = self
            .comparison
            .native_trace
            .get(frame_index)
            .map(NativeTraceFrame::authoritative_crc32);
        let oracle_crc = self.comparison.oracle.frame_crcs.get(frame_index).copied();
        if native_crc == oracle_crc {
            FrameStatus::Match
        } else {
            FrameStatus::Diverge
        }
    }

    fn render_current_frame(&mut self) -> Result<Vec<u8>> {
        let display_width = (NATIVE_FRAME_WIDTH * 3) as usize;
        let display_height = (NATIVE_FRAME_HEIGHT + VIEW_TAS_FOOTER_HEIGHT) as usize;
        let mut rgb = vec![0; display_width * display_height * 3];

        let frame_index = self.current_frame;
        let (composite_rgb, frame_diff) = {
            let display_frame = self.cached_display_frame(frame_index)?;
            (
                display_frame.composite_rgb.clone(),
                display_frame.frame_diff,
            )
        };
        let composite_height = NATIVE_FRAME_HEIGHT as usize;
        let composite_len = display_width * composite_height * 3;
        rgb[..composite_len].copy_from_slice(&composite_rgb);

        let native_frame = self
            .comparison
            .native_trace
            .get(frame_index)
            .ok_or_else(|| anyhow!("missing native frame {frame_index}"))?;
        let input_mask = self
            .comparison
            .scenario
            .script
            .get(frame_index)
            .copied()
            .unwrap_or(0);
        let input_summary = describe_frame_input(FrameInput::from_mask(input_mask));
        let first_divergence = self
            .comparison
            .first_divergence
            .map_or_else(|| String::from("none"), |frame| frame.to_string());
        let native_crc = native_frame.authoritative_crc32();
        let oracle_crc = self
            .comparison
            .oracle
            .frame_crcs
            .get(frame_index)
            .copied()
            .unwrap_or(0);
        let status = self.frame_status(frame_index);

        let lines = self.footer_lines(
            frame_index,
            status,
            &input_summary,
            native_crc,
            oracle_crc,
            frame_diff,
            native_frame,
        );

        let footer_y = NATIVE_FRAME_HEIGHT as usize + 4;
        for (index, line) in lines.iter().enumerate() {
            draw_text(&mut rgb, display_width, 4, footer_y + index * 8, line);
        }
        draw_text(
            &mut rgb,
            display_width,
            4,
            footer_y + 6 * 8,
            &format!(
                "SPACE play  LEFT/RIGHT step  PGUP/PGDN +/-60  HOME/END  D first={first_divergence}  N/P diff  I details  Q quit"
            ),
        );

        Ok(rgb)
    }

    fn footer_lines(
        &self,
        frame_index: usize,
        status: FrameStatus,
        input_summary: &str,
        native_crc: u32,
        oracle_crc: u32,
        frame_diff: FrameDiff,
        native_frame: &NativeTraceFrame,
    ) -> [String; 6] {
        let header = format!(
            "{} | frame {} [{}..{}) | {} | input {}",
            self.comparison.scenario.slug,
            frame_index,
            self.comparison.start,
            self.comparison.end,
            status.label(),
            input_summary
        );
        let state = format!(
            "mode {:?} | level {:02X} | kid {:08X},{:08X} | cam {:08X},{:08X}",
            native_frame.game_mode,
            native_frame.current_level.0,
            native_frame.kid_x_pos.unwrap_or(0),
            native_frame.kid_y_pos.unwrap_or(0),
            native_frame.camera_x_pos,
            native_frame.camera_y_pos
        );
        let crc_line = format!(
            "crc native={native_crc:08X} oracle={oracle_crc:08X} | mae {:.2} | max delta {}",
            frame_diff.mean_absolute_error, frame_diff.max_channel_delta
        );

        if !self.show_details {
            return [
                header,
                state,
                crc_line,
                String::from("details hidden"),
                String::new(),
                String::new(),
            ];
        }

        let mut diff_lines = [String::new(), String::new(), String::new()];
        for (slot, line) in diff_lines.iter_mut().zip(
            self.comparison
                .diff_by_frame
                .get(&frame_index)
                .into_iter()
                .flatten()
                .take(3)
                .map(format_diff_line),
        ) {
            *slot = line;
        }

        [
            header,
            state,
            crc_line,
            diff_lines[0].clone(),
            diff_lines[1].clone(),
            diff_lines[2].clone(),
        ]
    }

    fn cached_display_frame(&mut self, frame_index: usize) -> Result<&CachedDisplayFrame> {
        if !self.display_cache.contains_key(&frame_index) {
            let frame = self.render_display_frame(frame_index)?;
            self.display_cache.insert(frame_index, frame);
            self.prune_display_cache(frame_index);
        }
        self.display_cache
            .get(&frame_index)
            .ok_or_else(|| anyhow!("display cache lost frame {frame_index}"))
    }

    fn render_display_frame(&mut self, frame_index: usize) -> Result<CachedDisplayFrame> {
        let native_rgb = self.render_native_rgb(frame_index)?;
        let oracle_frame = self
            .comparison
            .oracle
            .frames
            .iter()
            .find(|frame| frame.frame_index == frame_index)
            .ok_or_else(|| anyhow!("missing oracle frame {frame_index}"))?;
        let oracle_rgb = load_png_rgb(&oracle_frame.png_path)?;
        let frame_diff = compare_rgb_frames(&native_rgb, &oracle_rgb)?;

        let (diff_width, _diff_height, mut composite_rgb) = generate_diff_composite(
            &native_rgb,
            &oracle_rgb,
            NATIVE_FRAME_WIDTH,
            NATIVE_FRAME_HEIGHT,
        );
        let level_id = self
            .comparison
            .native_trace
            .get(frame_index)
            .map_or(0, |frame| frame.current_level.0);
        annotate_diff(
            &mut composite_rgb,
            diff_width,
            level_id,
            &format!("{VIEW_TAS_LABEL_PREFIX}{frame_index}"),
            frame_diff.mean_absolute_error,
        );

        let diff_dir = self.comparison.run_dir.join("diff");
        fs::create_dir_all(&diff_dir)
            .with_context(|| format!("failed to create {}", diff_dir.display()))?;
        let diff_path = diff_dir.join(format!("{frame_index:06}.png"));
        write_rgb_png(
            &diff_path,
            diff_width as usize,
            NATIVE_FRAME_HEIGHT as usize,
            &composite_rgb,
        )?;

        Ok(CachedDisplayFrame {
            composite_rgb,
            frame_diff,
        })
    }

    fn render_native_rgb(&mut self, frame_index: usize) -> Result<Vec<u8>> {
        let frame = self
            .comparison
            .native_trace
            .get(frame_index)
            .ok_or_else(|| anyhow!("missing native frame {frame_index}"))?;
        let mut rgb = vec![0; (NATIVE_FRAME_WIDTH * NATIVE_FRAME_HEIGHT * 3) as usize];

        // Prefer the engine's own VDP reconstruction (title/Sega/intro/menu)
        // for pixel-faithful native-vs-oracle diffs; fall back to the
        // object-level scene during gameplay where `vdp_state` is `None`.
        if let Some(vdp) = self
            .comparison
            .vdp_snapshots
            .get(frame_index)
            .and_then(Option::as_ref)
        {
            render_native_vdp_rgb(vdp, &mut rgb);
        } else {
            let level_id = frame.current_level.0;
            if !self.asset_cache.contains_key(&level_id) {
                let assets = load_session_assets(&self.manifest, level_id)?;
                self.asset_cache.insert(level_id, assets);
            }

            let (level_visuals, block_grid) = self
                .asset_cache
                .get(&level_id)
                .ok_or_else(|| anyhow!("asset cache missing level {level_id:02X}"))?;
            let scene = build_scene(frame, false, Some(level_visuals), Some(block_grid));
            render_scene(&scene, &mut rgb);
        }

        let native_dir = self.comparison.run_dir.join("native");
        fs::create_dir_all(&native_dir)
            .with_context(|| format!("failed to create {}", native_dir.display()))?;
        let native_path = native_dir.join(format!("{frame_index:06}.png"));
        write_rgb_png(
            &native_path,
            NATIVE_FRAME_WIDTH as usize,
            NATIVE_FRAME_HEIGHT as usize,
            &rgb,
        )?;

        Ok(rgb)
    }

    fn prune_display_cache(&mut self, centre_frame: usize) {
        if self.display_cache.len() <= VIEW_TAS_COMPOSITE_CACHE_SIZE {
            return;
        }
        let keep_distance = VIEW_TAS_COMPOSITE_CACHE_SIZE / 3;
        self.display_cache
            .retain(|frame, _| frame.abs_diff(centre_frame) <= keep_distance);
    }
}

fn write_rgb_png(path: &Path, width: usize, height: usize, rgb: &[u8]) -> Result<()> {
    image::save_buffer_with_format(
        path,
        rgb,
        width as u32,
        height as u32,
        ColorType::Rgb8,
        ImageFormat::Png,
    )
    .with_context(|| format!("failed to write {}", path.display()))
}

fn format_diff_line(diff: &StateDiff) -> String {
    let advisory = if ADVISORY_FIELDS.contains(&diff.field.as_str()) {
        " [adv]"
    } else {
        ""
    };
    format!(
        "{} native={} oracle={}{}",
        diff.field, diff.native_value, diff.oracle_value, advisory
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sweep::DivergenceCategory;
    use wildside_core::LevelId;

    fn empty_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::new(),
            upstream_commit: String::new(),
            source_root: PathBuf::new(),
            output_root: PathBuf::new(),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    fn dynamic_scenario(frame_count: usize) -> DynamicScenario {
        DynamicScenario {
            slug: "view-tas-test".to_string(),
            script: vec![0; frame_count],
            frame_count,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        }
    }

    fn oracle_with_crcs(frame_crcs: Vec<u32>) -> OracleManifest {
        OracleManifest {
            sample_rate: 0,
            audio_path: PathBuf::new(),
            frames: Vec::new(),
            states: Vec::new(),
            frame_crcs,
            frame_rng_states: Vec::new(),
            memory_watches: Vec::new(),
            vdp_pixel_samples: Vec::new(),
            vdp_fifo_commits: Vec::new(),
            vdp_fifo_enqueues: Vec::new(),
            m68k_pc_trace: Vec::new(),
            settle_trace: Vec::new(),
        }
    }

    fn trace_frames(frame_count: usize) -> Vec<NativeTraceFrame> {
        (0..frame_count)
            .map(|frame_index| NativeTraceFrame {
                frame_index,
                current_level: LevelId(0x0A),
                camera_x_pos: 0x0100_0000 + u32::try_from(frame_index).unwrap(),
                camera_y_pos: 0x0200_0000,
                kid_x_pos: Some(0x0300_0000 + u32::try_from(frame_index).unwrap()),
                kid_y_pos: Some(0x0400_0000),
                ..NativeTraceFrame::default()
            })
            .collect()
    }

    fn matching_oracle_crcs(trace: &[NativeTraceFrame]) -> Vec<u32> {
        trace
            .iter()
            .map(NativeTraceFrame::authoritative_crc32)
            .collect()
    }

    fn state_diff(frame_index: usize, field: &str) -> StateDiff {
        StateDiff {
            checkpoint: format!("{VIEW_TAS_LABEL_PREFIX}{frame_index}"),
            field: field.to_string(),
            native_value: "native".to_string(),
            oracle_value: "oracle".to_string(),
            category: DivergenceCategory::Authoritative,
        }
    }

    fn view_app(
        current_frame: usize,
        native_trace: Vec<NativeTraceFrame>,
        oracle_crcs: Vec<u32>,
        diff_by_frame: BTreeMap<usize, Vec<StateDiff>>,
        divergent_frames: Vec<usize>,
    ) -> ViewTasApp {
        let frame_count = native_trace.len();
        ViewTasApp {
            manifest: empty_manifest(),
            comparison: TasComparisonRun {
                scenario: dynamic_scenario(frame_count),
                start: 0,
                end: frame_count,
                native_trace,
                vdp_snapshots: (0..frame_count).map(|_| None).collect(),
                oracle: oracle_with_crcs(oracle_crcs),
                diff_by_frame,
                divergent_frames,
                first_divergence: None,
                run_dir: PathBuf::new(),
            },
            asset_cache: HashMap::new(),
            display_cache: HashMap::new(),
            current_frame,
            show_details: true,
            autoplay: false,
        }
    }

    #[test]
    fn resolve_view_range_uses_whole_run_for_short_scenarios() {
        assert_eq!(
            resolve_view_tas_range(1_200, None, None, false).unwrap(),
            (0, 1_200)
        );
    }

    #[test]
    fn resolve_view_range_requires_explicit_opt_in_for_long_scenarios() {
        let error = resolve_view_tas_range(3_000, None, None, false).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pass --frame-start/--frame-end or --whole-run")
        );
    }

    #[test]
    fn resolve_view_range_accepts_explicit_slice() {
        assert_eq!(
            resolve_view_tas_range(3_000, Some(120), Some(240), false).unwrap(),
            (120, 240)
        );
    }

    #[test]
    fn resolve_view_range_treats_start_equal_to_length_as_empty_not_out_of_bounds() {
        let error = resolve_view_tas_range(10, Some(10), None, false).unwrap_err();
        assert!(error.to_string().contains("resolved frame range is empty"));
    }

    #[test]
    fn resolve_view_range_rejects_start_beyond_length_as_out_of_bounds() {
        let error = resolve_view_tas_range(10, Some(11), None, false).unwrap_err();
        assert!(error.to_string().contains("exceeds scenario length"));
    }

    #[test]
    fn resolve_view_range_accepts_start_only_slice_for_long_scenarios() {
        assert_eq!(
            resolve_view_tas_range(3_000, Some(2_800), None, false).unwrap(),
            (2_800, 3_000)
        );
    }

    #[test]
    fn resolve_view_range_accepts_end_only_slice_for_long_scenarios() {
        assert_eq!(
            resolve_view_tas_range(3_000, None, Some(120), false).unwrap(),
            (0, 120)
        );
    }

    #[test]
    fn resolve_view_range_rejects_whole_run_and_slice_together() {
        let error = resolve_view_tas_range(3_000, Some(120), None, true).unwrap_err();
        assert!(error.to_string().contains("--whole-run"));
    }

    #[test]
    fn frame_index_parser_accepts_expected_labels() {
        assert_eq!(frame_index_from_checkpoint_label("frame_719").unwrap(), 719);
    }

    #[test]
    fn frame_index_parser_rejects_unknown_labels() {
        let error = frame_index_from_checkpoint_label("level_settle").unwrap_err();
        assert!(error.to_string().contains("unsupported checkpoint label"));
    }

    #[test]
    fn frame_status_labels_are_stable_for_viewer_footer_and_title() {
        assert_eq!(FrameStatus::Match.label(), "MATCH");
        assert_eq!(FrameStatus::Advisory.label(), "ADVISORY");
        assert_eq!(FrameStatus::Diverge.label(), "DIVERGE");
    }

    #[test]
    fn index_state_diffs_groups_multiple_diffs_by_checkpoint_frame() {
        let indexed = index_state_diffs_by_frame(&[
            state_diff(7, "camera_x_pos"),
            state_diff(7, "kid_x_pos"),
            state_diff(9, "score"),
        ])
        .unwrap();

        assert_eq!(indexed.keys().copied().collect::<Vec<_>>(), vec![7, 9]);
        assert_eq!(indexed[&7].len(), 2);
        assert_eq!(indexed[&7][0].field, "camera_x_pos");
        assert_eq!(indexed[&7][1].field, "kid_x_pos");
        assert_eq!(indexed[&9][0].field, "score");
    }

    #[test]
    fn index_state_diffs_rejects_unknown_checkpoint_labels() {
        let error = index_state_diffs_by_frame(&[StateDiff {
            checkpoint: "level_settle".to_string(),
            field: "camera_x_pos".to_string(),
            native_value: "1".to_string(),
            oracle_value: "2".to_string(),
            category: DivergenceCategory::Authoritative,
        }])
        .unwrap_err();

        assert!(error.to_string().contains("unsupported checkpoint label"));
    }

    #[test]
    fn collect_divergent_frames_reports_window_relative_mismatches_as_absolute_frames() {
        assert_eq!(
            collect_divergent_frames(&[10, 20, 30, 40], &[10, 99, 30, 88], 1, 4),
            vec![1, 3]
        );
    }

    #[test]
    fn divergent_frame_navigation_skips_current_frame() {
        let trace = trace_frames(24);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let mut app = view_app(10, trace, oracle_crcs, BTreeMap::new(), vec![4, 10, 12, 20]);

        assert_eq!(app.next_divergent_frame(), Some(12));
        assert_eq!(app.previous_divergent_frame(), Some(4));

        app.current_frame = 20;
        assert_eq!(app.next_divergent_frame(), None);

        app.current_frame = 4;
        assert_eq!(app.previous_divergent_frame(), None);
    }

    #[test]
    fn frame_status_prefers_authoritative_state_diffs_over_matching_crc() {
        let trace = trace_frames(3);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let app = view_app(
            1,
            trace,
            oracle_crcs,
            BTreeMap::from([(1, vec![state_diff(1, "camera_x_pos")])]),
            vec![1],
        );

        assert_eq!(app.frame_status(1), FrameStatus::Diverge);
    }

    #[test]
    fn frame_status_treats_empty_diff_bucket_as_crc_match() {
        let trace = trace_frames(3);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let app = view_app(
            1,
            trace,
            oracle_crcs,
            BTreeMap::from([(1, Vec::new())]),
            Vec::new(),
        );

        assert_eq!(app.frame_status(1), FrameStatus::Match);
    }

    #[test]
    fn frame_status_uses_crc_when_no_state_diff_exists() {
        let trace = trace_frames(3);
        let mut oracle_crcs = matching_oracle_crcs(&trace);
        oracle_crcs[2] ^= 0xFFFF_0000;
        let app = view_app(1, trace, oracle_crcs, BTreeMap::new(), vec![2]);

        assert_eq!(app.frame_status(1), FrameStatus::Match);
        assert_eq!(app.frame_status(2), FrameStatus::Diverge);
    }

    #[test]
    fn footer_lines_hide_details_when_toggled_off() {
        let trace = trace_frames(2);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let mut app = view_app(1, trace.clone(), oracle_crcs, BTreeMap::new(), Vec::new());
        app.show_details = false;

        let lines = app.footer_lines(
            1,
            FrameStatus::Match,
            "none",
            0x1122_3344,
            0x1122_3344,
            FrameDiff {
                mean_absolute_error: 0.0,
                max_channel_delta: 0,
            },
            &trace[1],
        );

        assert!(lines[0].contains("view-tas-test | frame 1 [0..2) | MATCH"));
        assert_eq!(lines[3], "details hidden");
        assert_eq!(lines[4], "");
        assert_eq!(lines[5], "");
    }

    #[test]
    fn footer_lines_show_first_three_state_diffs() {
        let trace = trace_frames(2);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let app = view_app(
            1,
            trace.clone(),
            oracle_crcs,
            BTreeMap::from([(
                1,
                vec![
                    state_diff(1, "camera_x_pos"),
                    state_diff(1, "kid_x_pos"),
                    state_diff(1, "score"),
                    state_diff(1, "diamond_count"),
                ],
            )]),
            vec![1],
        );

        let lines = app.footer_lines(
            1,
            FrameStatus::Diverge,
            "A",
            0x1122_3344,
            0x5566_7788,
            FrameDiff {
                mean_absolute_error: 1.25,
                max_channel_delta: 9,
            },
            &trace[1],
        );

        assert_eq!(lines[3], "camera_x_pos native=native oracle=oracle");
        assert_eq!(lines[4], "kid_x_pos native=native oracle=oracle");
        assert_eq!(lines[5], "score native=native oracle=oracle");
        assert!(!lines.iter().any(|line| line.contains("diamond_count")));
    }

    #[test]
    fn footer_lines_pad_missing_state_diffs() {
        let trace = trace_frames(2);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let app = view_app(
            1,
            trace.clone(),
            oracle_crcs,
            BTreeMap::from([(1, vec![state_diff(1, "camera_x_pos")])]),
            vec![1],
        );

        let lines = app.footer_lines(
            1,
            FrameStatus::Diverge,
            "A",
            0,
            1,
            FrameDiff {
                mean_absolute_error: 1.0,
                max_channel_delta: 2,
            },
            &trace[1],
        );

        assert_eq!(lines[3], "camera_x_pos native=native oracle=oracle");
        assert_eq!(lines[4], "");
        assert_eq!(lines[5], "");
    }

    #[test]
    fn prune_display_cache_keeps_window_around_centre_frame() {
        let trace = trace_frames(1);
        let oracle_crcs = matching_oracle_crcs(&trace);
        let mut app = view_app(0, trace, oracle_crcs, BTreeMap::new(), Vec::new());
        for frame_index in 0..=200 {
            app.display_cache.insert(
                frame_index,
                CachedDisplayFrame {
                    composite_rgb: Vec::new(),
                    frame_diff: FrameDiff {
                        mean_absolute_error: 0.0,
                        max_channel_delta: 0,
                    },
                },
            );
        }

        app.prune_display_cache(100);

        assert_eq!(app.display_cache.len(), 121);
        assert!(app.display_cache.contains_key(&40));
        assert!(app.display_cache.contains_key(&100));
        assert!(app.display_cache.contains_key(&160));
        assert!(!app.display_cache.contains_key(&39));
        assert!(!app.display_cache.contains_key(&161));
    }

    #[test]
    fn format_diff_line_includes_field_values_and_advisory_suffix_policy() {
        assert_eq!(
            format_diff_line(&state_diff(12, "camera_x_pos")),
            "camera_x_pos native=native oracle=oracle"
        );
    }
}
