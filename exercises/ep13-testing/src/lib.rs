// แบบฝึกหัดตอนที่ 13: การเขียน test
// โจทย์: ฟังก์ชัน add ถูกต้องแล้ว — หน้าที่ของคุณคือเขียนเทสให้ครบ

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        todo!("เขียน assert_eq!(add(2, 3), 5)");
    }

    #[test]
    fn test_add_negative() {
        todo!("เขียน assert ที่ตรวจ add(-1, 1) == 0");
    }
}
