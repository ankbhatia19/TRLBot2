use std::any::Any;
use poise::serenity_prelude as serenity;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use std::path::Path;
use poise::futures_util::future::join_all;
use poise::serenity_prelude::{ChannelId, ChannelType};
use poise::serenity_prelude::ChannelType::Forum;
use crate::{player, r#match, stats, utility, team, Context, Error};

/// Collection of all match commands
#[poise::command(slash_command, subcommands("create", "submit", "info", "remove"))]
pub async fn r#match(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Submit a match given a match ID.
#[poise::command(slash_command)]
pub async fn submit(
    ctx: Context<'_>,
    #[description = "The match ID to submit"] match_id: i32,
    #[description = "Replay file"] game_1: Option<serenity::Attachment>,
    #[description = "Replay file"] game_2: Option<serenity::Attachment>,
    #[description = "Replay file"] game_3: Option<serenity::Attachment>,
    #[description = "Replay file"] game_4: Option<serenity::Attachment>,
    #[description = "Replay file"] game_5: Option<serenity::Attachment>,
    #[description = "Replay file"] game_6: Option<serenity::Attachment>,
    #[description = "Replay file"] game_7: Option<serenity::Attachment>
) -> Result<(), Error> {

    if !r#match::query::has_id(match_id).await? {
        r#match::response::err_submit_no_matchid(ctx, match_id).await?;
        return Ok(());
    }

    let msg = r#match::response::ok_submit_processing(ctx, match_id).await?;

    let save_dir = format!("Replays/{}", match_id);

    // Ensure the directory exists
    if !Path::new(&save_dir).exists() {
        fs::create_dir_all(&save_dir).await?;
    }

    // Collect non-None attachments into a vector
    let attachments = vec![game_1, game_2, game_3, game_4, game_5, game_6, game_7];
    let attachments: Vec<_> = attachments.into_iter().filter_map(|x| x).collect();

    let group_id = r#match::query::get_ballchasing_id(match_id).await?;

    // Map each attachment to an asynchronous task to download and save it
    let ballchasing_tasks = attachments.iter().enumerate().map(|(i, attachment)| {
        let save_dir = save_dir.clone(); // Clone the directory path for each task
        let url = attachment.url.clone();
        let group_id = group_id.clone();
        let file_path = format!("{}/{}", save_dir, attachment.filename);

        async move {
            // Check if file already exists
            if Path::new(&file_path).exists() {
                println!("File {} already exists, skipping download.", attachment.filename);
                return Ok(());
            }

            // Download and save the file if it does not exist
            let response = utility::ballchasing::CLIENT.get(&url).send().await?;
            let bytes = response.bytes().await?;
            let mut file = File::create(&file_path).await?;
            file.write_all(&bytes).await?;
            file.flush().await?;

            println!("Downloaded and saved file: {}", attachment.filename);

            // Upload to ballchasing
            let upload_data = utility::ballchasing::upload(
                &file_path,
                &attachment.filename
            ).await?;

            let ballchasing_id = upload_data["id"].as_str().ok_or_else(|| {
                match upload_data["error"].as_str() {
                    None => { String::from("Ballchasing provided no response.") }
                    Some(e) => { format!("Ballchasing Error: {}", e) }
                }
            })?;

            utility::ballchasing::group(
                &format!("{}_Game{}.replay", match_id, i+1),
                &group_id,
                ballchasing_id
            ).await?;

            let game_data = utility::ballchasing::pull(
                ballchasing_id
            ).await?;

            stats::query::insert_raw(
                match_id,
                ballchasing_id,
                &attachment.filename,
                &game_data
            ).await?;

            Ok::<(), Error>(())
        }
    });

    // Run all download tasks in parallel
    join_all(ballchasing_tasks).await.into_iter().collect::<Result<Vec<_>, _>>()?;
    println!("Ballchasing tasks complete for Match #{}.\nBeginning processing...", match_id);

    let mut unregistered: Vec<&str> = vec![];
    let mut teamless: Vec<u64> = vec![];

    let (team1_id, team2_id) = r#match::query::get_teams(match_id).await?;
    let data_per_game = stats::query::get_raw(match_id).await?;

    if data_per_game.is_empty() {
        r#match::response::err_submit_no_games_submitted(ctx, msg).await?;
        return Ok(())
    }

    // TODO: Much better logging needed
    for (game_num, data) in data_per_game.iter().enumerate() {

        for team in vec!["blue", "orange"].iter() {

            let players = data.get(team)
                .and_then(|team_data| {
                    team_data.get("players").and_then(serde_json::Value::as_array)
                })
                .ok_or("No players found in data json")?;

            for (_, player) in players.iter().enumerate() {

                let player_name = player.get("name")
                    .ok_or("No player name found in data json")?
                    .as_str()
                    .ok_or("Player name was not a valid string")?;


                if !player::query::has_name(player_name).await? {
                    if !unregistered.contains(&player_name) {
                        unregistered.push(player_name);
                    }
                    continue;
                }

                let player_id = player::query::get_id(player_name).await?;
                let player_team = team::query::get_team(player_id).await.unwrap_or_default();

                if player_team == team1_id || player_team == team2_id{
                    let stats = player.get("stats")
                    .ok_or("No player stats found in data json")?;

                    stats::query::insert(
                        player_id,
                        match_id,
                        (game_num + 1) as i32,
                        stats
                    ).await?;
                    println!("Inserted stats for {} in game {}", player_name, game_num);

                } else {
                    if !teamless.contains(&player_id) {
                        teamless.push(player_id);
                    }
                }
            }
        }
    }

    if !unregistered.is_empty() {
        r#match::response::err_submit_missing_usernames(ctx, msg, match_id, unregistered).await?;
    } else if !teamless.is_empty() {
        r#match::response::err_submit_missing_team(ctx, msg, match_id, teamless).await?;
    } else {
        r#match::response::ok_submit(ctx, msg, match_id).await?;
    }

    Ok(())
}

/// Create a new match ID for submission
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Team 1"] team_1: serenity::Role,
    #[description = "Team 2"] team_2: serenity::Role
) -> Result<(), Error> {

    if !team::query::has_id(team_1.id.get()).await?{
        r#match::response::err_create(ctx, team_1.id.get()).await?;
        return Ok(());
    }
    if !team::query::has_id(team_2.id.get()).await?{
        r#match::response::err_create(ctx, team_2.id.get()).await?;
        return Ok(());
    }

    let team1_id = team_1.id.get();
    let team2_id = team_2.id.get();

    let match_id = r#match::query::create(team1_id, team2_id).await?;

    let group_data = utility::ballchasing::create(
        match_id
    ).await?;

    match group_data["id"].as_str() {
        Some(ballchasing_id) => {
            r#match::query::set_ballchasing_id(match_id, ballchasing_id).await?;
        },
        None => { println!("Ballchasing did not return a group ID."); }
    }

    r#match::response::ok_create(ctx, match_id).await?;
    Ok(())
}

/// View the information of a match, including scheduling details.
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "The match ID to view"] match_id: i32
) -> Result<(), Error> {

    ctx.send(utility::response::wip()).await?;
    Ok(())
}


/// Remove a match. Also removes the match from the ballchasing group.
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The match ID to remove"] match_id: i32
) -> Result<(), Error> {

    if !r#match::query::has_id(match_id).await? {
        r#match::response::err_remove(ctx, match_id).await?;
    } else {
        r#match::response::ok_remove_in_progress(ctx, match_id).await?;
        let ids = stats::query::get_ballchasing_ids(match_id).await?;

        for id in ids {
            utility::ballchasing::ungroup(&id).await?;
        }

        let group_id = r#match::query::get_ballchasing_id(match_id).await?;
        utility::ballchasing::delete_group(&group_id).await?;

        r#match::query::remove(match_id).await?;
        stats::query::remove(match_id).await?;

        r#match::response::ok_remove_complete(ctx, match_id).await?;
    }

    Ok(())
}