pub mod gambling;
use crate::{
    Context, Error, Swear,
    db::{self, add_user_swears, get_user_swears, set_user_swears},
    serenity::{CreateEmbed, User},
    swear::SwearType,
};
use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbedAuthor},
};
use sqlx::Row;
use std::collections::HashMap;
#[poise::command(
    slash_command,
    description_localized("en-US", "Displays your or another user's account creation date")
)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Displays your or another user's current swears.")
)]
pub async fn swears(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!(
        "{} has {} swears.",
        u.name,
        get_user_swears(
            &ctx.data().pool,
            u.id.get(),
            match ctx.guild_id() {
                Some(x) => x.get(),
                None => 0,
            }
        )
        .await?
    );
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Adds a new swear."),
    owners_only
)]
pub async fn add_swear(
    ctx: Context<'_>,
    #[description = "Swear"] swear: String,
    #[description = "Swear Type"] swear_type: SwearType,
) -> Result<(), Error> {
    let swear = Swear::new(swear_type as i32, swear).await;
    db::add_swear(&ctx.data().pool, swear.clone()).await?;
    ctx.say("Swear Added.").await?;
    Ok(())
}

#[poise::command(
    slash_command,
    description_localized("en-US", "Get Guild Leaderboard"),
    owners_only
)]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "Max Entries"] max_entries: Option<u64>,
) -> Result<(), Error> {
    let query_results = sqlx::query(
        r#"
        SELECT gu.guild_id, gu.user_id, u.swear_score
        FROM guild_usages AS gu
        JOIN users AS u ON gu.user_id = u.id
        WHERE gu.guild_id = ?
        ORDER BY u.swear_score ASC
        LIMIT ?;
    "#,
    )
    .bind(match ctx.guild_id() {
        Some(x) => x.get(),
        None => 0,
    } as i64)
    .bind(max_entries.unwrap_or(10) as i64)
    .fetch_all(&ctx.data().pool)
    .await?;
    let mut leaderboard: HashMap<u64, u64> = HashMap::new();
    for i in query_results {
        leaderboard.insert(i.get("user_id"), i.get("swear_score"));
    }
    let content: String = leaderboard
        .iter()
        .enumerate()
        .map(|(i, x)| format!("{}. <@{}>: {} swears", i, x.0, x.1))
        .collect::<Vec<String>>()
        .join("\n");

    let embed = CreateEmbed::new()
        .title("Leaderboard")
        .description(content)
        .color(Colour::BLUE);
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, description_localized("en-US", "Send Swears"))]
pub async fn send(
    ctx: Context<'_>,
    #[description = "User to send to"] user: User,
    #[description = "Amount to send"] amount: u64,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(x) => x.get(),
        None => 0,
    };
    let author_swears = get_user_swears(&ctx.data().pool, ctx.author().id.get(), guild_id).await?;
    let embed = CreateEmbed::new().title("Send Swears").author(
        CreateEmbedAuthor::new(&ctx.author().name)
            .icon_url(ctx.author().avatar_url().unwrap_or("".to_string())),
    );

    if 0 < amount && amount <= author_swears as u64 {
        set_user_swears(
            &ctx.data().pool,
            ctx.author().id.get(),
            guild_id,
            author_swears - amount as i64,
        )
        .await?;
        add_user_swears(&ctx.data().pool, user.id.get(), guild_id, amount as i64).await?;
        let embed = embed
            .description(format!(
                "<@{}> sent <@{}> {} swears.",
                ctx.author().id.get(),
                user.id.get(),
                amount
            ))
            .color(Colour::from_rgb(0, 255, 0));
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = embed.description("Insufficient funds.").color(Colour::RED);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }
    Ok(())
}
