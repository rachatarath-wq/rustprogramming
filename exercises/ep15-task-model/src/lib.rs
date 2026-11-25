// แบบฝึกหัดตอนที่ 15: ออกแบบ data model (Task)
// โจทย์: เขียนฟังก์ชัน new_task สร้าง Task ใหม่ (ยังไม่เสร็จ = done: false)

#[derive(Debug, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
}

pub fn new_task(title: &str) -> Task {
    todo!("คืนค่า Task { title: title.to_string(), done: false }")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        assert_eq!(new_task("ซื้อของ").title, "ซื้อของ");
    }

    #[test]
    fn test_default_done() {
        assert_eq!(new_task("x").done, false);
    }
}
