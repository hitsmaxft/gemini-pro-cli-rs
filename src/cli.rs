extern crate shellexpand; // 1.0.0

use clap::{App, Arg};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::io::{stdout, AsyncWriteExt};

use google_generative_ai_rs::v1::gemini::response::{Candidate, GeminiResponse};
use google_generative_ai_rs::v1::gemini::Part;

pub async fn output_response(gemini: &GeminiResponse) -> String {
    if gemini.candidates.is_empty() {
        return "".to_string();
    }

    let first_candi: &Candidate = &gemini.candidates[0];

    if first_candi.content.parts.is_empty() {
        return "".to_string();
    }

    let first_part: &Part = &first_candi.content.parts[0];
    let may_text: &Option<String> = &first_part.text;

    match may_text {
        Some(text) => {
            let _ = stdout().write_all(text.as_bytes()).await;
            let _ = stdout().flush().await;

            "".to_string()
        }
        _ => "".to_string(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub generation_config: std::collections::HashMap<String, serde_json::Value>,
}

pub async fn read_config(input: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let real_path: &str = &shellexpand::tilde(input);

    info!("final config file path is {}", real_path);
    let contents = tokio::fs::read_to_string(real_path).await?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn create_cli_app() -> App<'static>{

    App::new("Gemini CLI")
        .version("0.1.0")
        .author("hitsmaxft")
        .about("Interacts with the Gemini model")
        .arg(
            Arg::with_name("prompt")
                .index(1)
                .value_name("PROMPT")
                .help("Sets the prompt for the Gemini model")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .help("output more logs"),
        )
        .arg(
            Arg::with_name("rich")
                .long("rich")
                .help("output the response in rich terminal"),
        )
        .arg(
            Arg::with_name("stream")
                .long("stream")
                .help("Streams the response from the model"),
        )
        .arg(
            Arg::with_name("config-file")
                .short('f')
                .long("config-file")
                .value_name("FILE")
                .help("Specify a custom TOML file for configuration")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("token")
                .long("token")
                .value_name("TOKEN")
                .help("Specify the API token directly")
                .takes_value(true),
        )
}