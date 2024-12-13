// authors.rs
use hyper::{Body, Response};
use reqwest::Client;
use serde_json::json;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use hyper::header::{CONTENT_TYPE};

pub fn fetch_author_detail_by_url_key(client: Client, url_key: String) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let es_url = format!("{}/authors/_search", es_host);

        let query = json!({
            "query": {
                "term": {
                    "url_key": url_key
                }
            },
            "size": 1
        });

        let response = client
            .post(&es_url)
            .basic_auth(es_username, Some(es_password))
            .json(&query)
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                let body = res.json::<serde_json::Value>().await.unwrap();

                let author = body["hits"]["hits"]
                    .as_array()
                    .and_then(|hits| hits.get(0))
                    .and_then(|hit| hit["_source"].as_object())
                    .cloned();

                if let Some(author) = author {
                    Ok(Response::builder()
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(serde_json::to_string(&author).unwrap()))
                        .unwrap())
                } else {
                    Ok(Response::builder()
                        .status(404)
                        .body(Body::from("Author not found"))
                        .unwrap())
                }
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Failed to fetch author"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}