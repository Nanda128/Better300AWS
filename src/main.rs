use better300::{db_init, get_now, BusResults};
use sqlx::{Pool, Sqlite};
use std::env;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};

#[derive(Clone)]
struct State {
    db: Pool<Sqlite>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let args: Vec<String> = env::args().collect();
    let database = if args.len() > 1 { &args[1] } else { "database.db" };
    let host_port = if args.len() > 2 { &args[2] } else { "127.0.0.1:8061" };

    let connection = db_init(database).await?;

    tide::log::start();

    let state = State { db: connection };

    let mut app = tide::with_state(state);

    app.at("/").get(|_| async {
        Ok(Response::builder(StatusCode::Ok)
            .body(include_str!("index.html"))
            .content_type("text/html")
            .build())
    });

    app.at("/bus").get(results_get);

    app.listen(host_port).await?;
    Ok(())
}

async fn results_get(req: Request<State>) -> tide::Result {
    let pool = &req.state().db;

    let hour_ago = get_now() - (60 * 15);

    let results = sqlx::query_as::<_, BusResults>("SELECT * FROM bus_results WHERE valid_estimate >= ?")
        .bind(hour_ago)
        .fetch_all(pool)
        .await?;

    for result in &results {
        println!("Retrieved result: {:?}", result);
    }

    Ok(json!(results).into())
}
