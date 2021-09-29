use std::{collections::HashMap};
use serde::{Serialize, Deserialize};
use std::env;
use rand::{Rng, distributions::Alphanumeric};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
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
        let mut c = Cluster {
            participants: HashMap::new(),
            data: HashMap::new(),
        };
        match env::var("MAGNET_URL") {
            Ok(address) => {
                match c.join(address) {
                    Ok(key) => println!("Joined lighthouse with key {}", key),
                    Err(e) => println!("Err: Failed to join lighthouse\t{}", &e)
                }
            },
            _ => (),
        }
        return c;
    }

    pub fn join(&mut self, address: String) -> Result<String, String> {
        let key = gen_key(16);
        if self.participants.get(&address).is_some() {
            return Err("Actor already present. Request /depart with your key to leave cluster".to_string())
        }
        self.participants.insert(address, key.to_string());
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

    pub fn get_participants(&self) {
        let s: Vec<String> = self.participants.keys().map(|f| f.clone()).collect();
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