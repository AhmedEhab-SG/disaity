use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Tell Cargo to re-run this build script if the source bin/ folder changes
    println!("cargo:rerun-if-changed=bin");

    // 1. Determine the target OS executable suffix
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    let exe_suffix = if target_family == "windows" {
        ".exe"
    } else {
        ""
    };

    // 2. Find the target profile directory (e.g., target/debug or target/release)
    // OUT_DIR is typically heavily nested: target/debug/build/disaity-<hash>/out
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir);

    // Navigate up 3 directories to reach the profile root
    let target_dir = out_path
        .parent()
        .unwrap() // out
        .parent()
        .unwrap() // disaity-<hash>
        .parent()
        .unwrap(); // build
    // Now at target/debug or target/release

    let dest_bin_dir = target_dir.join("bin");

    // 3. Define the binaries we want to copy
    let binaries = [
        ("engines", "media", "ffmpeg"),
        ("providers", "youtube", "yt-dlp"),
    ];

    let src_bin_dir = Path::new("bin");

    // 4. Copy files based on target OS
    if src_bin_dir.exists() {
        for (dir, subdir, base_name) in binaries {
            let file_name = format!("{}{}", base_name, exe_suffix);
            let src_path = src_bin_dir.join(dir).join(subdir).join(&file_name);

            if src_path.exists() {
                // Ensure the destination subdirectory exists
                let dest_subdir = dest_bin_dir.join(dir).join(subdir);
                fs::create_dir_all(&dest_subdir).expect("Failed to create destination directories");

                let dest_path = dest_subdir.join(&file_name);

                // Copy the file
                fs::copy(&src_path, &dest_path).unwrap_or_else(|e| {
                    panic!(
                        "Failed to copy {} to {}: {}",
                        src_path.display(),
                        dest_path.display(),
                        e
                    );
                });
            } else {
                // Emits a warning in the terminal if a file is missing
                println!(
                    "cargo:warning=Missing binary in source tree: {}",
                    src_path.display()
                );
            }
        }
    }
}
