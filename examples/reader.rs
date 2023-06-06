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

use iota_client::{
    block::address::Address,
    block::output::OutputId,
    Client
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();

    let tag = "wallet-lib";
    let client = Client::builder().with_node(&std::env::var("NODE_URL").unwrap())?.finish()?;
    let addr = "rms1qzr4f5uy29mc2f92xulgez9ja0jcj349yggwze9y4mm9te6yd9np5ewahuu";
    let mut id_set: HashSet<OutputId> = HashSet::new();
    loop {
        
        let outputs =  read(&client, tag, Address::try_from_bech32(addr)?.1).await?;
        outputs.iter().for_each(|output| {
            if !id_set.contains(&output) {
                id_set.insert(output.clone());
                println!("Output ID: {output:#?}");
            }
        });

    }



    Ok(())
}