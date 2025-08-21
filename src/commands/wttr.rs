use super::Command;
use crate::register_command;

use async_trait::async_trait;

#[derive(Default)]
pub struct Wttr;

#[async_trait]
impl Command for Wttr {
    fn name(&self) -> &'static str {
        "wttr"
    }

    async fn execute(&self, args: &str, _sender: &str) -> String {
        let url = build_weather_url(args);

        match fetch_weather(&url).await {
            Ok(body) => body,
            Err(e) => {
                log::error!("Failed to fetch weather: {e}");
                String::from("Could not retrieve weather data.")
            }
        }
    }
}

fn build_weather_url(args: &str) -> String {
    let location = args.replace(' ', "+");
    let format = r#"%l: %c 🌡️%t\n"#;
    format!("https://wttr.in/{location}?format={format}&m")
}

async fn fetch_weather(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}

register_command!(Wttr);
