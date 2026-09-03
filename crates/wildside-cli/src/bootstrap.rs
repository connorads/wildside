use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use std::fmt::Write as _;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use wildside_data::{
    AssetManifest, ensure_generated_bundle_contract, export_split_bundle, generated_bundle_dir,
    load_manifest, manifest_path as generated_manifest_path,
};
use wildside_rom::{VerifiedRom, verify_rom};

use crate::scenarios::GameplayScenario;

pub(crate) const ASL_REPO_URL: &str = "https://github.com/Macroassembler-AS/asl-releases.git";
pub(crate) const UPSTREAM_URL: &str = "https://github.com/sonicretro/kid-chameleon-disasm";
pub(crate) const UPSTREAM_COMMIT: &str = "a134567";
pub(crate) const M68K_REFERENCE_URL: &str =
    "https://github.com/prb28/m68k-instructions-documentation";
pub(crate) const M68K_REFERENCE_COMMIT: &str = "89744462bde926313aa273dde2109b680ed11c8a";
pub(crate) const GEMS_REFERENCE_URL: &str = "https://github.com/realmonster/GEMS";
pub(crate) const GEMS_REFERENCE_COMMIT: &str = "a09a734532231fb8a1170e55f018d8511329562e";
pub(crate) const GEMSPLAY_REFERENCE_URL: &str = "https://github.com/ValleyBell/GEMSPlay";
pub(crate) const GEMSPLAY_REFERENCE_COMMIT: &str = "8b2e490942e9e8c0cc8d6f9132f15c5b8507a5db";
pub(crate) const HARTE_VECTORS_URL: &str = "https://github.com/SingleStepTests/680x0";
pub(crate) const HARTE_VECTORS_COMMIT: &str = "e0d5ece9670205cc84a0101081837deb446f86a3";
pub(crate) const MAME_VECTORS_URL: &str = "https://github.com/SingleStepTests/m68000";
pub(crate) const MAME_VECTORS_COMMIT: &str = "64b253116a3de04aaac4346c43680960dc9b67e5";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ReferenceName {
    M68k,
    Gems,
    Gemsplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReferenceRepo {
    pub(crate) name: &'static str,
    pub(crate) dir_name: &'static str,
    pub(crate) url: &'static str,
    pub(crate) commit: &'static str,
    pub(crate) purpose: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrappedReference {
    pub(crate) name: &'static str,
    pub(crate) path: PathBuf,
    pub(crate) commit: &'static str,
    pub(crate) purpose: &'static str,
}

#[derive(Debug)]
pub(crate) struct NativeAslTools {
    pub(crate) root: PathBuf,
    pub(crate) asl: PathBuf,
    pub(crate) p2bin: PathBuf,
}

#[derive(Debug)]
pub(crate) struct PrepareLock {
    path: PathBuf,
}

impl Drop for PrepareLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) fn bootstrap_upstream(dir: &Path) -> Result<()> {
    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("upstream directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git").arg("clone").arg(UPSTREAM_URL).arg(dir),
            None,
        )?;
    }

    run_command(Command::new("git").arg("fetch").arg("--all"), Some(dir))?;
    run_command(
        Command::new("git")
            .arg("checkout")
            .arg("--detach")
            .arg(UPSTREAM_COMMIT),
        Some(dir),
    )?;
    Ok(())
}

pub(crate) fn bootstrap_references(
    reference_root: &Path,
    only: Option<ReferenceName>,
) -> Result<Vec<BootstrappedReference>> {
    fs::create_dir_all(reference_root)
        .with_context(|| format!("failed to create {}", reference_root.display()))?;

    let mut references = Vec::new();
    for reference in selected_reference_repos(only) {
        let path = reference_root.join(reference.dir_name);
        bootstrap_pinned_repo(&path, reference.url, reference.commit)?;
        references.push(BootstrappedReference {
            name: reference.name,
            path,
            commit: reference.commit,
            purpose: reference.purpose,
        });
    }
    Ok(references)
}

pub(crate) fn selected_reference_repos(only: Option<ReferenceName>) -> &'static [ReferenceRepo] {
    const M68K: ReferenceRepo = ReferenceRepo {
        name: "m68k",
        dir_name: "m68k-instructions-documentation",
        url: M68K_REFERENCE_URL,
        commit: M68K_REFERENCE_COMMIT,
        purpose: "Primary 68k instruction and directive reference",
    };
    const GEMS: ReferenceRepo = ReferenceRepo {
        name: "gems",
        dir_name: "gems",
        url: GEMS_REFERENCE_URL,
        commit: GEMS_REFERENCE_COMMIT,
        purpose: "Primary GEMS tooling and driver research reference",
    };
    const GEMSPLAY: ReferenceRepo = ReferenceRepo {
        name: "gemsplay",
        dir_name: "gemsplay",
        url: GEMSPLAY_REFERENCE_URL,
        commit: GEMSPLAY_REFERENCE_COMMIT,
        purpose: "Practical GEMS playback and inspection reference",
    };
    const ALL: &[ReferenceRepo] = &[M68K, GEMS, GEMSPLAY];
    const ONLY_M68K: &[ReferenceRepo] = &[M68K];
    const ONLY_GEMS: &[ReferenceRepo] = &[GEMS];
    const ONLY_GEMSPLAY: &[ReferenceRepo] = &[GEMSPLAY];

    match only {
        None => ALL,
        Some(ReferenceName::M68k) => ONLY_M68K,
        Some(ReferenceName::Gems) => ONLY_GEMS,
        Some(ReferenceName::Gemsplay) => ONLY_GEMSPLAY,
    }
}

pub(crate) fn bootstrap_pinned_repo(dir: &Path, url: &str, commit: &str) -> Result<()> {
    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("reference directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(Command::new("git").arg("clone").arg(url).arg(dir), None)?;
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

/// The two SingleStepTests 68000 vector repositories (#447): per-opcode
/// single-instruction vectors giving full state before and after, plus bus
/// transactions, used to validate the pinned `r68k-emu` core.
pub(crate) fn cpu_vector_repos() -> &'static [ReferenceRepo] {
    const HARTE: ReferenceRepo = ReferenceRepo {
        name: "singlesteptests-680x0",
        dir_name: "singlesteptests-680x0",
        url: HARTE_VECTORS_URL,
        commit: HARTE_VECTORS_COMMIT,
        purpose: "Tom Harte lineage 68000 per-opcode state + bus-cycle vectors",
    };
    const MAME: ReferenceRepo = ReferenceRepo {
        name: "singlesteptests-m68000",
        dir_name: "singlesteptests-m68000",
        url: MAME_VECTORS_URL,
        commit: MAME_VECTORS_COMMIT,
        purpose: "MAME-microcode-derived 68000 vectors; adds STOP, LINE A/F, re/we cycles",
    };
    const ALL: &[ReferenceRepo] = &[HARTE, MAME];
    ALL
}

/// Fetch both vector repositories into `reference_root`.
///
/// Kept out of `bootstrap-references` deliberately: these are ~600 MB of
/// generated test data used by one verification task, not reading references
/// every session needs. Shallow (`--depth 1`) for the same reason — the pinned
/// SHA is fetched directly, so the pin survives without the history.
pub(crate) fn bootstrap_cpu_vector_repos(
    reference_root: &Path,
) -> Result<Vec<BootstrappedReference>> {
    fs::create_dir_all(reference_root)
        .with_context(|| format!("failed to create {}", reference_root.display()))?;

    let mut references = Vec::new();
    for reference in cpu_vector_repos() {
        let path = reference_root.join(reference.dir_name);
        bootstrap_shallow_pinned_repo(&path, reference.url, reference.commit)?;
        references.push(BootstrappedReference {
            name: reference.name,
            path,
            commit: reference.commit,
            purpose: reference.purpose,
        });
    }
    Ok(references)
}

/// Shallow clone `url` into `dir` and detach at `commit`.
///
/// Returns early when `dir` is already detached at `commit`, so a re-run costs
/// no network. Unlike [`bootstrap_pinned_repo`] this never runs `git fetch
/// --all`: that would deepen the clone back to full history.
pub(crate) fn bootstrap_shallow_pinned_repo(dir: &Path, url: &str, commit: &str) -> Result<()> {
    if head_commit(dir).as_deref() == Some(commit) {
        return Ok(());
    }

    if !dir.exists() {
        let parent = dir
            .parent()
            .ok_or_else(|| anyhow!("vector directory has no parent: {}", dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg("--single-branch")
                .arg(url)
                .arg(dir),
            None,
        )?;
    }

    // GitHub serves reachable SHAs directly, so a pinned checkout stays shallow.
    run_command(
        Command::new("git")
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin")
            .arg(commit),
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

/// `dir`'s own checked-out commit, or `None` when `dir` is not a git checkout.
///
/// The git directory is named explicitly rather than inherited from the working
/// directory. These checkouts live under `.cache/`, which is inside *this*
/// repository's work tree, so a plain `rev-parse` in a not-yet-cloned directory
/// would confidently report wildside's own HEAD as the vector-set commit.
pub(crate) fn head_commit(dir: &Path) -> Option<String> {
    let mut command = Command::new("git");
    scrub_inherited_git_env(&mut command);
    let output = command
        .arg("--git-dir")
        .arg(dir.join(".git"))
        .arg("rev-parse")
        .arg("HEAD")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    if commit.is_empty() {
        None
    } else {
        Some(commit)
    }
}

pub(crate) fn prepare_generated_assets(
    repo_root: &Path,
    rom_path: &Path,
    upstream_dir: &Path,
) -> Result<AssetManifest> {
    prepare_generated_assets_with(
        repo_root,
        rom_path,
        upstream_dir,
        |path| Ok(verify_rom(path)?),
        bootstrap_upstream,
        sync_rom_into_upstream,
        |dir| run_command(Command::new("python3").arg("split.py"), Some(dir)),
        export_split_bundle,
    )
}

pub(crate) fn ensure_gameplay_level_layouts(
    rom_path: &Path,
    manifest: &AssetManifest,
    scenario: GameplayScenario,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    ensure_gameplay_level_layouts_with(&rom_bytes, manifest, scenario, super::dump_level_layout)
}

pub(crate) fn ensure_level_layouts(
    rom_path: &Path,
    manifest: &AssetManifest,
    level_ids: &[u8],
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    ensure_level_layouts_with(&rom_bytes, manifest, level_ids, super::dump_level_layout)
}

pub(crate) fn ensure_play_native_level_layouts(
    rom_path: &Path,
    manifest: &AssetManifest,
    warp_level: Option<u8>,
) -> Result<()> {
    let rom_bytes = fs::read(rom_path)
        .with_context(|| format!("failed to read ROM from {}", rom_path.display()))?;
    ensure_play_native_level_layouts_with(
        &rom_bytes,
        manifest,
        warp_level,
        super::dump_level_layout,
    )
}

pub(crate) fn ensure_play_native_level_layouts_with<FDump>(
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    warp_level: Option<u8>,
    dump_level_layout_fn: FDump,
) -> Result<()>
where
    FDump: Fn(&[u8], u8, &Path) -> Result<()>,
{
    ensure_gameplay_level_layouts_with(
        rom_bytes,
        manifest,
        GameplayScenario::SoakLevel10m,
        &dump_level_layout_fn,
    )?;
    if let Some(level_id) = warp_level {
        ensure_level_layouts_with(rom_bytes, manifest, &[level_id], &dump_level_layout_fn)?;
    }
    Ok(())
}

pub(crate) fn ensure_gameplay_level_layouts_with<FDump>(
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    scenario: GameplayScenario,
    dump_level_layout_fn: FDump,
) -> Result<()>
where
    FDump: Fn(&[u8], u8, &Path) -> Result<()>,
{
    let (start_level, end_level) = scenario.required_level_layout_range();
    let layout_dir = manifest.output_root.join("level-layout");
    let missing_levels =
        missing_oracle_dumpable_level_layouts(&layout_dir, start_level..=end_level);
    if missing_levels.is_empty() {
        return Ok(());
    }

    for level_id in missing_levels {
        dump_level_layout_fn(rom_bytes, level_id, &layout_dir)?;
    }
    Ok(())
}

pub(crate) fn ensure_level_layouts_with<FDump>(
    rom_bytes: &[u8],
    manifest: &AssetManifest,
    level_ids: &[u8],
    dump_level_layout_fn: FDump,
) -> Result<()>
where
    FDump: Fn(&[u8], u8, &Path) -> Result<()>,
{
    let layout_dir = manifest.output_root.join("level-layout");
    let missing_levels =
        missing_oracle_dumpable_level_layouts(&layout_dir, level_ids.iter().copied());
    if missing_levels.is_empty() {
        return Ok(());
    }

    for level_id in missing_levels {
        dump_level_layout_fn(rom_bytes, level_id, &layout_dir)?;
    }
    Ok(())
}

fn missing_oracle_dumpable_level_layouts(
    layout_dir: &Path,
    level_ids: impl IntoIterator<Item = u8>,
) -> Vec<u8> {
    level_ids
        .into_iter()
        .filter(|level_id| {
            !level_layout_dump_complete(layout_dir, *level_id)
                && !oracle_warp_broken_level(*level_id)
        })
        .collect()
}

fn level_layout_dump_complete(layout_dir: &Path, level_id: u8) -> bool {
    let data_path = layout_dir.join(format!("{level_id:02X}.bin"));
    let metadata_path = layout_dir.join(format!("{level_id:02X}.json"));
    let block_list_path = layout_dir.join(format!("{level_id:02X}.blocklist.bin"));
    data_path.is_file() && metadata_path.is_file() && block_list_path.is_file()
}

fn oracle_warp_broken_level(level_id: u8) -> bool {
    // The jgenesis level-skip path reaches invalid game mode `$4E73` for
    // these logical IDs. Native consumes the disassembly-decoded block layout
    // instead, so no CLI entry point should retry the impossible RAM dump.
    matches!(
        level_id,
        0x09 | 0x32 | 0x33 | 0x50 | 0x57 | 0x5A | 0x5D | 0x5E | 0x63
    )
}

pub(crate) fn prepare_generated_assets_with<FVerify, FBootstrap, FSync, FSplit, FExport>(
    repo_root: &Path,
    rom_path: &Path,
    upstream_dir: &Path,
    verify_rom_fn: FVerify,
    bootstrap_upstream_fn: FBootstrap,
    sync_rom_fn: FSync,
    split_fn: FSplit,
    export_bundle_fn: FExport,
) -> Result<AssetManifest>
where
    FVerify: FnOnce(&Path) -> Result<VerifiedRom>,
    FBootstrap: FnOnce(&Path) -> Result<()>,
    FSync: FnOnce(&Path, &Path) -> Result<()>,
    FSplit: FnOnce(&Path) -> Result<()>,
    FExport: FnOnce(&Path, &Path, &str, &str) -> Result<AssetManifest>,
{
    let _lock = acquire_prepare_lock(repo_root)?;
    let verified = verify_rom_fn(rom_path)?;
    let bundle_dir = generated_bundle_dir(repo_root, &verified.sha1);
    let bundle_manifest_path = generated_manifest_path(&bundle_dir);
    if bundle_manifest_path.is_file() {
        let manifest = load_manifest(&bundle_manifest_path)?;
        if ensure_generated_bundle_contract(&manifest).is_ok() {
            return Ok(manifest);
        }
    }

    bootstrap_upstream_fn(upstream_dir)?;
    sync_rom_fn(rom_path, upstream_dir)?;
    split_fn(upstream_dir)?;
    export_bundle_fn(upstream_dir, &bundle_dir, &verified.sha1, UPSTREAM_COMMIT)
}

pub(crate) fn acquire_prepare_lock(repo_root: &Path) -> Result<PrepareLock> {
    acquire_prepare_lock_with_clock(
        repo_root,
        Duration::from_secs(30),
        Duration::from_millis(200),
        Instant::now,
        thread::sleep,
    )
}

fn acquire_prepare_lock_with_clock<FNow, FSleep>(
    repo_root: &Path,
    timeout: Duration,
    retry_delay: Duration,
    mut now: FNow,
    mut sleep: FSleep,
) -> Result<PrepareLock>
where
    FNow: FnMut() -> Instant,
    FSleep: FnMut(Duration),
{
    let lock_dir = repo_root.join(".cache/generated");
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("failed to create {}", lock_dir.display()))?;
    let lock_path = lock_dir.join(".prepare.lock");
    let deadline = now() + timeout;
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => return Ok(PrepareLock { path: lock_path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if now() >= deadline {
                    bail!(
                        "timed out waiting for asset prepare lock at {}",
                        lock_path.display()
                    );
                }
                sleep(retry_delay);
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to open {}", lock_path.display()));
            }
        }
    }
}

pub(crate) fn sync_rom_into_upstream(rom_path: &Path, upstream_dir: &Path) -> Result<()> {
    let upstream_rom = upstream_dir.join("kid.bin");
    fs::copy(rom_path, &upstream_rom).with_context(|| {
        format!(
            "failed to copy ROM from {} to {}",
            rom_path.display(),
            upstream_rom.display()
        )
    })?;
    Ok(())
}

pub(crate) fn run_reference_build(
    repo_root: &Path,
    upstream_dir: &Path,
    verified: &VerifiedRom,
) -> Result<()> {
    sync_rom_into_upstream(&verified.path, upstream_dir)?;
    run_command(Command::new("python3").arg("split.py"), Some(upstream_dir))?;
    build_reference_rom(repo_root, upstream_dir)?;

    let rebuilt = verify_rom(&upstream_dir.join("kid_built.bin"))
        .context("rebuilt ROM did not pass Kid Chameleon header verification")?;

    println!("source sha1:  {}", verified.sha1);
    println!("rebuilt sha1: {}", rebuilt.sha1);
    if verified.sha1 != rebuilt.sha1 {
        bail!("reference build completed but ROM is not bit-perfect");
    }

    println!("reference build is bit-perfect");
    Ok(())
}

pub(crate) fn build_reference_rom(repo_root: &Path, upstream_dir: &Path) -> Result<()> {
    let bundled = Command::new("sh")
        .arg("build.sh")
        .current_dir(upstream_dir)
        .status();
    if let Ok(status) = bundled
        && status.success()
        && upstream_dir.join("kid_built.bin").is_file()
    {
        return Ok(());
    }

    let native_tools = ensure_native_asl_tools(repo_root)?;
    run_command(
        Command::new(&native_tools.asl)
            .env("AS_MSGPATH", &native_tools.root)
            .arg("-i")
            .arg(".")
            .arg("-xx")
            .arg("-c")
            .arg("-A")
            .arg("-U")
            .arg("kid.asm"),
        Some(upstream_dir),
    )?;
    run_command(
        Command::new(&native_tools.p2bin)
            .arg("-l")
            .arg("0")
            .arg("-r")
            .arg("0x-0x")
            .arg("kid.p")
            .arg("kid_built.bin"),
        Some(upstream_dir),
    )?;
    Ok(())
}

pub(crate) fn ensure_native_asl_tools(repo_root: &Path) -> Result<NativeAslTools> {
    let source_dir = repo_root.join(".cache/tooling/asl-releases");
    let built_asl = source_dir.join("asl");
    let built_p2bin = source_dir.join("p2bin");
    if built_asl.is_file() && built_p2bin.is_file() {
        // Warm cache: both binaries are present, so skip the network entirely.
        // Without this a repeat run pays a fetch it does not need, and the
        // toolchain is pinned by the checkout below rather than by freshness.
        return Ok(NativeAslTools {
            root: source_dir,
            asl: built_asl,
            p2bin: built_p2bin,
        });
    }

    if !source_dir.exists() {
        let parent = source_dir
            .parent()
            .ok_or_else(|| anyhow!("tooling directory has no parent: {}", source_dir.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        run_command(
            Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(ASL_REPO_URL)
                .arg(&source_dir),
            None,
        )?;
    }

    // Fetch into FETCH_HEAD and check out detached: `upstream:upstream` refuses
    // once the local `upstream` branch is checked out ("refusing to fetch into
    // branch"), which made a resumed bootstrap fail forever. `--force` is safe
    // and wanted here: the only tracked file this ever edits is Makefile.def,
    // rewritten immediately below, and .cache is regenerable state.
    run_command(
        Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg("upstream"),
        Some(&source_dir),
    )?;
    run_command(
        Command::new("git")
            .arg("checkout")
            .arg("--detach")
            .arg("--force")
            .arg("FETCH_HEAD"),
        Some(&source_dir),
    )?;

    let makefile = capture_stdout(
        Command::new("git")
            .arg("show")
            .arg("origin/main:Makefile.def")
            .current_dir(&source_dir),
    )
    .context("failed to load ASL Makefile.def from origin/main")?;
    fs::write(source_dir.join("Makefile.def"), makefile)
        .with_context(|| format!("failed to write {}/Makefile.def", source_dir.display()))?;

    run_command(Command::new("make").arg("-j4"), Some(&source_dir))?;

    Ok(NativeAslTools {
        root: source_dir,
        asl: built_asl,
        p2bin: built_p2bin,
    })
}

/// Variables git exports to a hook, which must not reach a child `git`.
///
/// The pre-commit gate runs bootstrap and oracle work, and both drive `git`
/// inside `.cache` checkouts. Inherited, they point that `git` at the outer
/// repository's index and object store, so a checkout of a cache repo's commit
/// fails ("unable to read tree", or promisor fetches of unrelated objects).
/// Every command spawned here is cache-local, so scrubbing is unconditional.
const INHERITED_GIT_ENV: [&str; 6] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// Drop any git-hook environment the parent process was handed.
pub(crate) fn scrub_inherited_git_env(command: &mut Command) {
    for name in INHERITED_GIT_ENV {
        command.env_remove(name);
    }
}

pub(crate) fn capture_stdout(command: &mut Command) -> Result<Vec<u8>> {
    scrub_inherited_git_env(command);
    let output = command.stderr(Stdio::inherit()).output().with_context(|| {
        format!(
            "failed to execute {}",
            render_command(command, Path::new("."))
        )
    })?;
    if !output.status.success() {
        bail!(
            "command failed: {}",
            render_command(command, Path::new("."))
        );
    }
    Ok(output.stdout)
}

pub(crate) fn run_command(command: &mut Command, cwd: Option<&Path>) -> Result<()> {
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    scrub_inherited_git_env(command);
    command.stdin(Stdio::null());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let status = command.status().with_context(|| {
        format!(
            "failed to execute command: {}",
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

pub(crate) fn print_bootstrapped_references(
    reference_root: &Path,
    references: &[BootstrappedReference],
) -> String {
    let output = format_bootstrapped_references(reference_root, references);
    print!("{output}");
    output
}

fn format_bootstrapped_references(
    reference_root: &Path,
    references: &[BootstrappedReference],
) -> String {
    let mut output = format!("reference root: {}\n", reference_root.display());
    for reference in references {
        writeln!(
            &mut output,
            "{}: {} @ {}",
            reference.name,
            reference.path.display(),
            reference.commit
        )
        .expect("writing to a String should not fail");
        writeln!(&mut output, "purpose: {}", reference.purpose)
            .expect("writing to a String should not fail");
    }
    output
}

fn render_command(command: &Command, cwd: &Path) -> String {
    let program = command.get_program().to_string_lossy();
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    format!("(cd {} && {} {})", cwd.display(), program, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use tempfile::TempDir;

    fn manifest_with_output_root(output_root: PathBuf) -> AssetManifest {
        AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::new(),
            upstream_commit: String::new(),
            source_root: PathBuf::new(),
            output_root,
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

    fn write_complete_layout(layout_dir: &Path, level_id: u8) {
        fs::create_dir_all(layout_dir).expect("layout dir");
        fs::write(layout_dir.join(format!("{level_id:02X}.bin")), b"layout").expect("bin");
        fs::write(layout_dir.join(format!("{level_id:02X}.json")), b"{}").expect("json");
        fs::write(
            layout_dir.join(format!("{level_id:02X}.blocklist.bin")),
            b"blocks",
        )
        .expect("blocklist");
    }

    fn file_child_path(temp: &TempDir) -> PathBuf {
        let file_parent = temp.path().join("not-a-dir");
        fs::write(&file_parent, b"file").expect("file parent");
        file_parent.join("child")
    }

    fn write_ascii(rom: &mut [u8], start: usize, len: usize, text: &str) {
        let bytes = text.as_bytes();
        let end = start + len.min(bytes.len());
        rom[start..end].copy_from_slice(&bytes[..end - start]);
    }

    fn write_test_rom(path: &Path) -> VerifiedRom {
        let mut rom = vec![0_u8; 0x200];
        write_ascii(&mut rom, 0x100, 0x10, "SEGA MEGA DRIVE");
        write_ascii(&mut rom, 0x120, 0x30, "KID CHAMELEON");
        write_ascii(&mut rom, 0x150, 0x30, "KID CHAMELEON");
        write_ascii(&mut rom, 0x180, 0x0E, "GM MK-1010");
        write_ascii(&mut rom, 0x1F0, 0x10, "JUE");
        fs::write(path, rom).expect("test ROM");
        verify_rom(path).expect("test ROM should verify")
    }

    /// Drive git in a scratch repo, immune to a hook's git environment.
    ///
    /// The pre-commit gate runs this suite, so without the scrub these helpers
    /// operate on the outer repository's index instead of the temp one.
    fn run_git(args: &[&str], cwd: &Path) {
        let mut command = Command::new("git");
        command.args(args).current_dir(cwd);
        scrub_inherited_git_env(&mut command);
        let status = command.status().expect("git should run");
        assert!(status.success(), "git {args:?} failed in {}", cwd.display());
    }

    fn setup_local_asl_checkout(repo_root: &Path, makefile: &str, existing_tools: &[&str]) {
        let remote_work = repo_root.join("origin-work");
        fs::create_dir_all(&remote_work).expect("origin worktree");
        run_git(&["init"], &remote_work);
        run_git(&["checkout", "-b", "main"], &remote_work);
        fs::write(remote_work.join("Makefile.def"), b"# generated for test\n")
            .expect("Makefile.def");
        run_git(&["add", "Makefile.def"], &remote_work);
        run_git(
            &[
                "-c",
                "user.email=connorads@users.noreply.github.com",
                "-c",
                "user.name=Wildside Test",
                "commit",
                "--no-verify",
                "-m",
                "seed",
            ],
            &remote_work,
        );
        run_git(&["branch", "upstream"], &remote_work);

        let source_dir = repo_root.join(".cache/tooling/asl-releases");
        fs::create_dir_all(source_dir.parent().expect("source parent")).expect("tooling parent");
        let mut clone = Command::new("git");
        clone
            .arg("clone")
            .arg(&remote_work)
            .arg(&source_dir)
            .current_dir(repo_root);
        scrub_inherited_git_env(&mut clone);
        let status = clone.status().expect("git clone should run");
        assert!(status.success(), "git clone failed");

        fs::write(source_dir.join("Makefile"), makefile).expect("Makefile");
        for tool in existing_tools {
            fs::write(source_dir.join(tool), b"tool").expect("existing tool");
        }
    }

    #[test]
    fn selected_reference_repos_preserves_full_and_single_reference_sets() {
        let all = selected_reference_repos(None);
        assert_eq!(
            all.iter().map(|repo| repo.name).collect::<Vec<_>>(),
            vec!["m68k", "gems", "gemsplay"]
        );

        assert_eq!(
            selected_reference_repos(Some(ReferenceName::M68k))[0].name,
            "m68k"
        );
        assert_eq!(
            selected_reference_repos(Some(ReferenceName::Gems))[0].name,
            "gems"
        );
        assert_eq!(
            selected_reference_repos(Some(ReferenceName::Gemsplay))[0].name,
            "gemsplay"
        );
    }

    #[test]
    fn level_layout_dump_complete_requires_data_metadata_and_blocklist() {
        let temp = TempDir::new().expect("tempdir");
        let layout_dir = temp.path();

        assert!(!level_layout_dump_complete(layout_dir, 0x02));

        fs::write(layout_dir.join("02.bin"), b"layout").expect("bin");
        assert!(!level_layout_dump_complete(layout_dir, 0x02));

        fs::write(layout_dir.join("02.json"), b"{}").expect("json");
        assert!(!level_layout_dump_complete(layout_dir, 0x02));

        fs::write(layout_dir.join("02.blocklist.bin"), b"blocks").expect("blocklist");
        assert!(level_layout_dump_complete(layout_dir, 0x02));
    }

    #[test]
    fn oracle_warp_broken_level_matches_known_jgenesis_skip_list() {
        for level_id in [0x09, 0x32, 0x33, 0x50, 0x57, 0x5A, 0x5D, 0x5E, 0x63] {
            assert!(oracle_warp_broken_level(level_id), "{level_id:02X}");
        }

        for level_id in [0x00, 0x01, 0x1F, 0x64] {
            assert!(!oracle_warp_broken_level(level_id), "{level_id:02X}");
        }
    }

    #[test]
    fn ensure_level_layouts_with_dumps_only_missing_requested_levels() {
        let temp = TempDir::new().expect("tempdir");
        let manifest = manifest_with_output_root(temp.path().to_path_buf());
        let layout_dir = temp.path().join("level-layout");
        write_complete_layout(&layout_dir, 0x01);

        let calls = RefCell::new(Vec::new());
        ensure_level_layouts_with(
            b"rom",
            &manifest,
            &[0x01, 0x02, 0x03, 0x33],
            |rom, level_id, dir| {
                assert_eq!(rom, b"rom");
                assert_eq!(dir, layout_dir.as_path());
                calls.borrow_mut().push(level_id);
                Ok(())
            },
        )
        .expect("missing explicit layouts should be dumped");

        assert_eq!(
            calls.into_inner(),
            vec![0x02, 0x03],
            "known jgenesis warp failures such as Level `$33` use the decoded native layout fallback instead of retrying an impossible RAM dump"
        );
    }

    #[test]
    fn ensure_gameplay_level_layouts_with_dumps_scenario_range_and_skips_oracle_broken_levels() {
        let temp = TempDir::new().expect("tempdir");
        let manifest = manifest_with_output_root(temp.path().to_path_buf());
        let layout_dir = temp.path().join("level-layout");
        write_complete_layout(&layout_dir, 0x00);

        let calls = RefCell::new(Vec::new());
        ensure_gameplay_level_layouts_with(
            b"rom",
            &manifest,
            GameplayScenario::SoakLevel10m,
            |rom, level_id, dir| {
                assert_eq!(rom, b"rom");
                assert_eq!(dir, layout_dir.as_path());
                calls.borrow_mut().push(level_id);
                Ok(())
            },
        )
        .expect("missing gameplay layouts should be dumped");

        let calls = calls.into_inner();
        assert_eq!(calls.len(), 30);
        assert!(!calls.contains(&0x00));
        assert!(!calls.contains(&0x09));
        assert!(calls.contains(&0x01));
        assert!(calls.contains(&0x1F));
    }

    #[test]
    fn ensure_play_native_level_layouts_with_includes_optional_warp_level() {
        let temp = TempDir::new().expect("tempdir");
        let manifest = manifest_with_output_root(temp.path().to_path_buf());

        let calls = RefCell::new(Vec::new());
        ensure_play_native_level_layouts_with(b"rom", &manifest, Some(0x35), |_, level_id, _| {
            calls.borrow_mut().push(level_id);
            Ok(())
        })
        .expect("play-native layouts should include soak range and explicit warp level");

        let calls = calls.into_inner();
        assert!(calls.contains(&0x00));
        assert!(calls.contains(&0x1F));
        assert!(calls.contains(&0x35));
        assert!(!calls.contains(&0x09));
    }

    #[test]
    fn public_layout_wrappers_report_missing_rom_before_dumping() {
        let temp = TempDir::new().expect("tempdir");
        let manifest = manifest_with_output_root(temp.path().to_path_buf());
        let missing_rom = temp.path().join("missing.bin");

        for error in [
            ensure_level_layouts(&missing_rom, &manifest, &[0x01])
                .expect_err("explicit layout wrapper should read ROM"),
            ensure_gameplay_level_layouts(
                &missing_rom,
                &manifest,
                GameplayScenario::PauseResumeLevel1,
            )
            .expect_err("gameplay layout wrapper should read ROM"),
            ensure_play_native_level_layouts(&missing_rom, &manifest, Some(0x01))
                .expect_err("play-native layout wrapper should read ROM"),
        ] {
            assert!(
                error.to_string().contains("failed to read ROM"),
                "unexpected error: {error:#}"
            );
        }
    }

    #[test]
    fn bootstrap_repo_entrypoints_surface_parent_create_errors() {
        let temp = TempDir::new().expect("tempdir");

        for error in [
            bootstrap_upstream(&file_child_path(&temp))
                .expect_err("upstream bootstrap should fail before git"),
            bootstrap_pinned_repo(
                &file_child_path(&temp),
                "https://example.invalid/repo.git",
                "abc",
            )
            .expect_err("pinned repo bootstrap should fail before git"),
            bootstrap_references(&file_child_path(&temp), Some(ReferenceName::M68k))
                .expect_err("reference bootstrap should fail before git"),
            bootstrap_shallow_pinned_repo(
                &file_child_path(&temp),
                "https://example.invalid/repo.git",
                "abc",
            )
            .expect_err("shallow repo bootstrap should fail before git"),
            bootstrap_cpu_vector_repos(&file_child_path(&temp))
                .expect_err("cpu vector bootstrap should fail before git"),
        ] {
            assert!(
                error.to_string().contains("failed to create"),
                "unexpected error: {error:#}"
            );
        }
    }

    /// The vector checkouts live under `.cache/`, inside this repository's work
    /// tree. A `rev-parse` that inherited the working directory would report
    /// wildside's own HEAD for a directory that was never cloned, silently
    /// stamping the wrong provenance onto a conformance baseline.
    #[test]
    fn head_commit_ignores_the_enclosing_repository() {
        let temp = TempDir::new().expect("tempdir");
        assert_eq!(head_commit(temp.path()), None);
        assert_eq!(head_commit(&temp.path().join("never-cloned")), None);

        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root");
        let inside = repo_root.join("crates");
        assert!(inside.is_dir(), "expected {} to exist", inside.display());
        assert_eq!(
            head_commit(&inside),
            None,
            "a plain directory inside the repo must not report the repo's HEAD"
        );
    }

    #[test]
    fn cpu_vector_repos_pin_both_published_sets() {
        let repos = cpu_vector_repos();
        assert_eq!(repos.len(), 2);
        for repo in repos {
            assert_eq!(
                repo.commit.len(),
                40,
                "{} must pin a full SHA, got {:?}",
                repo.name,
                repo.commit
            );
            assert!(repo.url.starts_with("https://github.com/SingleStepTests/"));
        }
    }

    #[test]
    fn prepare_lock_drop_removes_lock_file() {
        let temp = TempDir::new().expect("tempdir");
        let lock_path = temp.path().join(".cache/generated/.prepare.lock");

        {
            let _lock = acquire_prepare_lock(temp.path()).expect("lock should be acquired");
            assert!(lock_path.is_file());
        }

        assert!(
            !lock_path.exists(),
            "dropping the lock should remove the file"
        );
    }

    #[test]
    fn prepare_lock_waits_then_times_out_on_existing_lock() {
        let temp = TempDir::new().expect("tempdir");
        let lock_dir = temp.path().join(".cache/generated");
        fs::create_dir_all(&lock_dir).expect("lock dir");
        fs::write(lock_dir.join(".prepare.lock"), b"held").expect("held lock");

        let start = Instant::now();
        let now_calls = Cell::new(0);
        let sleep_calls = Cell::new(0);
        let error = acquire_prepare_lock_with_clock(
            temp.path(),
            Duration::from_millis(10),
            Duration::from_millis(3),
            || {
                let call = now_calls.get();
                now_calls.set(call + 1);
                match call {
                    0 => start,
                    1 => start + Duration::from_millis(1),
                    _ => start + Duration::from_millis(11),
                }
            },
            |delay| {
                assert_eq!(delay, Duration::from_millis(3));
                sleep_calls.set(sleep_calls.get() + 1);
            },
        )
        .expect_err("held lock should time out");

        assert!(
            error.to_string().contains("timed out waiting"),
            "unexpected error: {error:#}"
        );
        assert_eq!(sleep_calls.get(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn prepare_lock_reports_non_already_exists_open_errors_without_retry() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().expect("tempdir");
        let lock_dir = temp.path().join(".cache/generated");
        fs::create_dir_all(&lock_dir).expect("lock dir");
        let original_permissions = fs::metadata(&lock_dir)
            .expect("lock dir metadata")
            .permissions();
        fs::set_permissions(&lock_dir, fs::Permissions::from_mode(0o500))
            .expect("make lock dir read-only");

        let result = acquire_prepare_lock_with_clock(
            temp.path(),
            Duration::from_secs(1),
            Duration::from_millis(1),
            Instant::now,
            |_| panic!("non-AlreadyExists lock errors must not retry"),
        );

        fs::set_permissions(&lock_dir, original_permissions).expect("restore permissions");
        let error = result.expect_err("read-only lock dir should fail");
        assert!(
            error.to_string().contains("failed to open"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn sync_rom_into_upstream_copies_rom_to_kid_bin() {
        let temp = TempDir::new().expect("tempdir");
        let rom_path = temp.path().join("source.bin");
        let upstream_dir = temp.path().join("upstream");
        fs::create_dir_all(&upstream_dir).expect("upstream dir");
        fs::write(&rom_path, b"rom bytes").expect("source ROM");

        sync_rom_into_upstream(&rom_path, &upstream_dir).expect("ROM sync");

        assert_eq!(
            fs::read(upstream_dir.join("kid.bin")).expect("upstream ROM"),
            b"rom bytes"
        );
    }

    #[test]
    fn run_reference_build_accepts_bitperfect_local_build_script() {
        let temp = TempDir::new().expect("tempdir");
        let rom_path = temp.path().join("kid.bin");
        let verified = write_test_rom(&rom_path);
        let upstream_dir = temp.path().join("upstream");
        fs::create_dir_all(&upstream_dir).expect("upstream dir");
        fs::write(upstream_dir.join("split.py"), b"").expect("split script");
        fs::write(
            upstream_dir.join("build.sh"),
            b"#!/bin/sh\ncp kid.bin kid_built.bin\n",
        )
        .expect("build script");

        run_reference_build(temp.path(), &upstream_dir, &verified)
            .expect("local bit-perfect build should pass");
    }

    #[test]
    fn run_reference_build_reports_missing_source_rom() {
        let temp = TempDir::new().expect("tempdir");
        let rom_path = temp.path().join("kid.bin");
        let verified = write_test_rom(&rom_path);
        fs::remove_file(&rom_path).expect("remove source ROM");

        let error = run_reference_build(temp.path(), &temp.path().join("upstream"), &verified)
            .expect_err("missing source ROM should fail")
            .to_string();

        assert!(
            error.contains("failed to copy ROM"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn build_reference_rom_reports_failure_when_no_build_path_succeeds() {
        let temp = TempDir::new().expect("tempdir");
        let upstream_dir = temp.path().join("upstream");
        fs::create_dir_all(&upstream_dir).expect("upstream dir");
        fs::write(upstream_dir.join("build.sh"), b"#!/bin/sh\nexit 1\n").expect("build script");
        let source_dir = temp.path().join(".cache/tooling/asl-releases");
        fs::create_dir_all(source_dir.parent().expect("source parent")).expect("tooling parent");
        fs::write(&source_dir, b"not a directory").expect("file at tooling path");

        let error = build_reference_rom(temp.path(), &upstream_dir)
            .expect_err("failed bundled and native builds should error")
            .to_string();

        assert!(
            error.contains("failed to execute command") || error.contains("Not a directory"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn ensure_native_asl_tools_reports_parent_create_error_before_git() {
        let temp = TempDir::new().expect("tempdir");
        let tooling_parent = temp.path().join(".cache/tooling");
        fs::create_dir_all(tooling_parent.parent().expect("cache parent")).expect("cache dir");
        fs::write(&tooling_parent, b"file").expect("file at tooling parent");

        let error = ensure_native_asl_tools(temp.path())
            .expect_err("file parent should fail before git")
            .to_string();

        assert!(
            error.contains("failed to create"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn ensure_native_asl_tools_builds_when_either_binary_is_missing() {
        let temp = TempDir::new().expect("tempdir");
        setup_local_asl_checkout(
            temp.path(),
            "all:\n\tprintf asl > asl\n\tprintf p2bin > p2bin\n",
            &["asl"],
        );

        let tools = ensure_native_asl_tools(temp.path()).expect("ASL tools");

        assert!(tools.asl.is_file());
        assert!(tools.p2bin.is_file());
    }

    #[test]
    fn ensure_native_asl_tools_does_not_rebuild_when_binaries_exist() {
        let temp = TempDir::new().expect("tempdir");
        setup_local_asl_checkout(temp.path(), "all:\n\texit 42\n", &["asl", "p2bin"]);

        let tools = ensure_native_asl_tools(temp.path()).expect("ASL tools");

        assert_eq!(fs::read(&tools.asl).expect("asl"), b"tool");
        assert_eq!(fs::read(&tools.p2bin).expect("p2bin"), b"tool");
    }

    #[test]
    fn ensure_native_asl_tools_rebuilds_after_the_upstream_ref_is_already_checked_out() {
        // Regression: the bootstrap used to fetch `upstream:upstream`, which git
        // refuses once that branch is checked out, so any resumed bootstrap of a
        // half-built toolchain failed forever ("refusing to fetch into branch").
        let temp = TempDir::new().expect("tempdir");
        setup_local_asl_checkout(
            temp.path(),
            "all:\n\tprintf asl > asl\n\tprintf p2bin > p2bin\n",
            &[],
        );
        let source_dir = temp.path().join(".cache/tooling/asl-releases");
        run_git(
            &["checkout", "-b", "upstream", "--track", "origin/upstream"],
            &source_dir,
        );

        ensure_native_asl_tools(temp.path()).expect("first bootstrap");
        fs::remove_file(source_dir.join("p2bin")).expect("drop one binary");
        let tools = ensure_native_asl_tools(temp.path()).expect("resumed bootstrap");

        assert!(tools.p2bin.is_file());
    }

    #[test]
    fn command_helpers_drop_inherited_git_hook_environment() {
        // Regression: run from a git hook, cache-local `git` calls inherited the
        // outer repo's GIT_INDEX_FILE/GIT_DIR and failed to check out a cache
        // repo's own commit. The scrub runs last, so it wins over an explicit
        // `.env` too; nothing here ever wants to pass git's hook variables on.
        for name in INHERITED_GIT_ENV {
            let stdout = capture_stdout(
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("printf %s \"${{{name}-unset}}\""))
                    .env(name, "/leaked/from/hook"),
            )
            .expect("probe command should pass");

            assert_eq!(stdout, b"unset", "{name} leaked into the child");
        }
    }

    #[test]
    fn command_helpers_report_stdout_and_failures() {
        let stdout = capture_stdout(Command::new("sh").arg("-c").arg("printf wildside"))
            .expect("stdout command should pass");
        assert_eq!(stdout, b"wildside");

        let capture_error = capture_stdout(Command::new("sh").arg("-c").arg("exit 3"))
            .expect_err("failed stdout command should fail")
            .to_string();
        assert!(
            capture_error.contains("command failed"),
            "unexpected error: {capture_error}"
        );

        let run_error = run_command(Command::new("sh").arg("-c").arg("exit 7"), None)
            .expect_err("failed command should fail")
            .to_string();
        assert!(
            run_error.contains("command failed"),
            "unexpected error: {run_error}"
        );
    }

    #[test]
    fn rendered_and_printed_reference_summaries_include_root_and_purpose() {
        let reference = BootstrappedReference {
            name: "m68k",
            path: PathBuf::from("/refs/m68k"),
            commit: "abc123",
            purpose: "instruction reference",
        };

        let output = print_bootstrapped_references(Path::new("/refs"), &[reference]);

        assert!(output.contains("reference root: /refs"));
        assert!(output.contains("m68k: /refs/m68k @ abc123"));
        assert!(output.contains("purpose: instruction reference"));
    }

    #[test]
    fn render_command_includes_cwd_program_and_args() {
        let mut command = Command::new("python3");
        command.arg("split.py").arg("--flag");

        assert_eq!(
            render_command(&command, Path::new("/repo/upstream")),
            "(cd /repo/upstream && python3 split.py --flag)"
        );
    }
}
