
use std::{env, thread};
use std::time::Duration;

use rust_link_to_text::link_to_text;
use rust_link_to_text::service::spawn_link_to_text_socket_service;
use rust_link_to_text_socket_ipc::ipc::client_send_link_to_text_request;

const LINK: &'static str = "https://commonwealth.im/osmosis/discussion/8467-onboarding-of-new-major-token-pool-osmowmatic";



#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args: Vec<String> = env::args().collect();
    println!("env::args().collect(): {:?}",args);

    if args.len() <= 1 {
        let text = link_to_text(LINK).await?;
        println!("{}",text);
        Ok(())
    }else{
        match args[1].as_str() {
            "start_service" => {
                spawn_link_to_text_socket_service("./tmp/rust_link_to_text_socket").await.unwrap();
                Ok(())
            },
            "test_service" => {
                let result = client_send_link_to_text_request("./tmp/rust_link_to_text_socket",LINK.to_string())?;
                println!("{:?}",result);
                Ok(())
            }
            _ => {
                println!("invalid command");
                Ok(())
            }
        }
    }
}