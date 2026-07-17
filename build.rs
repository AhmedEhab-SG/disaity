use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

const VERSION: &str = "v0.1.1-deps";
const BASE_URL: &str = "https://github.com/AhmedEhab-SG/disaity/releases/download";

const BINARIES: [(&str, &str, &str); 2] = [
    ("engines", "media", "ffmpeg"),
    ("providers", "youtube", "yt-dlp"),
];

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    println!("cargo:warning=Downloading dependency from {}...", url);

    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("Failed to download {}: {}", url, e))?;

    let mut reader = response.into_body().into_reader();
    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755); // Make it executable on Linux/macOS
        fs::set_permissions(path, perms).unwrap();
    }

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=bin");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let mut target_dir = PathBuf::from(out_dir);

    while target_dir.ends_with("out") || target_dir.parent().map_or(false, |p| p.ends_with("build"))
    {
        target_dir.pop();
    }

    let dest_bin_dir = target_dir.join("bin");
    let src_root = Path::new("bin");

    // Check the target architecture being built, not the host machine running the build
    let target_os = env::var("TARGET").expect("TARGET environment variable not set");
    let exe_suffix = if target_os.contains("windows") {
        ".exe"
    } else {
        ""
    };

    for (dir, sub, name) in BINARIES {
        let file_name = format!("{}{}", name, exe_suffix);
        let folder_path = src_root.join(dir).join(sub);
        let src_path = folder_path.join(&file_name);
        let dest_path = dest_bin_dir.join(dir).join(sub).join(&file_name);

        // 1. Ensure the source directory exists
        if !folder_path.exists() {
            fs::create_dir_all(&folder_path).unwrap();
        }

        // 2. Download if missing from source
        if !src_path.exists() {
            let url = format!("{}/{}/{}", BASE_URL, VERSION, file_name);
            if let Err(e) = download_file(&url, &src_path) {
                panic!("Dependency missing and download failed: {}", e);
            }
        }

        // 3. Copy to target directory for the build
        fs::create_dir_all(dest_path.parent().unwrap()).ok();
        fs::copy(&src_path, &dest_path).unwrap_or_else(|e| {
            panic!("Failed to copy {}: {}", src_path.display(), e);
        });
    }
}
