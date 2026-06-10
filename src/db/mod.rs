use crate::{Error, Swear};
use futures::stream::StreamExt;
use sqlx::{Row, sqlite::SqlitePool};
use tracing::info;

async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, sqlx::Error> {
    let result: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?)",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn initialize(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                swear_score BIGINT
            );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS guild_usages (
                guild_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                PRIMARY KEY (guild_id, user_id)
                FOREIGN KEY(user_id) REFERENCES users(id)
            );
        "#,
    )
    .execute(pool)
    .await?;

    if !table_exists(pool, "swears").await? {
        info!("Building Swear Table.");
        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS swears (
                    value TEXT PRIMARY KEY,
                    type INT
                );
            "#,
        )
        .execute(pool)
        .await?;
        let resp: serde_json::Value = reqwest::get("https://api.swearjar.xyz/list")
            .await?
            .json()
            .await?;
        if let Some(swears) = resp["swears"].as_array() {
            for swear in swears {
                if let Some(swear) = swear.as_str() {
                    add_swear(pool, Swear::new(0, swear.to_string()).await).await?;
                }
            }
        }
        if let Some(slurs) = resp["slurs"].as_array() {
            for slur in slurs {
                if let Some(slur) = slur.as_str() {
                    add_swear(pool, Swear::new(1, slur.to_string()).await).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn get_swear_list(pool: &SqlitePool) -> Result<Vec<Swear>, Error> {
    let mut query_result = sqlx::query(
        "
        SELECT value, type FROM swears
    ",
    )
    .fetch(pool);
    let mut swear_list = Vec::new();
    while let Some(query_row) = query_result.next().await {
        let query_row = query_row?;
        swear_list.push(Swear::new(query_row.get("type"), query_row.get("value")).await);
    }
    Ok(swear_list)
}

pub async fn add_swear(pool: &SqlitePool, swear: Swear) -> Result<(), Error> {
    sqlx::query("INSERT INTO swears (value, type) VALUES (?, ?) ON CONFLICT(value) DO NOTHING")
        .bind(swear.get_value().await)
        .bind(swear.get_id().await)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_user_swears(pool: &SqlitePool, user_id: u64, guild_id: u64) -> Result<i64, Error> {
    let query_result = sqlx::query("SELECT swear_score FROM users WHERE id = ?")
        .bind(user_id as i64)
        .fetch_optional(pool)
        .await?;
    if let Some(row) = query_result {
        Ok(row.get("swear_score"))
    } else {
        set_user_swears(pool, user_id, guild_id, 0).await?;
        Ok(0)
    }
}

pub async fn set_user_swears(
    pool: &SqlitePool,
    user_id: u64,
    guild_id: u64,
    score: i64,
) -> Result<(), Error> {
    info!("Attempting to set swears to {} for user {}", score, user_id);
    info!("Added guild_usage.");
    sqlx::query(
        r#"
        INSERT INTO users (id, swear_score)
        VALUES (?, ?)
        ON CONFLICT(id)
        DO UPDATE SET swear_score = EXCLUDED.swear_score
        "#,
    )
    .bind(user_id as i64)
    .bind(score)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO guild_usages (guild_id, user_id)
        VALUES (?, ?)
        ON CONFLICT(guild_id, user_id) DO NOTHING;
    "#,
    )
    .bind(guild_id as i64)
    .bind(user_id as i64)
    .execute(pool)
    .await?;
    info!("Finished setting swears.");
    Ok(())
}

pub async fn add_user_swears(
    pool: &SqlitePool,
    user_id: u64,
    guild_id: u64,
    score: i64,
) -> Result<i64, Error> {
    let user_swears = get_user_swears(pool, user_id, guild_id).await?;
    if (user_swears as i128 + score as i128) < i64::MAX as i128 {
        let swear_score = user_swears + score;
        set_user_swears(pool, user_id, guild_id, swear_score).await?;
        Ok(swear_score)
    } else {
        Err(format!(
            "add_user_swears for user {} ({}+{}) would result in an overflow.",
            user_id, user_swears, score
        )
        .into())
    }
}
