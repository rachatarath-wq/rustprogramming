// แบบฝึกหัดตอนที่ 12: error handling (Result)
// โจทย์: เขียนฟังก์ชัน divide คืนค่า Result (Err เมื่อหารด้วยศูนย์)

pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    todo!("ถ้า b == 0 คืน Err, ไม่งั้นคืน Ok(a / b)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal() {
        assert_eq!(divide(10, 2), Ok(5));
    }

    #[test]
    fn test_negative() {
        assert_eq!(divide(-6, 3), Ok(-2));
    }

    #[test]
    fn test_by_zero() {
        assert!(divide(5, 0).is_err());
    }
}
