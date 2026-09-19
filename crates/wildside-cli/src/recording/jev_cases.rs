//! Private case adapter. Ordinary inputs and recording replay remain authoritative.

use super::*;
use anyhow::ensure;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseInput {
    issue: u16,
    output: PathBuf,
    script: Vec<u8>,
    checkpoints: Vec<usize>,
    #[serde(default)]
    live: bool,
}

fn collect(config: &CaseInput, manifest: &AssetManifest, start: RecordingStart) -> Result<Vec<u8>> {
    let initial = build_recording(manifest, "case-live", start, vec![], vec![])?;
    let mut runtime =
        initialise_recording_runtime(manifest, start, &initial.manifest.scenario)?.runtime;
    ensure!(
        config.script.is_empty() && start.settled,
        "live cases start from the settled level without a prefix"
    );
    let mut script = Vec::new();
    let mut observed = Vec::new();
    let parent = config.output.parent().context("case output parent")?;
    let mut journal = fs::File::create(parent.join("inputs.bin"))?;
    journal.write_all(&script)?;
    while script.len() < 3_600 {
        let trace = runtime.trace_frame();
        let x = trace.kid_x_pos.unwrap_or_default() as i32 >> 16;
        let y = trace.kid_y_pos.unwrap_or_default() as i32 >> 16;
        let mut terrain = vec![];
        if let Some(grid) = runtime.block_grid() {
            for row in (y / 16 - 8).max(0)..=(y / 16 + 3).max(0) {
                for col in (x / 16 - 6).max(0)..=(x / 16 + 12).max(0) {
                    let cell = grid.cell_at(col as u16, row as u16);
                    if cell.is_solid_floor() || cell.has_block {
                        terrain.push(json!({"x": col * 16, "y": row * 16,
                            "collision": format!("{:?}", cell.collision), "block": format!("{:?}", cell.block_type)}));
                    }
                }
            }
        }
        let state = json!({"goal": "Reach the BLW2 teleporter at x2503 y575, stand still for90 frames, then observe arrival near x448 y143. Stay alive and find a higher route around walls.",
            "frame": script.len(), "mode": trace.game_mode, "paused": trace.paused, "level": trace.current_level,
            "kid": {"x": x, "y": y, "vx": trace.kid_x_velocity.map(|v| f64::from(v)/65536.0),
                "vy": trace.kid_y_velocity.map(|v| f64::from(v)/65536.0), "grounded": trace.kid_on_ground,
                "hp": trace.number_hitpoints, "lives": trace.number_lives},
            "held_buttons": FrameInput::from_mask(script.last().copied().unwrap_or_default()), "terrain_cells_16px": terrain,
            "enemies": trace.active_enemy_details, "projectiles": trace.active_projectile_details});
        let mut child = Command::new("python3")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/jev_case_service.py"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let write_result = (|| -> Result<()> {
            let mut input = child.stdin.take().context("helper stdin")?;
            serde_json::to_writer(&mut input, &state)?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = child.kill();
        }
        let response = child.wait_with_output()?;
        if write_result.is_err() || !response.status.success() {
            fs::write(
                parent.join("collection-stop.txt"),
                "request failed or collection limit reached; inspect budget ledger",
            )?;
            break;
        }
        let mask: u8 = serde_json::from_slice(&response.stdout)?;
        for _ in 0..6 {
            if script.len() >= 3_600 {
                break;
            }
            let applied = script.last().copied().unwrap_or_default();
            observed.push(runtime.step_frame(FrameInput::from_mask(applied)));
            script.push(mask);
            journal.write_all(&[mask])?;
        }
        journal.flush()?;
        println!("Jev case {} frame {}", config.issue, script.len());
    }
    if start.settled {
        rebase_trace_frame_indices(&mut observed);
        for frame in &mut observed {
            frame.level_skip_cheat_enabled = 1;
        }
    }
    let recording = build_recording(manifest, "case-live-replay", start, script.clone(), vec![])?;
    let replay = run_recorded_trace(manifest, start, &recording.manifest.scenario)?;
    ensure!(
        serde_json::to_value(replay)? == serde_json::to_value(observed)?,
        "live input journal does not exactly reconstruct collected state"
    );
    Ok(script)
}

fn case_start(issue: u16) -> Result<RecordingStart> {
    let warp_level = match issue {
        210 | 254 => None,
        327 => Some(1),
        370 => Some(0),
        _ => bail!("case has no executable adapter: {issue}"),
    };
    Ok(RecordingStart {
        warp_level,
        settled: warp_level.is_some(),
        record_frames: None,
    })
}

#[test]
#[ignore = "private jev-case runner supplies inputs; requires ROM and generated assets"]
fn case_bundle() -> Result<()> {
    let config: CaseInput = serde_json::from_slice(&fs::read(std::env::var("JEV_CASE_INPUT")?)?)?;
    ensure!(!config.output.exists(), "case output must be fresh");
    ensure!(
        (config.live || !config.script.is_empty()) && config.script.len() <= 3_600,
        "invalid frame count"
    );
    ensure!(!config.checkpoints.is_empty(), "case needs checkpoints");
    ensure!(
        config.live
            || config
                .checkpoints
                .iter()
                .all(|frame| *frame < config.script.len()),
        "checkpoint outside route"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repo")?;
    let rom = crate::resolve_rom_path(root, None)?;
    let verified = wildside_rom::verify_rom(&rom)?;
    let bundle = root.join(".cache/generated").join(verified.sha1);
    let manifest = wildside_data::load_manifest(&wildside_data::manifest_path(&bundle))?;
    let start = case_start(config.issue)?;
    ensure!(
        !config.live || config.issue == 327,
        "paid discovery is only needed for the teleporter route"
    );
    let script = if config.live {
        collect(&config, &manifest, start)?
    } else {
        config.script
    };
    ensure!(!script.is_empty(), "no controller frames collected");
    let slug = format!("jev-case-{}", config.issue);
    let mut recording = build_recording(&manifest, &slug, start, script, vec![])?;
    recording.manifest.scenario.checkpoints = config
        .checkpoints
        .into_iter()
        .map(|frame_index| DynamicCheckpoint {
            label: format!("case-{frame_index}"),
            frame_index,
        })
        .collect();
    let trace = run_recorded_trace(&manifest, start, &recording.manifest.scenario)?;
    let checkpoints = recording.manifest.scenario.checkpoints;
    recording = build_recording(
        &manifest,
        &slug,
        start,
        recording.manifest.scenario.script,
        trace,
    )?;
    recording.manifest.scenario.checkpoints = checkpoints;
    save_recording_with_artifacts(
        &config.output,
        &recording,
        &RecordingBundleArtifacts::default(),
    )?;
    Ok(())
}

#[test]
fn gameplay_regression_jev_case_starts_do_not_seed_gameplay_state() -> Result<()> {
    assert_eq!(case_start(210)?.warp_level, None);
    assert!(!case_start(254)?.settled);
    assert_eq!(case_start(327)?.warp_level, Some(1));
    assert_eq!(case_start(370)?.warp_level, Some(0));
    assert!(case_start(372).is_err());
    Ok(())
}
