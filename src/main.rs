use std::env;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AwtrixPayload {
    color: Vec<i32>,
}

#[derive(Deserialize)]
struct TrainInfoResponse {
    departures: Vec<TrainInfo>,
}

#[derive(Deserialize)]
struct TrainInfo {
    delay: Option<i32>,
    cancelled: Option<bool>,
    line: Option<TrainLine>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrainLine {
    product_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let awtrix_hostname = env::var("AWTRIX_HOSTNAME").unwrap_or_default();
    let awtrix_indicator = env::var("AWTRIX_INDICATOR").unwrap_or_default();
    let db_rest_hostname = env::var("DB_REST_HOSTNAME").unwrap_or_default();
    let db_station = env::var("DB_STATION").unwrap_or_default();
    let db_direction = env::var("DB_DIRECTION").unwrap_or_default();
    let db_duration = env::var("DB_DURATION").unwrap_or_default();

    let station_result = get_station_info(
        &db_rest_hostname, &db_station, &db_direction, &db_duration
    ).await;

    if station_result.is_ok() {
        let color = determine_color(station_result?);
        return set_indicator(&awtrix_hostname, color, &awtrix_indicator).await;
    }

    Ok(())
}

fn determine_color(station_result: TrainInfoResponse) -> Vec<i32> {
    for departure in station_result.departures {
        if departure.cancelled.is_some_and(|cancelled| cancelled == true) {
            println!("Train is cancelled!");
            return vec![255, 0, 0]
        }

        if let Some(line) = &departure.line {
            if let Some(product) = &line.product_name {
                if product == "Bus" {
                    println!("Transport type is bus!");
                    return vec![255, 0, 0]
                }
            }
        }

        let delay = departure.delay.unwrap_or_default();

        if delay >= 600 {
            println!("Delay is over second threshold!");
            return vec![255, 0, 0]
        } else if delay >= 300 {
            println!("Delay is over first threshold!");
            return vec![255, 255, 0]
        }
    }

    vec![255, 0, 0]
}

async fn get_station_info(host: &str, from: &str, direction: &str, duration: &str)  -> Result<TrainInfoResponse, Box<dyn Error>> {
    let url = format!("http://{0}/stops/{1}/departures?direction={2}&duration={3}", host, from, direction, duration);

    let client = reqwest::Client::new();
    let response = client.get(url)
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        let res: TrainInfoResponse = response.json().await?;
        Ok(res)
    } else {
        println!("Failed to fetch station info. Status: {}", response.status());
        Err("Failed to fetch station info.".into())
    }
}

async fn set_indicator(host: &str, color: Vec<i32>, indicator: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("http://{0}/api/indicator{1}", host, indicator);

    let request_body = AwtrixPayload {
        color,
    };

    let client = reqwest::Client::new();
    let response = client.post(url)
        .json(&request_body)
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        println!("Successfully updated awtrix light.");
    } else {
        println!("Failed to update awtrix light. Status: {}", response.status());
    }

    Ok(())
}