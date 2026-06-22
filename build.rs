fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let target_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join(&profile);

    let configs = ["config.toml", "config-safe.toml", "config-simple.toml"];

    for cfg in &configs {
        let src = std::path::Path::new(&manifest_dir).join(cfg);
        if src.exists() {
            let dst = target_dir.join(cfg);
            match std::fs::copy(&src, &dst) {
                Ok(_) => println!("cargo:warning=Copied {} -> {}", src.display(), dst.display()),
                Err(e) => println!("cargo:warning=Failed to copy {}: {}", cfg, e),
            }
        }
    }
}
