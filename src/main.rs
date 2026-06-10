mod commands;
mod db;
mod event;
mod swear;
mod types;
use commands::{add_swear, age, gambling::coinflip, leaderboard, send, swears};
use poise::serenity_prelude as serenity;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions as PoolOptions};
use std::collections::HashSet;
use swear::Swear;
use tracing::debug;
use types::{Context, Data, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let options = SqliteConnectOptions::new()
        .filename("slurbot.db")
        .create_if_missing(true);
    let pool = PoolOptions::new().connect_with(options).await?;
    debug!("Connected to the database.");
    db::initialize(&pool).await?;

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                age(),
                add_swear(),
                coinflip::coinflip(),
                swears(),
                leaderboard(),
                send(),
            ],
            event_handler: |ctx, event, _framework, data| {
                Box::pin(async move {
                    let eh_result = event::event_handler(ctx, event, data).await;
                    debug!("Handled event: {:?}, result: {:?}", event, eh_result);
                    eh_result
                })
            },
            owners: HashSet::from([serenity::UserId::new(1015743206245290025)]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let pool = pool.clone();
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId::new(1514025036409864272),
                )
                .await?;

                Ok(Data { pool })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client?.start().await?;
    Ok(())
}
