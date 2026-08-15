@echo off
rem เปิด server + ลิงก์ HTTPS ชั่วคราวสำหรับทดสอบบนมือถือ (ดับเบิลคลิกไฟล์นี้)
cd /d %~dp0
if not exist target\release\teedet-server.exe (
  echo ยังไม่ได้ build server: รัน  cargo build --release -p teedet-server  ก่อน
  pause
  exit /b 1
)
start "teedet-server" target\release\teedet-server.exe
timeout /t 2 >nul
echo กำลังเปิดลิงก์ HTTPS ... รอสักครู่ แล้วเปิดลิงก์ที่ขึ้นว่า https://xxxx.trycloudflare.com บนมือถือ
"C:\Program Files (x86)\cloudflared\cloudflared.exe" tunnel --url http://localhost:8787 --no-autoupdate
