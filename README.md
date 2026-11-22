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

### 6. เกมเปิดได้แต่ช้ามาก (FPS ต่ำ / เปิดเกมใช้เวลาหลายนาทีกว่าจะเข้า)
สาเหตุ: เครื่องมีการ์ดจอ (เช่น NVIDIA) แต่ driver พัง/ไม่ถูกโหลด → ระบบหลุดไป render ด้วย CPU (software rendering / llvmpipe) แทนการ์ดจอ ทำให้วาดเฟรมช้ามาก โดยเฉพาะเฟรมแรก
วิธีเช็ค: รัน `nvidia-smi` — ถ้าขึ้น error "couldn't communicate with the NVIDIA driver" แปลว่า driver พัง
วิธีแก้ (Ubuntu):
```
ubuntu-drivers devices          # ดู driver ที่แนะนำสำหรับการ์ดจอ
sudo ubuntu-drivers autoinstall # ติดตั้ง driver ที่แนะนำ
sudo reboot                     # reboot แล้วตรวจ nvidia-smi อีกครั้ง
```
หลังซ่อมเสร็จเกมจะเร็วขึ้นมาก (เข้าเกมภายในไม่กี่วินาที)

### 7. เกมค้างทั้งเครื่อง (หน้าจอ freeze ต้องกดปุ่มปิดเครื่อง / รีเซ็ตเครื่อง)
สาเหตุ: การ์ดจอ NVIDIA รุ่นเก่ามาก (GeForce 8xxx/9xxx, ประมาณปี 2008) ใช้ driver 340.xx ซึ่ง NVIDIA เลิกซัพพอร์ตแล้ว — kernel module ของ driver ค้างตอนเกมขอใช้ OpenGL 3.3 ทำให้ทั้งเครื่อง freeze (ไม่ใช่บั๊กของโค้ดเกม)
วิธีเช็ค: รัน `nvidia-smi` — ถ้าขึ้น `Driver Version: 340.xx` และการ์ดเป็น GeForce รุ่น 8/9 series แปลว่าใช่สาเหตุนี้
วิธีแก้ (ทดสอบว่าตัวเกมเองรันได้ โดยบังคับ render ด้วย CPU แทนการ์ด NVIDIA):
```
LIBGL_ALWAYS_SOFTWARE=1 ./miniroblox
```
เกมจะไม่ค้าง แต่ FPS ช้าลง (เป็นเรื่องปกติของการ render ด้วย CPU) — อาจมีเสียงสะดุดเป็นครั้งคราว (ข้อความ `ALSA underrun` / `-EPIPE`) เพราะ CPU ยุ่งกับการ render ทำให้เสียงตามไม่ทัน เกมยังเล่นได้ ไม่ใช่ error ร้ายแรง
วิธีแก้ถาวร (เลือกอย่างใดอย่างหนึ่ง):
- เล่น/ทดสอบบนเครื่องที่มีการ์ดจอใหม่กว่า (รองรับ OpenGL 3.3 ขึ้นไป) — แนะนำสำหรับการเล่นจริง
- ลบ driver nvidia-340 แล้วใช้ driver โอเพนซอร์ส `nouveau` แทน (เสถียรกว่าบน kernel ใหม่ แต่ GPU เก่าขนาดนี้ก็ยังช้าอยู่ดี)
