// ตอนที่ 15: ออกแบบแอป Task Manager

// โครงสร้างข้อมูล Task
struct Task {
    id: i64,
    title: String,
    due: Option<String>,
    done: bool,
}
