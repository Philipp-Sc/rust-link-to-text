use serde::{Deserialize, Serialize};

pub mod socket;

use socket::{client_send_request, spawn_socket_service};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn client_send_link_to_text_request(socket_path: &str, link: String) -> anyhow::Result<LinkToTextResult> {
    println!("client_send_request initiating");
    client_send_request(socket_path,LinkToTextRequest{link})
}

#[derive(Serialize,Deserialize,Debug,Hash,Clone)]
pub struct LinkToTextRequest {
    pub link: String,
}
impl LinkToTextRequest {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}


impl TryFrom<Vec<u8>> for LinkToTextRequest {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<LinkToTextRequest> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: LinkToTextRequest) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct LinkToTextResult {
    pub result: String,
    pub request: LinkToTextRequest,
}

impl TryFrom<Vec<u8>> for LinkToTextResult {
    type Error = anyhow::Error;
    fn try_from(item: Vec<u8>) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(&item[..])?)
    }
}

impl TryFrom<LinkToTextResult> for Vec<u8> {
    type Error = anyhow::Error;
    fn try_from(item: LinkToTextResult) -> anyhow::Result<Self> {
        Ok(bincode::serialize(&item)?)
    }
}

