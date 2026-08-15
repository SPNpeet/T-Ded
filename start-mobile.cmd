@echo off
title ทีเด็ดปลาน้ำจืด - เซิร์ฟเวอร์ + ลิงก์มือถือ
cd /d %~dp0
if not exist target\release\teedet-server.exe (
  echo ยังไม่ได้ build: รัน  cargo build --release -p teedet-server  ก่อน
  pause
  exit /b 1
)
rem ปิดตัวเก่าที่ค้างอยู่ ป้องกันพอร์ตชนกัน
taskkill /f /im teedet-server.exe >nul 2>&1
taskkill /f /im cloudflared.exe >nul 2>&1
start "teedet-server" /min cmd /c "cd /d %~dp0 && :loop && target\release\teedet-server.exe && timeout /t 3 >nul && goto loop"
timeout /t 3 >nul
echo.
echo กำลังเปิดลิงก์ HTTPS สำหรับมือถือ (ถ้าหลุดจะต่อใหม่เองอัตโนมัติ)
echo เปิดลิงก์ที่ขึ้นว่า https://xxxx.trycloudflare.com บนมือถือได้เลย
echo.
:tunnel
"C:\Program Files (x86)\cloudflared\cloudflared.exe" tunnel --url http://localhost:8787 --no-autoupdate
echo ลิงก์หลุด กำลังต่อใหม่...
timeout /t 3 >nul
goto tunnel
