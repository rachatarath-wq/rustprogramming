// แบบฝึกหัดตอนที่ 52: ระบบเหรียญ
// โจทย์: ทุบบล็อกได้เหรียญ — เขียนฟังก์ชันบวกเหรียญตามจำนวนบล็อกที่ทุบ

pub fn break_blocks(coins: i64, count: i64) -> i64 {
    todo!("คืนค่า coins + count (ทุบ 1 บล็อก = +1 เหรียญ)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_break() {
        assert_eq!(break_blocks(0, 1), 1);
    }

    #[test]
    fn test_accumulate() {
        assert_eq!(break_blocks(5, 3), 8);
    }

    #[test]
    fn test_no_break() {
        assert_eq!(break_blocks(10, 0), 10);
    }
}
