# แบบฝึกหัดตอนที่ 43 — โปรโตคอล + serde/bincode

## โจทย์
เข้ารหัส `Message` (enum) เป็น bytes และถอดรหัสกลับ ด้วย `bincode`

## ขั้นตอน
1. เปิด `src/lib.rs` (enum `Message` + `#[derive(Serialize, Deserialize)]` มีให้แล้ว)
2. แทนที่ `todo!()` ทั้ง 2 จุดด้วย `bincode::serialize` / `bincode::deserialize`
3. รัน `cargo test` — ต้องผ่านทั้ง 2 เทส

## แนวคิดที่ได้ฝึก
- `bincode` = แปลง struct/enum เป็น bytes เล็ก ๆ ไว้ส่งผ่านเน็ตเวิร์ก (เล็กกว่า JSON)
- enum ที่มีข้อมูล (Chat / Move) serialize ได้อัตโนมัติด้วย serde

## ตารางตรวจคำตอบ
| encode → decode | ผลลัพธ์ |
|-----------------|---------|
| Chat { ... } | ได้ Message เดิม |
| Move { ... } | ได้ Message เดิม |
