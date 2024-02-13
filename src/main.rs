use gemini_pro_cli::cli;
use clap::ArgMatches;
use env_logger::Env;
use gemini_pro_cli::llm;
use google_generative_ai_rs::v1::gemini::response::GeminiResponse;
use log::info;
use std::io::{stdin, Read, Write};
use tokio::io::{AsyncWriteExt};
//use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::crossterm::{cursor, ExecutableCommand};
use termimad::{FmtText, MadSkin};
use termimad::crossterm::terminal::Clear;
use termimad::crossterm::terminal::ClearType::FromCursorDown;
use std::sync::Arc;
use tokio::sync::Mutex; // 注意：我们使用的是tokio的Mutex，它对异步代码友好

use google_generative_ai_rs::v1::api::Client;


struct StreamCtx {
    pub skin:  MadSkin,
    pub buffer: String,
}

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
    let is_rich = matches.contains_id("rich");
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

    let response = llm::request(client, llm::LLMRequest {
        stream : is_stream,
        rich : is_rich,
        token : token,
        prompt : Some(prompt),
    }).await?;



    if is_rich {
        info!("output in markdown\n");
    }


    if is_stream {
        info!("streaming output");
        if let Some(stream_response) = response.streamed() {
            if let Some(json_stream) = stream_response.response_stream {

                let skin: MadSkin = MadSkin::default();

                // skin.bold.set_fg(Yellow);
                // skin.paragraph.set_fgbg(Magenta, rgb(30, 30, 40));
                // skin.italic.add_attr(Underlined);
                let holder = Arc::new(
                    Mutex::new(StreamCtx {
                        skin,
                        buffer: String::new(),
                    })
                );

                Client::for_each_async(
                    json_stream, move |gr: GeminiResponse| {
                    let holder = Arc::clone(&holder);

                    async move {
                        if let Some(chunk) = cli::get_text(&gr) {
                            if is_rich {
                                let mut holder = holder.lock().await;

                                //blocking code
                                std::io::stdout()
                                .execute(cursor::SavePosition).unwrap()
                                .execute(Clear(FromCursorDown)).unwrap();

                                let mut _buffer =  holder.buffer.clone();

                                _buffer.push_str(chunk);
                                let src: &str = _buffer.as_str();
                                let skin : &MadSkin = & mut (holder.skin);

                                let tx:FmtText = FmtText::from(&skin, src, None); 

                                print!("{}", tx);
                                std::io::stdout().flush().unwrap();
                                holder.buffer = _buffer;
                                // let _ = stdout().write_all(tx).await;
                                // let _ = stdout().flush().await;
                            } else {
                                let _ = tokio::io::stdout().write_all(chunk.as_bytes()).await;
                                let _ = tokio::io::stdout().flush().await;

                            }
                        }
                    }
                })
                .await;

                std::io::stdout().execute(cursor::RestorePosition).unwrap();
                std::io::stdout().execute(cursor::Show).unwrap();

            }
        }
    } else if let Some(gemini) = response.rest() {
        if let Some(text) = &gemini
            .candidates
            .first()
            .and_then(|c| c.content.parts.first()
            .and_then(|p| p.text.as_ref()))
        {
            if is_rich {
                termimad::print_inline(text);
            } else {
                println!("{}", text);
            }
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
