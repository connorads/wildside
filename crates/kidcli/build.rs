fn main() {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("xcrun")
            .args(["--show-sdk-path"])
            .output()
            && output.status.success()
        {
            let sdk_path = String::from_utf8_lossy(&output.stdout);
            let sdk_path = sdk_path.trim();
            println!("cargo:rustc-link-search=native={sdk_path}/usr/lib");
        }
    }
}
