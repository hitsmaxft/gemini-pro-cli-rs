use google_generative_ai_rs::v1::api::PostResult;
use google_generative_ai_rs::v1::gemini::{Role, Content, Part};
use google_generative_ai_rs::v1::gemini::request::Request;
use google_generative_ai_rs::v1::api::Client;
use google_generative_ai_rs::v1::errors::GoogleAPIError;

pub struct LLMRequest<'a> {
    pub stream: bool,
    pub rich: bool,
    pub token: &'a str,
    pub prompt: Option<String>,

}

pub async fn request(client: Client, req: LLMRequest<'_>) -> Result<PostResult, GoogleAPIError> {
    let txt_request = Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: req.prompt,
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

    return client.post(30, &txt_request).await;

}