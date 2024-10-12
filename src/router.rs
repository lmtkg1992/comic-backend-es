use hyper::{Body, Request, Response};
use reqwest::Client;
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::future::Future;
use crate::stories;
use crate::chapters;

type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>;

type Handler = Box<dyn Fn(Client, Vec<String>) -> ResponseFuture + Send + Sync>;

pub struct Router {
    routes: HashMap<String, Handler>,
}

impl Router {
    pub fn new() -> Self {
        let mut routes: HashMap<String, Handler> = HashMap::new();

        // Move `Client` into the closure and pass it by value to `fetch_story_detail`
        routes.insert("/stories/detail_by_url_key".to_string(), Box::new(move |client: Client, path_parts: Vec<String>| {
            stories::fetch_story_detail(client, path_parts)
        }));

        // Move `Client` into the closure and pass it by value to `fetch_chapter_detail`
        routes.insert("/chapters/detail_by_url".to_string(), Box::new(move |client: Client, path_parts: Vec<String>| {
            chapters::fetch_chapter_detail(client, path_parts)
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

        // Match the path and call the corresponding handler
        for (route_path, handler) in &self.routes {
            if path.starts_with(route_path) {
                return handler(client.clone(), parts).await; // Clone `Client` here and pass by value
            }
        }

        // Default response for unknown routes
        Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap())
    }
}