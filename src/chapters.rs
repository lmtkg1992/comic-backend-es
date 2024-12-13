use hyper::{Body, Response};
use reqwest::Client;
use serde_json::json;
use std::convert::Infallible;
use std::pin::Pin;
use std::future::Future;
use hyper::header::{CONTENT_TYPE};

pub fn fetch_chapters_by_story_id(client: Client, story_id: String, page: usize, size: usize) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let es_url = format!("{}/chapters/_search", es_host);

        let from = (page - 1) * size;

        let query = json!({
            "query": {
                "term": {
                    "story_id": story_id
                }
            },
            "from": from,
            "size": size,
            "sort": [{ "ordered": { "order": "asc" } }]
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

                let empty_vec = vec![];
                let chapters: Vec<serde_json::Value> = body["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec)
                    .iter()
                    .map(|hit| {
                        let source = &hit["_source"];
                        json!({
                            "chapter_id": source["chapter_id"],
                            "story_id": source["story_id"],
                            "increment_id": source["increment_id"],
                            "title": source["title"],
                            "short_title": source["short_title"],
                            "url_key": source["url_key"],
                            "ordered": source["ordered"],
                            "status": source["status"],
                            "created_date": source["created_date"],
                            "updated_date": source["updated_date"]
                        })
                    })
                    .collect();

                let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
                let total_page = (total as f64 / size as f64).ceil() as usize;

                let response_body = json!({
                    "message": "Successfully",
                    "error": false,
                    "data": {
                        "list": chapters,
                        "total": total,
                        "total_page": total_page
                    }
                });

                Ok(Response::builder()
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body.to_string()))
                    .unwrap())
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Failed to fetch chapters"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}

pub fn fetch_chapter_detail(client: Client, path_parts: Vec<String>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        if path_parts.len() < 5 {
            return Ok(Response::builder().status(400).body(Body::from("Bad Request: Missing story/chapter keys")).unwrap());
        }

        // Extract the story_key and chapter_key from the path
        let story_key = &path_parts[3];
        let chapter_key = &path_parts[4];

        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "story_url_key": story_key }},
                        { "term": { "url_key": chapter_key }}
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
                // Parse the response to extract the first _source object from hits.hits
                let body = res.json::<serde_json::Value>().await.unwrap();

                // Extract the first _source object
                let source = body["hits"]["hits"]
                    .as_array()
                    .and_then(|hits| hits.get(0))  // Get the first hit
                    .and_then(|hit| hit["_source"].as_object())  // Extract _source as an object
                    .cloned();  // Clone to move it out of Option

                // If no chapter is found, return "Chapter not found"
                if let Some(source) = source {
                    let response_body = serde_json::to_string(&source).unwrap();
                    Ok(Response::builder()
                        .header(CONTENT_TYPE, "application/json")  // Set Content-Type to application/json
                        .body(Body::from(response_body))
                        .unwrap())
                } else {
                    Ok(Response::builder()
                        .status(404)
                        .body(Body::from("Chapter not found"))
                        .unwrap())
                }
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