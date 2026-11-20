// ตอนที่ 10: trait และ generics

// ตัวอย่าง generics
fn largest<T>(list: &[T]) -> &T {
    &list[0]
}

// ตัวอย่าง trait
trait Summary {
    fn summarize(&self) -> String;
}
