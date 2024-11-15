use poise::serenity_prelude::{Mentionable, RoleId, UserId};
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

pub async fn ok_submit(ctx: Context<'_>, match_id: i32) -> Result<(), Error> {

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
        .title(format!("Match # {}", match_id))
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
            format!("{} ({} - {})", RoleId::new(team1_id).mention(), team1_score, team2_score),
            false
        )
    } else if team2_score > team1_score {
        embed.field(
            "Winner",
            format!("{} ({} - {})", RoleId::new(team2_id).mention(), team2_score, team1_score),
            false
        )
    } else {
        embed.field(
            "Winner",
            "None",
            false
        )
    };

    embed = embed.field(
        "Ballchasing Group",
        format!("https://ballchasing.com/group/{}", ballchasing_id),
        false
    );

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed)
    ).await?;

    Ok(())

}

pub async fn err_submit_missing_usernames(ctx: Context<'_>, match_id: i32, missing: Vec<&str>) -> Result<(), Error> {

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

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed)
    ).await?;

    Ok(())
}

pub async fn err_submit_missing_team(ctx: Context<'_>, match_id: i32, missing: Vec<u64>) -> Result<(), Error> {
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
            "Missing Team",
            mentions,
            false
        );

    ctx.send(
        poise::reply::CreateReply::default()
            .reply(true)
            .embed(embed)
    ).await?;

    Ok(())
}