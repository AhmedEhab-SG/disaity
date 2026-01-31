pub fn load_bin() {
    #[cfg(any(target_os = "windows", target_family = "unix"))]
    {
        use std::env;

        let project_bin = env::current_dir().unwrap().join("bin");

        let mut exe_bin = env::current_exe().unwrap();
        exe_bin.pop();
        let exe_bin = exe_bin.join("bin");

        if let Some(path) = env::var_os("PATH") {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();

            paths.insert(0, exe_bin);
            paths.insert(0, project_bin);

            let new_path = env::join_paths(paths).unwrap();
            unsafe {
                env::set_var("PATH", &new_path);
            }
        }
    }

    {
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
}
