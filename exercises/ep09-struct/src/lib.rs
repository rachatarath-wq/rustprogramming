// แบบฝึกหัดตอนที่ 9: struct + method
// โจทย์: เติม method area() ให้คืนค่าพื้นที่ของ Rectangle

pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn area(&self) -> u32 {
        todo!("คืนค่า self.width * self.height")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let r = Rectangle { width: 4, height: 5 };
        assert_eq!(r.area(), 20);
    }

    #[test]
    fn test_square() {
        let r = Rectangle { width: 7, height: 7 };
        assert_eq!(r.area(), 49);
    }
}
