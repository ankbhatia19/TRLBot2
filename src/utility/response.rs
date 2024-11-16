use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbedFooter;
use serde_json::Value;
use rand::seq::SliceRandom;
use std::fs;
pub fn quote() -> Result<String, serenity::Error> {
    // Read the file content
    let data = fs::read_to_string("quotes.json")?;

    // Parse the JSON file
    let quotes: Vec<Value> = serde_json::from_str(&data)?;

    // Randomly select a quote
    let selected = quotes.choose(&mut rand::thread_rng())
        .expect("Quotes list is empty");

    let result = format!(
        "\"{}\" - {}",
        selected["quote"].as_str().unwrap_or_default(),
        selected["author"].as_str().unwrap_or_default()
    );

    Ok(result)
}

pub fn base() -> serenity::CreateEmbed {
    serenity::CreateEmbed::default()
        .color(serenity::Color::from_rgb(15, 15, 150))
        .thumbnail("https://raw.githubusercontent.com/ankbhatia19/TRLBot/master/assets/TRL_logo_topright_noBG.png")
        .footer(
            CreateEmbedFooter::new(
                quote().unwrap_or("\"This is a sample quote.\" - Waycey".to_string())
            )
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