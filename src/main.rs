use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::lock::Mutex;
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};
use rand::Rng;

mod server;

use crate::server::cluster::{Cluster, JoinRequest};

use self::server::Context;

fn rand_between(m: i32, n: i32) -> i32 {
    rand::thread_rng().gen_range(m..n)
}

#[tokio::main]
async fn main() {
    let port = match env::var("PORT") {
        Ok(v) => match v.parse::<i32>() {
            Ok(v) => v,
            Err(_) => rand_between(5555, 8080),
        },
        Err(_) => rand_between(5555, 8080),
    };
    let server_details = format!("127.0.0.1:{}", port);
    let addr: SocketAddr = server_details
        .parse()
        .expect("Unable to parse socket address");
    
    let ctx = Arc::new(Context {
        cluster: Mutex::new(Cluster::new())
    });

    // Create a handle and move that to maintain ownership
    let c = &ctx;

    let make_svc = make_service_fn(move |_conn| {
        let ctx = c.clone();
        let svc_fn = service_fn(move |req| {
            let ctx = ctx.clone();
            async move {
                server::serve(ctx, req).await
            }
        });
        async { Ok::<_, Infallible>(svc_fn) }
    });


    let server = Server::bind(&addr).serve(make_svc);
    match env::var("MAGNET_URL") {
        Ok(remote_address) => {
            match ctx.cluster.lock().await.join(JoinRequest {local_address: addr.to_string(), remote_address}).await {
                Ok(key) => println!("Joined lighthouse with key {}", key),
                Err(e) => println!("Err: Failed to join lighthouse\t{}", &e)
            }
        },
        _ => (),
    };

    println!("Started on port {}", port);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}