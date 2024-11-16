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
use crate::{player, r#match, team, utility, Context, Error};

pub async fn ok_add(ctx: Context<'_>, team_id: u64, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Success")
                    .field(
                        "Added Player",
                        format!(
                            "{} to {}",
                            UserId::new(player_id).mention(),
                            RoleId::new(team_id).mention()
                        ),
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn err_add_already_on_team(ctx: Context<'_>, team_id: u64, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(
                        "Could Not Add Player",
                        format!(
                            "{} is already on team {}",
                            UserId::new(player_id).mention(),
                            RoleId::new(team_id).mention()
                        ),
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn err_add(ctx: Context<'_>, team_id: u64, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(
                        "Could Not Add Player",
                        format!(
                            "{} is already on team {}. (Or, the team is full.)",
                            UserId::new(player_id).mention(),
                            RoleId::new(team_id).mention()
                        ),
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn ok_remove(ctx: Context<'_>, team_id: u64, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Success")
                    .field(
                        "Removed Player",
                        format!(
                            "{} from team {}.",
                            UserId::new(player_id).mention(),
                            RoleId::new(team_id).mention()
                        ),
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn err_remove(ctx: Context<'_>, player_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(
                        "Could not remove Player",
                        format!(
                            "{} was not found to be on any team.",
                            UserId::new(player_id).mention()
                        ),
                        false
                    )
            )
    ).await?;

    Ok(())
}

async fn info_base(ctx: Context<'_>, team_id: u64) -> Result<CreateEmbed, Error> {

    let players = team::query::get_players(team_id).await?;


    let team_mentions = if players.is_empty() {
        "None".to_string()
    } else {
        players
            .iter()
            .map(|p| UserId::new(*p).mention().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    };


    let mention_str: String;
    if team_id == 0 {
        mention_str = "None".to_string();
    } else {
        mention_str = format!("{}", RoleId::new(team_id).mention());
    }


    Ok(
        utility::response::base()
            .title("Team Card")
            .field(
                "Team",
                mention_str,
                true
            )
            .field(
                "Roster",
                team_mentions,
                true
            )
        // .thumbnail(user.face())

    )
}

async fn info_core(ctx: Context<'_>, team_id: u64) -> Result<CreateEmbed, Error> {

    let players = team::query::get_players(team_id).await?;
    let num_players = players.len() as f64;

    let mut stats_avg = (0.0, 0.0, 0.0, 0.0, 0.0);

    for player in players {
        let stats = player::query::stats_core(player).await?;
        stats_avg.0 += stats.0 as f64;
        stats_avg.1 += stats.1;
        stats_avg.2 += stats.2;
        stats_avg.3 += stats.3;
        stats_avg.4 += stats.4;
    }
    stats_avg.0 /= num_players;
    stats_avg.1 /= num_players;
    stats_avg.2 /= num_players;
    stats_avg.3 /= num_players;
    stats_avg.4 /= num_players;


    let avg_mvpr = 250.0 + (
            250.0 * (
                stats_avg.1 + (
                    stats_avg.2 / 3.0
                ) + (
                    stats_avg.3 * 0.75
                ) + (
                    stats_avg.4 * 0.6
                )
            )
        );


    Ok(
        info_base(ctx, team_id).await?
            .field(
                "Core Stats",
                format!("```\
                    Games:         {}\n\
                    Avg. Goals:    {:.2}\n\
                    Avg. Shots:    {:.2}\n\
                    Avg. Assists:  {:.2}\n\
                    Avg. Saves:    {:.2}\n\
                    Avg. TRL MMR:  {:.2}\n\
                    ```", stats_avg.0, stats_avg.1, stats_avg.2, stats_avg.3, stats_avg.4, avg_mvpr
                ),
                false
            )
    )
}

async fn info_demos(ctx: Context<'_>, team_id: u64) -> Result<CreateEmbed, Error> {

    let players = team::query::get_players(team_id).await?;
    let num_players = players.len() as f64;
    let mut stats_avg= (0.0, 0.0, 0.0);

    for player in players {
        let stats = player::query::stats_demos(player).await?;
        stats_avg.0 += stats.0 as f64;
        stats_avg.1 += stats.1;
        stats_avg.2 += stats.2;
    }
    stats_avg.0 /= num_players;
    stats_avg.1 /= num_players;
    stats_avg.2 /= num_players;

    Ok(
        info_base(ctx, team_id).await?
            .field(
                "Demo Stats",
                format!("```\
                    Games:                   {}\n\
                    Avg. Demos Inflicted:    {:.2}\n\
                    Avg. Demos Taken:        {:.2}\n\
                    ```", stats_avg.0, stats_avg.1, stats_avg.2
                ),
                false
            )
    )
}

async fn info_boost(ctx: Context<'_>, team_id: u64) -> Result<CreateEmbed, Error> {
    let players = team::query::get_players(team_id).await?;
    let num_players = players.len() as f64;
    let mut stats_avg = (0.0, 0.0, 0.0, 0.0, 0.0);

    for player in players {
        let stats = player::query::stats_core(player).await?;
        stats_avg.0 += stats.0 as f64;
        stats_avg.1 += stats.1;
        stats_avg.2 += stats.2;
        stats_avg.3 += stats.3;
        stats_avg.4 += stats.4;
    }
    stats_avg.0 /= num_players;
    stats_avg.1 /= num_players;
    stats_avg.2 /= num_players;
    stats_avg.3 /= num_players;
    stats_avg.4 /= num_players;

    Ok(
        info_base(ctx, team_id).await?
            .field(
                "Boost Stats",
                format!("```\
                    Games:                {}\n\
                    Avg. Boost Amount:    {:.2}\n\
                    % Zero Boost:         {:.2}\n\
                    % Full Boost:         {:.2}\n\
                    Avg. Amount Overfill: {:.2}\n\
                    ```", stats_avg.0, stats_avg.1, stats_avg.2, stats_avg.3, stats_avg.4
                ),
                false
            )
    )
}

async fn info_positioning(ctx: Context<'_>, team_id: u64) -> Result<CreateEmbed, Error> {
    let players = team::query::get_players(team_id).await?;
    let num_players = players.len() as f64;
    let mut stats_avg = (0.0, 0.0, 0.0, 0.0, 0.0);

    for player in players {
        let stats = player::query::stats_core(player).await?;
        stats_avg.0 += stats.0 as f64;
        stats_avg.1 += stats.1;
        stats_avg.2 += stats.2;
        stats_avg.3 += stats.3;
        stats_avg.4 += stats.4;
    }
    stats_avg.0 /= num_players;
    stats_avg.1 /= num_players;
    stats_avg.2 /= num_players;
    stats_avg.3 /= num_players;
    stats_avg.4 /= num_players;

    Ok(
        info_base(ctx, team_id).await?
            .field(
                "Positioning Stats",
                format!("```\
                    Games:              {}\n\
                    % Defensive Third:  {:.2}\n\
                    % Neutral Third:    {:.2}\n\
                    % Offensive Third:  {:.2}\n\
                    % Closest to Ball:  {:.2}\n\
                    ```", stats_avg.0, stats_avg.1, stats_avg.2, stats_avg.3, stats_avg.4
                ),
                false
            )
    )
}

pub async fn info(ctx: Context<'_>, team_id: u64) -> Result<(), Error> {

    let response = poise::reply::CreateReply::default()
        .reply(true)
        .embed(info_core(ctx, team_id).await?)
        .components(vec![CreateActionRow::Buttons(
            vec![
                CreateButton::new("team_stats_core").label("Core"),
                CreateButton::new("team_stats_boost").label("Boost"),
                CreateButton::new("team_stats_positioning").label("Positioning"),
                CreateButton::new("team_stats_demos").label("Demos"),
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
            "team_stats_core" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_core(ctx, team_id).await?)
                    )
                ).await?;
            }
            "team_stats_demos" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_demos(ctx, team_id).await?)
                    )
                ).await?;
            }
            "team_stats_boost" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_boost(ctx, team_id).await?)
                    )
                ).await?;
            }
            "team_stats_positioning" => {
                interaction.create_response(
                    &ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(info_positioning(ctx, team_id).await?)
                    )
                ).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn err_info(ctx: Context<'_>) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(
                        "Team Not Registered",
                        "Please add at least one player to this team.",
                        false
                    )
            )
    ).await?;

    Ok(())

}