use std::collections::HashMap;

use hyper::{Request, Body};
use hyper::body;
use serde::de::DeserializeOwned;
use anyhow::Result;
use url::form_urlencoded;

// TODO, these should have hyper errors mapped from serde
pub async fn get_body<T>(req: Request<Body>) -> Result<T>
where T:DeserializeOwned {
    let bytes = body::to_bytes(req.into_body()).await?;
    let t:T = serde_json::from_slice(&bytes)?;
    Ok(t)
}

pub async fn get_params(req: Request<Body>) -> HashMap<String, String> {
    let query = match req.uri().query() {
        Some(q) => q,
        None => return HashMap::new()
    };
    let params = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();
    return params
}