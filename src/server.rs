
use hyper::{Body, Request, Method, StatusCode, Response};
use std::{collections::HashMap};
use url::form_urlencoded;



static MISSING: &[u8] = b"Missing field";

pub struct Server {
    data: HashMap<String, String>
}

impl Server {
    pub fn new() -> Self {
        Server {
            data: HashMap::new()
        }
    }
    pub fn routes(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
       
        match (req.method(), req.uri().path()) {
            // Serve some instructions at /
            (&Method::GET, "/") => Ok(Response::new(Body::from(
                "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
            ))),
    
            (&Method::POST, "/join") => {
                Ok(Response::new(Body::from("JOINING!")))
            }
    
            (&Method::GET, "/data") => {
                let query = if let Some(q) = req.uri().query() {
                    q
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(MISSING.into())
                        .unwrap());
                };
                let params = form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect::<HashMap<String, String>>();
                let page = if let Some(p) = params.get("id") {
                    p
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(MISSING.into())
                        .unwrap());
                };
                let body = format!("You requested {}", page);
                Ok(Response::new(body.into()))
            }
    
            (&Method::POST, "/data") => {
                Ok(Response::new(Body::from("Fart")))
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