// แบบฝึกหัดตอนที่ 2: โปรแกรมแรก (ฟังก์ชัน)
// โจทย์: เขียนฟังก์ชัน greeting คืนค่าคำทักทาย

pub fn greeting(name: &str) -> String {
    todo!("คืนค่า format!(\"Hello, {}!\", name)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(greeting("Rust"), "Hello, Rust!");
    }

    #[test]
    fn test_empty() {
        assert_eq!(greeting(""), "Hello, !");
    }
}
