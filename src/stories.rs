use hyper::{Body, Response};
use reqwest::Client;
use serde_json::json;
use std::convert::Infallible;
use std::pin::Pin;
use std::future::Future;
use hyper::header::{CONTENT_TYPE};
use std::collections::HashMap;
use urlencoding::decode;

pub fn fetch_stories(client: Client, query_params: HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let page = query_params.get("page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(1);
        let size = query_params.get("size").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
        let from = (page - 1) * size;

        // Elasticsearch query construction
        let mut must_clauses = vec![];

        if let Some(title) = query_params.get("title") {
            let decoded_title = decode(title).unwrap_or_else(|_| title.to_string().into());
            must_clauses.push(json!({ "match": { "title": decoded_title } }));
        }

        if let Some(author_id) = query_params.get("author_id") {
            must_clauses.push(json!({ "term": { "author.author_id.keyword": author_id } }));
        }

        if let Some(is_full) = query_params.get("is_full") {
            if is_full == "true" {
                must_clauses.push(json!({ "term": { "is_full": true } }));
            }
        }

        let mut query = json!({
            "query": {
                "bool": {
                    "must": must_clauses
                }
            },
            "from": from,
            "size": size
        });

        // Add sorting by latest if required
        if let Some(sort_by_latest) = query_params.get("sort_by_latest") {
            if sort_by_latest == "true" {
                query["sort"] = json!([{ "updated_date": { "order": "desc" } }]);
            }
        }

       // Print the constructed Elasticsearch query for debugging
        println!("Elasticsearch Query: {}", query);

        let es_url = format!("{}/stories/_search", es_host);

        // Send the request to Elasticsearch
        let response = client
            .post(&es_url)
            .basic_auth(es_username, Some(es_password))
            .json(&query)
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                // Parse the response to extract the stories and pagination info
                let body = res.json::<serde_json::Value>().await.unwrap();

                let empty_vec = vec![];
                let stories: Vec<&serde_json::Value> = body["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec)
                    .iter()
                    .map(|hit| &hit["_source"])
                    .collect();

                let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
                let total_page = (total as f64 / size as f64).ceil() as usize;

                // Build the final response
                let response_body = json!({
                    "message": "Successfully",
                    "error": false,
                    "data": {
                        "list": stories,
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
                .body(Body::from("Failed to fetch stories"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}

pub fn fetch_stories_by_category(client: Client, category_id: String, page: usize, size: usize, sort_by_latest: bool) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        // Build Elasticsearch query
        let mut query = json!({
            "query": {
                "bool": {
                    "must": [
                        { "term": { "categories.category_id.keyword": category_id } }
                    ]
                }
            },
            "from": (page - 1) * size,
            "size": size
        });

        // Add sorting if `sort_by_latest` is true
        if sort_by_latest {
           query["sort"] = json!([{ "updated_date": { "order": "desc" } }]);
        }

        let es_url = format!("{}/stories/_search", es_host);

        // Send the request to Elasticsearch
        let response = client
            .post(&es_url)
            .basic_auth(es_username, Some(es_password))
            .json(&query)
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                // Parse the response to extract the stories and pagination info
                let body = res.json::<serde_json::Value>().await.unwrap();

                // Bind the empty vector to avoid lifetime issues
                let empty_vec = vec![];

                // Extract the list of stories and total count
                let stories: Vec<&serde_json::Value> = body["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec) // Use reference to the empty vector
                    .iter()
                    .map(|hit| &hit["_source"])
                    .collect();

                let total = body["hits"]["total"]["value"].as_u64().unwrap_or(0);
                let total_page = (total as f64 / size as f64).ceil() as usize;

                // Build the final response
                let response_body = json!({
                    "message": "Successfully",
                    "error": false,
                    "data": {
                        "list": stories,
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
                .body(Body::from("Failed to fetch stories"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}

pub fn fetch_story_detail(client: Client, path_parts: Vec<String>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        if path_parts.len() < 4 {
            return Ok(Response::builder().status(400).body(Body::from("Bad Request: Missing URL key")).unwrap());
        }

        // Extract the url_key from the last part of the path
        let url_key = &path_parts[3];

        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let query = json!({
            "query": {
                "term": {
                    "url_key": url_key
                }
            }
        });


        let es_url = format!("{}/stories/_search", es_host);

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

                // If no story is found, return "Story not found"
                if let Some(source) = source {
                    let response_body = serde_json::to_string(&source).unwrap();
                    Ok(Response::builder()
                        .header(CONTENT_TYPE, "application/json")  // Set Content-Type to application/json
                        .body(Body::from(response_body))
                        .unwrap())
                } else {
                    Ok(Response::builder()
                        .status(404)
                        .body(Body::from("Story not found"))
                        .unwrap())
                }
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Story not found"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}