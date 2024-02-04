extern crate shellexpand; // 1.0.0


use log::info;
use clap::{App, Arg, ArgMatches};
use env_logger::Env;
use google_generative_ai_rs::v1::gemini::response::{Candidate, GeminiResponse};
use std::io::{stdout, Write};
use serde::{Deserialize, Serialize};

use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{request::Request, Content, Part, Role},
};

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    generation_config: std::collections::HashMap<String, serde_json::Value>,
}

async fn read_config(input: &str) -> Result<Config, Box<dyn std::error::Error>> {

    let real_path: &str = &shellexpand::tilde(input);

    info!("final config file path is {}", real_path);
    let contents = tokio::fs::read_to_string(real_path).await?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

async fn output_response(gemini: &GeminiResponse) -> String {

    if gemini.candidates.len() ==0 {
        return "".to_string();
    }

    let first_candi:&Candidate = &gemini.candidates[0];

    if first_candi.content.parts.len() == 0 {
        return "".to_string();
    }

    let first_part : &Part = &first_candi.content.parts[0];
    let may_text: &Option<String> = &first_part.text;

    match may_text  {
        Some(text ) => {
            let mut lock = stdout().lock();
            let _  = write!(lock, "{}", text);
            "".to_string()
        }
        _ => "".to_string(),
    }
}

async fn run(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {


    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", match matches.contains_id("verbose") {
           true => "info",
           _  => "warn", })
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);


    // Parse command-line arguments
    let prompt = matches.value_of("prompt").unwrap_or_else(|| {
        eprintln!("No prompt provided. Please use --prompt to specify the prompt.");
        std::process::exit(1);
    });

    let config_path = matches
        .value_of("config-file")
        .unwrap_or("~/.config/gemini-cli.toml");

    let is_stream = matches.contains_id("stream");
    let config = read_config(config_path).await?;

    let token = matches
        .value_of("token")
        .or_else(|| Some(config.token.as_str()))
        .expect("No token provided. Please use --token or configure in the TOML file.");

    let client = match is_stream {
        true => Client::new_from_model_response_type(
            google_generative_ai_rs::v1::gemini::Model::GeminiPro,
            token.to_string(),
            google_generative_ai_rs::v1::gemini::ResponseType::StreamGenerateContent,
        ),
        _ => Client::new_from_model(
            google_generative_ai_rs::v1::gemini::Model::GeminiPro,
            token.to_string(),
        ),
    };

    let txt_request = Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
        }],

        tools: vec![],
        safety_settings: vec![],
        //TODO read from config
        generation_config: None,
    };

    let response = client.post(30, &txt_request).await?;

    if is_stream {
        info!("streaming output");
        if let Some(stream_response) = response.streamed() {
            if let Some(json_stream) = stream_response.response_stream {
                Client::for_each_async(json_stream, move |gr:GeminiResponse| async move {
                    output_response(&gr).await;
                }).await
            }
        }
    } else {
        if let Some(gemini) = response.rest() {
            if let Some(text) = &gemini.candidates.get(0).and_then(|c| c.content.parts.get(0).and_then(|p| p.text.as_ref())) {
                print!("{}", text);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the interval duration

    let matches = App::new("Gemini CLI")
        .version("0.1.0")
        .author("Your Name")
        .about("Interacts with the Gemini model")
        .arg(
            Arg::with_name("prompt")
            .long("prompt")
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
        .get_matches();

    run(matches).await
}
