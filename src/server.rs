
use futures::lock::Mutex;
use hyper::{Body, Request, Method, StatusCode, Response};
use std::sync::Arc;
use std::{collections::HashMap};

use self::cluster::{GetResponse};

pub mod cluster;
mod requests;

static MISSING: &[u8] = b"Missing field";
static NOT_FOUND: &[u8] = b"Not Found";
static EMPTY: &[u8] = b"";
#[allow(dead_code)]
static UNAUTHORIZED: &[u8] = b"Unauthorized";

pub struct Context {
    pub cluster: Mutex<cluster::Cluster>,
}

pub async fn serve(ctx: Arc<Context>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(
            Response::builder()
                .status(StatusCode::OK)
                .body("Nothing here".into())
                .unwrap()
        ),

        (&Method::POST, "/join") => {
            let s: cluster::JoinRequest = requests::get_body(req).await.unwrap();
            match ctx.cluster.lock().await.join(s).await {
                Ok(key) => Ok(Response::new(Body::from(key))),
                Err(e) => Ok(
                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(e.to_string().into())
                        .unwrap()
                )
            }
        }

        (&Method::POST, "/ping") => {
            let s: cluster::PingRequest = requests::get_body(req).await.unwrap();
            println!("Noise received from {}", s.address);
            Ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .body("OK!".to_string().into())
                    .unwrap()
            )
        }

        (&Method::POST, "/depart") => {
            let s: cluster::DepartRequest = requests::get_body(req).await.unwrap();
            
            match ctx.cluster.lock().await.depart(s.address, &s.key) {
                Ok(_) => Ok(Response::new(Body::from(EMPTY))),
                Err(e) => Ok(
                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(e.to_string().into())
                        .unwrap()
                )
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
            let value = ctx.cluster.lock().await.get(key.to_string()).await;
            if value.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(NOT_FOUND.into())
                    .unwrap())
            }

            let res = GetResponse {
                value: value.unwrap()
            };
            return Ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&res).unwrap().into())
                    .unwrap()
            )
       
        }

        (&Method::POST, "/data") => {
            let s: cluster::PutRequest = requests::get_body(req).await.unwrap();
            ctx.cluster.lock().await.put(s.key, s.value);
            Ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(EMPTY.into())
                    .unwrap()
            )
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}