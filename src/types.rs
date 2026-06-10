use crate::SqlitePool;
pub struct Data {
    pub pool: SqlitePool,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
