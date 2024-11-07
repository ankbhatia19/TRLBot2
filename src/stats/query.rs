use rusqlite::{params, Connection, Result};
use serde_json;
use crate::utility;

pub async fn init() -> Result<()> {

    let db = utility::query::db().await?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS stats (
            player_id INTEGER NOT NULL,
            match_id INTEGER NOT NULL,
            game_num INTEGER NOT NULL,
            ballchasing_id TEXT NOT NULL,
            shots INTEGER,
            shots_against INTEGER,
            goals INTEGER,
            goals_against INTEGER,
            saves INTEGER,
            assists INTEGER,
            score INTEGER,
            mvp BOOLEAN,
            shooting_percentage REAL,
            bpm INTEGER,
            bcpm REAL,
            avg_amount REAL,
            amount_collected INTEGER,
            amount_stolen INTEGER,
            amount_collected_big INTEGER,
            amount_stolen_big INTEGER,
            amount_collected_small INTEGER,
            amount_stolen_small INTEGER,
            count_collected_big INTEGER,
            count_stolen_big INTEGER,
            count_collected_small INTEGER,
            count_stolen_small INTEGER,
            amount_overfill INTEGER,
            amount_overfill_stolen INTEGER,
            amount_used_while_supersonic INTEGER,
            time_zero_boost REAL,
            percent_zero_boost REAL,
            time_full_boost REAL,
            percent_full_boost REAL,
            time_boost_0_25 REAL,
            time_boost_25_50 REAL,
            time_boost_50_75 REAL,
            time_boost_75_100 REAL,
            percent_boost_0_25 REAL,
            percent_boost_25_50 REAL,
            percent_boost_50_75 REAL,
            percent_boost_75_100 REAL,
            avg_speed INTEGER,
            total_distance INTEGER,
            time_supersonic_speed REAL,
            time_boost_speed REAL,
            time_slow_speed REAL,
            time_ground REAL,
            time_low_air REAL,
            time_high_air REAL,
            time_powerslide REAL,
            count_powerslide INTEGER,
            avg_powerslide_duration REAL,
            avg_speed_percentage REAL,
            percent_slow_speed REAL,
            percent_boost_speed REAL,
            percent_supersonic_speed REAL,
            percent_ground REAL,
            percent_low_air REAL,
            percent_high_air REAL,
            avg_distance_to_ball INTEGER,
            avg_distance_to_ball_possession INTEGER,
            avg_distance_to_ball_no_possession INTEGER,
            avg_distance_to_mates INTEGER,
            time_defensive_third REAL,
            time_neutral_third REAL,
            time_offensive_third REAL,
            time_defensive_half REAL,
            time_offensive_half REAL,
            time_behind_ball REAL,
            time_infront_ball REAL,
            time_most_back REAL,
            time_most_forward REAL,
            time_closest_to_ball REAL,
            time_farthest_from_ball REAL,
            percent_defensive_third REAL,
            percent_offensive_third REAL,
            percent_neutral_third REAL,
            percent_defensive_half REAL,
            percent_offensive_half REAL,
            percent_behind_ball REAL,
            percent_infront_ball REAL,
            percent_most_back REAL,
            percent_most_forward REAL,
            percent_closest_to_ball REAL,
            percent_farthest_from_ball REAL,
            demos_inflicted INTEGER,
            demos_taken INTEGER,
            PRIMARY KEY (player_id, match_id, game_num)
        );",
        params![] // No parameters
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS stats_raw (
            filename TEXT NOT NULL,
            ballchasing_id TEXT NOT NULL,
            data TEXT,
            PRIMARY KEY (filename)
        );",
        params![]
    )?;

    Ok(())
}

pub async fn insert_raw(
    filename: &str,
    ballchasing_id: &str,
    data: &serde_json::Value
) -> Result<()> {

    let db = utility::query::db().await?;

    let mut query = db.prepare("
        INSERT INTO stats_raw (filename, ballchasing_id, data)
        VALUES (?, ?, ?);"
    )?;

    query.execute(params![
        filename,
        ballchasing_id,
        serde_json::to_string_pretty(data).unwrap_or(String::new())
    ])?;

    Ok(())

}

pub async fn get_raw(
    filename: &str
) -> Result<String> {

    let db = utility::query::db().await?;

    let mut query = db.prepare("
        SELECT filename, data FROM stats_raw WHERE filename = ?"
    )?;

    query.query_row(params![filename], |row| row.get(0))
}

pub async fn insert(
    player_id: u64,
    match_id: i32,
    game_num: i32,
    ballchasing_id: &str,
    stats: &serde_json::Value
) -> Result<()> {
    let query = "
        INSERT OR REPLACE INTO player_stats (
            player_id, match_id, game_num, ballchasing_id,
            shots, shots_against, goals, goals_against, saves, assists, score, mvp,
            shooting_percentage, bpm, bcpm, avg_amount, amount_collected, amount_stolen, amount_collected_big,
            amount_stolen_big, amount_collected_small, amount_stolen_small, count_collected_big, count_stolen_big,
            count_collected_small, count_stolen_small, amount_overfill, amount_overfill_stolen,
            amount_used_while_supersonic, time_zero_boost, percent_zero_boost, time_full_boost, percent_full_boost,
            time_boost_0_25, time_boost_25_50, time_boost_50_75, time_boost_75_100, percent_boost_0_25,
            percent_boost_25_50, percent_boost_50_75, percent_boost_75_100, avg_speed, total_distance,
            time_supersonic_speed, time_boost_speed, time_slow_speed, time_ground, time_low_air, time_high_air,
            time_powerslide, count_powerslide, avg_powerslide_duration, avg_speed_percentage, percent_slow_speed,
            percent_boost_speed, percent_supersonic_speed, percent_ground, percent_low_air, percent_high_air,
            avg_distance_to_ball, avg_distance_to_ball_possession, avg_distance_to_ball_no_possession,
            avg_distance_to_mates, time_defensive_third, time_neutral_third, time_offensive_third,
            time_defensive_half, time_offensive_half, time_behind_ball, time_infront_ball, time_most_back,
            time_most_forward, time_closest_to_ball, time_farthest_from_ball, percent_defensive_third,
            percent_offensive_third, percent_neutral_thistmtrd, percent_defensive_half, percent_offensive_half,
            percent_behind_ball, percent_infront_ball, percent_most_back, percent_most_forward, percent_closest_to_ball,
            percent_farthest_from_ball, demos_inflicted, demos_taken
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    ";

    let db = utility::query::db().await?;

    // Create a prepared statement
    let mut statement = db.prepare(query)?;

    // Bind the values
    statement.execute(params![
        player_id, match_id, game_num, ballchasing_id,
        stats["core"]["shots"].as_i64().unwrap_or(0),
        stats["core"]["shots_against"].as_i64().unwrap_or(0),
        stats["core"]["goals"].as_i64().unwrap_or(0),
        stats["core"]["goals_against"].as_i64().unwrap_or(0),
        stats["core"]["saves"].as_i64().unwrap_or(0),
        stats["core"]["assists"].as_i64().unwrap_or(0),
        stats["core"]["score"].as_i64().unwrap_or(0),
        stats["core"]["mvp"].as_bool().unwrap_or(false),
        stats["core"]["shooting_percentage"].as_f64().unwrap_or(0.0),
        stats["boost"]["bpm"].as_i64().unwrap_or(0),
        stats["boost"]["bcpm"].as_f64().unwrap_or(0.0),
        stats["boost"]["avg_amount"].as_f64().unwrap_or(0.0),
        stats["boost"]["amount_collected"].as_i64().unwrap_or(0),
        stats["boost"]["amount_stolen"].as_i64().unwrap_or(0),
        stats["boost"]["amount_collected_big"].as_i64().unwrap_or(0),
        stats["boost"]["amount_stolen_big"].as_i64().unwrap_or(0),
        stats["boost"]["amount_collected_small"].as_i64().unwrap_or(0),
        stats["boost"]["amount_stolen_small"].as_i64().unwrap_or(0),
        stats["boost"]["count_collected_big"].as_i64().unwrap_or(0),
        stats["boost"]["count_stolen_big"].as_i64().unwrap_or(0),
        stats["boost"]["count_collected_small"].as_i64().unwrap_or(0),
        stats["boost"]["count_stolen_small"].as_i64().unwrap_or(0),
        stats["boost"]["amount_overfill"].as_i64().unwrap_or(0),
        stats["boost"]["amount_overfill_stolen"].as_i64().unwrap_or(0),
        stats["boost"]["amount_used_while_supersonic"].as_i64().unwrap_or(0),
        stats["boost"]["time_zero_boost"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_zero_boost"].as_f64().unwrap_or(0.0),
        stats["boost"]["time_full_boost"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_full_boost"].as_f64().unwrap_or(0.0),
        stats["boost"]["time_boost_0_25"].as_f64().unwrap_or(0.0),
        stats["boost"]["time_boost_25_50"].as_f64().unwrap_or(0.0),
        stats["boost"]["time_boost_50_75"].as_f64().unwrap_or(0.0),
        stats["boost"]["time_boost_75_100"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_boost_0_25"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_boost_25_50"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_boost_50_75"].as_f64().unwrap_or(0.0),
        stats["boost"]["percent_boost_75_100"].as_f64().unwrap_or(0.0),
        stats["movement"]["avg_speed"].as_i64().unwrap_or(0),
        stats["movement"]["total_distance"].as_i64().unwrap_or(0),
        stats["movement"]["time_supersonic_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_boost_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_slow_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_ground"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_low_air"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_high_air"].as_f64().unwrap_or(0.0),
        stats["movement"]["time_powerslide"].as_f64().unwrap_or(0.0),
        stats["movement"]["count_powerslide"].as_i64().unwrap_or(0),
        stats["movement"]["avg_powerslide_duration"].as_f64().unwrap_or(0.0),
        stats["movement"]["avg_speed_percentage"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_slow_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_boost_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_supersonic_speed"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_ground"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_low_air"].as_f64().unwrap_or(0.0),
        stats["movement"]["percent_high_air"].as_f64().unwrap_or(0.0),
        stats["positioning"]["avg_distance_to_ball"].as_i64().unwrap_or(0),
        stats["positioning"]["avg_distance_to_ball_possession"].as_i64().unwrap_or(0),
        stats["positioning"]["avg_distance_to_ball_no_possession"].as_i64().unwrap_or(0),
        stats["positioning"]["avg_distance_to_mates"].as_i64().unwrap_or(0),
        stats["positioning"]["time_defensive_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_neutral_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_offensive_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_defensive_half"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_offensive_half"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_behind_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_infront_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_most_back"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_most_forward"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_closest_to_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["time_farthest_from_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_defensive_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_offensive_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_neutral_third"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_defensive_half"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_offensive_half"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_behind_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_infront_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_most_back"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_most_forward"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_closest_to_ball"].as_f64().unwrap_or(0.0),
        stats["positioning"]["percent_farthest_from_ball"].as_f64().unwrap_or(0.0),
        stats["demo"]["inflicted"].as_i64().unwrap_or(0),
        stats["demo"]["taken"].as_i64().unwrap_or(0)
    ])?;

    Ok(())
}
