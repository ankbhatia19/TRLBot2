use rusqlite::{params, Connection, Result};

pub async fn db()->Result<Connection>{
    let conn = Connection::open("trl.sqlite")?;
    Ok(conn)
}