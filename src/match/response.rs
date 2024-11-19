use std::fmt::format;
use poise::ReplyHandle;
use poise::serenity_prelude::{ChannelId, Mentionable, RoleId, UserId};
use crate::{player, r#match, team, utility, Context, Error};

pub async fn ok_create(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {

    let (team1_id, team2_id) = r#match::query::get_teams(match_id).await?;
    let team1_players = team::query::get_players(team1_id).await?;
    let team2_players = team::query::get_players(team2_id).await?;

    let team1_mentions = if team1_players.is_empty() {
        "None".to_string()
    } else {
        team1_players
            .iter()
            .map(|p| UserId::new(*p).mention().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    };

    let team2_mentions = if team2_players.is_empty() {
        "None".to_string()
    } else {
        team2_players
            .iter()
            .map(|p| UserId::new(*p).mention().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    };

    let embed = utility::response::base()
        .title(format!("Match ID: {}", match_id))
        .field(
            "Team 1",
            RoleId::new(team1_id).mention().to_string(),
            true
        )
        .field(
            "Team 2",
            RoleId::new(team2_id).mention().to_string(),
            true
        )
        .field(
            "_ _",
            "_ _",
            false
        )
        .field(
            "Roster",
            team1_mentions,
            true
        )
        .field(
            "Roster",
            team2_mentions,
            true
        );

    ctx.send(
             poise::reply::CreateReply::default()
                 .reply(true)
                 .embed(embed)
    ).await?;

    Ok(())

}

pub async fn err_create(ctx: Context<'_>, team_id: u64) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Error: Could not create match.")
                    .field(
                        "This team does not have any players:",
                        format!("{}", RoleId::new(team_id).mention()),
                        false
                    )
            )
    ).await?;

    Ok(())

}

pub async fn ok_remove_in_progress(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Removing...")
                    .field(format!("Match ID: {}", match_id), "_ _", false)
            )
    ).await?;

    Ok(())
}

pub async fn ok_remove_complete(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Removed")
                    .field(format!("Match ID: {}", match_id), "_ _", false)
            )
    ).await?;

    Ok(())
}

pub async fn err_remove(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {

    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Error")
                    .field(format!("Match ID {} does not exist.", match_id), "_ _", false)
            )
    ).await?;

    Ok(())
}


pub async fn ok_submit(ctx: Context<'_>, msg: ReplyHandle<'_>, match_id: i32) -> Result<(), Error> {

    let (team1_id, team2_id, team1_score, team2_score) = r#match::query::score(match_id).await?;
    let game_scores = r#match::query::tally(match_id).await?;

    let ballchasing_id = r#match::query::get_ballchasing_id(match_id).await?;

    let game_scores_str = game_scores.iter()
        .map(|score| {
            format!("Game #{}:      {}       {}", score.0, score.1, score.2)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let mut embed = utility::response::base()
        .title(format!("Match #{}", match_id))
        .field(
            "Team 1",
            format!("{}", RoleId::new(team1_id).mention()),
            true
        )
        .field(
            "Team 2",
            format!("{}", RoleId::new(team2_id).mention()),
            true
        )
        .field(
            "Game Stats",
            format!("```            Team 1  Team 2\n{}```", game_scores_str),
            false
        );

    embed = if team1_score > team2_score {
        embed.field(
            "Winner",
            format!("{} **({} - {})**", RoleId::new(team1_id).mention(), team1_score, team2_score),
            false
        )
    } else if team2_score > team1_score {
        embed.field(
            "Winner",
            format!("{} **({} - {})**", RoleId::new(team2_id).mention(), team2_score, team1_score),
            false
        )
    } else {
        embed.field(
            "Winner",
            "**None**",
            false
        )
    };

    embed = embed.field(
        "Ballchasing Group",
        format!("https://ballchasing.com/group/{}", ballchasing_id),
        false
    );

    msg.edit(
        ctx,
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed.clone())
    ).await?;

    ChannelId::new(
        std::env::var("REPORT_CHANNEL")
            .expect("REPORT_CHANNEL must be set")
            .parse()
            .unwrap()
    ).send_message(
        ctx.http(),
        poise::serenity_prelude::CreateMessage::default()
            .embed(embed)
        ).await?;

    Ok(())

}

pub async fn ok_submit_processing(ctx: Context<'_>, match_id: i32) -> Result<(poise::ReplyHandle), Error> {

    let reply = ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title(format!("Submitting Match #{}...", match_id))
                    .field(
                        "_ _",
                        "Please allow up to 5 minutes for match to process.",
                        false
                    )
            )
    ).await?;

    Ok((reply))
}

pub async fn err_submit_no_matchid(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {
    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Error Submitting")
                    .field(
                        format!("Match ${} does not exist.", match_id),
                        "Use `/match create` to create a new match,",
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn err_submit_no_games_submitted(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), Error> {
    ctx.send(
        poise::reply::CreateReply::default()
            .embed(
                utility::response::base()
                    .title("Error Submitting")
                    .field(
                        "Must provide at least one replay.",
                        "Use `/match submit` to drag/drop replays.",
                        false
                    )
            )
    ).await?;

    Ok(())
}

pub async fn err_submit_missing_usernames(ctx: Context<'_>, msg: ReplyHandle<'_>, match_id: i32, missing: Vec<&str>) -> Result<(), Error> {

    let missing_str = missing.iter()
        .map(|m| format!("`{}`", m))
        .collect::<Vec<String>>()
        .join("\n");

    let embed = utility::response::base()
        .title(format!("Error Processing Match #{}", match_id))
        .field(
            "Missing Usernames",
            missing_str,
            false
        );

    msg.edit(
        ctx,
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed)
    ).await?;

    Ok(())
}

pub async fn err_submit_missing_team(ctx: Context<'_>, msg: ReplyHandle<'_>, match_id: i32, missing: Vec<u64>) -> Result<(), Error> {
    let mentions = if missing.is_empty() {
        "None".to_string()
    } else {
        missing
            .iter()
            .map(|p| UserId::new(*p).mention().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    };

    let embed = utility::response::base()
        .title(format!("Error Processing Match #{}", match_id))
        .field(
            "These players are not registered to either team:",
            mentions,
            false
        );

    msg.edit(
        ctx,
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed)
    ).await?;

    Ok(())
}