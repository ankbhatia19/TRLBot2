use poise::serenity_prelude as serenity;
use crate::{player, Context, Error};


/// Register and view a player
#[poise::command(slash_command, subcommands("register", "info", "remove"))]
pub async fn player(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Displays your or another user's per-game statistics
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "Selected player"] player: Option<serenity::User>
) -> Result<(), Error> {

    let p = player.as_ref().unwrap_or(ctx.author());

    player::response::info(ctx, p.id.get()).await?;

    Ok(())
}

/// Register an in-game username
#[poise::command(slash_command)]
pub async fn register(
    ctx: Context<'_>,
    #[description = "New Username"] username: String,
    #[description = "Player to Register"] player: Option<serenity::User>
) -> Result<(), Error> {

    let p = player.as_ref().unwrap_or(ctx.author());

    if player::query::register(p.id.get(), &username).await? {
        ctx.send(player::response::ok_register(ctx, &username, p.id.get()).await).await?;
    } else {
        ctx.send(player::response::err_register(ctx, &username, p.id.get()).await).await?;
    }

    Ok(())
}

/// Removes a username from whichever player it is registered to
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Username to remove"] username: String
) -> Result<(), Error> {

    if !player::query::has_name(&username).await? {
        player::response::err_remove(ctx, &username).await?;
    } else {
        let player_id = player::query::get_id(&username).await?;
        player::query::remove(&username, player_id).await?;
        player::response::ok_remove(ctx, &username, player_id).await?;
    }

    Ok(())
}