use rusqlite::{params, Connection, Result};
use serde_json;
use crate::utility;

pub async fn init() -> Result<()> {

    let db = utility::query::db().await?;

    let mut query = db.prepare("CREATE TABLE IF NOT EXISTS teams (
        team_id INTEGER NOT NULL,
        player1_id INTEGER,
        player2_id INTEGER,
        player3_id INTEGER,
        PRIMARY KEY(team_id)
    );")?;

    query.execute(params![])?;

    Ok(())
}

pub async fn get_team(player_id: u64) -> Result<u64> {

    let db = utility::query::db().await?;

    let mut query = db.prepare(
        "SELECT team_id
            FROM teams
            WHERE player1_id = ? OR player2_id = ? OR player3_id = ?;"
    )?;

    query.query_row(params![player_id, player_id, player_id], |row| row.get(0) )
}

pub async fn get_players(team_id: u64) -> Result<Vec<u64>> {
    let db = utility::query::db().await?;

    let mut players: Vec<u64> = vec![];

    db.query_row(
        "SELECT player1_id, player2_id, player3_id FROM teams WHERE team_id = ?",
        params![team_id],
        |row| {
            let player1_id: u64 = row.get(0).unwrap_or_default();
            let player2_id: u64 = row.get(1).unwrap_or_default();
            let player3_id: u64 = row.get(2).unwrap_or_default();

            if player1_id != 0 { players.push(player1_id); }
            if player2_id != 0 { players.push(player2_id); }
            if player3_id != 0 { players.push(player3_id); }

            Ok(players)
        }
    )

}

pub async fn add(team_id: u64, player_id: u64) -> Result<bool> {

    let db = utility::query::db().await?;

    // Attempt to retrieve a single row for the given team_id
    let row = db.query_row(
        "SELECT player1_id, player2_id, player3_id FROM teams WHERE team_id = ?",
        params![team_id],
        |row| Ok((row.get::<_, u64>(0)?, row.get::<_, u64>(1)?, row.get::<_, u64>(2)?)),
    );

    match row {
        // If a row is found, check if the player_id is already in one of the slots
        Ok((player1_id, player2_id, player3_id)) => {
            if player1_id == player_id || player2_id == player_id || player3_id == player_id {
                return Ok(false); // Player is already part of the team
            }

            // Determine which slot is open and prepare the update query accordingly
            let update_query = if player1_id == 0 {
                "UPDATE teams SET player1_id = ? WHERE team_id = ?"
            } else if player2_id == 0 {
                "UPDATE teams SET player2_id = ? WHERE team_id = ?"
            } else if player3_id == 0 {
                "UPDATE teams SET player3_id = ? WHERE team_id = ?"
            } else {
                return Ok(false); // All slots are full
            };

            // Execute the update query to add the player to an open slot
            db.execute(update_query, params![player_id, team_id])?;
            Ok(true) // Successfully added player
        }
        // If no row exists for the team_id, insert a new row with player_1 set to player_id
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            db.execute(
                "INSERT INTO teams (team_id, player1_id, player2_id, player3_id) VALUES (?, ?, 0, 0)",
                params![team_id, player_id],
            )?;
            Ok(true) // New row created with the player added to player_1 slot
        }
        // For any other error, return it
        Err(e) => Err(e)
    }
}