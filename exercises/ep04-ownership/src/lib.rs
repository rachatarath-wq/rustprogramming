// แบบฝึกหัดตอนที่ 4: Ownership
// โจทย์: รับ String เข้ามา (ย้าย ownership) เติม " world" ต่อท้าย แล้วคืนค่ากลับ

pub fn append_world(mut s: String) -> String {
    // แทนที่ todo!() ด้วยโค้ดของคุณ
    todo!("ใช้ s.push_str(\" world\") แล้วคืนค่า s")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append() {
        assert_eq!(append_world(String::from("hello")), "hello world");
    }

    #[test]
    fn test_empty() {
        assert_eq!(append_world(String::from("")), " world");
    }
}
