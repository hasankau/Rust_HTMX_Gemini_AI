use axum::{
    http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Form, Json, Router
};
use askama::Template;
use std::sync::Mutex;
use lazy_static::lazy_static;
use google_generative_ai_rs::v1::{api::{Client, PostResult}, gemini::{request::Request, Content, Part, Role}};


lazy_static! {
    static ref TODO: Mutex<String> = Mutex::new(String::new());
}

#[derive(serde::Deserialize)]
pub struct NewTodo {
    new_to_do: String,
}

#[derive(Template)]
#[template(path = "index.html")]
#[allow(dead_code)]
struct IndexTemplate {
    greeting: String,
}

async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        greeting: "Hello, world!".into(),
    };

    match template.render() {
        Ok(html) => Html(html), // Wrap HTML content
        Err(e) => Html(e.to_string()), // Handle the error case, still as HTML
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(index))
    .route("/greet", get(greeting))
    .route("/todo", post(add_todo))
    .route("/gemini", post(response_gemini));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn greeting() -> String{
    "Greetings".to_owned()
}

 
async fn add_todo(Form(input): Form<NewTodo>) -> impl IntoResponse {
    let mut todo = TODO.lock().unwrap();
    
    todo.push_str(&input.new_to_do);
    todo.push_str("<br>");
    todo.clone()
}

async fn response_gemini(Form(input): Form<NewTodo>) -> Result<impl IntoResponse, (StatusCode, String)> {
    match request_gemini(input.new_to_do.to_string()).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn request_gemini(input: String) -> Result<String, Box<dyn std::error::Error>> {
    // Either run as a standard text request or a stream generate content request
    let client = Client::new("API_KEY".to_string());

    let txt_request = Request {
        contents: vec![Content {
            role: Role::User,
            parts: vec![Part {
                text: Some(input.to_string()),
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
    let text = match response {
        PostResult::Rest(gemini_response) => {
            gemini_response.candidates.get(0)
                .and_then(|candidate| candidate.content.parts.get(0))
                .and_then(|part| part.text.clone())
                .unwrap_or_default()
        },
        PostResult::Streamed(_) => String::new(), // Handle differently if needed
    };

    let formatted_text = text.replace("\n", "<br>");

    Ok(formatted_text)
}