# แบบฝึกหัดตอนที่ 18 — export JSON + tests

## โจทย์
เขียนฟังก์ชัน `export_json` ให้แปลง `Vec<Task>` เป็นสตริง JSON (ใช้ serde)

## ขั้นตอน
1. เปิด `src/lib.rs`
2. แทนที่ `todo!()` ด้วย `serde_json::to_string(tasks).unwrap()`
3. รัน `cargo test` — ต้องผ่านทั้ง 2 เทส

## แนวคิดที่ได้ฝึก
- `serde_json::to_string` รับ slice `&[Task]` แล้วคืน String แบบ JSON
- `#[derive(Serialize)]` = serde สร้างโค้ด serialize ให้อัตโนมัติ

## ตารางตรวจคำตอบ
| input | ผลลัพธ์ |
|-------|---------|
| รายการงาน 2 ชิ้น | JSON ที่ parse กลับมาได้เท่าเดิม |
| รายการว่าง | "[]" |
