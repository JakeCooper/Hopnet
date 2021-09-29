use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::lock::Mutex;
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};
use rand::Rng;

mod server;

fn rand_between(m: i32, n: i32) -> i32 {
    rand::thread_rng().gen_range(m..n)
}

#[tokio::main]
async fn main() {
    let port = match env::var("PORT") {
        Ok(v) => v,
        Err(_) => rand_between(5555, 8080).to_string()
    };
    let server_details = format!("127.0.0.1:{}", port);
    let addr: SocketAddr = server_details
        .parse()
        .expect("Unable to parse socket address");

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    
    let srv = Arc::new(Mutex::new(server::Server::new()));

    // Join lighthouse if available
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

    let server = Server::bind(&addr).serve(make_svc);


    println!("Started on port {}", port);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}