use std::{
    env, fs,
    path::{Path, PathBuf},
};

const BINARIES: [(&str, &str, &str); 2] = [
    ("engines", "media", "ffmpeg"),
    ("providers", "youtube", "yt-dlp"),
];

fn main() {
    println!("cargo:rerun-if-changed=bin");

    // Get the profile root (target/debug or target/release)
    // Using a more robust way to find the profile directory
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let mut target_dir = PathBuf::from(out_dir);

    // Pop until we are out of the /build/pkg-hash/out/ directory
    while target_dir.ends_with("out") || target_dir.parent().map_or(false, |p| p.ends_with("build"))
    {
        target_dir.pop();
    }
    // Now at target/debug or target/release

    let dest_bin_dir = target_dir.join("bin");
    let src_root = Path::new("bin");

    if src_root.exists() {
        for (dir, sub, name) in BINARIES {
            let file_name = format!("{}{}", name, env::consts::EXE_SUFFIX);
            let src_path = src_root.join(dir).join(sub).join(&file_name);
            let dest_path = dest_bin_dir.join(dir).join(sub).join(&file_name);

            if src_path.exists() {
                fs::create_dir_all(dest_path.parent().unwrap()).ok();
                fs::copy(&src_path, &dest_path).unwrap_or_else(|e| {
                    panic!("Failed to copy {}: {}", src_path.display(), e);
                });
            } else {
                println!(
                    "cargo:warning=Binary not found in source: {}",
                    src_path.display()
                );
            }
        }
    }
}
