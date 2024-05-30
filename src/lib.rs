use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Error, Pool, Sqlite};

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tide::prelude::*;

pub async fn db_init(database: &str) -> Result<Pool<Sqlite>, Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(SqliteConnectOptions::from_str(&format!("sqlite://{}", database))?.create_if_missing(true))
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bus_results (
            id text primary key,
            place_from text not null,
            place_to text not null,
            last_arriving text not null,
            last_timestamp integer not null,
            valid_arriving text,
            valid_timestamp integer,
            valid_estimate integer DEFAULT 1,
            journey_id integer not null,
            journey_no integer not null
         )",
    )
    .execute(&pool)
    .await?;

    // set up indexes?
    sqlx::query("CREATE INDEX IF NOT EXISTS index_estimate ON bus_results (valid_estimate)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[derive(Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(default)]
pub struct BusResults {
    id: String,

    place_from: String,
    place_to: String,

    last_arriving: String,
    last_timestamp: i64,

    valid_arriving: String,
    valid_timestamp: i64,
    valid_estimate: i64,
    // these are to double check against the ID
    journey_id: i32,
    journey_no: i32,
}

pub fn get_now() -> i64 {
    if let Ok(x) = SystemTime::now().duration_since(UNIX_EPOCH) {
        x.as_secs() as i64
    } else {
        0
    }
}
