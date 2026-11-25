// แบบฝึกหัดตอนที่ 7: if/else และ match
// โจทย์: เขียนฟังก์ชัน grade คืนค่าเกรดตามคะแนน (ใช้ match)

pub fn grade(score: u32) -> &'static str {
    todo!("ใช้ match: >=80=\"A\", >=70=\"B\", >=60=\"C\", อื่น=\"F\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a() {
        assert_eq!(grade(85), "A");
    }

    #[test]
    fn test_b() {
        assert_eq!(grade(75), "B");
    }

    #[test]
    fn test_c() {
        assert_eq!(grade(65), "C");
    }

    #[test]
    fn test_f() {
        assert_eq!(grade(40), "F");
    }
}
