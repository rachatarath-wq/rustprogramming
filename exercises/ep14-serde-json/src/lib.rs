// แบบฝึกหัดตอนที่ 14: serde/JSON
// โจทย์: แปลง struct User เป็น JSON และแปลงกลับ (ใช้ serde)

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub name: String,
    pub age: u32,
}

pub fn to_json(user: &User) -> String {
    todo!("ใช้ serde_json::to_string(user).unwrap()")
}

pub fn from_json(s: &str) -> User {
    todo!("ใช้ serde_json::from_str(s).unwrap()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let u = User { name: "สมชาย".to_string(), age: 20 };
        assert_eq!(from_json(&to_json(&u)), u);
    }

    #[test]
    fn test_parse() {
        let u = from_json(r#"{"name":"Ann","age":30}"#);
        assert_eq!(u.name, "Ann");
        assert_eq!(u.age, 30);
    }
}
