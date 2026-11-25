# แบบฝึกหัดตอนที่ 16 — สร้าง CLI + add/list

## โจทย์
เขียน 2 ฟังก์ชัน: `add` (เพิ่มงานใหม่) และ `list` (คืนรายชื่อ title ทั้งหมด)

## ขั้นตอน
1. เปิด `src/lib.rs`
2. แทนที่ `todo!()` ทั้ง 2 จุด
   - `add`: `tasks.push(Task { title: title.to_string(), done: false })`
   - `list`: `tasks.iter().map(|t| t.title.clone()).collect()`
3. รัน `cargo test` — ต้องผ่านทั้ง 2 เทส

## แนวคิดที่ได้ฝึก
- `&mut Vec<Task>` = ส่ง reference แบบแก้ไขได้ (push เพิ่มของ)
- `iter().map(...).collect()` = แปลงแต่ละ element แล้วเก็บกลับเป็น Vec

## ตารางตรวจคำตอบ
| ทำ | list(...) ที่ได้ |
|----|------------------|
| add "ซื้อของ", add "ทำการบ้าน" | ["ซื้อของ", "ทำการบ้าน"] |
| (ว่าง) | [] (empty) |
