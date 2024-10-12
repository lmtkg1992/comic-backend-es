use hyper::{Body, Response};
use reqwest::Client;
use serde_json::json;
use std::convert::Infallible;
use std::pin::Pin;
use std::future::Future;

pub fn fetch_chapter_detail(client: Client, path_parts: Vec<String>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        if path_parts.len() < 4 {
            return Ok(Response::builder().status(400).body(Body::from("Bad Request: Missing story/chapter keys")).unwrap());
        }
        let story_key = &path_parts[2];
        let chapter_key = &path_parts[3];

        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "story_id.keyword": story_key }},
                        { "term": { "url_key.keyword": chapter_key }}
                    ]
                }
            }
        });

        let es_url = format!("{}/chapters/_search", es_host);

        let response = client
            .post(&es_url)
            .basic_auth(es_username, Some(es_password))
            .json(&query)
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                let body = res.text().await.unwrap();
                Ok(Response::new(Body::from(body)))
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Chapter not found"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}