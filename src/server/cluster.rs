use std::{collections::HashMap};
use hyper::StatusCode;
use serde::{Serialize, Deserialize};
use rand::{Rng, distributions::Alphanumeric};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub local_address: String,
    pub remote_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingRequest {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartRequest {
    pub address: String,
    pub key: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResponse {
    pub value: String,
}

type Participants = HashMap<String, String>;
type Data = HashMap<String, String>;

pub struct Cluster {
    participants: Participants,
    data: Data,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClusterError {
    // Empty Address Error
    #[error("Address cannot be empty")]
    EmptyAddress,

    #[error("Actor {0} already present. Request /depart with your key to leave cluster")]
    NoopError(String),

    #[error("Failed to ping cluster")]
    ConnectionError(),

    /// Failed to fire a connection request.
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}

pub fn gen_key(length: usize) -> String {
    let action_key = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    return action_key;
}

impl Cluster {
    pub fn new() -> Self {
        return Cluster {
            participants: HashMap::new(),
            data: HashMap::new(),
        };
    }

    pub async fn join(&mut self, req: JoinRequest) -> Result<String, ClusterError> {
        let JoinRequest {remote_address, local_address} = req;
        if remote_address.len() == 0 {
            return Err(ClusterError::EmptyAddress)
        }
        let key = gen_key(16);
        if self.participants.get(&remote_address).is_some() {
            return Err(ClusterError::NoopError(remote_address))
        }
        let jr = PingRequest {
            address: local_address,
        };
        let res = reqwest::Client::new()
            .post(format!("http://{}/ping", remote_address))
            .json(&jr)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => (),
            _ => return Err(ClusterError::ConnectionError())
        }
        self.participants.insert(remote_address, key.to_string());
        return Ok(key);
    }

    pub fn depart(&mut self, address: String, key: &String) -> Result<(), String> {
        let opt = self.participants.get(&address);
        match opt {
            Some(s) => {
                if s == key {
                    self.participants.remove(&address);
                    return Ok(())
                } else {
                    return Err("Invalid".to_string())
                }
            }
            None => return Ok(())
        }
    }
    #[allow(dead_code)]
    pub fn get_participants(&self) {
        let _s: Vec<String> = self.participants.keys().map(|f| f.clone()).collect();
    }

    pub fn put(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub async fn get(&self, key: String) -> Option<String> {
        let v = self.data.get(&key);
        return match v {
            Some(val) => Some(val.to_string().clone()),
            None => { 
                for v in self.participants.keys() {
                    let via = self.participants.keys().into_iter().map(|f| format!("&via={}", urlencoding::encode(f))).collect::<String>();
                    let url = format!("http://{}/data?key={}{}", v.to_string(), &key, via);
                    let v = reqwest::get(url).await.unwrap();
                    if v.status().is_success() {
                        let result = v.text().await.unwrap();
                        let data: GetResponse = serde_json::from_str(&result).unwrap();
                        println!("{:?}", data);
                        return Some(data.value);
                    }
                }
                return None;
            },
        };
    }
}