pub fn load_bin() {
    #[cfg(any(target_os = "windows", target_family = "unix"))]
    {
        use std::env;

        let current_dir = env::current_dir().unwrap();
        let bin_path = current_dir.join("bin");

        if let Some(path) = env::var_os("PATH") {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();
            paths.insert(0, bin_path);
            let new_path = env::join_paths(paths).unwrap();

            unsafe {
                env::set_var("PATH", &new_path);
            }
        }
    }

    let check = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output();
    match check {
        Ok(_) => println!("✅ FFmpeg is ready to go!"),
        Err(_) => println!("❌ FFmpeg NOT FOUND. Install it or check /bin folder."),
    }

    let check_yt = std::process::Command::new("yt-dlp")
        .arg("-version")
        .output();
    match check_yt {
        Ok(_) => println!("✅ yt-dlp is ready to go!"),
        Err(_) => println!("❌ yt-dlp NOT FOUND. Install it or check /bin folder."),
    }
}
