use better300::{db_init, get_now, BusResults};
use sqlx::{Pool, Sqlite};
use std::env;
use tide::prelude::*;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let args: Vec<String> = env::args().collect();
    let database = if args.len() > 1 { &args[1] } else { "database.db" };

    let connection = db_init(database).await?;

    get_data_controller(&connection).await;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct BusResponseDataInner {
    #[serde(rename = "JourneyID")]
    id: i64,

    #[serde(rename = "JrnyID")]
    journey_id: i32,
    #[serde(rename = "JourneyNo")]
    journey_no: i32,
    #[serde(rename = "JourneyType")]
    journey_type: u32,
    #[serde(rename = "RouteNo")]
    route_no: String,
    #[serde(rename = "LeavingIn")]
    leaving_in: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BusResponseData {
    #[serde(rename = "JourneyList")]
    journey_list: Vec<BusResponseDataInner>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BusResponse {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Data")]
    data: BusResponseData,
}

struct GetDataItem<'a> {
    from: &'a str,
    to_array: Vec<&'a str>,
}

async fn get_data_controller(pool: &Pool<Sqlite>) {
    let routes = vec![
        GetDataItem {
            from: "Kildare Village",
            to_array: vec!["Dublin City", "Ennis", "Killarney", "Tralee"],
        },
        GetDataItem {
            from: "University of Limerick",
            to_array: vec!["Dublin City", "Ennis", "Killarney", "Tralee"],
        },
        GetDataItem {
            from: "Arthurs Quay Limerick",
            to_array: vec!["Dublin City", "Ennis", "Killarney", "Tralee"],
        },
        GetDataItem {
            from: "Bunratty",
            to_array: vec!["Dublin City", "Ennis"],
        },
        GetDataItem {
            from: "Ennis",
            to_array: vec!["Dublin City"],
        },
    ];

    for route in routes {
        get_data(pool, &route).await
    }
}

async fn get_data(pool: &Pool<Sqlite>, route: &GetDataItem<'_>) {
    let current_time = get_now();

    for to in &route.to_array {
        let url = format!("https://ticketbooking.dublincoach.ie/MobileAPI/MobileBooking/GetJourneyList?FromStageName={}&ToStageName={}&JourneyType=0&RouteID=0&JrEndStageID=0&IsStageSelection=1", route.from, to);
        if let Ok(result) = surf::get(url).recv_json::<BusResponse>().await {
            if result.message != "SUCCESS" {
                println!("Failed to fetch data for route: {} to {}. Status: {}", route.from, to, result.message);
                continue;
            }

            for item in result.data.journey_list {
                if item.id == 0 {
                    println!("Skipping invalid journey with ID 0.");
                    continue;
                }

                if item.leaving_in == "No Data" {
                    println!("Inserting 'No Data' for journey {} from {} to {}.", item.id, route.from, to);
                    sqlx::query_as::<_, BusResults>(
                        r#"
                    INSERT OR REPLACE INTO bus_results (id, place_from, place_to, last_arriving, last_timestamp, journey_id, journey_no)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    ON CONFLICT(id) DO UPDATE SET
                        last_arriving=excluded.last_arriving,
                        last_timestamp=excluded.last_timestamp
                    "#,
                    )
                    .bind(format!("{}_{}_{}", item.id, route.from, to).replace(" ", "-"))
                    .bind(route.from)
                    .bind(to)
                    .bind(item.leaving_in.to_string())
                    .bind(current_time)
                    .bind(item.journey_id)
                    .bind(item.journey_no)
                    .fetch_optional(pool)
                    .await
                    .ok();
                } else {
                    let estimate = current_time + arrival_to_s(&item.leaving_in);

                    println!("Inserting journey {} from {} to {} with estimate {}.", item.id, route.from, to, estimate);
                    sqlx::query_as::<_, BusResults>(
                        r#"
                    INSERT OR REPLACE INTO bus_results (id, place_from, place_to, last_arriving, last_timestamp, journey_id, journey_no, valid_arriving, valid_timestamp, valid_estimate)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    )
                    .bind(format!("{}_{}_{}", item.id, route.from, to).replace(" ", "-"))
                    .bind(route.from)
                    .bind(to)
                    .bind(item.leaving_in.to_string())
                    .bind(current_time)
                    .bind(item.journey_id)
                    .bind(item.journey_no)
                    .bind(item.leaving_in.to_string())
                    .bind(current_time)
                    .bind(estimate)
                    .fetch_optional(pool)
                    .await
                    .ok();
                }
            }
        } else {
            println!("Failed to fetch data from URL");
        }
    }
}

fn arrival_to_s(eta: &str) -> i64 {
    let mut seconds: i64 = 0;

    let split_pos = if eta.len() == 5 { eta.len() - 5 } else { eta.len() - 6 };

    let split = eta.split_at(split_pos);

    if let Ok(result) = split.1.replace("min", "").trim().parse::<i64>() {
        seconds += result * 60;
    }

    if let Ok(result) = split.0.replace("hr", "").trim().parse::<i64>() {
        seconds += result * 60 * 60;
    }

    seconds
}
