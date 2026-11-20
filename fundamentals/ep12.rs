// ตอนที่ 12: การจัดการข้อผิดพลาด: Result

// ตัวอย่าง Result
let r: Result<i32, String> = Ok(42);
match r {
    Ok(v) => println!("{}", v),
    Err(e) => println!("{}", e),
}

// ตัวอย่างตัวดำเนินการ ?
fn read() -> Result<String, Error> {
    let s = fs::read_to_string("f.txt")?;
    Ok(s)
}
