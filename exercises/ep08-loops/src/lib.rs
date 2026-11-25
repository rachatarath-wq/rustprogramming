// แบบฝึกหัดตอนที่ 8: ลูป (loop / while / for)
// โจทย์: เขียนฟังก์ชัน sum_to คืนค่าผลรวม 1 + 2 + ... + n (ใช้ for)

pub fn sum_to(n: u32) -> u32 {
    todo!("ใช้ for วนบวก 1..=n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_five() {
        assert_eq!(sum_to(5), 15);
    }

    #[test]
    fn test_one() {
        assert_eq!(sum_to(1), 1);
    }

    #[test]
    fn test_zero() {
        assert_eq!(sum_to(0), 0);
    }
}
