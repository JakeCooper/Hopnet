
use hyper::{Body, Request, Method, StatusCode, Response};
use std::net::SocketAddr;
use std::{collections::HashMap};
use std::env;
use rand::Rng;


use self::cluster::GetResponse;


static MISSING: &[u8] = b"Missing field";
static NOT_FOUND: &[u8] = b"Not Found";
static EMPTY: &[u8] = b"";
#[allow(dead_code)]
static UNAUTHORIZED: &[u8] = b"Unauthorized";

mod requests;

mod cluster;
pub struct Server {
    pub addr: std::net::SocketAddr,
    cluster: cluster::Cluster,
}

fn rand_between(m: i32, n: i32) -> i32 {
    rand::thread_rng().gen_range(m..n)
}

impl Server {
    pub async fn new() -> Self {
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
    
        Server {
            cluster: cluster::Cluster::new(addr.to_string()).await,
            addr: addr, 
        }
    }

    pub async fn routes(&mut self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        // self.data.insert(String::from("key"), "value".to_string());
        match (req.method(), req.uri().path()) {
            // Serve some instructions at /
            (&Method::GET, "/") => Ok(Response::new(Body::from(
                "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
            ))),
    
            (&Method::POST, "/join") => {
                let s: cluster::JoinRequest = requests::get_body(req).await.unwrap();
                match self.cluster.join(s).await {
                    Ok(key) => Ok(Response::new(Body::from(key))),
                    Err(e) => Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(e.to_string().into())
                        .unwrap())
                }
            }

            (&Method::POST, "/ping") => {
                let s: cluster::PingRequest = requests::get_body(req).await.unwrap();
                println!("Noise received from {}", s.address);
                Ok(Response::new("Ok!".into()))
            }

            (&Method::POST, "/depart") => {
                let s: cluster::DepartRequest = requests::get_body(req).await.unwrap();
                
                match self.cluster.depart(s.address, &s.key) {
                    Ok(_) => Ok(Response::new(Body::from(EMPTY))),
                    Err(e) => Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(e.into())
                        .unwrap())
                }
            }
    
            (&Method::GET, "/data") => {
                let params: HashMap<String, String> = requests::get_params(req).await;
                let k = params.get("key");

                if k.is_none() {
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(MISSING.into())
                        .unwrap())
                } 

                let key = k.unwrap();
                let value = self.cluster.get(key.to_string()).await;
                if value.is_none() {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(NOT_FOUND.into())
                        .unwrap())
                }

                let res = GetResponse {
                    value: value.unwrap()
                };
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&res).unwrap().into())
                    .unwrap())
           
            }
    
            (&Method::POST, "/data") => {
                let s: cluster::PutRequest = requests::get_body(req).await.unwrap();
                self.cluster.put(s.key, s.value);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(EMPTY.into())
                    .unwrap())
            }
    
            // Return the 404 Not Found for other routes.
            _ => {
                let mut not_found = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }
}