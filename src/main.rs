
use gemini_pro_cli::cli;
use clap::ArgMatches;
use env_logger::Env;
use google_generative_ai_rs::v1::gemini::response::GeminiResponse;
use google_generative_ai_rs::v1::gemini::request::Request;
use google_generative_ai_rs::v1::gemini::Content;
use google_generative_ai_rs::v1::gemini::Role;
use google_generative_ai_rs::v1::gemini::Part;
use log::info;
use std::io::{stdin, Read};

use google_generative_ai_rs::v1::api::Client;

async fn run(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or(
            "MY_LOG_LEVEL",
            match matches.contains_id("verbose") {
                true => "info",
                _ => "warn",
            },
        )
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    // Parse command-line arguments
    let prompt : String = (if let Some(p) = matches.value_of("prompt") {
        Some(p.to_string())
    } else {
        let mut buffer = String::new();
        if stdin().read_to_string(&mut buffer).is_ok() {
            Some(buffer.trim_end().to_string()) 
        } else {
            None
        }
    })
    .unwrap_or_else(|| {
        eprintln!("No prompt provided. Please use --prompt to specify the prompt or stdin");
        std::process::exit(1);
    });

    let config_path = matches
        .value_of("config-file")
        .unwrap_or("~/.config/gemini-cli.toml");

    let is_stream = matches.contains_id("stream");
    let config = cli::read_config(config_path).await?;

    let token = matches
        .value_of("token")
        .or(Some(config.token.as_str()))
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
                text: Some(prompt),
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
                Client::for_each_async(json_stream, move |gr: GeminiResponse| async move {
                    cli::output_response(&gr).await;
                })
                .await
            }
        }
    } else if let Some(gemini) = response.rest() {
        if let Some(text) = &gemini
            .candidates
            .first()
            .and_then(|c| c.content.parts.first().and_then(|p| p.text.as_ref()))
        {
            print!("{}", text);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // read input from stdin
    // let mut buffer = String::new();
	// io::stdin().read_line(&mut buffer).unwrap();
	// let strings = string_to_args(&buffer);
    let matches:ArgMatches = cli::create_cli_app().get_matches();

    run(matches).await
}
