use std::{env, path::PathBuf, process::Command};

fn find_binary(sub_dir: &str, bin_name: &str) -> Option<PathBuf> {
    let exe_name = format!("{}{}", bin_name, env::consts::EXE_SUFFIX);

    let relative_path = PathBuf::from("bin").join(sub_dir).join(&exe_name);

    if let Ok(cwd) = env::current_dir() {
        let path = cwd.join(&relative_path);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(mut exe_dir) = env::current_exe() {
        exe_dir.pop(); // Remove the executable name to get its directory
        let path = exe_dir.join(&relative_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn verify_execution(cmd: &str, arg: Option<&str>) {
    if Command::new(cmd)
        .arg(arg.unwrap_or("-version"))
        .output()
        .is_ok()
    {
        println!("✅ {cmd} are globally accessible to the app!");
    } else {
        println!("⚠️ Binay {cmd} found, but still not accessible via PATH.");
    }
}

pub fn load_bin() {
    const BINARIES: [(&str, &str); 2] = [("ffmpeg", "ffmpeg"), ("yt-dlp", "yt-dlp")];

    let mut extra_paths: Vec<(&str, PathBuf)> = Vec::new();

    for (dir, bin) in BINARIES {
        if let Some(p) = find_binary(dir, bin) {
            if let Some(parent) = p.parent() {
                extra_paths.push((bin, parent.to_path_buf()));
            }
        }
    }

    if let Some(old_path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&old_path).collect::<Vec<_>>();

        for (_, path) in extra_paths.clone() {
            if !paths.contains(&path) {
                paths.insert(0, path);
            }
        }

        let new_path = env::join_paths(paths).expect("Failed to join paths");

        unsafe {
            env::set_var("PATH", &new_path);
        }
    }

    for (name, _) in extra_paths {
        verify_execution(name, None);
    }
}
