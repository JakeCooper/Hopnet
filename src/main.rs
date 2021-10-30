use std::convert::Infallible;
use std::sync::Arc;
use futures::lock::Mutex;
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};

mod server;

#[tokio::main]
async fn main() {

    let srv = Arc::new(Mutex::new(server::Server::new().await));

    let make_svc = make_service_fn(|_conn| {
        let srv = srv.clone();
        let svc_fn = service_fn(move |req| {
            let srv = srv.clone();
            async move {
                srv.lock().await.routes(req).await
            }
        });
        async move { Ok::<_, Infallible>(svc_fn) }
    });

    let addr = srv.lock().await.addr.clone();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Started on port {}", addr);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}