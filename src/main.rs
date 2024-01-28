use clap::{App, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use log::{warn, info};
use env_logger::Env;
//use std::env;


use google_generative_ai_rs::v1::{
    api::{Client},
    gemini::{request::Request, response::Candidate, Content, Part, Role},
};

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    generation_config: std::collections::HashMap<String, serde_json::Value>,
}

fn read_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[tokio::main]
async fn run(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>>  {
        // Parse command-line arguments
    let prompt = matches.value_of("prompt").unwrap_or_else(|| {
        eprintln!("No prompt provided. Please use --prompt to specify the prompt.");
        std::process::exit(1);
    });

    let config_path = matches.value_of("config-file").unwrap_or("~/.config/gemini.toml");

    let stream = match matches.value_of("stream").unwrap() {
        "true" => true,
        _ => false
    }

    let config = read_config(config_path)?;

    let token = matches
        .value_of("token")
        .or_else(|| Some(config.token.as_str()))
        .expect("No token provided. Please use --token or configure in the TOML file.");

    let client = match stream {
        true => Client::new_from_model_reponse_type(google_generative_ai_rs::v1::gemini::Model::GeminiPro, token.to_string(), google_generative_ai_rs::v1::gemini::ResponseType::StreamGenerateContent) ,
        _ => Client::new_from_model(google_generative_ai_rs::v1::gemini::Model::GeminiPro, token.to_string()),
    }

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
        generation_config: None,
    };

    let response = client.post(30, &txt_request).await?;

    match stream {
        true => {
             match response.streamed() {
                Some(gemini) => ,

                _=> print!("empty response")
            },

    }
        },
        _=> {
             match response.rest() {
        Some(gemini) => 
            match &(gemini.candidates[0].content.parts[0].text) {
                Some(text) =>
                    print!("{}", text.to_string()),
                    _=>
                    print!("{}", "text is empty"),
            }
        _=> print!("empty response")

    }
},

    }

    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "warn")
    .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

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

    run(matches)
}
