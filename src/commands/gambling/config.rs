use poise::{CreateReply, serenity_prelude::CreateEmbedAuthor};
use rand::{RngExt, rngs::ThreadRng};

use crate::{
    Context, Error, SqlitePool,
    db::{get_user_swears, set_user_swears},
    serenity::{Colour, CreateEmbed, User},
};

enum FlipResult {
    Win(u64),
    Loss(u64),
    InsufficientFunds,
}

async fn flip(
    pool: &SqlitePool,
    user_id: u64,
    guild_id: u64,
    bet: u64,
) -> Result<FlipResult, Error> {
    let current_swears = get_user_swears(pool, user_id, guild_id).await?;
    Ok(if current_swears >= bet as i64 {
        if ThreadRng::default().random_bool(1.0 / 2.0) {
            set_user_swears(pool, user_id, guild_id, current_swears + bet as i64).await?;
            FlipResult::Win(bet)
        } else {
            set_user_swears(pool, user_id, guild_id, current_swears - bet as i64).await?;
            FlipResult::Loss(bet)
        }
    } else {
        FlipResult::InsufficientFunds
    })
}

async fn construct_flip_embed(result: FlipResult, user: &User) -> Result<CreateEmbed, Error> {
    let (color, message) = match result {
        FlipResult::Win(value) => (
            Colour::from_rgb(0, 255, 0),
            format!("{} flipped a coin and won {} swears.", user.name, value),
        ),
        FlipResult::Loss(value) => (
            Colour::RED,
            format!("{} flipped a coin and lost {} swears.", user.name, value),
        ),
        FlipResult::InsufficientFunds => (
            Colour::BLUE,
            "Insufficient funds for current bet.".to_string(),
        ),
    };
    let embed = CreateEmbed::new()
        .color(color)
        .description(message)
        .author(
            CreateEmbedAuthor::new(&user.name)
                .icon_url(user.avatar_url().unwrap_or("".to_string())),
        )
        .title("Coin Flip");
    Ok(embed)
}

#[poise::command(slash_command, description_localized("en-US", "Flip a coin."))]
pub async fn coinflip(ctx: Context<'_>, #[description = "Bet"] bet: u64) -> Result<(), Error> {
    let flip_result = flip(
        &ctx.data().pool,
        ctx.author().id.get(),
        match ctx.guild_id() {
            Some(x) => x.get(),
            None => 0,
        },
        bet,
    )
    .await?;
    let embed = construct_flip_embed(flip_result, ctx.author()).await?;
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
