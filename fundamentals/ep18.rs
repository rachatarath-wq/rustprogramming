// ตอนที่ 18: export JSON + test + สรุปคอร์ส

// ตัวอย่าง export
let json = serde_json::to_string(&tasks)?;
fs::write("backup.json", json)?;
