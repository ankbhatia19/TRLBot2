use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbed;
use serenity::{Mentionable, UserId, CacheHttp};
use serenity::builder::{
    CreateButton,
    CreateActionRow,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{player, team, utility, Context, Error};

pub async fn ok_register(ctx: Context<'_>, username: &str, player_id: u64) -> poise::reply::CreateReply {

    let user = UserId::new(player_id).to_user(ctx.http()).await;

    match user {
        Ok(u) => {
            poise::reply::CreateReply::default()
                .reply(true)
                .embed(utility::response::base()
                    .field(
                        "Success",
                        format!("\nUsername `{}` has been added to {}.\n\n
                        Use `player info` to view.", username, u.mention()),
                        false
                    )
                )
        }
        Err(_) => {
            utility::response::err_no_id(player_id)
        }
    }
}

pub async fn err_register(ctx: Context<'_>, username: &str, player_id: u64) -> poise::reply::CreateReply {

    let user = UserId::new(player_id).to_user(ctx.http()).await;

    let registered_to = player::query::get_id(username).await.unwrap_or_default();

    let mut registered_to_str: String;

    if registered_to != 0 {
        registered_to_str = UserId::new(registered_to).mention().to_string();
    } else {
        registered_to_str = "Nobody".to_string();
    }

    match user {
        Ok(u) => {
            poise::reply::CreateReply::default()
                .reply(true)
                .embed(utility::response::base()
                    .field(
                        "Error",
                        format!(
                            "Username `{}` was not added to {}.\n\n\
                            It is registered to {}.",
                            username, u.mention(), registered_to_str
                        ),
                        false
                    )
                )
        }
        Err(_) => {
            utility::response::err_no_id(player_id)
        }
    }
}

pub async fn ok_remove(ctx: Context<'_>, username: &str, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Success")
                    .field(
                        "Removed Username",
                        format!("`{}` from {}", username, UserId::new(player_id).mention()),
                        false
                    )
            )
    ).await?;

    Ok(())

}

pub async fn err_remove(ctx: Context<'_>, username: &str) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(
                        "Username Not Found",
                        format!("`{}` was not found to be registered to any player.", username),
                        false
                    )
            )
    ).await?;

    Ok(())

}

async fn info_base(ctx: Context<'_>, player_id: u64) -> Result<CreateEmbed, Error> {
    let user = UserId::new(player_id).to_user(ctx.http()).await?;
    let team_id = team::query::get_team(player_id).await.unwrap_or_default();

    let mention_str: String;
    if team_id == 0 {
        mention_str = "None".to_string();
    } else {
        mention_str = format!("{}", RoleId::new(team_id).mention());
    }

    Ok(
        utility::response::base()
            .title("Player Card")
            .field(
                "Player",
                user.mention().to_string(),
                true
            )
            .field(
                "Team",
                mention_str,
                true
            )
            .thumbnail(user.face())

    )
}

async fn info_core(ctx: Context<'_>, player_id: u64) -> Result<CreateEmbed, Error> {
    let stats = player::query::stats_core(player_id).await.unwrap_or_default();
    let names = player::query::get_names(player_id).await.unwrap_or(vec!["None".to_string()]);

    let avg_mvpr = 250.0 + (250.0 * (stats.1 + (stats.2 / 3.0) + (stats.3 * 0.75) + (stats.4 * 0.6)));

    Ok(
        info_base(ctx, player_id).await?
            .field(
                "Core Stats",
                format!("```\
                    Games:         {}\n\
                    Avg. Goals:    {:.2}\n\
                    Avg. Shots:    {:.2}\n\
                    Avg. Assists:  {:.2}\n\
                    Avg. Saves:    {:.2}\n\
                    TRL MMR:       {:.2}\n\
                    ```", stats.0, stats.1, stats.2, stats.3, stats.4, avg_mvpr
                ),
                false
            )
            .field(
                "Registered Usernames",
                format!("{}", names.join("\n")),
                false
            )
    )
}

async fn info_demos(ctx: Context<'_>, player_id: u64) -> Result<CreateEmbed, Error> {

    let stats = player::query::stats_demos(player_id).await.unwrap_or_default();
    let names = player::query::get_names(player_id).await.unwrap_or(vec!["None".to_string()]);

    Ok(
        info_base(ctx, player_id).await?
            .field(
                "Demo Stats",
                format!("```\
                    Games:                   {}\n\
                    Avg. Demos Inflicted:    {:.2}\n\
                    Avg. Demos Taken:        {:.2}\n\
                    ```", stats.0, stats.1, stats.2
                ),
                false
            )
            .field(
                "Registered Usernames",
                format!("{}", names.join("\n")),
                false
            )
    )
}

async fn info_boost(ctx: Context<'_>, player_id: u64) -> Result<CreateEmbed, Error> {
    let stats = player::query::stats_boost(player_id).await.unwrap_or_default();
    let names = player::query::get_names(player_id).await.unwrap_or(vec!["None".to_string()]);

    Ok(
        info_base(ctx, player_id).await?
            .field(
                "Boost Stats",
                format!("```\
                    Games:                {}\n\
                    Avg. Boost Amount:    {:.2}\n\
                    % Zero Boost:         {:.2}\n\
                    % Full Boost:         {:.2}\n\
                    Avg. Amount Overfill: {:.2}\n\
                    ```", stats.0, stats.1, stats.2, stats.3, stats.4
                ),
                false
            )
            .field(
                "Registered Usernames",
                format!("{}", names.join("\n")),
                false
            )
    )
}

async fn info_positioning(ctx: Context<'_>, player_id: u64) -> Result<CreateEmbed, Error> {
    let stats = player::query::stats_positioning(player_id).await.unwrap_or_default();
    let names = player::query::get_names(player_id).await.unwrap_or(vec!["None".to_string()]);

    Ok(
        info_base(ctx, player_id).await?
            .field(
                "Positioning Stats",
                format!("```\
                    Games:              {}\n\
                    % Defensive Third:  {:.2}\n\
                    % Neutral Third:    {:.2}\n\
                    % Offensive Third:  {:.2}\n\
                    % Closest to Ball:  {:.2}\n\
                    ```", stats.0, stats.1, stats.2, stats.3, stats.4
                ),
                false
            )
            .field(
                "Registered Usernames",
                format!("{}", names.join("\n")),
                false
            )
    )
}

pub async fn info(ctx: Context<'_>, player_id: u64) -> Result<(), Error> {

    let response = poise::reply::CreateReply::default()
        .reply(true)
        .embed(info_core(ctx, player_id).await?)
        .components(vec![CreateActionRow::Buttons(
            vec![
                CreateButton::new("player_stats_core").label("Core"),
                CreateButton::new("player_stats_boost").label("Boost"),
                CreateButton::new("player_stats_positioning").label("Positioning"),
                CreateButton::new("player_stats_demos").label("Demos"),
            ])
        ]);


    // Fire the message, which has two buttons attached
    let m = ctx.send(response).await?;

    // Allow the user to press multiple buttons via a stream
    let mut interaction_stream = m
        .into_message().await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(300))
        .stream();

    // Waiting for the user to press the button...
    while let Some (interaction) = interaction_stream.next().await {

        match interaction.data.custom_id.as_str() {
            "player_stats_core" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_core(ctx, player_id).await?)
                    )
                ).await?;
            }
            "player_stats_demos" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_demos(ctx, player_id).await?)
                    )
                ).await?;
            }
            "player_stats_boost" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_boost(ctx, player_id).await?)
                    )
                ).await?;
            }
            "player_stats_positioning" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_positioning(ctx, player_id).await?)
                    )
                ).await?;
            }
            _ => {}
        }
    }

    Ok(())
}