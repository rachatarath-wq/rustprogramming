// แบบฝึกหัดตอนที่ 10: generics + trait
// โจทย์: เขียนฟังก์ชัน generic largest คืนค่าที่มากกว่า

pub fn largest<T: PartialOrd>(a: T, b: T) -> T {
    todo!("คืนค่าที่มากกว่า (ใช้ if a > b)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        assert_eq!(largest(3, 7), 7);
    }

    #[test]
    fn test_str() {
        assert_eq!(largest("a", "z"), "z");
    }
}
