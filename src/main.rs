mod stories;
mod chapters;
mod router;

use hyper::{Body, Request, Server, Response, Method};
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
    let router = Arc::new(router::Router::new());

    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        let router = router.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let client = client.clone();
                let router = router.clone();
                async move {
                    if req.method() == Method::OPTIONS {
                        // Handle preflight CORS requests
                        return Ok::<_, Infallible>(Response::builder()
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                            .header("Access-Control-Allow-Headers", "Content-Type")
                            .body(Body::empty())
                            .unwrap());
                    }

                    let mut response = router.route_request(&client, req).await?;
                    // Add CORS headers to every response
                    response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                    Ok::<_, Infallible>(response)
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