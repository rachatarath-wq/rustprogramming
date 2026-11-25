// แบบฝึกหัดตอนที่ 16: สร้าง CLI + add/list
// โจทย์: เขียนฟังก์ชัน add (เพิ่มงาน) และ list (คืนรายชื่อ title ทั้งหมด)

#[derive(Debug, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
}

pub fn add(tasks: &mut Vec<Task>, title: &str) {
    todo!("push Task ใหม่เข้าไป")
}

pub fn list(tasks: &[Task]) -> Vec<String> {
    todo!("คืนค่า Vec ของ title ทั้งหมด")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_list() {
        let mut tasks = Vec::new();
        add(&mut tasks, "ซื้อของ");
        add(&mut tasks, "ทำการบ้าน");
        assert_eq!(list(&tasks), vec!["ซื้อของ", "ทำการบ้าน"]);
    }

    #[test]
    fn test_empty_list() {
        let tasks: Vec<Task> = Vec::new();
        assert!(list(&tasks).is_empty());
    }
}
