use std::any::Any;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateActionRow, Mentionable};
use crate::{player, r#match, stats, utility, team, Context, Error};

/// Register for TRL!
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {

    match ctx {
        Context::Application(atx) => {

            let response = poise::reply::CreateReply::default()
                .ephemeral(true)
                .embed(
                    utility::response::base()
                        .title("TRL Winter 2025 Registration")
                        .field("Please visit the calendar before continuing.", "\nPress 'Next' once you've reviewed all dates.", false)
                        .field("Calendar Link", "\nhttps://docs.google.com/spreadsheets/d/1h3azGagXryQyObMPhHlIAJyWxEXxsD0buFH_coLEAeY/edit?usp=sharing", false)
                )
                .components(vec![CreateActionRow::Buttons(
                    vec![
                        serenity::CreateButton::new("register_next").label("Next"),
                    ])
                ]);

            let button_handler = atx.send(response).await?;

            let interaction = button_handler
                .into_message().await?
                .await_component_interaction(&ctx.serenity_context().shard)
                .timeout(std::time::Duration::from_secs(300))
                .await.unwrap();

            //let interaction = atx.interaction;


            let modal = serenity::CreateQuickModal::new("TRL Winter 2025 Registration")
                .timeout(std::time::Duration::from_secs(1800))
                .short_field("Which dates (if any) will you be missing?")
                .short_field("Peak 2s MMR")
                .short_field("Peak 3s MMR")
                .short_field("RL Tracker Link")
                .short_field("Availability (eg. Monday, Tuesday, Saturday)");


            let response = interaction.quick_modal(atx.serenity_context, modal).await?.unwrap();
            let inputs = response.inputs;
            let (missing_dates, peak_2s, peak_3s, tracker, availability) = (&inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4]);

            response.interaction.create_response(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponse::Acknowledge
            ).await?;


            interaction.create_followup(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponseFollowup::default()
                    .embed(
                        utility::response::base()
                            .title("Successful Registration")
                            .field(
                                "You have successfully registered for TRL Winter 2025",
                                "\nPlease reach out to League Staff for any further questions.",
                                false
                            )
                    )
                    .ephemeral(true)
            ).await?;

            ChannelId::new(
                std::env::var("REGISTER_CHANNEL")
                    .expect("REGISTER_CHANNEL must be set")
                    .parse()
                    .unwrap()
            ).send_message(
                ctx.http(),
                poise::serenity_prelude::CreateMessage::default()
                    .embed(
                        utility::response::base()
                            .title("New Registration")
                            .field(
                                "User",
                                atx.author().mention().to_string(),
                                false
                            )
                            .field("Peak 2s", peak_2s, true)
                            .field("Peak 3s", peak_3s, true)
                            .field("Availability", availability, false)
                            .field("Missing Dates", missing_dates, false)
                            .field("Tracker", tracker, false),
                    )
            ).await?;

        }
        Context::Prefix(_) => {}
    }


    Ok(())
}