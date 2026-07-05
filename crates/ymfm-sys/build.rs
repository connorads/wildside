use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root");
    let ymfm_dir = repo_root.join("third_party/ymfm/src");

    println!("cargo:rerun-if-changed={}", ymfm_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("src/bridge.cpp").display()
    );

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .flag_if_supported("-std=c++14")
        .include(&ymfm_dir)
        .file(manifest_dir.join("src/bridge.cpp"))
        .file(ymfm_dir.join("ymfm_adpcm.cpp"))
        .file(ymfm_dir.join("ymfm_misc.cpp"))
        .file(ymfm_dir.join("ymfm_opn.cpp"))
        .file(ymfm_dir.join("ymfm_pcm.cpp"))
        .file(ymfm_dir.join("ymfm_ssg.cpp"));

    // Bare wasm has no host C++ toolchain: compile via zig's clang with its
    // bundled wasm32-wasi libc++ headers (#20). The objects link fine into a
    // wasm32-unknown-unknown module; the handful of C++ runtime symbols the
    // code needs (operator new/delete, abort hooks) are provided as Rust
    // shims in src/wasm_shims.rs, so no C++ stdlib is linked.
    if env::var("TARGET").as_deref() == Ok("wasm32-unknown-unknown") {
        configure_zig_for_wasm(&mut build);
        build.cpp_link_stdlib(None);
        // ymfm is computation-only; drop the stack-protector calls libc++
        // headers would otherwise import (__stack_chk_fail).
        build.flag("-fno-stack-protector");
    }

    build.compile("ymfm_bridge");
}

/// Route cc-rs through generated wrapper scripts that call `zig c++`/`zig ar`
/// for the wasm target. zig has no libc++ for `wasm32-unknown-unknown` (only
/// freestanding, header-less), so the wrapper compiles for `wasm32-wasi` and
/// strips the clang-style `--target` flag cc-rs injects. The zig cache is
/// pinned inside OUT_DIR: build scripts can run under sandboxes that deny
/// zig's default global cache directory.
fn configure_zig_for_wasm(build: &mut cc::Build) {
    let zig = find_zig();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let cache_dir = out_dir.join("zig-cache");

    let cxx_wrapper = out_dir.join("zig-cxx-wasm32");
    write_executable(
        &cxx_wrapper,
        &format!(
            "#!/bin/sh\n\
             export ZIG_GLOBAL_CACHE_DIR=\"{cache}\"\n\
             export ZIG_LOCAL_CACHE_DIR=\"{cache}\"\n\
             i=0; n=$#\n\
             while [ $i -lt $n ]; do\n\
             \ta=$1; shift\n\
             \t[ \"$a\" = \"--target=wasm32-unknown-unknown\" ] || set -- \"$@\" \"$a\"\n\
             \ti=$((i+1))\n\
             done\n\
             exec \"{zig}\" c++ -target wasm32-wasi \"$@\"\n",
            cache = cache_dir.display(),
            zig = zig.display(),
        ),
    );

    let ar_wrapper = out_dir.join("zig-ar-wasm32");
    write_executable(
        &ar_wrapper,
        &format!("#!/bin/sh\nexec \"{}\" ar \"$@\"\n", zig.display()),
    );

    build.compiler(&cxx_wrapper);
    build.archiver(&ar_wrapper);
}

/// Locate zig: `WILDSIDE_ZIG` env override first, then `zig` on PATH (the
/// repo's .mise.toml pins it).
fn find_zig() -> PathBuf {
    println!("cargo:rerun-if-env-changed=WILDSIDE_ZIG");
    if let Ok(explicit) = env::var("WILDSIDE_ZIG") {
        let explicit = PathBuf::from(explicit);
        assert!(
            explicit.is_file(),
            "WILDSIDE_ZIG is set but {} does not exist",
            explicit.display()
        );
        return explicit;
    }
    let path = env::var_os("PATH").unwrap_or_default();
    for dir in env::split_paths(&path) {
        let candidate = dir.join("zig");
        if candidate.is_file() {
            return candidate;
        }
    }
    panic!(
        "building ymfm-sys for wasm32-unknown-unknown needs zig (bundled \
         wasm libc++). Install via mise (`mise install zig`, pinned in \
         .mise.toml) or point WILDSIDE_ZIG at a zig binary."
    );
}

fn write_executable(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap_or_else(|error| {
        panic!("failed to write {}: {error}", path.display());
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap_or_else(
            |error| {
                panic!("failed to chmod {}: {error}", path.display());
            },
        );
    }
}
