# แบบฝึกหัดตอนที่ 14 — serde/JSON

## โจทย์
แปลง struct `User` เป็น JSON และแปลงกลับ (serialize / deserialize) ด้วย `serde`

## ขั้นตอน
1. เปิด `src/lib.rs` (โครงสร้าง struct + `#[derive(Serialize, Deserialize)]` มีให้แล้ว)
2. แทนที่ `todo!()` ทั้ง 2 จุดด้วย `serde_json::to_string` / `serde_json::from_str`
3. รัน `cargo test` — ต้องผ่านทั้ง 2 เทส

## แนวคิดที่ได้ฝึก
- `#[derive(Serialize, Deserialize)]` = ให้ serde สร้างโค้ดแปลง struct ↔ JSON ให้อัตโนมัติ
- `serde_json::to_string` = struct → String, `serde_json::from_str` = String → struct

## ตารางตรวจคำตอบ
| การทำงาน | ผลลัพธ์ |
|----------|---------|
| to_json → from_json | ได้ struct เดิมกลับมา (round-trip) |
| from_json(`{"name":"Ann","age":30}`) | User { name: "Ann", age: 30 } |
