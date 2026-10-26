# MiniCut V3 — Next.js 15 + Tauri 2 + Rust + FFmpeg

MiniCut là ứng dụng Windows 11 dùng để **cắt và ghép video**, có giao diện ghi sáng bằng Next.js 15, TypeScript, Tailwind CSS và Lucide Icons.

## Tính năng

- Thêm MP4, MOV, MKV, AVI, WebM; tạo preview H.264 nhẹ để xem bằng WebView2.
- Phát/tạm dừng preview, tua video; chọn điểm đầu/cuối bằng thanh trượt hoặc vị trí tua.
- Đổi thứ tự và xóa clip, timeline ghép đơn giản.
- Xuất MP4 (1280x720, 30fps, H.264/AAC) với FFmpeg; thêm âm thanh im lặng cho video không có audio.
- Tự tìm ffmpeg.exe và ffprobe.exe trong PATH, cạnh EXE, C:\\ffmpeg\\bin, Chocolatey và một số đường dẫn phổ biến; chọn thủ công thư mục bin.
- Build EXE + installer Windows bằng GitHub Actions.

## Build trên GitHub Desktop (Windows 11)

1. Giải nén ZIP. Trong GitHub Desktop mở `Repository > Show in Explorer` của repo `sku-minicut`.
2. **Xóa source cũ** `src/`, `Cargo.toml`, `BUILD-WINDOWS.bat` (của egui). **Không xóa `.git`**.
3. Copy toàn bộ file và thư mục của bản V3 vào gốc repo: `app`, `src-tauri`, `.github`, `package.json`, `next.config.ts`, ... (cho phép ghi đè).
4. Quay lại GitHub Desktop: Commit to main -> Push origin.
5. Mở GitHub -> Actions -> `Build MiniCut V3 Windows`. Chờ dấu xanh. Download artifact `MiniCut-V3-Windows-Installer` để cài, hoặc `MiniCut-V3-Windows-EXE` để dùng EXE.
6. Máy phải có FFmpeg và ffprobe trong PATH, hoặc vào Cài đặt -> Chọn thư mục bin.

## Chạy trên máy (nếu muốn)

Cần Node.js 22, Rust stable, MSVC C++ Build Tools, WebView2, FFmpeg.

```powershell
npm install
npm run tauri dev
```

Build:

```powershell
npm run tauri build
```

## Giới hạn

- Preview được **chuyển mã H.264** bằng FFmpeg mỗi lần import, vì WebView2 không phát được mọi codec; video dài có thể cần thời gian tạo preview và thêm dung lượng cache.
- Cắt chính xác bằng re-encode (không phải stream copy); ghép clip theo thứ tự. Export chạy ở background nhưng chưa có phần trăm tiến độ/hủy.
- Xuất mặc định 720p/30fps để phù hợp máy Intel Iris Xe; đây chưa phải editor nhiều track như CapCut.
- Đường dẫn FFmpeg chọn thủ công được sử dụng trong phiên chạy hiện tại; khởi động lại sẽ tự quét PATH.
- Cần thử build và chạy thực tế trên Windows; ZIP chứa **source**, không phải EXE build sẵn.
