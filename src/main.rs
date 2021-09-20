use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};
use redis::AsyncCommands;
use rand::Rng;

mod server;

mod client;

fn rand_between(m: i32, n: i32) -> i32 {
    rand::thread_rng().gen_range(m..n)
}

#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let port = rand_between(5555, 8080);
    let server_details = format!("127.0.0.1:{}", port);
    let addr: SocketAddr = server_details
        .parse()
        .expect("Unable to parse socket address");

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(server::echo))
    });

    let server = Server::bind(&addr).serve(make_svc);

    let client = client::redis().await;

    let mut conn = client.get_async_connection().await.unwrap();

    let _: () = conn.set("key", "testing").await.unwrap();

    let val: Option<String> = conn.get("key").await.unwrap();

    match val {
        Some(s) => println!("Value is {}", s),
        _ => (),
    };

    println!("Started on port {}", port);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}