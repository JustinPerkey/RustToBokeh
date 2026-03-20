use std::path::{Path, PathBuf};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    // Copy Python DLLs from vendor/python/ to the target directory so the
    // OS loader can find them next to the executable at runtime.
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let vendor_dir = manifest_dir.join("vendor").join("python");

    if !vendor_dir.exists() {
        println!(
            "cargo:warning=vendor/python/ not found — run `bash scripts/setup_vendor.sh` first"
        );
        return;
    }

    // OUT_DIR is something like target/release/build/<crate>-<hash>/out/
    // Walk up to find the profile directory (target/release/ or target/debug/).
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "release" || n == "debug"))
        .map(Path::to_path_buf);

    let Some(target_dir) = target_dir else {
        println!("cargo:warning=Could not determine target directory from OUT_DIR");
        return;
    };

    let dlls = ["python3.dll", "python312.dll"];
    for dll in &dlls {
        let src = vendor_dir.join(dll);
        let dst = target_dir.join(dll);
        if src.exists() && !dst.exists() {
            std::fs::copy(&src, &dst).unwrap_or_else(|e| {
                panic!("Failed to copy {} to {}: {}", src.display(), dst.display(), e)
            });
            println!("cargo:warning=Copied {} to {}", dll, target_dir.display());
        }
    }

    // Re-run if the vendor directory changes.
    println!("cargo:rerun-if-changed=vendor/python/python3.dll");
    println!("cargo:rerun-if-changed=vendor/python/python312.dll");
}
