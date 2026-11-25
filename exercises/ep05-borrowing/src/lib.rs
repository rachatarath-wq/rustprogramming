// แบบฝึกหัดตอนที่ 5: Borrowing และ References
// โจทย์: นับจำนวนคำ (คั่นด้วยช่องว่าง) โดยยืม &str ไม่ย้าย ownership

pub fn word_count(s: &str) -> usize {
    // แทนที่ todo!() ด้วยโค้ดของคุณ
    todo!("ใช้ s.split_whitespace().count()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_words() {
        assert_eq!(word_count("hello world"), 2);
    }

    #[test]
    fn test_three_words() {
        assert_eq!(word_count("rust programming language"), 3);
    }

    #[test]
    fn test_empty() {
        assert_eq!(word_count(""), 0);
    }
}
