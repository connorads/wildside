use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use kid_data::{
    AssetManifest, ensure_generated_bundle_contract, export_split_bundle, generated_bundle_dir,
    load_manifest, manifest_path as generated_manifest_path,
};
use kid_rom::{VerifiedRom, verify_rom};
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

pub(crate) struct NativeAslTools {
    pub(crate) root: PathBuf,
    pub(crate) asl: PathBuf,
    pub(crate) p2bin: PathBuf,
}

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
    let missing_levels = (start_level..=end_level)
        .filter(|level_id| {
            let data_path = layout_dir.join(format!("{level_id:02X}.bin"));
            let metadata_path = layout_dir.join(format!("{level_id:02X}.json"));
            !(data_path.is_file() && metadata_path.is_file())
        })
        .collect::<Vec<_>>();
    if missing_levels.is_empty() {
        return Ok(());
    }

    for level_id in missing_levels {
        if oracle_warp_broken_level(level_id) {
            // jgenesis still fails warped entry on a small set of levels with
            // `game_mode=0x4E73`; native can fall back to the decoded block
            // layout for those levels, so do not block gameplay verification
            // on an oracle-only extraction failure.
            continue;
        }
        dump_level_layout_fn(rom_bytes, level_id, &layout_dir)?;
    }
    Ok(())
}

fn oracle_warp_broken_level(level_id: u8) -> bool {
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
    let lock_dir = repo_root.join(".cache/generated");
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("failed to create {}", lock_dir.display()))?;
    let lock_path = lock_dir.join(".prepare.lock");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => return Ok(PrepareLock { path: lock_path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if Instant::now() >= deadline {
                    bail!(
                        "timed out waiting for asset prepare lock at {}",
                        lock_path.display()
                    );
                }
                thread::sleep(Duration::from_millis(200));
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

    run_command(
        Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg("upstream:upstream"),
        Some(&source_dir),
    )?;
    run_command(
        Command::new("git").arg("checkout").arg("upstream"),
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

    let asl = source_dir.join("asl");
    let p2bin = source_dir.join("p2bin");
    if !asl.is_file() || !p2bin.is_file() {
        run_command(Command::new("make").arg("-j4"), Some(&source_dir))?;
    }

    Ok(NativeAslTools {
        root: source_dir,
        asl,
        p2bin,
    })
}

pub(crate) fn capture_stdout(command: &mut Command) -> Result<Vec<u8>> {
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
) {
    println!("reference root: {}", reference_root.display());
    for reference in references {
        println!(
            "{}: {} @ {}",
            reference.name,
            reference.path.display(),
            reference.commit
        );
        println!("purpose: {}", reference.purpose);
    }
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
