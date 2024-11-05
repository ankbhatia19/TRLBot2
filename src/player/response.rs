use poise::serenity_prelude as serenity;
use serenity::{Mentionable, UserId, CacheHttp};

use crate::{utility, Context};

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

    match user {
        Ok(u) => {
            poise::reply::CreateReply::default()
                .reply(true)
                .embed(utility::response::base()
                    .field(
                        "Error",
                        format!("Username `{}` was not added to {}.\n\n
                        It is registered to another player.", username, u.mention()),
                        false
                    )
                )
        }
        Err(_) => {
            utility::response::err_no_id(player_id)
        }
    }

}