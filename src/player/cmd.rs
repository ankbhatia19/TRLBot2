use poise::serenity_prelude as serenity;

use crate::{player, Context, Error};


/// Register and view a player
#[poise::command(slash_command, subcommands("register", "info"))]
pub async fn player(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

/// Displays your or another user's per-game statistics
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "Selected player"] player: Option<serenity::User>
) -> Result<(), Error> {

    let p = player.as_ref().unwrap_or(ctx.author());

    Ok(())
}

/// Register an in-game username to yourself
#[poise::command(slash_command)]
pub async fn register(
    ctx: Context<'_>,
    #[description = "New Username"] username: String,
    #[description = "Player to Register"] player: Option<serenity::User>
) -> Result<(), Error> {

    println!("Calling register");

    let p = player.as_ref().unwrap_or(ctx.author());

    println!("Registering {} to {}", username, p.name);

    if player::query::register(p.id.get(), &username).await? {
        ctx.send(player::response::ok_register(ctx, &username, p.id.get()).await).await?;
    } else {
        ctx.send(player::response::err_register(ctx, &username, p.id.get()).await).await?;
    }

    Ok(())
}