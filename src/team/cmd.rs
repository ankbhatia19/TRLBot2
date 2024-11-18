use poise::serenity_prelude as serenity;
use crate::{r#match, stats, team, utility, Context, Error};

/// Group of team commands
#[poise::command(slash_command, subcommands("add", "remove", "info"))]
pub async fn team(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Add player(s) to a team
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Team to which players should be added"] team: serenity::Role,
    #[description = "Player to be added"] player1: serenity::User,
    #[description = "Player to be added"] player2: Option<serenity::User>,
    #[description = "Player to be added"] player3: Option<serenity::User>
) -> Result<(), Error> {

    let mut players = vec![player1];

    match player2 {
        Some(p) => { players.push(p); },
        _ => {}
    }
    match player3 {
        Some(p) => { players.push(p); },
        _ => {}
    }

    for player in players {

        let current_team = team::query::get_team(player.id.get()).await.unwrap_or_default();

        if current_team != 0 {
            team::response::err_add_already_on_team(ctx, current_team, player.id.get()).await?;
        } else if team::query::add(team.id.get(), player.id.get()).await? {
            team::response::ok_add(ctx, team.id.get(), player.id.get()).await?;
        } else {
            team::response::err_add(ctx, team.id.get(), player.id.get()).await?;
        }
    }

    Ok(())
}

/// View the information of a selected team
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "The team to view"] team: serenity::Role
) -> Result<(), Error>{

    if !team::query::has_id(team.id.get()).await? {
        team::response::err_info(ctx).await?;
    } else {
        team::response::info(ctx, team.id.get()).await?;
    }

    Ok(())
}

/// Remove a player from whichever team they are registered to
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The player to remove"] player: serenity::User
) -> Result<(), Error>{

    let current_team = team::query::get_team(player.id.get()).await.unwrap_or_default();

    if current_team == 0 {
        team::response::err_remove(ctx, player.id.get()).await?;
    } else {
        team::query::remove(player.id.get()).await?;
        team::response::ok_remove(ctx, current_team, player.id.get()).await?;
    }

    Ok(())
}