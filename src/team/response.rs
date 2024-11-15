use poise::serenity_prelude::{Mentionable, RoleId, UserId};
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