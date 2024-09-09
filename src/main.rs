use axum::{extract::Query, response::Html, response::IntoResponse, routing::get, Router};
use reqwest::Error;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct LatLong {
    latitude: String,
    longitude: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind listener");
    println!(
        "listening on {}",
        listener.local_addr().expect("Failed to get local address")
    );
    axum::serve(listener, app)
        .await
        .expect("Failed to serve app");
}

async fn get_weather(latitude: String, longitude: String) -> Result<String, Error> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&timezone=America%2FChicago&forecast_days=1",
        latitude, longitude
    );
    println!("GET {}", url);
    let response = reqwest::get(&url).await?.text().await?;
    Ok(response)
}

async fn handler(Query(lat_long): Query<LatLong>) -> impl IntoResponse {
    match get_weather(lat_long.latitude.clone(), lat_long.longitude.clone()).await {
        Ok(json_response) => {
            match format_response(lat_long.latitude, lat_long.longitude, &json_response) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    eprintln!("Error formatting response: {}", e);
                    Html("<p>Failed to format response</p>".to_string()).into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            Html("<p>Failed to fetch weather data</p>".to_string()).into_response()
        }
    }
}

fn format_response(latitude: String, longitude: String, json: &str) -> Result<String, String> {
    let parsed_json: Value = serde_json::from_str(json).map_err(|e| {
        format!(
            "Failed to parse returned JSON payload from weather API: {}",
            e
        )
    })?;
    let current_weather = parsed_json["current"].as_object().ok_or_else(|| {
        "Missing 'current' key in returned JSON payload from weather API".to_string()
    })?;
    let temperature = current_weather["temperature_2m"].as_f64().ok_or_else(|| {
        "Missing 'temperature_2m' key or invalid type in returned JSON payload from weather API"
            .to_string()
    })?;
    Ok(format!(
        "<p>Current temperature at latitude {}, longitude {}: {} degrees Celsius.</p>",
        latitude, longitude, temperature
    ))
}
