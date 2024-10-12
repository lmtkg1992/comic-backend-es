use hyper::{Body, Request, Response};
use std::convert::Infallible;
use reqwest::Client;

use crate::elasticsearch::es_story_detail;

pub async fn handle_request(client: &Client, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path().to_string();

    if path.starts_with("/es/detail/") {
        let story_id = path.trim_start_matches("/es/detail/").to_string();
        return es_story_detail(client, &story_id).await;
    }

    // Default response for unknown routes
    Ok(Response::builder()
        .status(404)
        .body(Body::from("Not Found"))
        .unwrap())
}