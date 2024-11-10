use rusqlite::{params, Connection, Result};
use serde_json;
use rand::Rng;
use crate::utility;

pub async fn init() -> Result<()> {

    let db = utility::query::db().await?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS matches (
            match_id INTEGER NOT NULL,
            team1_id INTEGER NOT NULL,
            team2_id INTEGER NOT NULL,
            match_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            match_status INTEGER,
            ballchasing_id TEXT,
            PRIMARY KEY (match_id)
        );",
        params![]
    )?;

    Ok(())
}

pub async fn has_id(match_id: i32) -> Result<bool> {

    let db = utility::query::db().await?;

    let mut query = db.prepare("SELECT COUNT(*) FROM matches WHERE match_id = ?")?;

    let count: i32 = query.query_row(params![match_id], |row| row.get(0))?;

    Ok(count > 0)
}

pub async fn create(team1_id: u64, team2_id: u64) -> Result<i32> {

    let mut num: i32;
    let db = utility::query::db().await?;

    loop {
        num = rand::thread_rng().gen_range(10000..99999);

        if !has_id(num).await? {
            break;
        }
    }

    let mut query = db.prepare(
        "INSERT INTO matches (match_id, team1_id, team2_id, match_status)
        VALUES (?, ?, ?, ?)"
    )?;

    query.execute(params![num, team1_id, team2_id, 0])?;

    Ok(num)
}

pub async fn set_ballchasing_id(match_id: i32, ballchasing_id: &str) -> Result<()> {

    let db = utility::query::db().await?;

    db.execute("
        UPDATE matches
        SET ballchasing_id = ?
        WHERE match_id = ?",
        params![ballchasing_id, match_id]
    )?;

    Ok(())
}

pub async fn get_ballchasing_id(match_id: i32) -> Result<String> {

    let db = utility::query::db().await?;

    let mut query = db.prepare("
        SELECT ballchasing_id
        FROM matches
        WHERE match_id = ?")?;

    query.query_row(params![match_id], |row| Ok(row.get(0)) )?
}

pub async fn get_teams(match_id: i32) -> Result<(u64, u64)> {

    let db = utility::query::db().await?;

    let mut query = db.prepare(
        "SELECT team1_id, team2_id FROM matches WHERE match_id = ?"
    )?;

    query.query_row(params![match_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
}