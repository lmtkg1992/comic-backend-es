mod stories;
mod chapters;
mod router;

use hyper::{Body, Request, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use reqwest::Client;
use dotenv::dotenv;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = Client::new();
    let addr = ([0, 0, 0, 0], 8084).into();
    let router = Arc::new(router::Router::new()); // Shared router

    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        let router = router.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let client = client.clone();
                let router = router.clone();
                async move {
                    router.route_request(&client, req).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Starting server on port 8084...");

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}