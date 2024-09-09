use std::collections::HashMap;
use std::result;
use axum::{response::Html, routing::get, Router};
use axum::extract::Query;
use axum::response::IntoResponse;
use reqwest::Error;
use serde_json::{Result, Value};
use serde::Deserialize;

#[derive(Deserialize)]
struct LatLong {
    latitude: String,
    longitude: String,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}



async fn get_weather(latitude: String, longitude: String) -> result::Result<String, Error> {
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={:}&longitude={:}&current=temperature_2m&timezone=America%2FChicago&forecast_days=1", latitude, longitude);
    println!("GET {}", url);
    let response = reqwest::get(url)
        .await?.text().await?;
    Ok(response)
}

async fn handler(Query(lat_long): Query<LatLong>) -> Html<String> {
    let latitude = lat_long.latitude;
    let longitude = lat_long.longitude;
    let json = get_weather(latitude.clone(), longitude.clone()).await.unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let current = v["current"].as_object().unwrap();
    let temp = current["temperature_2m"].as_number().unwrap();
    let html = Html(format!("<p>Current temperature at latitude {:}, longitude {:}: {} degrees Celsius.</p>", latitude, longitude, temp));
    html
}
