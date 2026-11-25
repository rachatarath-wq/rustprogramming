# แบบฝึกหัดตอนที่ 15 — ออกแบบ data model (Task)

## โจทย์
เขียนฟังก์ชัน `new_task` ให้สร้าง `Task` ใหม่ โดย `done` เริ่มต้นเป็น `false`

## ขั้นตอน
1. เปิด `src/lib.rs` (struct `Task` มีให้แล้ว)
2. แทนที่ `todo!()` ด้วยโค้ดที่คืนค่า `Task { title: title.to_string(), done: false }`
3. รัน `cargo test` — ต้องผ่านทั้ง 2 เทส

## แนวคิดที่ได้ฝึก
- `struct` = เก็บข้อมูลหลายช่อง (title + done)
- ค่า default ของงานใหม่ = ยังไม่เสร็จ (`done: false`)

## ตารางตรวจคำตอบ
| new_task(...)  | title      | done  |
|----------------|------------|-------|
| new_task("ซื้อของ") | "ซื้อของ" | false |
| new_task("x")  | "x"        | false |
