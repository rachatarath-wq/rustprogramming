// แบบฝึกหัดตอนที่ 11: modules
// โจทย์: เติมฟังก์ชัน square ใน module math ให้คืนค่า x ยกกำลังสอง

pub mod math {
    pub fn square(x: i32) -> i32 {
        todo!("คืนค่า x * x")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive() {
        assert_eq!(math::square(4), 16);
    }

    #[test]
    fn test_negative() {
        assert_eq!(math::square(-3), 9);
    }
}
