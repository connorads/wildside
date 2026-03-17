use crate::scenarios::Scenario;
use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BLASTEM_URL: &str = "https://github.com/libretro/blastem";
const BLASTEM_COMMIT: &str = "277e4a62668597d4f59cadda1cbafb844f981d45";
const JGENESIS_URL: &str = "https://github.com/jsgroth/jgenesis";
const JGENESIS_COMMIT: &str = "8e5ed096d90ef08060813fc157222fe466b967d2";
const HELPER_TEMPLATE: &str = include_str!("oracle_helper_main.rs.txt");

#[derive(Debug, Clone)]
pub struct OracleEnvironment {
    pub jgenesis_helper_bin: PathBuf,
    pub jgenesis_dir: PathBuf,
    pub blastem_dir: PathBuf,
    pub blastem_binary: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct OracleManifest {
    pub sample_rate: u32,
    pub audio_path: PathBuf,
    pub frames: Vec<OracleFrame>,
    pub states: Vec<OracleState>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleEnemyLayoutTraceManifest {
    pub scenario: String,
    pub checkpoints: Vec<OracleEnemyLayoutCheckpoint>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct OracleFrame {
    pub label: String,
    pub frame_index: usize,
    pub png_path: PathBuf,
    pub sha1: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EnemyFacing {
    Left,
    Right,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleEnemyState {
    pub object_type: u8,
    pub x_pos: u32,
    pub y_pos: u32,
    pub x_velocity: i32,
    pub y_velocity: i32,
    pub facing: EnemyFacing,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleEnemyLayoutCheckpoint {
    pub label: String,
    pub frame_index: usize,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub header_ptr: u32,
    pub object_ptr: u32,
    pub axis_uses_y_position: bool,
    pub enemy_type_slots: Vec<u16>,
    pub upper_bound: u16,
    pub object_batch_flags: u8,
    pub object_count: u8,
    pub objects: Vec<OracleEnemyLayoutObject>,
    pub visible_enemies: Vec<OracleVisibleEnemyLayout>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleEnemyLayoutObject {
    pub object_type: u8,
    pub resolved_object_type: Option<u8>,
    pub flags: u8,
    pub hit_points_plus_one: u16,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleVisibleEnemyLayout {
    pub object_type: u8,
    pub layout_ptr: u32,
    pub parent_object_ptr: u32,
    pub x_pos: u32,
    pub y_pos: u32,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleState {
    pub label: String,
    pub frame_index: usize,
    pub game_mode: u16,
    pub current_player: u8,
    pub number_hitpoints: u16,
    pub current_helmet_id: u16,
    pub current_level_id: u16,
    pub player_1_level_id: u16,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
    #[serde(default)]
    pub game_paused: bool,
    pub active_enemies: Vec<OracleEnemyState>,
}

pub fn ensure_oracles(repo_root: &Path) -> Result<OracleEnvironment> {
    let oracle_root = repo_root.join(".cache/oracles");
    fs::create_dir_all(&oracle_root)
        .with_context(|| format!("failed to create {}", oracle_root.display()))?;

    let jgenesis_dir = oracle_root.join("jgenesis");
    checkout_repo(&jgenesis_dir, JGENESIS_URL, JGENESIS_COMMIT)?;

    let helper_dir = oracle_root.join("jgenesis-helper");
    write_helper_project(&helper_dir, &jgenesis_dir)?;
    run_command(
        Command::new("cargo").arg("build").arg("--release").arg("-q"),
        Some(&helper_dir),
    )?;

    let blastem_dir = oracle_root.join("blastem");
    checkout_repo(&blastem_dir, BLASTEM_URL, BLASTEM_COMMIT)?;
    let blastem_binary = maybe_build_blastem(&blastem_dir)?;

    Ok(OracleEnvironment {
        jgenesis_helper_bin: helper_dir.join("target/release/kid-jgenesis-oracle"),
        jgenesis_dir,
        blastem_dir,
        blastem_binary,
    })
}

pub fn run_jgenesis_capture(
    env: &OracleEnvironment,
    rom_path: &Path,
    scenario: Scenario,
    out_dir: &Path,
) -> Result<OracleManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let stdout = capture_stdout(
        Command::new(&env.jgenesis_helper_bin)
            .arg("--rom")
            .arg(rom_path)
            .arg("--out-dir")
            .arg(out_dir)
            .arg("--scenario")
            .arg(scenario.slug()),
        None,
    )?;
    serde_json::from_slice(&stdout).context("failed to parse jgenesis oracle manifest")
}

/// Run the jgenesis oracle using a JSON scenario file (DynamicScenario).
pub fn run_jgenesis_capture_dynamic(
    env: &OracleEnvironment,
    rom_path: &Path,
    scenario_file: &Path,
    out_dir: &Path,
) -> Result<OracleManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let stdout = capture_stdout(
        Command::new(&env.jgenesis_helper_bin)
            .arg("--rom")
            .arg(rom_path)
            .arg("--out-dir")
            .arg(out_dir)
            .arg("--scenario-file")
            .arg(scenario_file),
        None,
    )?;
    serde_json::from_slice(&stdout).context("failed to parse jgenesis oracle manifest")
}

pub fn run_jgenesis_enemy_layout_trace(
    env: &OracleEnvironment,
    rom_path: &Path,
    scenario: Scenario,
    out_dir: &Path,
) -> Result<OracleEnemyLayoutTraceManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let stdout = capture_stdout(
        Command::new(&env.jgenesis_helper_bin)
            .arg("--rom")
            .arg(rom_path)
            .arg("--out-dir")
            .arg(out_dir)
            .arg("--scenario")
            .arg(scenario.slug())
            .arg("--mode")
            .arg("enemy-layout-trace"),
        None,
    )?;
    serde_json::from_slice(&stdout).context("failed to parse jgenesis enemy-layout trace")
}

fn write_helper_project(helper_dir: &Path, jgenesis_dir: &Path) -> Result<()> {
    let src_dir = helper_dir.join("src");
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("failed to create {}", src_dir.display()))?;

    let cargo_toml = format!(
        "[package]\nname = \"kid-jgenesis-oracle\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\nanyhow = \"1\"\nbincode = \"2\"\ngenesis-config = {{ path = {:?} }}\ngenesis-core = {{ path = {:?} }}\nhound = \"3.5.1\"\nimage = {{ version = \"0.25.8\", default-features = false, features = [\"png\"] }}\njgenesis-common = {{ path = {:?} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\nsha1 = \"0.10.6\"\n",
        jgenesis_dir.join("config/genesis-config"),
        jgenesis_dir.join("backend/genesis-core"),
        jgenesis_dir.join("common/jgenesis-common"),
    );

    write_if_changed(&helper_dir.join("Cargo.toml"), cargo_toml.as_bytes())?;
    write_if_changed(&src_dir.join("main.rs"), HELPER_TEMPLATE.as_bytes())?;
    Ok(())
}

fn maybe_build_blastem(dir: &Path) -> Result<Option<PathBuf>> {
    if !pkg_config_exists(&["sdl2", "glew", "gl"])? {
        return Ok(None);
    }

    run_command(Command::new("make").arg("-j4"), Some(dir))?;
    let blastem_bin = dir.join("blastem");
    if blastem_bin.is_file() {
        Ok(Some(blastem_bin))
    } else {
        Ok(None)
    }
}

fn checkout_repo(dir: &Path, url: &str, commit: &str) -> Result<()> {
    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("repo directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git")
                .arg("clone")
                .arg("--filter=blob:none")
                .arg(url)
                .arg(dir),
            None,
        )?;
    }

    run_command(
        Command::new("git").arg("fetch").arg("--all").arg("--tags"),
        Some(dir),
    )?;
    run_command(
        Command::new("git")
            .arg("checkout")
            .arg("--detach")
            .arg(commit),
        Some(dir),
    )?;
    Ok(())
}

fn pkg_config_exists(packages: &[&str]) -> Result<bool> {
    let status = Command::new("pkg-config")
        .arg("--exists")
        .args(packages)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to execute pkg-config")?;
    Ok(status.success())
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<()> {
    if let Ok(existing) = fs::read(path)
        && existing == contents
    {
        return Ok(());
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn capture_stdout(command: &mut Command, cwd: Option<&Path>) -> Result<Vec<u8>> {
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| {
            format!(
                "failed to execute {}",
                render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
            )
        })?;
    if !output.status.success() {
        bail!(
            "command failed: {}",
            render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
        );
    }
    Ok(output.stdout)
}

fn run_command(command: &mut Command, cwd: Option<&Path>) -> Result<()> {
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "failed to execute {}",
                render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
            )
        })?;
    if !status.success() {
        bail!(
            "command failed: {}",
            render_command(command, cwd.unwrap_or_else(|| Path::new(".")))
        );
    }
    Ok(())
}

fn render_command(command: &Command, cwd: &Path) -> String {
    let program = command.get_program().to_string_lossy();
    let args = command
        .get_args()
        .map(quote_argument)
        .collect::<Vec<_>>()
        .join(" ");
    format!("(cd {} && {} {})", cwd.display(), program, args)
}

fn quote_argument(value: &OsStr) -> String {
    let rendered = value.to_string_lossy();
    if rendered
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-_./:=+".contains(&byte))
    {
        rendered.into_owned()
    } else {
        format!("{rendered:?}")
    }
}
