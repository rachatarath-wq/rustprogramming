# เรียนเขียนโปรแกรมภาษา Rust

ซีรีส์สอนเขียนโปรแกรมภาษา Rust สำหรับผู้เริ่มต้น โดย รร.ศูนย์อบรมคอมพิวเตอร์ ชลบุรี

## โครงสร้างโฟลเดอร์
- `fundamentals/` — โค้ดตัวอย่างตอนที่ 1-18 (พื้นฐานภาษา Rust)
- `game/` — โปรเจกต์เกม Mini Roblox (ตอนที่ 19+) สร้างด้วย Macroquad

## วิธีใช้งาน (ดูคลิปแล้วทำตาม)
1. ติดตั้ง Rust: https://rustup.rs
2. โคลนโปรเจกต์:
   ```
   git clone https://github.com/rachatarath-wq/rustprogramming.git
   ```
3. เปิดวิดีโอตอนที่ต้องการ แล้วทำตามโค้ดในโฟลเดอร์ที่ตรงกับตอน

## ลิงก์
- YouTube playlist: https://www.youtube.com/playlist?list=PLVjdx7mvLqSk

## รันเกม Mini Roblox
```
cd game
cargo run
```

การควบคุม: WASD เดิน, Space กระโดด, เมาส์มองรอบ, คลิกซ้ายทุบบล็อก, คลิกขวาวางบล็อก, ปุ่ม 1-8 เลือกชนิดบล็อก, R บันทึกโลก

## ปัญหาและการแก้ไข (Troubleshooting)

### 1. รัน ./miniroblox แล้วขึ้น `GLIBC_2.xx not found`
สาเหตุ: ไฟล์ binary ถูก build บนเครื่องที่ใช้ glibc ใหม่กว่าเครื่องของคุณ
วิธีแก้ (เลือกอย่างใดอย่างหนึ่ง):
- ดาวน์โหลดเวอร์ชัน static (musl) ที่รันได้กับ Linux ทุกรุ่น
- build เองจากโค้ด: `cd game && cargo build --release`
- อัปเกรดระบบเป็น Ubuntu 20.04 ขึ้นไป

### 2. `cargo: command not found`
สาเหตุ: ยังไม่ได้ติดตั้ง Rust
วิธีแก้:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 3. cargo build ขึ้น `error: linker 'cc' not found`
สาเหตุ: ไม่มี C compiler
วิธีแก้: `sudo apt install build-essential`

### 4. รันเกมแล้วขึ้น `error while loading shared libraries` หรือ `LibraryNotFound (libX11.so.6)`
สาเหตุ: ขาดไลบรารีระบบสำหรับกราฟิก/เสียง (เกมต้องใช้ X11 + OpenGL + ALSA ตอนรัน)
วิธีแก้:
- สำหรับรัน binary สำเร็จรูป: `sudo apt install -y libx11-6 libxi6 libxrandr2 libgl1 libasound2 libxcursor1`
- สำหรับ build เองจากโค้ด: `sudo apt install -y libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev pkg-config`
- หมายเหตุ: ต้องรันบนเครื่องที่มีหน้าจอ desktop ถ้าเป็น server ไม่มีจอจะรันไม่ได้ (ขึ้น "cannot open display")

### 5. เกมไม่เปิดหน้าต่าง (รันบน server / ไม่มีจอ)
สาเหตุ: เกมต้องการหน้าจอ + การ์ดจอ (desktop environment)
วิธีแก้: รันบนเครื่องที่มี desktop (Linux/Windows/Mac) ไม่ใช่ server แบบไม่มีจอ
