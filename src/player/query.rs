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

pub async fn remove(username: &str, player_id: u64)->Result<()>{
    let db = utility::query::db().await?;

    // Step 1: Find the index of the username
    let index: Option<i64> = db.query_row(
        "
        SELECT json_each.key
        FROM players, json_each(players.usernames)
        WHERE players.player_id = ? AND json_each.value = ?
        ",
        params![player_id, username],
        |row| row.get(0),
    )?; // Optional handles cases where no match is found

    if let Some(index) = index {
        // Step 2: Remove the username by index
        db.execute(
            "
            UPDATE players
            SET usernames = json_remove(usernames, '$[' || ? || ']')
            WHERE player_id = ?
            ",
            params![index, player_id],
        )?;
    }

    Ok(())
}

pub async fn get_names(player_id: u64) -> Result<Vec<String>> {
    let db = utility::query::db().await?;

    db.query_row(
        "SELECT usernames FROM players WHERE player_id = ?",
        params![player_id],
        |row| {
            let names: String = row.get(0)?;
            let names_parsed = serde_json::from_str(&names)
                .unwrap_or(vec!["None".to_string()]);
            Ok(names_parsed)
        }
    )
}

pub async fn stats_core(player_id: u64) -> Result<(i32, f64, f64, f64, f64)> {
    let db = utility::query::db().await?;

    db.query_row(
        "SELECT
            COUNT(DISTINCT match_id || '-' || game_num) AS games,
            COALESCE(AVG(goals), 0) AS avg_goals,
            COALESCE(AVG(shots), 0) AS avg_shots,
            COALESCE(AVG(assists), 0) AS avg_assists,
            COALESCE(AVG(saves), 0) AS avg_saves
        FROM
            stats
        WHERE
            player_id = ?;",
        params![player_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )
}


pub async fn stats_demos(player_id: u64) -> Result<(i32, f64, f64)> {
    let db = utility::query::db().await?;

    db.query_row(
        "SELECT
            COUNT(DISTINCT match_id || '-' || game_num) AS games,
            COALESCE(AVG(demos_inflicted), 0) AS avg_demos_inflicted,
            COALESCE(AVG(demos_taken), 0) AS avg_demos_taken
        FROM
            stats
        WHERE
            player_id = ?;",
        params![player_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        },
    )
}

pub async fn stats_boost(player_id: u64) -> Result<(i32, f64, f64, f64, f64)> {
    let db = utility::query::db().await?;

    db.query_row(
        "SELECT
            COUNT(DISTINCT match_id || '-' || game_num) AS games,
            COALESCE(AVG(avg_amount), 0) AS avg_amount,
            COALESCE(AVG(percent_zero_boost), 0) AS avg_zero_boost,
            COALESCE(AVG(percent_full_boost), 0) AS avg_full_boost,
            COALESCE(AVG(amount_overfill), 0) AS avg_overfill
        FROM
            stats
        WHERE
            player_id = ?;",
        params![player_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )
}

pub async fn stats_positioning(player_id: u64) -> Result<(i32, f64, f64, f64, f64)> {
    let db = utility::query::db().await?;

    db.query_row(
        "SELECT
            COUNT(DISTINCT match_id || '-' || game_num) AS games,
            COALESCE(AVG(percent_defensive_third), 0) AS avg_defensive_third,
            COALESCE(AVG(percent_neutral_third), 0) AS avg_neutral_third,
            COALESCE(AVG(percent_offensive_third), 0) AS avg_offensive_third,
            COALESCE(AVG(percent_closest_to_ball), 0) AS avg_closest_to_ball
        FROM
            stats
        WHERE
            player_id = ?;",
        params![player_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?
            ))
        },
    )
}