use crate::capture::sha1_hex;
use crate::scenarios::Scenario;
use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const BLASTEM_URL: &str = "https://github.com/libretro/blastem";
const BLASTEM_COMMIT: &str = "277e4a62668597d4f59cadda1cbafb844f981d45";
const JGENESIS_URL: &str = "https://github.com/jsgroth/jgenesis";
const JGENESIS_COMMIT: &str = "8e5ed096d90ef08060813fc157222fe466b967d2";
const HELPER_TEMPLATE: &str = include_str!("oracle_helper_main.rs.txt");

/// SHA1 of the helper template. Folded into `OracleCacheKey::script_sha1`
/// so that any change to the helper's boot/warp/capture logic unreaches
/// stale oracle-cache entries without manual deletion.
pub(crate) fn jgenesis_helper_cache_version() -> &'static str {
    static CACHE_VERSION: OnceLock<String> = OnceLock::new();
    CACHE_VERSION.get_or_init(|| sha1_hex(HELPER_TEMPLATE.as_bytes()))
}

/// Extra options for oracle runs beyond the base scenario.
#[derive(Debug, Clone, Default)]
pub(crate) struct OracleRunOptions {
    pub(crate) skip_frames_before: Option<usize>,
    pub(crate) memory_watches: Option<String>,
    pub(crate) wram_dump_frames: Option<String>,
    pub(crate) vram_dump_frames: Option<String>,
    pub(crate) cram_dump_frames: Option<String>,
    pub(crate) vsram_dump_frames: Option<String>,
    pub(crate) vdp_pixel_samples: Option<String>,
    pub(crate) capture_cpu_state: bool,
    pub(crate) capture_settle_trace: bool,
    pub(crate) capture_rng_stream: bool,
}

/// Abstraction over emulator backends for oracle comparison.
pub(crate) trait OracleBackend: Send + Sync {
    fn name(&self) -> &'static str;
    /// Version string identifying the current helper binary. Must change
    /// whenever the helper's observable behaviour changes. Folded into the
    /// oracle cache key so stale entries become unreachable on upgrade.
    fn cache_version(&self) -> &str;
    fn run_scenario(
        &self,
        rom_path: &Path,
        scenario_file: &Path,
        out_dir: &Path,
        skip_frames_before: Option<usize>,
    ) -> Result<OracleManifest>;
}

/// jgenesis-based oracle backend (default).
pub(crate) struct JgenesisBackend {
    pub(crate) env: OracleEnvironment,
}

impl OracleBackend for JgenesisBackend {
    fn name(&self) -> &'static str {
        "jgenesis"
    }

    fn cache_version(&self) -> &str {
        jgenesis_helper_cache_version()
    }

    fn run_scenario(
        &self,
        rom_path: &Path,
        scenario_file: &Path,
        out_dir: &Path,
        skip_frames_before: Option<usize>,
    ) -> Result<OracleManifest> {
        run_jgenesis_capture_dynamic(
            &self.env,
            rom_path,
            scenario_file,
            out_dir,
            skip_frames_before,
        )
    }
}

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
    #[serde(default)]
    pub frame_crcs: Vec<u32>,
    #[serde(default)]
    pub frame_rng_states: Vec<Option<wildside_core::RomRandomFrameState>>,
    #[serde(default)]
    pub memory_watches: Vec<HashMap<String, i64>>,
    #[serde(default)]
    pub vdp_pixel_samples: Vec<OracleVdpPixelSampleRow>,
    #[serde(default)]
    pub settle_trace: Vec<OracleSettleTraceRow>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct OracleVdpPixelSampleRow {
    pub frame: usize,
    pub samples: String,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct OracleSettleTraceRow {
    pub phase: String,
    pub step: usize,
    pub game_mode: u16,
    pub current_level_id: u16,
    pub rng_offset: u16,
    #[serde(default)]
    pub rng_last_byte: u8,
    #[serde(default)]
    pub rng_ram: Vec<u8>,
    pub paused: bool,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub kid_x_vel: i32,
    pub kid_y_vel: i32,
    pub number_objects: u16,
    #[serde(default)]
    pub object_slots_loc_e1e: String,
    #[serde(default)]
    pub drips_object_slots_loc_33488: String,
    #[serde(default)]
    pub background_scroll_loc_3050a: String,
    #[serde(default)]
    pub background_row_offsets_loc_30744: String,
    #[serde(default)]
    pub background_source_window_loc_30744: String,
    #[serde(default)]
    pub background_source_absolute_loc_30744: String,
    #[serde(default)]
    pub background_source_rows_loc_12848: String,
    #[serde(default)]
    pub background_plane_b_rows_loc_307d8: String,
    #[serde(default)]
    pub background_plane_b_spans_loc_307d8: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision_type: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision_source: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_ptr: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_object_ptr: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gfx_slot_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_ground: Option<bool>,
    /// Enemy `current_hp` word from the parent Object struct (offset $44 per
    /// `variables.asm`).  Drops by one each time a friendly hit lands at
    /// `loc_3D6E0` / `loc_C20A` and similar damage paths, so any "did the
    /// attack land" assertion needs both sides to agree on this byte.
    /// Stored u16 to mirror the M68K word; native populates from
    /// `NativeObjectSnapshot.hit_points` widened to u16.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_hp: Option<u16>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub debug_fields: BTreeMap<String, String>,
}

/// M68K CPU register snapshot for sub-routine-level comparison.
#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleCpuRegisters {
    pub pc: u32,
    pub data_registers: [u32; 8],
    pub address_registers: [u32; 7],
    pub status_register: u16,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, PartialEq, Eq)]
pub struct OracleState {
    pub label: String,
    pub frame_index: usize,
    pub game_mode: u16,
    #[serde(default)]
    pub boot_phase: Option<String>,
    #[serde(default)]
    pub boot_phase_elapsed: Option<usize>,
    #[serde(default)]
    pub title_screen_ready: bool,
    #[serde(default)]
    pub main_menu_option: Option<u8>,
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
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub number_lives: u16,
    #[serde(default)]
    pub number_diamonds: u16,
    #[serde(default)]
    pub diamond_power_active: bool,
    #[serde(default)]
    pub murder_wall_x: Option<i32>,
    #[serde(default)]
    pub kid_x_vel: Option<i32>,
    #[serde(default)]
    pub kid_y_vel: Option<i32>,
    #[serde(default)]
    pub character_movement: Option<u16>,
    #[serde(default)]
    pub just_received_damage: Option<bool>,
    #[serde(default)]
    pub kid_collision_type: Option<u16>,
    #[serde(default)]
    pub kid_collision_source: Option<u16>,
    /// Kid GfxObject `x_direction` byte (offset $16): 0 = facing right (default
    /// sprite orientation), $FF = facing left (sprite flipped). Native and
    /// oracle MUST agree every frame — divergence is the moonwalk-up-slopes
    /// signature (#137) and the only way to detect facing-direction bugs in
    /// the sweep.
    #[serde(default)]
    pub kid_x_direction: Option<u8>,
    #[serde(default)]
    pub invulnerable_frames: Option<u8>,
    #[serde(default)]
    pub currently_transforming: Option<bool>,
    #[serde(default)]
    pub rng_offset: Option<u16>,
    #[serde(default)]
    pub rng_last_byte: Option<u8>,
    #[serde(default)]
    pub rng_ring: Option<wildside_core::RomRandomRing>,
    #[serde(default)]
    pub level_special_effects: Option<u16>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gameplay_debug_fields: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_registers: Option<OracleCpuRegisters>,
}

pub fn ensure_oracles(repo_root: &Path) -> Result<OracleEnvironment> {
    let oracle_root = repo_root.join(".cache/oracles");
    fs::create_dir_all(&oracle_root)
        .with_context(|| format!("failed to create {}", oracle_root.display()))?;

    let jgenesis_dir = oracle_root.join("jgenesis");
    checkout_repo(&jgenesis_dir, JGENESIS_URL, JGENESIS_COMMIT)?;
    patch_jgenesis_cpu_registers(&jgenesis_dir)?;

    let helper_dir = oracle_root.join("jgenesis-helper");
    write_helper_project(&helper_dir, repo_root, &jgenesis_dir)?;
    run_command(
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("-q"),
        Some(&helper_dir),
    )?;

    let blastem_dir = oracle_root.join("blastem");
    checkout_repo(&blastem_dir, BLASTEM_URL, BLASTEM_COMMIT)?;
    let blastem_binary = maybe_build_blastem(&blastem_dir)?;

    Ok(OracleEnvironment {
        jgenesis_helper_bin: helper_dir.join("target/release/wildside-jgenesis-oracle"),
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
///
/// When `skip_frames_before` is `Some(N)`, the oracle still runs from frame 0
/// but only captures PNGs and state from frame N onwards. CRC computation
/// runs for all frames regardless, keeping the `frame_crcs` array complete.
pub fn run_jgenesis_capture_dynamic(
    env: &OracleEnvironment,
    rom_path: &Path,
    scenario_file: &Path,
    out_dir: &Path,
    skip_frames_before: Option<usize>,
) -> Result<OracleManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let mut cmd = Command::new(&env.jgenesis_helper_bin);
    cmd.arg("--rom")
        .arg(rom_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--scenario-file")
        .arg(scenario_file);
    if let Some(skip) = skip_frames_before {
        cmd.arg("--skip-frames-before").arg(skip.to_string());
    }
    let stdout = capture_stdout(&mut cmd, None)?;
    serde_json::from_slice(&stdout).context("failed to parse jgenesis oracle manifest")
}

/// Run the jgenesis oracle with full options including memory watches and WRAM dumps.
pub fn run_jgenesis_capture_dynamic_with_options(
    env: &OracleEnvironment,
    rom_path: &Path,
    scenario_file: &Path,
    out_dir: &Path,
    options: &OracleRunOptions,
) -> Result<OracleManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let mut cmd = Command::new(&env.jgenesis_helper_bin);
    cmd.arg("--rom")
        .arg(rom_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--scenario-file")
        .arg(scenario_file);
    if let Some(skip) = options.skip_frames_before {
        cmd.arg("--skip-frames-before").arg(skip.to_string());
    }
    if let Some(ref watches) = options.memory_watches {
        cmd.arg("--memory-watches").arg(watches);
    }
    if let Some(ref frames) = options.wram_dump_frames {
        cmd.arg("--wram-dump-frames").arg(frames);
    }
    if let Some(ref frames) = options.vram_dump_frames {
        cmd.arg("--vram-dump-frames").arg(frames);
    }
    if let Some(ref frames) = options.cram_dump_frames {
        cmd.arg("--cram-dump-frames").arg(frames);
    }
    if let Some(ref frames) = options.vsram_dump_frames {
        cmd.arg("--vsram-dump-frames").arg(frames);
    }
    if let Some(ref samples) = options.vdp_pixel_samples {
        cmd.arg("--vdp-pixel-samples").arg(samples);
    }
    if options.capture_cpu_state {
        cmd.arg("--capture-cpu-state");
    }
    if options.capture_settle_trace {
        cmd.arg("--capture-settle-trace");
    }
    if options.capture_rng_stream {
        cmd.arg("--capture-rng-stream");
    }
    let stdout = capture_stdout(&mut cmd, None)?;
    serde_json::from_slice(&stdout).context("failed to parse jgenesis oracle manifest")
}

fn write_helper_project(helper_dir: &Path, repo_root: &Path, jgenesis_dir: &Path) -> Result<()> {
    let src_dir = helper_dir.join("src");
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("failed to create {}", src_dir.display()))?;

    let cargo_toml = format!(
        "[package]\nname = \"wildside-jgenesis-oracle\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\nanyhow = \"1\"\nbincode = \"2\"\ncrc32fast = \"1.4\"\ngenesis-config = {{ path = \"{}\" }}\ngenesis-core = {{ path = \"{}\" }}\nhound = \"3.5.1\"\nimage = {{ version = \"0.25.8\", default-features = false, features = [\"png\"] }}\njgenesis-common = {{ path = \"{}\" }}\nwildside-core = {{ path = \"{}\" }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\nsha1 = \"0.10.6\"\n",
        jgenesis_dir.join("config/genesis-config").display(),
        jgenesis_dir.join("backend/genesis-core").display(),
        jgenesis_dir.join("common/jgenesis-common").display(),
        repo_root.join("crates/wildside-core").display(),
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

/// Patch jgenesis to expose M68K CPU registers through the debug API.
/// Idempotent: skips if the patch marker is already present.
fn patch_jgenesis_cpu_registers(jgenesis_dir: &Path) -> Result<()> {
    let debug_rs = jgenesis_dir.join("backend/genesis-core/src/api/debug.rs");
    let contents = fs::read_to_string(&debug_rs)
        .with_context(|| format!("failed to read {}", debug_rs.display()))?;

    // Idempotency: skip if already patched
    if contents.contains("m68k_pc") {
        return Ok(());
    }

    let patch = r"
    /// M68K program counter.
    #[must_use]
    pub fn m68k_pc(&self) -> u32 {
        self.0.m68k.pc()
    }

    /// M68K data registers D0–D7.
    #[must_use]
    pub fn m68k_data_registers(&self) -> [u32; 8] {
        self.0.m68k.data_registers()
    }

    /// M68K address registers A0–A6.
    #[must_use]
    pub fn m68k_address_registers(&self) -> [u32; 7] {
        self.0.m68k.address_registers()
    }

    /// M68K status register (condition codes, interrupt mask, supervisor flag).
    #[must_use]
    pub fn m68k_status_register(&self) -> u16 {
        self.0.m68k.status_register()
    }
";

    // Insert before the final closing brace of impl<'emu> GenesisDebugView
    let patched = if let Some(pos) = contents.rfind('}') {
        format!("{}{}{}", &contents[..pos], patch, &contents[pos..])
    } else {
        return Err(anyhow!("could not find closing brace in debug.rs"));
    };

    fs::write(&debug_rs, patched)
        .with_context(|| format!("failed to write patched {}", debug_rs.display()))
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
    let status = match Command::new("pkg-config")
        .arg("--exists")
        .args(packages)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => status,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error).context("failed to execute pkg-config"),
    };
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

#[cfg(test)]
mod tests {
    use super::{
        HELPER_TEMPLATE, JgenesisBackend, OracleBackend, OracleEnvironment, OracleManifest,
        jgenesis_helper_cache_version, quote_argument, render_command, write_if_changed,
    };
    use crate::capture::sha1_hex;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn jgenesis_backend_reports_stable_name_and_helper_cache_version() {
        let backend = JgenesisBackend {
            env: OracleEnvironment {
                jgenesis_helper_bin: PathBuf::from("helper-bin"),
                jgenesis_dir: PathBuf::from("jgenesis"),
                blastem_dir: PathBuf::from("blastem"),
                blastem_binary: None,
            },
        };
        let cache_version = jgenesis_helper_cache_version();

        assert_eq!(backend.name(), "jgenesis");
        assert_eq!(backend.cache_version(), cache_version);
        assert_eq!(cache_version, sha1_hex(HELPER_TEMPLATE.as_bytes()));
        assert_eq!(cache_version.len(), 40);
        assert!(
            cache_version
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        );
    }

    #[test]
    fn helper_excludes_prize_metadata_from_direct_enemy_capture_loc_104ae() {
        assert!(
            HELPER_TEMPLATE.contains("const FIRST_NON_ENEMY_OBJECT_META: u8 = 0x64;")
                && HELPER_TEMPLATE
                    .contains("object_type != 0 && object_type < FIRST_NON_ENEMY_OBJECT_META"),
            "ASM constants.asm starts prize metadata at objid_prizediamond=$64, and PrizeHelmetCyclone_Init writes objid_cyclone=$70 before loc_104AE; neither may enter the enemy roster",
        );
    }

    #[test]
    fn helper_excludes_kid_gfx_from_direct_enemy_capture_loc_1cf2() {
        assert!(
            HELPER_TEMPLATE.contains("const ADDR_GFX_OBJECT_KID_ADDR: usize = 0xFF_F85E;")
                && HELPER_TEMPLATE.contains("let kid_gfx_object = read_wram_u32(")
                && HELPER_TEMPLATE.contains("if base == kid_gfx_object"),
            "ASM GfxObjects_CollisionKid loc_1CF2 copies an enemy collision source into Kid's GfxObject, which must not make Kid appear in the enemy roster",
        );
    }

    #[test]
    fn write_if_changed_writes_missing_or_changed_files_only() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("generated.txt");

        write_if_changed(&path, b"first").expect("initial write");
        assert_eq!(fs::read(&path).expect("initial contents"), b"first");

        write_if_changed(&path, b"second").expect("changed write");
        assert_eq!(fs::read(&path).expect("changed contents"), b"second");

        write_if_changed(&path, b"second").expect("same contents");
        assert_eq!(fs::read(&path).expect("unchanged contents"), b"second");
    }

    #[test]
    fn render_command_quotes_only_arguments_that_need_shell_escaping() {
        let mut command = Command::new("git");
        command
            .arg("checkout")
            .arg("--detach")
            .arg("abc-123_./:=+")
            .arg("branch with space");

        assert_eq!(quote_argument(OsStr::new("abc-123_./:=+")), "abc-123_./:=+");
        assert_eq!(
            quote_argument(OsStr::new("branch with space")),
            "\"branch with space\""
        );
        assert_eq!(
            render_command(&command, Path::new("/repo root")),
            "(cd /repo root && git checkout --detach abc-123_./:=+ \"branch with space\")"
        );
    }

    #[test]
    fn helper_settles_level_zero_recording_oracles() {
        assert!(
            HELPER_TEMPLATE.contains("if target_level == 0")
                && HELPER_TEMPLATE.contains("settle_title_card_target_level")
                && HELPER_TEMPLATE.contains("ASM loc_1AE2E")
                && HELPER_TEMPLATE.contains("read_game_mode(emulator) != 0x0008"),
            "settled Level 00 recording oracles must dismiss the title card before scenario frame 0"
        );
    }

    #[test]
    fn helper_waits_for_live_object_tables_before_settled_recording_frames() {
        assert!(
            HELPER_TEMPLATE.contains("fn target_level_is_live")
                && HELPER_TEMPLATE.contains("read_kid_x_pos(emulator) != 0")
                && HELPER_TEMPLATE.contains("read_number_objects(emulator) != 0"),
            "settled recording oracle bootstrap must not start scenario frame 0 on an empty Game_Mode=$000C shell"
        );
    }

    #[test]
    fn helper_forces_target_level_for_direct_probe_warps() {
        assert!(
            HELPER_TEMPLATE.contains("let force_target_level_for_probe =")
                && HELPER_TEMPLATE.contains("resolved.start_position.is_some()")
                && HELPER_TEMPLATE.contains("|| resolved.camera_position.is_some()")
                && HELPER_TEMPLATE.contains("resolved.playtest_form.is_some()")
                && HELPER_TEMPLATE.contains("force_target_level_for_probe,")
                && HELPER_TEMPLATE.contains(
                    "} else if settle_oracle_warp || force_target_level_for_probe {"
                )
                && HELPER_TEMPLATE.contains("ASM loc_74B0")
                && HELPER_TEMPLATE.contains("drain_direct_probe_title_start_latch")
                && HELPER_TEMPLATE.contains("seed_input_latches")
                && HELPER_TEMPLATE.contains("ASM ReadJoypad loc_C4E")
                && HELPER_TEMPLATE.contains("if settle_oracle_warp || force_target_level_for_probe {\n            clear_input_latches(emulator);\n        }"),
            "direct probe scenarios patch live WRAM after warp_to_level, so the oracle helper must target the same level as native before frame 0"
        );
    }

    #[test]
    fn helper_samples_kid_velocity_from_live_kid_gfx_slot() {
        assert!(
            HELPER_TEMPLATE.contains("const KID_X_VEL_ADDR: usize = 0xFF_2506;")
                && HELPER_TEMPLATE.contains("const KID_Y_VEL_ADDR: usize = 0xFF_250A;")
                && HELPER_TEMPLATE.contains("Some(read_wram_i32(&work_ram, KID_X_VEL_ADDR))")
                && HELPER_TEMPLATE.contains("Some(read_wram_i32(&work_ram, KID_Y_VEL_ADDR))"),
            "oracle Kid velocity must come from $FFFF24E0 + x_vel/y_vel, not the stale first gfx slot"
        );
    }

    #[test]
    fn helper_exposes_platform_collision_scratch_loc_257e() {
        assert!(
            HELPER_TEMPLATE
                .contains("const PLATFORM_COLLISION_SCRATCH_FAA2_ADDR: usize = 0xFF_FAA2;")
                && HELPER_TEMPLATE.contains(
                    "const PLATFORM_COLLISION_JUMP_PLATFORM_REF_FA96_ADDR: usize = 0xFF_FA96;"
                )
                && HELPER_TEMPLATE.contains("fn platform_collision_scratch_debug_loc_257e")
                && HELPER_TEMPLATE.contains("ASM Platforms_CheckCollision loc_257E")
                && HELPER_TEMPLATE.contains("\"platform_collision_scratch_loc_257e\""),
            "oracle gameplay trace must expose the same Platforms_CheckCollision scratch cells as native"
        );
    }

    #[test]
    fn helper_exposes_oracle_platform_slot_order_loc_257e() {
        assert!(
            HELPER_TEMPLATE.contains("const ADDR_FIRST_PLATFORM_SLOT_ADDR: usize = 0xFF_FA8E;")
                && HELPER_TEMPLATE.contains("const NUMBER_PLATFORMS_ADDR: usize = 0xFF_FA92;")
                && HELPER_TEMPLATE.contains("const PLATFORM_RAM_ADDR: usize = 0xFFFF_EDBA;")
                && HELPER_TEMPLATE.contains("fn oracle_platform_slots_debug_loc_257e")
                && HELPER_TEMPLATE.contains("Addr_FirstPlatformSlot")
                && HELPER_TEMPLATE.contains("\"oracle_platform_slots_loc_257e\""),
            "oracle gameplay trace must expose the ROM platform linked-list order for native vector-order audits"
        );
    }

    #[test]
    fn oracle_manifest_deserialises_hidden_settle_trace_rows() {
        let manifest: OracleManifest = serde_json::from_str(
            r#"{
                "sample_rate": 44100,
                "audio_path": "audio.wav",
                "frames": [],
                "states": [],
                "settle_trace": [
                    {
                        "phase": "target_live",
                        "step": 34,
                        "game_mode": 12,
                        "current_level_id": 1,
                        "rng_offset": 65491,
                        "rng_ram": [1, 2, 3],
                        "paused": false,
                        "camera_x_pos": 0,
                        "camera_y_pos": 1835008,
                        "kid_x_pos": 2097152,
                        "kid_y_pos": 12517376,
                        "kid_x_vel": 0,
                        "kid_y_vel": 0,
                        "number_objects": 4,
                        "background_scroll_loc_3050a": "slots=08:col=0031,vdp=0062,cur=0072,prev=0071",
                        "background_row_offsets_loc_30744": "cam_chunk=0011;width=0046;rows=0F:03CA,10:0410,11:0456",
                        "background_source_window_loc_30744": "B18@8C9E[28..38]=00,01,02",
                        "background_source_absolute_loc_30744": "8C9E[28..38]=00,01,02",
                        "background_source_rows_loc_12848": "8C9E[00..45]=00,01,02",
                        "background_plane_b_rows_loc_307d8": "B18=0780",
                        "background_plane_b_spans_loc_307d8": "B18=05..2F:2780"
                    }
                ]
            }"#,
        )
        .expect("manifest should accept diagnostic hidden settle rows");

        let row = &manifest.settle_trace[0];
        assert_eq!(row.phase, "target_live");
        assert_eq!(row.step, 34);
        assert_eq!(row.game_mode, 0x000C);
        assert_eq!(row.current_level_id, 0x0001);
        assert_eq!(row.rng_offset, 0xFFD3);
        assert_eq!(row.rng_ram, [1, 2, 3]);
        assert_eq!(row.camera_y_pos, 0x001C_0000);
        assert_eq!(row.kid_x_pos, 0x0020_0000);
        assert_eq!(row.kid_y_pos, 0x00BF_0000);
        assert_eq!(row.number_objects, 4);
        assert_eq!(
            row.background_scroll_loc_3050a,
            "slots=08:col=0031,vdp=0062,cur=0072,prev=0071"
        );
        assert_eq!(
            row.background_row_offsets_loc_30744,
            "cam_chunk=0011;width=0046;rows=0F:03CA,10:0410,11:0456"
        );
        assert_eq!(
            row.background_source_window_loc_30744,
            "B18@8C9E[28..38]=00,01,02"
        );
        assert_eq!(
            row.background_source_absolute_loc_30744,
            "8C9E[28..38]=00,01,02"
        );
        assert_eq!(
            row.background_source_rows_loc_12848,
            "8C9E[00..45]=00,01,02"
        );
        assert_eq!(row.background_plane_b_rows_loc_307d8, "B18=0780");
        assert_eq!(row.background_plane_b_spans_loc_307d8, "B18=05..2F:2780");
    }

    #[test]
    fn oracle_manifest_deserialises_typed_per_frame_rng_stream_loc_1186() {
        let ring = vec![0x5Au8; wildside_core::RomRandomRing::LENGTH];
        let manifest: OracleManifest = serde_json::from_value(serde_json::json!({
            "sample_rate": 44100,
            "audio_path": "audio.wav",
            "frames": [],
            "states": [],
            "frame_rng_states": [null, {"offset": 0xFFEDu16, "ring": ring}],
        }))
        .expect("oracle boundary should parse complete post-Init_RNG frame state");

        assert_eq!(manifest.frame_rng_states.len(), 2);
        let state = manifest.frame_rng_states[1]
            .as_ref()
            .expect("second frame is initialised");
        assert_eq!(state.offset(), 0xFFED);
        assert_eq!(state.ring().as_bytes(), &[0x5A; 55]);
    }

    #[test]
    fn helper_seeds_playtest_form_collision_half_width() {
        assert!(
            HELPER_TEMPLATE.contains(
                "const PLAYER_COLLISION_HALF_WIDTH_ADDR: usize = 0xFF_FA78;"
            ) && HELPER_TEMPLATE.contains("fn collision_half_width_for_helmet_id")
                && HELPER_TEMPLATE.contains("ASM unk_B408/sub_B41C")
                && HELPER_TEMPLATE.contains("write_emulator_wram_u16(\n        emulator,\n        PLAYER_COLLISION_HALF_WIDTH_ADDR,\n        collision_half_width,\n    );"),
            "oracle playtest_form must seed ASM sub_B41C's $FFFFFA78 half-width alongside Current_Helmet"
        );
    }

    #[test]
    fn helper_seeds_probe_lives_count_for_pause_menu_routes() {
        assert!(
            HELPER_TEMPLATE.contains("const NUMBER_LIVES_ADDR: usize = 0xFF_FC3E;")
                && HELPER_TEMPLATE.contains("const PLAYER_1_LIVES_ADDR: usize = 0xFF_FC54;")
                && HELPER_TEMPLATE.contains("fn apply_probe_lives_count")
                && HELPER_TEMPLATE.contains("Game_Paused_ChkStart")
                && HELPER_TEMPLATE
                    .contains("write_emulator_wram_u16(emulator, NUMBER_LIVES_ADDR, count);")
                && HELPER_TEMPLATE
                    .contains("write_emulator_wram_u16(emulator, PLAYER_1_LIVES_ADDR, count);"),
            "oracle last-life pause probes must seed ASM Number_Lives and the player slot save copy"
        );
    }
}
