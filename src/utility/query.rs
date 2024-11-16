use rusqlite::{Connection, Result};

pub async fn db()->Result<Connection>{
    let conn = Connection::open(format!("{}.sqlite", env!("BALLCHASING_GROUP")))?;
    Ok(conn)
}