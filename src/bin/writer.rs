use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
// use std::collections::BinaryHeap;
// use iota_client::block::BlockId;
use mylib::channel::{
    read, write_with_client, setup_with_client
};

use iota_client::{
    block::address::Address,
    secret::{SecretManager},
    Client
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();
    
    // let mut storage: Vec<BlockId> = Vec::new();

    let tag = "licat-10";
    let metadata = "this is metadata";

    let ( mut secret_manager, client, address ): (SecretManager, Client, Address) = setup_with_client().await?;

    let expiration = (SystemTime::now() + Duration::from_secs(120))
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_secs()
        .try_into()
        .unwrap();

    write_with_client(&mut secret_manager, &client, address, tag, metadata, Some(expiration)).await?;

    sleep(Duration::from_millis(7000));

    write_with_client(&mut secret_manager, &client, address, tag, metadata, None).await?;

    sleep(Duration::from_millis(5000));
    read(&client, tag).await?;
    


    Ok(())
}
