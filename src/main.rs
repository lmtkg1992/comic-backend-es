mod stories;
mod chapters;
mod router;

use hyper::{Body, Request, Response, Server, Method};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use reqwest::Client;
use dotenv::dotenv;
use std::sync::Arc;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

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

                // Clone the headers before the request is moved
                let accept_encoding = req.headers().get("Accept-Encoding").cloned();

                async move {
                    if req.method() == Method::OPTIONS {
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

                    // Apply gzip compression if the client supports it
                    if let Some(accept_encoding) = accept_encoding {
                        if accept_encoding.to_str().unwrap_or("").contains("gzip") {
                            // Explicitly handle the result without `?`
                            response = match gzip_response(response).await {
                                Ok(res) => res,
                                Err(_) => return Ok::<_, Infallible>(Response::builder()
                                    .status(500)
                                    .body(Body::from("Failed to compress response"))
                                    .unwrap()),
                            };
                        }
                    }

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

async fn gzip_response(mut response: Response<Body>) -> Result<Response<Body>, hyper::http::Error> {
    let body_bytes = hyper::body::to_bytes(response.body_mut()).await.unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&body_bytes).unwrap();
    let compressed_body = encoder.finish().unwrap();

    response.headers_mut().insert("Content-Encoding", "gzip".parse().unwrap());
    *response.body_mut() = Body::from(compressed_body);

    Ok(response)
}