// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::collections::{BinaryHeap, HashSet};
use iota_client::block::BlockId;
use purity_lib::purity::{
    read, write_with_client, setup_with_client
};

use iota_client::{
    block::address::Address,
    block::output::OutputId,
    secret::{SecretManager},
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