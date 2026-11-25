// แบบฝึกหัดตอนที่ 43: โปรโตคอล + serde/bincode
// โจทย์: เข้ารหัส (encode) และถอดรหัส (decode) ข้อความ Message เป็น bytes

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    Chat { name: String, text: String },
    Move { x: f32, y: f32, z: f32 },
}

pub fn encode(msg: &Message) -> Vec<u8> {
    todo!("ใช้ bincode::serialize(msg).unwrap()")
}

pub fn decode(bytes: &[u8]) -> Message {
    todo!("ใช้ bincode::deserialize(bytes).unwrap()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_round_trip() {
        let m = Message::Chat { name: "สมชาย".to_string(), text: "สวัสดี".to_string() };
        assert_eq!(decode(&encode(&m)), m);
    }

    #[test]
    fn test_move_round_trip() {
        let m = Message::Move { x: 1.5, y: 2.0, z: -3.5 };
        assert_eq!(decode(&encode(&m)), m);
    }
}
