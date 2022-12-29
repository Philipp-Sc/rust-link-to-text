use std::sync::{Arc, LockResult, Mutex};
use rust_link_to_text_socket_ipc::ipc::{LinkToTextRequest, LinkToTextResult};
use rust_link_to_text_socket_ipc::ipc::socket::{spawn_socket_service};
use async_trait::async_trait;
use tokio::task::JoinHandle;


use lazy_static::lazy_static;
use crate::cache::HashValueStore;
use crate::link_to_text;


lazy_static!{
   static ref LINK_TO_TEXT_STORE: HashValueStore = load_store("./tmp/rust_link_to_text_sled_db");
}


pub fn load_store(path: &str) -> HashValueStore {
    let db: sled::Db = sled::Config::default()
        .path(path)
        .cache_capacity(1024 * 1024 * 1024 / 2)
        .use_compression(false)
        .compression_factor(22)
        .flush_every_ms(Some(1000))
        .open().unwrap();
    HashValueStore::new(&db)
}


pub fn spawn_link_to_text_socket_service(socket_path: &str) -> JoinHandle<()> {
    println!("spawn_socket_service startup");
    let task = spawn_socket_service(socket_path,|bytes| { process(bytes)
    });
    println!("spawn_socket_service ready");
    task
}

pub async fn process(bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {

    let request: LinkToTextRequest = bytes.try_into()?;

    let hash = request.get_hash();

    let result;
    if LINK_TO_TEXT_STORE.contains_hash(hash)? {
        result = LINK_TO_TEXT_STORE.get_item_by_hash::<LinkToTextResult>(hash)?.unwrap();
    } else {
        let v = link_to_text(request.link.as_str()).await?;
        result = LinkToTextResult {
            text_nodes: v.0,
            hierarchical_segmentation: v.1,
            request,
        };
        LINK_TO_TEXT_STORE.insert_item(hash,result.clone()).ok();
    };

    let into_bytes: Vec<u8> = result.try_into()?;
    Ok(into_bytes)
}
