use poise::serenity_prelude as serenity;

mod player;
mod utility;
mod stats;
mod team;
mod r#match;


struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Initialize database & all tables
    player::query::init().await.unwrap();
    stats::query::init().await.unwrap();
    r#match::query::init().await.unwrap();
    team::query::init().await.unwrap();

    println!("Launching TRLBot2...");

    let token = std::env::var("DISCORD_TOKEN").expect("Discord Token is required");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // Command list
                player::cmd::player(),
                r#match::cmd::r#match(),
                team::cmd::team(),
                // End command list
            ], ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let create_commands = poise::builtins::create_application_commands(
                    &framework.options().commands
                );

                serenity::Command::set_global_commands(ctx, create_commands).await?;
                // serenity::GuildId::new(483325630692327434)
                //     .set_commands(ctx, create_commands)
                //     .await?;

                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    println!("TRLBot2 launched successfully.");
    client.unwrap().start().await.unwrap();
}