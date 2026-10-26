# MiniCut — ứng dụng cắt/ghép video cho Windows 11

Rust + egui (eframe) + FFmpeg. Có preview khung hình khi kéo thanh thời gian, chọn khoảng cắt, xếp thứ tự và xuất MP4. **Preview là khung hình tĩnh, không phát liên tục hoặc có âm thanh.**

## Cài trên Windows 11

1. Cài [Rust](https://rustup.rs/) bằng rustup-init.exe. Khi được hỏi, cài thêm **Visual Studio C++ Build Tools** với workload **Desktop development with C++** (MSVC và Windows SDK). Đóng/mở lại PowerShell.
2. Cài **FFmpeg** gồm `ffmpeg.exe` và `ffprobe.exe`. Có thể chạy PowerShell: `winget install --id Gyan.FFmpeg -e` (nếu winget tìm thấy gói); sau đó đóng/mở lại PowerShell. Hoặc tải bản build FFmpeg từ https://ffmpeg.org/download.html#build-windows rồi thêm thư mục `bin` chứa hai file exe vào biến môi trường `PATH`.
3. Kiểm tra trong PowerShell:

   ```powershell
   cargo --version
   ffmpeg -version
   ffprobe -version
   ```

4. Giải nén ZIP, mở PowerShell tại thư mục có `Cargo.toml`, chạy:

   ```powershell
   cargo run --release
   ```

5. Sau khi build, file chạy ở `target\release\minicut.exe`. **Máy chạy ứng dụng vẫn cần FFmpeg và ffprobe trong PATH.**

## Cách dùng

- **+ Thêm video:** Chọn nhiều file MP4/MOV/MKV/WebM/AVI.
- **Danh sách clip:** Click chọn, bấm ↑ / ↓ để đổi thứ tự, ✕ để bỏ clip.
- **Tua video:** Kéo thanh tua để xem khung hình tại vị trí chọn.
- **Đặt bắt đầu/kết thúc tại đây:** Gán mốc cắt từ thanh tua. Có thể chỉnh hai thanh trượt bên dưới.
- **Xuất MP4:** Chọn tên file đầu ra; app sẽ lần lượt chuẩn hóa từng đoạn rồi ghép thành MP4.

## Giới hạn và lưu ý

- Xuất 1280×720, 30 fps, H.264/AAC, stereo 48 kHz; video khác tỉ lệ được thêm viền đen.
- Không có audio playback, transition, nhiều track, undo hay phần trăm tiến độ.
- Preview chỉ là khung hình tĩnh tại thời điểm tua; khi đang xử lý có thể mất một chút thời gian.
- Export re-encode tất cả các clip (có thể chậm với video dài). Cắt gần đúng theo frame ở 30fps.
- File không có âm thanh được bổ sung âm thanh im lặng; file có âm thanh quá ngắn sẽ được nối thêm im lặng.
- **Không được chọn tên output trùng với file input.**
- Cần đủ dung lượng trống cho các file tạm trong `%TEMP%`.
- Mã nguồn chưa được compile/test tự động tại môi trường tạo ZIP (không có Rust/Cargo). Nếu gặp lỗi, chạy `cargo build --release` và xem thông báo compiler.

## Build EXE tự động bằng GitHub Actions

1. Mở GitHub Desktop > File > Add local repository (nếu chưa là repository, chọn Create a repository).
2. Chọn thư mục `minicut-rust` (thư mục có `Cargo.toml` và `.github`).
3. Commit lên nhánh `main`, chọn Publish repository. Có thể chọn **Keep this code private**.
4. Vào repository trên github.com > Actions > Build MiniCut Windows > Run workflow, hoặc push lên `main` để tự build.
5. Khi workflow thành công, mở run > Artifacts > tải `MiniCut-Windows-11`, giải nén và chạy `minicut.exe`.

**Lưu ý:** `minicut.exe` gọi `ffmpeg` và `ffprobe` từ PATH của Windows; cần giữ FFmpeg đã cài trên máy. Artifact không chứa FFmpeg. Chưa kiểm chứng workflow thực tế trên GitHub.

## Giao diện sáng màu ghi (V2)

- Nền xám nhạt, thẻ trắng, thanh công cụ gọn.
- Tự tải Segoe UI từ `C:\Windows\Fonts\segoeui.ttf` để hiển thị tiếng Việt có dấu trên Windows 11.
- Preview là ảnh tĩnh tại vị trí thanh tua, không phải phát video liên tục.
- Cắt và ghép vẫn dùng FFmpeg. GitHub Actions tự build EXE khi push lên main.
