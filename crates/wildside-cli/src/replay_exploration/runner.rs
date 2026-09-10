use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use sha1::{Digest, Sha1};

use super::{Evidence, FRAMES, assess, run_comparison, seed_scenario};
use crate::oracle::{OracleEnvironment, OracleManifest};
use crate::scenarios::{DynamicScenario, NativeDynamicRunOptions, run_native_dynamic_scenario};

pub(super) trait Runner {
    fn evaluate(&mut self, scenario: &DynamicScenario, name: &str) -> Result<Evidence>;
    fn exploration_time_left(&self) -> bool;
    fn save(&self, name: &str, value: &impl Serialize) -> Result<()>;
}

pub(super) struct LiveRunner {
    output: PathBuf,
    rom: PathBuf,
    manifest: wildside_data::AssetManifest,
    oracle: OracleEnvironment,
    started: Instant,
    deadline: Instant,
    search_deadline: Instant,
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    // Finish a complete file before replacing an incremental report.
    let temporary = path.with_extension("json.pending");
    serde_json::to_writer(File::create(&temporary)?, value)?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn wait_for_child(child: &mut std::process::Child, deadline: Instant) -> Result<()> {
    loop {
        if let Some(status) = child.try_wait()? {
            ensure!(
                status.success(),
                "oracle exited {status}; see oracle/stderr.log"
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("oracle execution deadline reached");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

impl LiveRunner {
    fn prepare() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .context("repository root")?
            .to_path_buf();
        let rom = crate::resolve_rom_path(&root, None)?;
        let verified = wildside_rom::verify_rom(&rom)?;
        let bundle = root.join(".cache/generated").join(&verified.sha1);
        let manifest = wildside_data::load_manifest(&wildside_data::manifest_path(&bundle))?;
        ensure!(
            manifest.rom_sha1 == verified.sha1,
            "asset/ROM identity mismatch"
        );
        let oracle = crate::oracle::ensure_oracles(&root)?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let output = root
            .join(".cache/replay-exploration")
            .join(format!("{stamp}-{}", std::process::id()));
        fs::create_dir_all(&output)?;
        let revision = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&root)
            .output()?;
        ensure!(revision.status.success(), "cannot record source revision");
        let diff = Command::new("git")
            .args(["diff", "HEAD", "--"])
            .current_dir(&root)
            .output()?;
        ensure!(diff.status.success(), "cannot record source changes");
        fs::write(output.join("working-tree.patch"), &diff.stdout)?;
        write_json(
            &output.join("provenance.json"),
            &serde_json::json!({
                "revision": String::from_utf8(revision.stdout)?.trim(),
                "rom_sha1": verified.sha1,
                "oracle_template_version": crate::oracle::jgenesis_helper_cache_version(),
                "oracle_binary_sha1": format!("{:x}", Sha1::digest(fs::read(&oracle.jgenesis_helper_bin)?)),
                "random_algorithm": "SplitMix64 with rejection sampling",
                "seeds": [0, 1, 2], "candidates_per_method_per_seed": super::CANDIDATES,
                "scenario_frames_per_candidate": FRAMES,
                "budget": "equal complete 420-frame replays, each with the identical settled-warp bootstrap",
                "execution_cap_seconds": 7200, "reproduction_reserve_seconds": 900,
                "elapsed_includes": "control, native execution, oracle bootstrap/replay, evidence IO and reproduction; excludes build/setup",
            }),
        )?;
        println!("replay exploration output: {}", output.display());
        let started = Instant::now();
        Ok(Self {
            output,
            rom,
            manifest,
            oracle,
            started,
            deadline: started + Duration::from_secs(7200),
            search_deadline: started + Duration::from_secs(6300),
        })
    }

    fn oracle_capture(&self, scenario_file: &Path, directory: &Path) -> Result<OracleManifest> {
        ensure!(
            Instant::now() < self.deadline,
            "pilot execution deadline reached"
        );
        fs::create_dir_all(directory)?;
        let path = directory.join("manifest.json");
        // File-backed output cannot fill a pipe while the parent polls the deadline.
        let mut child = Command::new(&self.oracle.jgenesis_helper_bin)
            .args(["--rom"])
            .arg(&self.rom)
            .arg("--scenario-file")
            .arg(scenario_file)
            .arg("--out-dir")
            .arg(directory)
            .arg("--capture-rng-stream")
            .stdout(Stdio::from(File::create(&path)?))
            .stderr(Stdio::from(File::create(directory.join("stderr.log"))?))
            .spawn()
            .context("start oracle")?;
        // A hung individual probe cannot consume the entire pilot.
        wait_for_child(
            &mut child,
            self.deadline.min(Instant::now() + Duration::from_secs(120)),
        )?;
        serde_json::from_reader(File::open(path)?).context("decode complete oracle evidence")
    }
}

impl Runner for LiveRunner {
    fn evaluate(&mut self, scenario: &DynamicScenario, name: &str) -> Result<Evidence> {
        ensure!(
            Instant::now() < self.deadline,
            "pilot execution deadline reached"
        );
        let directory = self.output.join(name);
        fs::create_dir(&directory).context("candidate output must be fresh")?;
        let scenario_path = directory.join("scenario.json");
        write_json(&scenario_path, scenario)?;
        let started = Instant::now();
        let native = run_native_dynamic_scenario(
            self.manifest.clone(),
            scenario,
            NativeDynamicRunOptions::default(),
        )?;
        let native_states = crate::sweep::extract_checkpoint_states(&native, &scenario.checkpoints);
        write_json(&directory.join("native-states.json"), &native_states)?;
        let oracle = self.oracle_capture(&scenario_path, &directory.join("oracle"))?;
        let native_crcs = native
            .iter()
            .map(wildside_core::NativeTraceFrame::authoritative_crc32)
            .collect::<Vec<_>>();
        let result = assess(
            &native_states,
            &oracle.states,
            &native_crcs,
            &oracle.frame_crcs,
            &oracle.frame_rng_states,
        );
        write_json(&directory.join("evidence.json"), &result)?;
        write_json(
            &directory.join("timing.json"),
            &serde_json::json!({
                "elapsed_seconds": started.elapsed().as_secs_f64(),
                "pilot_elapsed_seconds": self.started.elapsed().as_secs_f64(),
                "scenario_frames": FRAMES,
            }),
        )?;
        println!(
            "{name}: {} ({:.1}s)",
            result.description(),
            started.elapsed().as_secs_f64()
        );
        Ok(result)
    }

    fn exploration_time_left(&self) -> bool {
        Instant::now() < self.search_deadline
    }

    fn save(&self, name: &str, value: &impl Serialize) -> Result<()> {
        write_json(&self.output.join(format!("{name}.json")), value)
    }
}

#[test]
#[ignore = "requires ROM, generated assets and oracle; explicitly runs the bounded two-hour pilot"]
fn pilot() -> Result<()> {
    let mut runner = LiveRunner::prepare()?;
    let result = run_comparison(&mut runner);
    if let Err(error) = &result {
        runner.save(
            "failure",
            &serde_json::json!({
                "verdict": "Inconclusive", "error": format!("{error:#}"),
                "elapsed_seconds": runner.started.elapsed().as_secs_f64(),
            }),
        )?;
    }
    result?;
    println!("pilot evidence: {}", runner.output.display());
    Ok(())
}

#[test]
#[ignore = "requires ROM and oracle; reruns only the neutral adapter control"]
fn control() -> Result<()> {
    let mut runner = LiveRunner::prepare()?;
    let evidence = runner.evaluate(&seed_scenario(), "control")?;
    ensure!(
        evidence.accepted(),
        "neutral control failed: {}",
        evidence.description()
    );
    Ok(())
}

#[test]
fn child_deadline_kills_and_reaps_the_process() -> Result<()> {
    let mut child = Command::new("sleep").arg("60").spawn()?;
    assert!(wait_for_child(&mut child, Instant::now()).is_err());
    assert!(child.try_wait()?.is_some());
    Ok(())
}
