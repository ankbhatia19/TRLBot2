use poise::serenity_prelude as serenity;

use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use std::path::Path;
use poise::futures_util::future::join_all;
use crate::{r#match, stats, utility, Context, Error};

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

    // TODO: Check if the match_id already has a ballchasing_id in matches table
    let group_data = utility::ballchasing::create(
        match_id
    ).await?;

    let group_id = group_data["id"].as_str().ok_or_else(|| {
        match group_data["error"].as_str() {
            None => { String::from("Ballchasing provided no response.") }
            Some(e) => { format!("Ballchasing Error: {}", e) }
        }
    })?;

    //println!("{}", serde_json::to_string_pretty(&group_data).unwrap_or(String::new()));


    // Map each attachment to an asynchronous task to download and save it
    let ballchasing_tasks = attachments.iter().enumerate().map(|(i, attachment)| {
        let save_dir = save_dir.clone(); // Clone the directory path for each task
        let url = attachment.url.clone();
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
            //println!("{}", serde_json::to_string_pretty(&upload_data).unwrap_or(String::new()));

            let ballchasing_id = upload_data["id"].as_str().ok_or_else(|| {
                match upload_data["error"].as_str() {
                    None => { String::from("Ballchasing provided no response.") }
                    Some(e) => { format!("Ballchasing Error: {}", e) }
                }
            })?;

            utility::ballchasing::group(
                &format!("{}_Game{}.replay", match_id, i+1),
                group_id,
                ballchasing_id
            ).await?;

            let game_data = utility::ballchasing::pull(
                ballchasing_id
            ).await?;
            //println!("{}", serde_json::to_string_pretty(&game_data).unwrap_or(String::new()));

            stats::query::insert_raw(
                &attachment.filename,
                ballchasing_id,
                &game_data
            ).await?;

            Ok::<(), Error>(())
        }
    });

    // Run all download tasks in parallel
    join_all(ballchasing_tasks).await.into_iter().collect::<Result<Vec<_>, _>>()?;
    println!("Uploading complete for Match #{}.\nBeginning processing...", match_id);


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

    // TODO: Embed for succesful match id creation
    ctx.reply(format!("Created match {}", match_id)).await?;

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