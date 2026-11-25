// แบบฝึกหัดตอนที่ 17: done / edit / delete
// โจทย์: เขียนฟังก์ชัน mark_done ทำเครื่องหมายงานที่ title ตรงกันว่าเสร็จแล้ว

#[derive(Debug, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
}

pub fn mark_done(tasks: &mut Vec<Task>, title: &str) -> bool {
    todo!("หา task ที่ title ตรง แล้ว set done=true คืน true; ถ้าไม่เจอคืน false")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_existing() {
        let mut tasks = vec![
            Task { title: "ซื้อของ".to_string(), done: false },
            Task { title: "อ่านหนังสือ".to_string(), done: false },
        ];
        assert_eq!(mark_done(&mut tasks, "ซื้อของ"), true);
        assert_eq!(tasks[0].done, true);
        assert_eq!(tasks[1].done, false);
    }

    #[test]
    fn test_mark_missing() {
        let mut tasks = vec![Task { title: "x".to_string(), done: false }];
        assert_eq!(mark_done(&mut tasks, "ไม่มีงานนี้"), false);
    }
}
