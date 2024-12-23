use hyper::{Body, Request, Response};
use reqwest::Client;
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::future::Future;
use crate::stories;
use crate::chapters;
use crate::categories;
use crate::authors;

type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>;

type Handler = Box<dyn Fn(Client, Vec<String>, HashMap<String, String>) -> ResponseFuture + Send + Sync>;

pub struct Router {
    routes: HashMap<String, Handler>,
}

impl Router {
    pub fn new() -> Self {
        let mut routes: HashMap<String, Handler> = HashMap::new();

        // STORIES ROUTERS
        // Route for fetching stories
        routes.insert("/stories/list".to_string(), Box::new(move |client, _path_parts, query_params| {
            stories::fetch_stories(client, query_params)
        }));

        // Route for fetching stories by category
        routes.insert("/stories/list_by_category".to_string(), Box::new(move |client, path_parts, query_params| {
            if path_parts.len() < 4 {
                return Box::pin(async {
                    Ok(Response::builder()
                        .status(400)
                        .body(Body::from("Bad Request: Missing category ID"))
                        .unwrap())
                });
            }
            let category_id = path_parts[3].clone();
            let page = query_params.get("page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(1);
            let size = query_params.get("size").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
            let sort_by_latest = query_params.get("sort_by_latest").map_or(false, |v| v == "true");

            stories::fetch_stories_by_category(client, category_id, page, size, sort_by_latest)
        }));

        // Add other routes like fetch_story_detail, fetch_chapter_detail, etc.
        routes.insert("/stories/detail_by_url_key".to_string(), Box::new(move |client, path_parts, _| {
            stories::fetch_story_detail(client, path_parts)
        }));

        // CHAPTERS ROUTERS
        routes.insert("/chapters/list".to_string(), Box::new(move |client, path_parts, query_params| {
            if path_parts.len() < 4 {
                return Box::pin(async {
                    Ok(Response::builder()
                        .status(400)
                        .body(Body::from("Bad Request: Missing story ID"))
                        .unwrap())
                });
            }
        
            let story_id = path_parts[3].clone();
            let page = query_params.get("page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(1);
            let size = query_params.get("size").and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
        
            chapters::fetch_chapters_by_story_id(client, story_id, page, size)
        }));

        routes.insert("/chapters/detail_by_url".to_string(), Box::new(move |client, path_parts, _| {
            chapters::fetch_chapter_detail(client, path_parts)
        }));

        // CATEGORIES ROUTERS
        routes.insert("/categories/list".to_string(), Box::new(move |client, _path_parts, query_params| {
            categories::fetch_categories(client, query_params)
        }));

        routes.insert("/categories/detail_by_url_key".to_string(), Box::new(move |client, path_parts, _query_params| {
            if path_parts.len() < 4 {
                return Box::pin(async {
                    Ok(Response::builder()
                        .status(400)
                        .body(Body::from("Bad Request: Missing URL key"))
                        .unwrap())
                });
            }
        
            let url_key = path_parts[3].clone(); // Extract the URL key from the path
            categories::fetch_category_detail_by_url_key(client, url_key)
        }));

        // AUTHORS ROUTERS
        // Route for fetching authors by URL key
        routes.insert("/authors/detail_by_url_key".to_string(), Box::new(move |client, path_parts, _query_params| {
            if path_parts.len() < 4 {
                return Box::pin(async {
                    Ok(Response::builder()
                        .status(400)
                        .body(Body::from("Bad Request: Missing URL key"))
                        .unwrap())
                });
            }
        
            let url_key = path_parts[3].clone(); // Extract the URL key from the path
            authors::fetch_author_detail_by_url_key(client, url_key)
        }));
        

        Router { routes }
    }

    pub async fn route_request(
        &self,
        client: &Client,
        req: Request<Body>
    ) -> Result<Response<Body>, Infallible> {
        let path = req.uri().path().to_string();
        let parts: Vec<String> = path.split('/').map(|s| s.to_string()).collect();
        let query_params: HashMap<String, String> = req.uri().query()
            .map(|query| {
                query.split('&')
                    .map(|pair| {
                        let mut iter = pair.split('=');
                        (iter.next().unwrap_or("").to_string(), iter.next().unwrap_or("").to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Match the path and call the corresponding handler
        for (route_path, handler) in &self.routes {
            if path.starts_with(route_path) {
                return handler(client.clone(), parts, query_params).await;
            }
        }

        // Default response for unknown routes
        Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap())
    }
}