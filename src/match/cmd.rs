use std::future::Future;
use poise::serenity_prelude as serenity;

use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use std::path::Path;
use poise::futures_util::future::join_all;
use crate::{player, r#match, stats, utility, team, Context, Error};

/// TODO: Description
#[poise::command(slash_command, subcommands("create", "submit", "info"))]
pub async fn r#match(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// TODO: Description
#[poise::command(slash_command)]
pub async fn submit(
    ctx: Context<'_>,
    #[description = "TODO: Description"] match_id: i32,
    #[description = "TODO: Description"] game_1: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_2: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_3: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_4: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_5: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_6: Option<serenity::Attachment>,
    #[description = "TODO: Description"] game_7: Option<serenity::Attachment>
) -> Result<(), Error> {

    // TODO: Send 'In Progress' embed
    ctx.reply(format!("Submitting Match #{}", match_id)).await?;

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
            let response = reqwest::get(&url).await?;
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
        r#match::response::err_submit_missing_usernames(ctx, match_id, unregistered).await?;
    } else if !teamless.is_empty() {
        r#match::response::err_submit_missing_team(ctx, match_id, teamless).await?;
    } else {
        r#match::response::ok_submit(ctx, match_id).await?;
    }

    Ok(())
}

/// TODO: Description
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "TODO: Description"] team_1: serenity::Role,
    #[description = "TODO: Description"] team_2: serenity::Role
) -> Result<(), Error> {

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
        None => {  }
    }

    r#match::response::ok_create(ctx, match_id).await?;
    Ok(())
}

/// TODO: Description
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "TODO: Description"] match_id: i32
) -> Result<(), Error> {
    unimplemented!()
}

// This proof-of-concept exists just in case Discord ever adds file uploads to modals
#[poise::command(slash_command)]
pub async fn modalsubmit(ctx: Context<'_>) -> Result<(), Error> {

    match ctx {
        Context::Application(atx) => {

            let interaction = atx.interaction;

            let modal = serenity::CreateQuickModal::new("About you")
                .timeout(std::time::Duration::from_secs(600))
                .short_field("First name")
                .short_field("Last name")
                .paragraph_field("Hobbies and interests");


            let response = interaction.quick_modal(atx.serenity_context, modal).await?.unwrap();
            let inputs = response.inputs;
            let (first_name, last_name, hobbies) = (&inputs[0], &inputs[1], &inputs[2]);

            response.interaction.create_response(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponse::Acknowledge
            ).await?;


            interaction.create_followup(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponseFollowup::default()
                    .content(format!("Thank you for your response, {} {}", first_name, last_name))
            ).await?;

        }
        Context::Prefix(_) => {}
    }

    Ok(())
}