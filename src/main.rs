use poise::serenity_prelude as serenity;

mod player;
mod utility;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Initialize database & all tables
    player::query::init().await.unwrap();

    let token = env!("DISCORD_TOKEN");  
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![player::cmd::player()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let create_commands = poise::builtins::create_application_commands(&framework.options().commands);
                // serenity::Command::set_global_commands(ctx, create_commands).await?;
                let guild = serenity::GuildId::new(1288596199712096357);
                guild.set_commands(ctx, create_commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}