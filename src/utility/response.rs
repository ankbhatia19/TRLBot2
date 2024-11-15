use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbedFooter;

pub fn base() -> serenity::CreateEmbed {
    serenity::CreateEmbed::default()
        .color(serenity::Color::from_rgb(15, 15, 150))
        .thumbnail("https://raw.githubusercontent.com/ankbhatia19/TRLBot/master/assets/TRL_logo_topright_noBG.png")
        .footer(
            CreateEmbedFooter::new("\"This is a sample quote.\" - Waycey")
        )
}

pub fn wip() -> poise::reply::CreateReply {
    poise::reply::CreateReply::default()
        .reply(true)
        .embed(base()
            .field(
                "This command has not yet been implemented.",
                "Contact a dev for details.",
                false
            )
        )
}

pub fn err_no_id(player_id: u64) -> poise::reply::CreateReply {
    poise::reply::CreateReply::default()
        .reply(true)
        .embed(base()
            .field(
                "Error",
                format!("\nError: User ID {} was not found.\n\nContact a dev for details.", player_id),
                false
            )
        )
}