// แบบฝึกหัดตอนที่ 6: ฟังก์ชัน
// โจทย์: เขียนฟังก์ชัน is_even คืนค่า true ถ้า n เป็นเลขคู่

pub fn is_even(n: i32) -> bool {
    todo!("คืนค่า (n % 2 == 0)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even() {
        assert_eq!(is_even(4), true);
    }

    #[test]
    fn test_odd() {
        assert_eq!(is_even(7), false);
    }

    #[test]
    fn test_zero() {
        assert_eq!(is_even(0), true);
    }
}
