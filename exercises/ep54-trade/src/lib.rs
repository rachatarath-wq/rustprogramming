// แบบฝึกหัดตอนที่ 54: เทรดเหรียญ
// โจทย์: โอนเหรียญจาก sender ไป receiver — ถ้า sender มีพอคืน Ok(ยอดใหม่ทั้งคู่) ถ้าไม่พอ Err

pub fn transfer(sender: i64, receiver: i64, amount: i64) -> Result<(i64, i64), String> {
    todo!("ถ้า sender >= amount คืน Ok((sender-amount, receiver+amount)) ไม่งั้น Err")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() {
        assert_eq!(transfer(10, 0, 3), Ok((7, 3)));
    }

    #[test]
    fn test_not_enough() {
        assert!(transfer(2, 0, 5).is_err());
    }

    #[test]
    fn test_exact_amount() {
        assert_eq!(transfer(5, 2, 5), Ok((0, 7)));
    }
}
