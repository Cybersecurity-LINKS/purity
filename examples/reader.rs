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

use std::collections::HashSet;
use purity::client::read;

use iota_sdk::{
    types::block::{output::OutputId, address::Bech32Address}, 
    client::Client
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();

    let tag = "wallet-lib";
    let client = Client::builder().with_node(&std::env::var("NODE_URL").unwrap())?.finish().await?;
    let addr = "rms1qplyhddljvsu7sx68d4gsk3sxq9zj797mvzalq09q2r9tx6yknne6gxqw26";
    let mut id_set: HashSet<OutputId> = HashSet::new();
    loop {
        
        let outputs =  read(&client, tag, Bech32Address::try_from_str(addr)?).await?;
        outputs.iter().for_each(|output| {
            if !id_set.contains(&output) {
                id_set.insert(output.clone());
                println!("Output ID: {output:#?}");
            }
        });

    }
    Ok(())
}