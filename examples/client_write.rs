// Copyright 2023 Fondazione LINKS

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use purity::client::read;
use purity::client::write_with_client;
use purity::client::setup_with_client;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();
    
    // let mut storage: Vec<BlockId> = Vec::new();

    let tag = "licat-10";
    let metadata = "this is metadata";

    let ( mut secret_manager, client, address ) = setup_with_client().await?;

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
    read(&client, tag, address).await?;
    


    Ok(())
}
