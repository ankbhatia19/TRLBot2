use rusqlite::{params, Result};
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

pub async fn tally(match_id: i32) -> Result<Vec<(i32, i32, i32)>> {

    let db = utility::query::db().await?;
    let mut game_scores: Vec<(i32, i32, i32)> = vec![];

    let mut query = db.prepare(
        "SELECT
            s.game_num,
            team1.team_id AS team1_id,
            team2.team_id AS team2_id,
            COALESCE(SUM(CASE WHEN s.player_id IN (team1.player1_id, team1.player2_id, team1.player3_id) THEN s.goals END), 0) AS team1_goals,
            COALESCE(SUM(CASE WHEN s.player_id IN (team2.player1_id, team2.player2_id, team2.player3_id) THEN s.goals END), 0) AS team2_goals
        FROM
            stats as s
        JOIN
            matches AS m ON s.match_id = m.match_id
        JOIN
            teams AS team1 ON m.team1_id = team1.team_id
        JOIN
            teams AS team2 ON m.team2_id = team2.team_id
        WHERE
            s.match_id = ?
        GROUP BY
            s.game_num;"
    )?;

    let rows = query.query_map(params![match_id], |row| {
        Ok((row.get(0)?, row.get(3)?, row.get(4)?))
    })?;

    for row_result in rows {
        game_scores.push(row_result?); // Unwrap each Result from `rows` and push to `game_scores`
    }

    Ok(game_scores)
}

pub async fn score(match_id: i32) -> Result<(u64, u64, i32, i32)> {

    let db = utility::query::db().await?;

    db.query_row(
        "
        WITH game_results AS (
            SELECT
                s.game_num,
                team1.team_id AS team1_id,
                team2.team_id AS team2_id,
                COALESCE(SUM(CASE WHEN s.player_id IN (team1.player1_id, team1.player2_id, team1.player3_id) THEN s.goals END), 0) AS team1_goals,
                COALESCE(SUM(CASE WHEN s.player_id IN (team2.player1_id, team2.player2_id, team2.player3_id) THEN s.goals END), 0) AS team2_goals
            FROM
                stats as s
            JOIN
                matches AS m ON s.match_id = m.match_id
            JOIN
                teams AS team1 ON m.team1_id = team1.team_id
            JOIN
                teams AS team2 ON m.team2_id = team2.team_id
            WHERE
                s.match_id = ?
            GROUP BY
                s.game_num
        )
        SELECT
            team1_id,
            team2_id,
            SUM(CASE WHEN team1_goals > team2_goals THEN 1 ELSE 0 END) AS team1_score,
            SUM(CASE WHEN team2_goals > team1_goals THEN 1 ELSE 0 END) AS team2_score
        FROM
            game_results
        WHERE
            team1_goals IS NOT NULL OR team2_goals IS NOT NULL;",
        params![match_id],
        |row| {
            Ok ((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        }
    )

}