use sqlx::SqlitePool;

use crate::types::{DBVersion, Error};

async fn migrate_db(pool: &SqlitePool, from: &DBVersion, to: &DBVersion) -> Result<(), Error> {
    Ok(())
}
