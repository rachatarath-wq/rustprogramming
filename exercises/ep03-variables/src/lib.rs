// แบบฝึกหัดตอนที่ 3: ตัวแปรและประเภทข้อมูล
// โจทย์: คืนค่าพื้นที่สี่เหลี่ยม = กว้าง × ยาว

pub fn area_of_rectangle(width: u32, height: u32) -> u32 {
    // แทนที่ todo!() ด้วยโค้ดของคุณ
    todo!("คืนค่า width * height")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(area_of_rectangle(4, 5), 20);
    }

    #[test]
    fn test_square() {
        assert_eq!(area_of_rectangle(7, 7), 49);
    }

    #[test]
    fn test_zero() {
        assert_eq!(area_of_rectangle(0, 100), 0);
    }
}
