//! Private #372 experiment. The saved controller stream is replay authority.

use super::*;
use anyhow::ensure;
use serde_json::{Value, json};
use wildside_core::{BlockBounceTraceKind, terrain::BlockType};

const BATCH: usize = 6;
const MAX_FRAMES: usize = 3_600;
const START: RecordingStart = RecordingStart {
    warp_level: Some(0),
    settled: true,
    record_frames: None,
};

#[derive(Default, Debug)]
struct Encounter {
    first: Option<usize>,
    last: Option<usize>,
}

impl Encounter {
    /// ASM sub_1007A (kid.asm): the copied prize sprite owns the wobble.
    fn observe(&mut self, frame: usize, trace: &NativeTraceFrame, cell: (u16, u16)) {
        if trace.block_bounce_details.iter().any(|bounce| {
            (bounce.cell_x, bounce.cell_y) == cell && bounce.kind == BlockBounceTraceKind::Prize
        }) {
            self.first.get_or_insert(frame);
            self.last = Some(frame);
        }
    }

    fn complete(&self, frame: usize) -> bool {
        self.last.is_some_and(|last| frame >= last + 30)
    }

    fn checkpoints(&self, frames: usize) -> Vec<DynamicCheckpoint> {
        let begin = self
            .first
            .unwrap_or(frames.saturating_sub(1))
            .saturating_sub(30);
        (begin..frames)
            .map(|frame_index| DynamicCheckpoint {
                label: format!("jev-{frame_index}"),
                frame_index,
            })
            .collect()
    }
}

struct Episode {
    runtime: NativeRuntime,
    script: Vec<u8>,
    trace: Vec<NativeTraceFrame>,
    encounter: Encounter,
    target: (u16, u16),
}

impl Episode {
    fn new(manifest: &AssetManifest, target: (u16, u16)) -> Result<Self> {
        let empty = build_recording(manifest, "jev-initial", START, vec![], vec![])?;
        let initial = initialise_recording_runtime(manifest, START, &empty.manifest.scenario)?;
        Ok(Self {
            runtime: initial.runtime,
            script: vec![],
            trace: vec![],
            encounter: Encounter::default(),
            target,
        })
    }

    fn advance(&mut self, mask: u8) {
        for _ in 0..BATCH {
            if self.script.len() >= MAX_FRAMES {
                break;
            }
            let mut trace = self.runtime.step_frame(FrameInput::from_mask(mask));
            trace.level_skip_cheat_enabled = 1;
            self.encounter
                .observe(self.script.len(), &trace, self.target);
            self.script.push(mask);
            self.trace.push(trace);
            if self.encounter.complete(self.script.len() - 1) {
                break;
            }
        }
    }

    fn observation(&self) -> Value {
        let trace = self.runtime.trace_frame();
        let x = trace.kid_x_pos.unwrap_or_default() as i32 >> 16;
        let y = trace.kid_y_pos.unwrap_or_default() as i32 >> 16;
        let mut terrain = vec![];
        let mut prizes = vec![];
        if let Some(grid) = self.runtime.block_grid() {
            for row in 0..grid.height_cells {
                for col in 0..grid.width_cells {
                    let cell = grid.cell_at(col, row);
                    let px = i32::from(col) * 16;
                    let py = i32::from(row) * 16;
                    if cell.block_type == BlockType::Prize {
                        prizes.push(json!({"cell": [col, row], "x": px, "y": py}));
                    }
                    if (px - x).abs() <= 112 && (py - y).abs() <= 96 {
                        terrain.push(json!({"cell": [col, row], "x": px, "y": py,
                            "collision": format!("{:?}", cell.collision),
                            "block": format!("{:?}", cell.block_type)}));
                    }
                }
            }
        }
        json!({
            "goal": "Hit the target prize block from below with your head, then release controls to watch its animation.",
            "coordinates": "Pixels; x grows right, y grows down. Terrain cells are 16x16. Kid position is his centre.",
            "frame": self.script.len(), "target_cell": self.target,
            "target_centre_x": i32::from(self.target.0) * 16 + 8,
            "target_bottom_y": i32::from(self.target.1) * 16 + 16,
            "target_dx": i32::from(self.target.0) * 16 + 8 - x,
            "kid": {"x": x, "y": y, "vx": trace.kid_x_velocity.map(|v| f64::from(v) / 65536.0),
                "vy": trace.kid_y_velocity.map(|v| f64::from(v) / 65536.0),
                "grounded": trace.kid_on_ground, "hp": trace.number_hitpoints,
                "form": trace.current_helmet_id},
            "terrain": terrain, "prizes": prizes,
            "enemies": trace.active_enemy_details,
            "recent_masks": self.script.iter().rev().take(24).copied().collect::<Vec<_>>(),
            "target_hit": self.encounter.first.is_some()
        })
    }

    fn save(mut self, manifest: &AssetManifest, directory: &Path, reason: &str) -> Result<()> {
        ensure!(!self.script.is_empty(), "no frames applied");
        rebase_trace_frame_indices(&mut self.trace);
        let slug = directory
            .file_name()
            .and_then(|s| s.to_str())
            .context("episode slug")?;
        let mut recording = build_recording(manifest, slug, START, self.script, self.trace)?;
        recording.manifest.scenario.checkpoints = self.encounter.checkpoints(recording.trace.len());
        save_recording_with_artifacts(directory, &recording, &RecordingBundleArtifacts::default())?;
        fs::write(
            directory.join("assessment.json"),
            serde_json::to_vec_pretty(&json!({
                "status": if self.encounter.first.is_some() {"awaiting_visual_review"} else {"not_exercised"},
                "stop_reason": reason, "target_cell": self.target,
                "first_hit_frame": self.encounter.first, "last_animation_frame": self.encounter.last,
                "complete_window": self.encounter.complete(recording.trace.len() - 1)
            }))?,
        )?;
        let replay = run_recorded_trace(manifest, START, &recording.manifest.scenario)?;
        ensure!(
            serde_json::to_value(&replay)? == serde_json::to_value(&recording.trace)?,
            "recorded input does not exactly reconstruct native trace"
        );
        let frames =
            render_dynamic_checkpoint_frames(manifest, &START, &recording.manifest.scenario, 0)?;
        let images = directory.join("native");
        fs::create_dir(&images)?;
        write_rendered_frames(&images, &frames)?;
        Ok(())
    }
}

fn prepare() -> Result<(PathBuf, AssetManifest)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repo")?;
    let rom = crate::resolve_rom_path(root, None)?;
    let verified = wildside_rom::verify_rom(&rom)?;
    let bundle = root.join(".cache/generated").join(verified.sha1);
    let manifest = wildside_data::load_manifest(&wildside_data::manifest_path(&bundle))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    let output = root.join(".cache/jev-playtest").join(format!("{stamp}"));
    fs::create_dir_all(&output)?;
    println!("Jev evidence: {}", output.display());
    Ok((output, manifest))
}

#[test]
fn gameplay_regression_jev_requires_target_prize_worker_sub_1007a() {
    let mut encounter = Encounter::default();
    let mut trace = NativeTraceFrame::default();
    trace
        .block_bounce_details
        .push(wildside_core::BlockBounceTrace {
            cell_x: 8,
            cell_y: 12,
            x_offset_px: 0,
            remaining: 28,
            y_offset_px: -6,
            kind: BlockBounceTraceKind::Metal,
        });
    encounter.observe(50, &trace, (8, 12));
    assert_eq!(encounter.first, None);
    trace.block_bounce_details[0].kind = BlockBounceTraceKind::Prize;
    encounter.observe(51, &trace, (9, 12));
    assert_eq!(encounter.first, None);
    encounter.observe(52, &trace, (8, 12));
    encounter.observe(80, &trace, (8, 12));
    assert!(!encounter.complete(109));
    assert!(encounter.complete(110));
    let checkpoints = encounter.checkpoints(111);
    assert_eq!(checkpoints.first().unwrap().frame_index, 22);
    assert_eq!(checkpoints.last().unwrap().frame_index, 110);
}

#[test]
#[ignore = "requires ROM and generated assets; records the private no-network control"]
fn control() -> Result<()> {
    let (output, manifest) = prepare()?;
    let mut episode = Episode::new(&manifest, (8, 12))?;
    fs::write(
        output.join("initial.json"),
        serde_json::to_vec_pretty(&episode.observation())?,
    )?;
    let before = serde_json::to_value(episode.runtime.trace_frame())?;
    let _ = episode.observation();
    ensure!(
        before == serde_json::to_value(episode.runtime.trace_frame())?,
        "observation advanced time"
    );
    episode.advance(0);
    ensure!(
        episode.script == vec![0; BATCH],
        "incorrect applied frame count"
    );
    episode.save(&manifest, &output.join("control"), "control_complete")
}
