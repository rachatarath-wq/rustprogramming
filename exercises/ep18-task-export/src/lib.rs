// แบบฝึกหัดตอนที่ 18: export JSON + tests
// โจทย์: เขียนฟังก์ชัน export_json แปลง Vec<Task> เป็นสตริง JSON

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
}

pub fn export_json(tasks: &[Task]) -> String {
    todo!("ใช้ serde_json::to_string(tasks).unwrap()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() {
        let tasks = vec![
            Task { title: "ซื้อของ".to_string(), done: true },
            Task { title: "อ่านหนังสือ".to_string(), done: false },
        ];
        let json = export_json(&tasks);
        let back: Vec<Task> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, tasks);
    }

    #[test]
    fn test_export_empty() {
        let tasks: Vec<Task> = Vec::new();
        assert_eq!(export_json(&tasks), "[]");
    }
}
