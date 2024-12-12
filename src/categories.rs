use hyper::{Body, Response};
use reqwest::Client;
use serde_json::json;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::collections::HashMap;
use hyper::header::{CONTENT_TYPE};

pub fn fetch_categories(client: Client, query_params: HashMap<String, String>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        // Replace with your Elasticsearch host and credentials
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let es_url = format!("{}/categories/_search", es_host);

        let mut must_clauses = vec![];

        // Add type_category filter if present
        if let Some(type_category) = query_params.get("type_category") {
            must_clauses.push(json!({
                "term": { "type_category": type_category }
            }));
        }

        // Construct the Elasticsearch query
        let query = json!({
            "query": {
                "bool": {
                    "must": must_clauses
                }
            },
            "size": 1000 // Adjust size as needed
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
                let categories: Vec<&serde_json::Value> = body["hits"]["hits"]
                    .as_array()
                    .unwrap_or(&empty_vec)
                    .iter()
                    .map(|hit| &hit["_source"])
                    .collect();

                let response_body = json!({
                    "message": "Successfully",
                    "error": false,
                    "data": {
                        "list": categories
                    }
                });

                Ok(Response::builder()
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body.to_string()))
                    .unwrap())
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Failed to fetch categories"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}

pub fn fetch_category_detail_by_url_key(client: Client, url_key: String) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> {
    Box::pin(async move {
        // Replace with your Elasticsearch host and credentials
        let es_host = std::env::var("ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let es_username = std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".to_string());
        let es_password = std::env::var("ES_PASSWORD").unwrap_or_else(|_| "password".to_string());

        let es_url = format!("{}/categories/_search", es_host);

        // Elasticsearch query to fetch the category by `url_key`
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

                let category = body["hits"]["hits"]
                    .as_array()
                    .and_then(|hits| hits.get(0))  // Get the first result
                    .and_then(|hit| hit["_source"].as_object())  // Extract _source as an object
                    .cloned();  // Clone the object

                if let Some(category) = category {
                    Ok(Response::builder()
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(serde_json::to_string(&category).unwrap()))
                        .unwrap())
                } else {
                    Ok(Response::builder()
                        .status(404)
                        .body(Body::from("Category not found"))
                        .unwrap())
                }
            }
            Ok(res) => Ok(Response::builder()
                .status(res.status())
                .body(Body::from("Failed to fetch category"))
                .unwrap()),
            Err(err) => Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("Elasticsearch error: {:?}", err)))
                .unwrap()),
        }
    })
}