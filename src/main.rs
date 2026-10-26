use eframe::egui;
use std::{path::{Path, PathBuf}, process::{Command, Stdio}, sync::mpsc::{self, Receiver, Sender}, thread, time::{Duration, Instant}};

#[derive(Clone)]
struct Clip {
    path: PathBuf,
    duration: f64,
    start: f64,
    end: f64,
    has_audio: bool,
}
impl Clip { fn length(&self) -> f64 { (self.end - self.start).max(0.0) } }

struct FrameRequest { path: PathBuf, time: f64, generation: u64 }
struct FrameResult { png: Result<Vec<u8>, String>, generation: u64 }
struct ExportResult(Result<PathBuf, String>);

struct MiniCut {
    clips: Vec<Clip>, selected: Option<usize>, playhead: f64,
    texture: Option<egui::TextureHandle>,
    frame_tx: Sender<FrameRequest>, frame_rx: Receiver<FrameResult>,
    last_request: Instant, needs_frame: bool, generation: u64,
    export_rx: Option<Receiver<ExportResult>>, exporting: bool,
    status: String,
}

fn ffprobe(path: &Path) -> Result<(f64, bool), String> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration:stream=codec_type", "-of", "json"])
        .arg(path).output().map_err(|e| format!("Không chạy được ffprobe: {e}"))?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let duration = value["format"]["duration"].as_str()
        .ok_or("Không đọc được thời lượng video")?
        .parse::<f64>().map_err(|e| e.to_string())?;
    if !duration.is_finite() || duration <= 0.0 { return Err("Thời lượng video không hợp lệ".into()); }
    let streams = value["streams"].as_array().ok_or("Không đọc được streams")?;
    if !streams.iter().any(|s| s["codec_type"] == "video") { return Err("File không có video".into()); }
    let has_audio = streams.iter().any(|s| s["codec_type"] == "audio");
    Ok((duration, has_audio))
}

fn get_frame(req: FrameRequest) -> Result<Vec<u8>, String> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{:.3}", req.time))
        .arg("-i").arg(req.path)
        .args(["-frames:v", "1", "-vf", "scale=960:-2", "-f", "image2pipe", "-vcodec", "png", "-"])
        .output().map_err(|e| format!("Không chạy được ffmpeg: {e}"))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(format!("Không tạo được preview: {}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

fn run_checked(mut command: Command) -> Result<(), String> {
    let output = command.stdout(Stdio::null()).output().map_err(|e| e.to_string())?;
    if output.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&output.stderr).chars().take(3000).collect()) }
}

fn export_video(clips: Vec<Clip>, output: PathBuf) -> Result<PathBuf, String> {
    if clips.is_empty() { return Err("Chưa có video".into()); }
    if clips.iter().any(|c| c.length() < 0.05) { return Err("Mỗi clip phải dài ít nhất 0.05 giây".into()); }
    let temp = std::env::temp_dir().join(format!("minicut-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
    std::fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
    let result = (|| -> Result<PathBuf, String> {
        let mut parts = Vec::new();
        for (i, clip) in clips.iter().enumerate() {
            let part = temp.join(format!("part_{i:04}.mp4"));
            let mut cmd = Command::new("ffmpeg");
            cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-ss"])
                .arg(format!("{:.3}", clip.start))
                .arg("-t").arg(format!("{:.3}", clip.length()))
                .arg("-i").arg(&clip.path);
            if !clip.has_audio {
                cmd.args(["-f", "lavfi", "-i", "anullsrc=channel_layout=stereo:sample_rate=48000"]);
            }
            cmd.args(["-map", "0:v:0"]);
            if clip.has_audio { cmd.args(["-map", "0:a:0"]); }
            else { cmd.args(["-map", "1:a:0"]); }
            cmd.args(["-vf", "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30,format=yuv420p",
                "-c:v", "libx264", "-preset", "veryfast", "-crf", "21",
                "-c:a", "aac", "-ar", "48000", "-ac", "2", "-b:a", "160k",
                "-af", "aresample=async=1:first_pts=0,apad", "-movflags", "+faststart"]);
            // Keep output length bounded even if source audio ends early.
            cmd.args(["-t", &format!("{:.3}", clip.length())]);
            cmd.arg(&part);
            run_checked(cmd).map_err(|e| format!("Lỗi xử lý clip {}: {e}", i + 1))?;
            parts.push(part);
        }
        // Concat demuxer requires matching video/audio codecs and stream parameters.
        let list = temp.join("concat.txt");
        let mut contents = String::new();
        for (i, _) in parts.iter().enumerate() {
            // Relative paths are ASCII-only and work on Windows, including paths with apostrophes.
            contents.push_str(&format!("file 'part_{i:04}.mp4'\n"));
        }
        std::fs::write(&list, contents).map_err(|e| e.to_string())?;
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i"])
            .arg(&list).args(["-c", "copy", "-movflags", "+faststart"]).arg(&output);
        run_checked(cmd)?;
        Ok(output)
    })();
    let _ = std::fs::remove_dir_all(&temp);
    result
}

impl MiniCut {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.2);
        let (frame_tx, requests) = mpsc::channel::<FrameRequest>();
        let (results, frame_rx) = mpsc::channel::<FrameResult>();
        thread::spawn(move || {
            while let Ok(mut request) = requests.recv() {
                while let Ok(newer) = requests.try_recv() { request = newer; }
                let generation = request.generation;
                let png = get_frame(request);
                if results.send(FrameResult { png, generation }).is_err() { break; }
            }
        });
        Self { clips: vec![], selected: None, playhead: 0.0, texture: None,
            frame_tx, frame_rx, last_request: Instant::now() - Duration::from_secs(1),
            needs_frame: false, generation: 0, export_rx: None, exporting: false,
            status: "Sẵn sàng. Cần cài FFmpeg và ffprobe trong PATH.".into() }
    }
    fn select(&mut self, i: usize) {
        self.selected = Some(i);
        self.playhead = self.clips[i].start;
        self.texture = None;
        self.generation += 1;
        self.needs_frame = true;
    }
    fn add_files(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("Video", &["mp4", "mov", "mkv", "webm", "avi"])
            .pick_files() {
            for path in files {
                match ffprobe(&path) {
                    Ok((duration, has_audio)) => {
                        self.clips.push(Clip { path, duration, start: 0.0, end: duration, has_audio });
                        self.status = "Đã thêm video".into();
                    }
                    Err(e) => self.status = format!("Lỗi import {}: {e}", path.display()),
                }
            }
            if self.selected.is_none() && !self.clips.is_empty() { self.select(0); }
        }
    }
    fn export(&mut self) {
        if self.exporting || self.clips.is_empty() { return; }
        if let Some(path) = rfd::FileDialog::new().set_file_name("output.mp4")
            .add_filter("MP4", &["mp4"]).save_file() {
            let path = path.with_extension("mp4");
            if self.clips.iter().any(|c| c.path == path) {
                self.status = "Không được xuất đè lên file video đầu vào!".into();
                return;
            }
            let clips = self.clips.clone();
            let (tx, rx) = mpsc::channel();
            self.export_rx = Some(rx);
            self.exporting = true;
            self.status = "Đang xuất video, vui lòng đợi...".into();
            thread::spawn(move || { let _ = tx.send(ExportResult(export_video(clips, path))); });
        }
    }
}

impl eframe::App for MiniCut {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(frame) = self.frame_rx.try_recv() {
            if frame.generation != self.generation { continue; }
            match frame.png {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                        self.texture = Some(ctx.load_texture("preview", color, egui::TextureOptions::LINEAR));
                    }
                    Err(e) => self.status = format!("Lỗi ảnh preview: {e}"),
                },
                Err(e) => self.status = e,
            }
        }
        if let Some(rx) = &self.export_rx {
            if let Ok(ExportResult(result)) = rx.try_recv() {
                self.exporting = false;
                self.status = match result {
                    Ok(path) => format!("Đã xuất: {}", path.display()),
                    Err(e) => format!("Xuất thất bại: {e}"),
                };
                self.export_rx = None;
            }
        }
        if self.needs_frame && self.last_request.elapsed() >= Duration::from_millis(180) {
            if let Some(i) = self.selected {
                let _ = self.frame_tx.send(FrameRequest {
                    path: self.clips[i].path.clone(), time: self.playhead, generation: { self.generation += 1; self.generation },
                });
                self.last_request = Instant::now();
                self.needs_frame = false;
            }
        }
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("MiniCut");
                if ui.button("+ Thêm video").clicked() { self.add_files(); }
                if ui.add_enabled(!self.exporting && !self.clips.is_empty(),
                    egui::Button::new("Xuất MP4")).clicked() { self.export(); }
                if self.exporting { ui.spinner(); }
            });
        });
        egui::SidePanel::left("clips").min_width(260.0).show(ctx, |ui| {
            ui.heading("Danh sách clip (thứ tự ghép)");
            let mut action: Option<(usize, &str)> = None;
            for i in 0..self.clips.len() {
                ui.horizontal(|ui| {
                    let name = self.clips[i].path.file_name().unwrap_or_default().to_string_lossy();
                    if ui.selectable_label(self.selected == Some(i),
                        format!("{}. {} ({:.1}s)", i + 1, name, self.clips[i].length())).clicked() {
                        action = Some((i, "select"));
                    }
                    if ui.small_button("↑").clicked() { action = Some((i, "up")); }
                    if ui.small_button("↓").clicked() { action = Some((i, "down")); }
                    if ui.small_button("✕").clicked() { action = Some((i, "delete")); }
                });
            }
            if let Some((i, action)) = action {
                match action {
                    "select" => self.select(i),
                    "up" if i > 0 => { self.clips.swap(i, i - 1); self.select(i - 1); }
                    "down" if i + 1 < self.clips.len() => { self.clips.swap(i, i + 1); self.select(i + 1); }
                    "delete" => {
                        self.clips.remove(i);
                        if self.clips.is_empty() { self.selected = None; self.texture = None; }
                        else { self.select(i.min(self.clips.len() - 1)); }
                    }
                    _ => {}
                }
            }
            ui.separator();
            let total: f64 = self.clips.iter().map(Clip::length).sum();
            ui.label(format!("Tổng thời lượng: {:.2} giây", total));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(i) = self.selected {
                ui.heading("Preview & Cắt video");
                if let Some(tex) = &self.texture {
                    let available = ui.available_width().min(960.0);
                    let scale = (available / tex.size_vec2().x).min(1.0);
                    ui.image((tex.id(), tex.size_vec2() * scale));
                } else {
                    ui.label("Đang tải khung hình preview...");
                }
                ui.separator();
                let clip = &mut self.clips[i];
                ui.label(format!("Thời điểm: {:.2}s / {:.2}s", self.playhead, clip.duration));
                if ui.add(egui::Slider::new(&mut self.playhead, 0.0..=clip.duration)
                    .text("Tua video")).changed() { self.needs_frame = true; }
                ui.horizontal(|ui| {
                    if ui.button("Đặt bắt đầu tại đây").clicked() {
                        clip.start = self.playhead.min(clip.end - 0.05).max(0.0);
                    }
                    if ui.button("Đặt kết thúc tại đây").clicked() {
                        clip.end = self.playhead.max(clip.start + 0.05).min(clip.duration);
                    }
                });
                ui.label("Chọn khoảng thời gian cần GIỮ LẠI:");
                let old_start = clip.start;
                let old_end = clip.end;
                ui.add(egui::Slider::new(&mut clip.start, 0.0..=clip.duration).text("Bắt đầu (s)"));
                ui.add(egui::Slider::new(&mut clip.end, 0.0..=clip.duration).text("Kết thúc (s)"));
                clip.start = clip.start.clamp(0.0, (clip.duration - 0.05).max(0.0));
                clip.end = clip.end.clamp(clip.start + 0.05, clip.duration);
                if (clip.start - old_start).abs() > 0.0001 || (clip.end - old_end).abs() > 0.0001 {
                    self.playhead = self.playhead.clamp(clip.start, clip.end);
                    self.needs_frame = true;
                }
                ui.label(format!("Đoạn giữ lại: {:.2}s → {:.2}s ({:.2}s)", clip.start, clip.end, clip.length()));
                if ui.button("Xem khung hình đầu đoạn").clicked() {
                    self.playhead = clip.start; self.needs_frame = true;
                }
            } else {
                ui.centered_and_justified(|ui| { ui.label("Bấm '+ Thêm video' để bắt đầu."); });
            }
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 740.0]),
        ..Default::default()
    };
    eframe::run_native("MiniCut - Rust Video Editor", options,
        Box::new(|cc| Ok(Box::new(MiniCut::new(cc)))))
}
