// ตอนที่ 17: คำสั่ง done / edit / delete

// ตัวอย่าง UPDATE
conn.execute(
    "UPDATE tasks SET done = 1 WHERE id = ?1",
    [task_id],
)?;

// ตัวอย่าง DELETE
conn.execute(
    "DELETE FROM tasks WHERE id = ?1",
    [task_id],
)?;
