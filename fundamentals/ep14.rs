// ตอนที่ 14: ข้อมูลจริง: JSON / ไฟล์ / SQLite

// ตัวอย่าง serde
#[derive(Serialize, Deserialize)]
struct Task {
    title: String,
    done: bool,
}
