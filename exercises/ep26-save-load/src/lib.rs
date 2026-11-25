// แบบฝึกหัดตอนที่ 26: บันทึก / โหลดโลก
// โจทย์: แปลงโลก (Vec<Block>) เป็นข้อความบันทึกได้ และอ่านกลับ

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u8,
}

pub fn save_world(blocks: &[Block]) -> String {
    todo!("แต่ละ block เป็นบรรทัด \"x,y,z,block\" ต่อด้วย \\n")
}

pub fn load_world(s: &str) -> Vec<Block> {
    todo!("parse กลับเป็น Vec<Block> (ข้ามบรรทัดว่าง)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let world = vec![
            Block { x: 0, y: 0, z: 0, block: 1 },
            Block { x: 1, y: 2, z: 3, block: 5 },
        ];
        assert_eq!(load_world(&save_world(&world)), world);
    }

    #[test]
    fn test_empty() {
        assert!(load_world("").is_empty());
    }
}
