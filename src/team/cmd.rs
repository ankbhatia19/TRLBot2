use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Mentionable};
use crate::{r#match, stats, team, utility, Context, Error};
use crate::player::cmd::player;

/// TODO: Description
#[poise::command(slash_command, subcommands("add", "remove", "info"))]
pub async fn team(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// TODO: Description
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "TODO: Description"] team: serenity::Role,
    #[description = "TODO: Description"] player1: serenity::User,
    #[description = "TODO: Description"] player2: Option<serenity::User>,
    #[description = "TODO: Description"] player3: Option<serenity::User>
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

/// TODO: Description
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "TODO: Description"] team: serenity::Role
) -> Result<(), Error>{

    if !team::query::has_id(team.id.get()).await? {
        ctx.reply("Team does not exist").await?;
    } else {
        team::response::info(ctx, team.id.get()).await?;
    }

    Ok(())

}

/// TODO: Description
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "TODO: Description"] player: serenity::User
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