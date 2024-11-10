use rusqlite::{params, Result};
use serde_json::json;

use crate::utility;

pub async fn init() -> Result<()> {

    let db = utility::query::db().await?;

    // Create the table if it does not exist
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS players (
        player_id INTEGER NOT NULL,
        usernames TEXT,
        PRIMARY KEY (player_id));
        "#,
        params![],
    )?;

    Ok(())
}

pub async fn has_name(username: &str)->Result<bool>{

    let db = utility::query::db().await?;

    let mut statement = db.prepare(
        r#"
        SELECT
            COUNT(*)
        FROM
            players,
            json_each(usernames) AS username
        WHERE
            username.value = ?;
        "#
    )?;

    // Execute the query with the bound parameter and fetch the result
    let count: i32 = statement.query_row(params![username], |row| row.get(0))?;

    // Return true if count > 0, otherwise false
    Ok(count > 0)
}

pub async fn get_id(username: &str) -> Result<u64>{

    let db = utility::query::db().await?;

    let mut query = db.prepare(
        "SELECT player_id
            FROM players
            JOIN json_each(players.usernames) AS username
            WHERE username.value = ?"
    )?;

    query.query_row(params![username], |row| row.get(0) )
}


pub async fn has_id(player_id: u64)->Result<bool>{
    let db = utility::query::db().await?;

    let mut statement = db.prepare(
        r#"SELECT COUNT(*) FROM players WHERE player_id = ?;"#
    )?;

    // Execute the query with the bound parameter and fetch the result
    let count: i32 = statement.query_row(params![player_id], |row| row.get(0))?;

    // Return true if count > 0, otherwise false
    Ok(count > 0)
}

pub async fn register(player_id: u64, username: &str)->Result<bool>{
    if (has_name(username)).await?{
        return Ok(false);
    }

    let db = utility::query::db().await?;

    if !has_id(player_id).await? {
        let usernames_json = json!([username]).to_string();

        let mut query = db.prepare("
                INSERT INTO players (player_id, usernames) VALUES (?, ?)
            ")?;
        query.execute(params![player_id, usernames_json])?;
        println!("New User Created: {} ({})", player_id, username);

        Ok(true)
    } else {
        let mut query = db.prepare("
                UPDATE players
                SET usernames = json_set(usernames, '$[#]', ?)
                WHERE player_id = ?
            ")?;
        query.execute(params![username, player_id])?;
        println!("Username Added: {} ({})", player_id, username);

        Ok(true)
    }
}