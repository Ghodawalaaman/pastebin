use axum::{extract::Path, response::Html, routing::get, routing::post, Form, Router};
use http::{StatusCode, Uri};
use serde::Deserialize;
use std::fs::{read_to_string, remove_file, File};
use std::io::prelude::*;
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 1024 * 1024 * 10;

#[derive(Debug, Deserialize)]
struct Upload {
    pasted_data: String,
}

async fn handle(Path(id): Path<String>) -> (StatusCode, String) {
    match Uuid::parse_str(&id) {
        Err(e) => {
            println!("{e}");
            return (StatusCode::NOT_FOUND, String::from("invalid path\n"));
        }
        _ => {}
    }
    match read_to_string(id) {
        Ok(content) => (StatusCode::OK, content),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()),
    }
}

async fn submit_handle(upload: Form<Upload>) -> String {
    if let Form(u) = upload {
        let pasted_data = u.pasted_data;
        if pasted_data.len() > MAX_FILE_SIZE {
            // Don't store the file if it exceeds max size
            return String::from("ERROR: max size exceeded");
        }
        let path = Uuid::new_v4();
        let mut output = File::create(path.to_string()).unwrap();
        write!(output, "{}", pasted_data).unwrap();
        let mut url = String::from("https://paste1.duckdns.org/");
        url.push_str(&path.to_string());
        url.push_str("\n"); // appending a newline
        return url;
    } else {
        return "ERROR: unknown error".to_string();
    }
}

async fn delete_handle(Path(id): Path<String>) -> String {
    match remove_file(id) {
        Ok(_) => String::from("Sucessfully deleted the file"),
        Err(e) => e.to_string(),
    }
}

async fn index_page() -> Html<String> {
    let content = read_to_string("index.html").unwrap();
    Html(content)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_page))
        .route("/{id}", get(handle))
        .route("/submit", post(submit_handle))
        .route("/del/{id}", get(delete_handle))
        .fallback(fallback);

    async fn fallback(uri: Uri) -> (StatusCode, String) {
        (StatusCode::NOT_FOUND, format!("No route for {uri}"))
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
