# sku media Split Merge — V5

Ứng dụng Windows 11 dùng Next.js 15 + Tauri 2 + Rust + FFmpeg.

## Thay đổi
- Dùng logo SKU do người dùng cung cấp ở header, icon EXE và icon bộ cài.
- Đổi tên cửa sổ và ứng dụng thành `sku media Split Merge`.
- Bỏ nút Thêm video trên thanh công cụ cạnh Cài đặt; giữ nút Thêm video ở thư viện bên trái.
- Preview dùng khung letterbox `object-fit: contain`, không crop video và giữ đúng tỷ lệ gốc.
- Xuất video mặc định 480p (giữ nguyên thiết lập từ bản trước).

## Build bằng GitHub Desktop
1. Copy nội dung thư mục này vào repository hiện tại, giữ nguyên `.git`.
2. Commit và Push origin.
3. Vào GitHub Actions → Build SKU Media Windows, tải artifact sau khi build xanh.

Lưu ý: bản source chưa được xác nhận chạy thực tế trên Windows. Windows có thể cache icon cũ, cần tạo shortcut mới hoặc khởi động lại Explorer.
