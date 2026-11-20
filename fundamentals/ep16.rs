// ตอนที่ 16: สร้าง CLI + คำสั่ง add/list

// โครงสร้าง CLI
#[derive(Parser)]
enum Cmd {
    Add { title: String },
    List,
}

// คำสั่ง list
for task in tasks {
    println!("{} {}", task.id, task.title);
}
