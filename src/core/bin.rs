use std::{env, path::PathBuf, process::Command};

pub struct BinariesExt<'a> {
    pub binaries: [(&'a str, &'a str, &'a str); 2],
}

impl BinariesExt<'_> {
    fn find_binary(dir: &str, sub_dir: &str, bin_name: &str) -> Option<PathBuf> {
        let exe_name = format!("{}{}", bin_name, env::consts::EXE_SUFFIX);

        let relative_path = PathBuf::from("bin").join(dir).join(sub_dir).join(&exe_name);

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

    fn ensure_executable(path: &PathBuf) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use std::{fs, os::unix::fs::PermissionsExt};

            let mut perms = fs::metadata(path)?.permissions();
            let mode = perms.mode();

            // 0o111 represents execute permissions for owner, group, and others
            if mode & 0o111 != 0o111 {
                perms.set_mode(mode | 0o111);
                fs::set_permissions(path, perms)?;
                println!("🔒 Set execution permissions for {}", path.display());
            }
        }
        Ok(())
    }

    pub fn load() -> Self {
        const BINARIES: [(&str, &str, &str); 2] = [
            ("engines", "media", "ffmpeg"),
            ("providers", "youtube", "yt-dlp"),
        ];

        let mut extra_paths: Vec<(&str, PathBuf)> = Vec::new();

        for (dir, sub_dir, bin) in BINARIES {
            if let Some(p) = Self::find_binary(dir, sub_dir, bin) {
                if let Err(e) = Self::ensure_executable(&p) {
                    println!(
                        "⚠️ Failed to set execution permissions for {}: {}",
                        p.display(),
                        e
                    );
                }

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
            Self::verify_execution(name, None);
        }

        Self { binaries: BINARIES }
    }
}
