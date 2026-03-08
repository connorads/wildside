use std::env;
use std::path::PathBuf;

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

    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .flag_if_supported("-std=c++14")
        .include(&ymfm_dir)
        .file(manifest_dir.join("src/bridge.cpp"))
        .file(ymfm_dir.join("ymfm_adpcm.cpp"))
        .file(ymfm_dir.join("ymfm_misc.cpp"))
        .file(ymfm_dir.join("ymfm_opn.cpp"))
        .file(ymfm_dir.join("ymfm_pcm.cpp"))
        .file(ymfm_dir.join("ymfm_ssg.cpp"))
        .compile("ymfm_bridge");
}
