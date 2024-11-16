use rusqlite::{Connection, Result};

pub async fn db()->Result<Connection>{
    let conn = Connection::open(
        format!(
            "{}.sqlite",
            std::env::var("BALLCHASING_GROUP").expect("BALLCHASING_GROUP not set")
        )
    )?;
    Ok(conn)
}