# sku media Split Merge V6 — Merge trên timeline

Giữ nguyên giao diện V5 và Split (Ctrl+B), bổ sung kéo-thả đổi thứ tự clip ở danh sách bên trái hoặc timeline. Khi xuất, toàn bộ clip trên timeline được ghép theo thứ tự hiện tại thành MP4 480p / 30fps, H.264 + AAC. Clip khác độ phân giải và không có âm thanh được chuẩn hóa bằng FFmpeg trước khi ghép.

## Cách sử dụng
1. Bấm **Thêm video** để nhập ít nhất hai video.
2. Kéo một clip lên clip khác trong danh sách hoặc timeline để thay đổi thứ tự ghép. Có thể dùng nút mũi tên lên/xuống như trước.
3. Dùng **Ctrl+B** để tách, **Delete** để xóa, kéo mép để trim.
4. Bấm **Ghép N đoạn · Xuất MP4** và chọn đường dẫn. Kết quả là một file MP4 duy nhất.
5. **Ctrl+Z** hoàn tác đổi thứ tự.

## Build trên Windows 11
Giải nén ZIP, sao chép **nội dung** thư mục vào repository GitHub Desktop hiện tại (không xóa `.git`), commit và Push origin. Workflow `.github/workflows/build-windows.yml` sẽ build bản Windows.

**Lưu ý:** Chưa kiểm thử runtime Windows trong môi trường này. Import vẫn tạo proxy preview, vì vậy video dài có thể import chậm. Export chuẩn hóa và mã hóa từng đoạn, nên ghép video dài có thể mất thời gian.
