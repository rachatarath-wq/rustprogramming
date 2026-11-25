// แบบฝึกหัดตอนที่ 53: ร้านค้า
// โจทย์: ซื้อของ — ถ้าเหรียญพอให้หักแล้วคืนยอดใหม่ (Some) ถ้าไม่พอคืน None

pub fn buy(coins: i64, price: i64) -> Option<i64> {
    todo!("ถ้า coins >= price คืน Some(coins - price) ไม่งั้น None")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enough() {
        assert_eq!(buy(10, 3), Some(7));
    }

    #[test]
    fn test_not_enough() {
        assert_eq!(buy(2, 5), None);
    }

    #[test]
    fn test_exact() {
        assert_eq!(buy(5, 5), Some(0));
    }
}
