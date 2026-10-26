use serde::{Deserialize, Serialize};
use std::{env, fs, path::{Path, PathBuf}, process::{Command, Stdio}, sync::Mutex};
use tauri::{Manager, State};

#[derive(Default)]
struct ToolState(Mutex<Option<Tools>>);
#[derive(Clone)]
struct Tools { ffmpeg: PathBuf, ffprobe: PathBuf }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolInfo { ffmpeg: String, ffprobe: String, version: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoInfo { path: String, name: String, duration: f64, preview_path: String, has_audio: bool }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Segment { path: String, start: f64, end: f64, has_audio: bool }

#[cfg(windows)]
fn hidden(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000)
}
#[cfg(not(windows))]
fn hidden(cmd: &mut Command) -> &mut Command { cmd }
fn video_command(path: &Path) -> Command {
    let mut cmd = Command::new(path);
    hidden(&mut cmd);
    cmd
}
fn valid_binary(path: &Path) -> bool {
    path.is_file() && video_command(path).arg("-version").stdout(Stdio::null()).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}
fn find_binary(name: &str, manual: Option<&str>) -> Option<PathBuf> {
    let exe = format!("{name}.exe");
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(m) = manual { dirs.push(PathBuf::from(m)); }
    if let Some(p) = env::var_os("PATH") { dirs.extend(env::split_paths(&p)); }
    if let Ok(current) = env::current_exe() { if let Some(parent) = current.parent() { dirs.push(parent.to_path_buf()); dirs.push(parent.join("bin")); } }
    dirs.extend([PathBuf::from(r"C:\ffmpeg\bin"), PathBuf::from(r"C:\Program Files\ffmpeg\bin"), PathBuf::from(r"C:\ProgramData\chocolatey\bin")]);
    for dir in dirs { let candidate = if dir.is_file() { dir } else { dir.join(&exe) }; if valid_binary(&candidate) { return Some(candidate); } }
    None
}
fn resolve_tools(manual: Option<&str>) -> Result<Tools, String> {
    let ffmpeg = find_binary("ffmpeg", manual).ok_or("Không tìm thấy ffmpeg.exe. Hãy chọn thư mục bin chứa FFmpeg.")?;
    let ffprobe = find_binary("ffprobe", manual).ok_or("Không tìm thấy ffprobe.exe. Hãy chọn thư mục bin chứa ffprobe.")?;
    Ok(Tools { ffmpeg, ffprobe })
}
fn tools_from_state(state: &State<'_, ToolState>) -> Result<Tools, String> {
    state.0.lock().map_err(|e| e.to_string())?.clone().ok_or("Chưa cấu hình FFmpeg. Mở Cài đặt để quét lại.".to_string())
}
#[tauri::command]
fn detect_ffmpeg(state: State<'_, ToolState>, folder: Option<String>) -> Result<ToolInfo, String> {
    let tools = resolve_tools(folder.as_deref())?;
    let output = video_command(&tools.ffmpeg).arg("-version").output().map_err(|e| e.to_string())?;
    let version = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("FFmpeg").to_string();
    let result = ToolInfo { ffmpeg: tools.ffmpeg.display().to_string(), ffprobe: tools.ffprobe.display().to_string(), version };
    *state.0.lock().map_err(|e| e.to_string())? = Some(tools);
    Ok(result)
}
fn probe_video(tools: &Tools, path: &str) -> Result<(f64, bool), String> {
    let output = video_command(&tools.ffprobe).args(["-v", "error", "-show_entries", "format=duration:stream=codec_type", "-of", "json", path]).output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Err(format!("ffprobe không đọc được file: {}", String::from_utf8_lossy(&output.stderr))); }
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let duration = data["format"]["duration"].as_str().and_then(|v| v.parse::<f64>().ok()).ok_or("Không đọc được thời lượng video")?;
    let has_audio = data["streams"].as_array().map(|a| a.iter().any(|v| v["codec_type"] == "audio")).unwrap_or(false);
    if !duration.is_finite() || duration <= 0.0 { return Err("Thời lượng video không hợp lệ".into()); }
    Ok((duration, has_audio))
}
#[tauri::command]
async fn import_video(app: tauri::AppHandle, state: State<'_, ToolState>, path: String) -> Result<VideoInfo, String> {
    let tools = tools_from_state(&state)?;
    let cache = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("previews");
    tauri::async_runtime::spawn_blocking(move || {
        if !Path::new(&path).is_file() { return Err("Không tìm thấy video".into()); }
        let (duration, has_audio) = probe_video(&tools, &path)?;
        fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
        let name = Path::new(&path).file_name().and_then(|s| s.to_str()).unwrap_or("Video").to_string();
        // Generate a compact H.264 preview for WebView2, including MKV/HEVC sources.
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
        let preview = cache.join(format!("preview-{stamp}.mp4"));
        let output = video_command(&tools.ffmpeg).args(["-hide_banner", "-loglevel", "error", "-y", "-i", &path, "-vf", "scale=w=960:h=540:force_original_aspect_ratio=decrease,format=yuv420p", "-r", "24", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "29", "-an", "-movflags", "+faststart"]).arg(&preview).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(format!("Không tạo được preview: {}", String::from_utf8_lossy(&output.stderr))); }
        Ok(VideoInfo { path, name, duration, preview_path: preview.to_string_lossy().to_string(), has_audio })
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn export_video(state: State<'_, ToolState>, segments: Vec<Segment>, output_path: String) -> Result<String, String> {
    let tools = tools_from_state(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        if segments.is_empty() { return Err("Chưa có clip để xuất".into()); }
        let dest = PathBuf::from(&output_path);
        let parent = dest.parent().ok_or("Đường dẫn xuất không hợp lệ")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = parent.join(format!(".minicut-{}", std::process::id()));
        fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            let mut clips = Vec::new();
            for (index, s) in segments.iter().enumerate() {
                if !s.start.is_finite() || !s.end.is_finite() || s.end <= s.start || s.start < 0.0 { return Err(format!("Đoạn {} có thời gian không hợp lệ", index+1)); }
                let file = temp.join(format!("clip-{index:04}.mp4"));
                let duration = s.end - s.start;
                let mut cmd = video_command(&tools.ffmpeg);
                cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-ss", &format!("{:.3}", s.start), "-t", &format!("{:.3}", duration), "-i", &s.path]);
                if !s.has_audio { cmd.args(["-f", "lavfi", "-t", &format!("{:.3}", duration), "-i", "anullsrc=channel_layout=stereo:sample_rate=48000"]); }
                cmd.args(["-map", "0:v:0"]);
                if s.has_audio { cmd.args(["-map", "0:a:0"]); } else { cmd.args(["-map", "1:a:0"]); }
                cmd.args(["-vf", "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30,format=yuv420p", "-c:v", "libx264", "-preset", "veryfast", "-crf", "22", "-c:a", "aac", "-ar", "48000", "-ac", "2", "-b:a", "160k", "-shortest", "-movflags", "+faststart"]);
                let out = cmd.arg(&file).output().map_err(|e| e.to_string())?;
                if !out.status.success() { return Err(format!("Lỗi xử lý clip {}: {}", index+1, String::from_utf8_lossy(&out.stderr))); }
                clips.push(file);
            }
            // concat demuxer with file list stored in a temporary directory.
            let list = temp.join("concat.txt");
            let lines = clips.iter().map(|p| format!("file '{}'", p.file_name().unwrap().to_string_lossy())).collect::<Vec<_>>().join("\n");
            fs::write(&list, lines).map_err(|e| e.to_string())?;
            let out = video_command(&tools.ffmpeg).current_dir(&temp).args(["-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", "concat.txt", "-c", "copy", "-movflags", "+faststart"]).arg(&dest).output().map_err(|e| e.to_string())?;
            if !out.status.success() { return Err(format!("Ghép video thất bại: {}", String::from_utf8_lossy(&out.stderr))); }
            Ok(())
        })();
        let _ = fs::remove_dir_all(&temp);
        result.map(|_| output_path)
    }).await.map_err(|e| e.to_string())?
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ToolState::default())
        .invoke_handler(tauri::generate_handler![detect_ffmpeg, import_video, export_video])
        .run(tauri::generate_context!())
        .expect("MiniCut startup failed");
}
