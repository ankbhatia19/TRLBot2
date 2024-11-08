use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Mentionable};
use crate::{r#match, stats, team, utility, Context, Error};
use crate::player::cmd::player;

/// TODO: Description
#[poise::command(slash_command, subcommands("add", "remove"))]
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

    // TODO: Add embeds for success and failure
    for (i, player) in players.iter().enumerate() {
        if team::query::add(team.id.get(), player.id.get()).await? {
            ctx.reply(format!("Added {} to team {}", player.name, team.name)).await?;
        } else {
            ctx.reply(format!("Failed to add {} to team {}", player.name, team.name)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "TODO: Description"] player: serenity::User
) -> Result<(), Error>{
    unimplemented!()
}