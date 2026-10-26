# MiniCut V4 — Windows 11

Next.js 15 + Tauri 2 + Rust + FFmpeg. Giao diện ghi sáng, timeline tách clip bằng Ctrl+B, kéo hai mép clip để trim, Delete xóa, Ctrl+Z hoàn tác, Space phát/dừng. Các đoạn còn lại được ghép nối tiếp khi xuất.

## Build bằng GitHub Desktop

1. Copy **nội dung** thư mục này vào repository hiện tại (không xóa `.git`).
2. Xóa các file egui cũ nếu còn. Commit và Push nhánh `main`.
3. GitHub → Actions → Build MiniCut V3 Windows → tải artifact EXE hoặc installer sau khi build xanh.

## Lưu ý

- FFmpeg và ffprobe được tìm trong PATH hoặc chọn thủ công ở Cài đặt.
- Preview video được FFmpeg chuyển sang H.264 trước khi thêm; file dài có thể mất thời gian, dù xử lý chạy nền.
- Xuất MP4 1280×720, 30 FPS. Mỗi đoạn được mã hóa lại trước khi ghép.
- Trình phát preview phụ thuộc WebView2 và codec.
- Ctrl+B chia clip ở vị trí con trỏ timeline. Kéo mép clip thay đổi đoạn giữ lại; Undo lưu 40 bước.
- Dự án chưa được kiểm chứng build trên Windows trong môi trường tạo file; nếu Actions báo lỗi hãy gửi build log.
