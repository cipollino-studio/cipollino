
#[cfg(target_os = "macos")]
#[cfg(debug_assertions)]
pub const FFMPEG_PATH: &'static str = "./libs/bin/macos_arm64/ffmpeg"; 

#[cfg(target_os = "macos")]
#[cfg(not(debug_assertions))]
pub const FFMPEG_PATH: &'static str = "../ffmpeg"; 

#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
pub const FFMPEG_PATH: &'static str = "./libs/bin/windows_x86/ffmpeg.exe"; 

#[cfg(target_os = "windows")]
#[cfg(not(debug_assertions))]
pub const FFMPEG_PATH: &'static str = "./ffmpeg.exe"; 