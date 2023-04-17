// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

//! cargo run --bin test

use std::thread::sleep;
use std::{env, path::PathBuf};
use std::time::{Instant};
use rand::prelude::*;
use dotenv::dotenv;

use iota_client::{
    utils::request_funds_from_faucet
};

use iota_wallet::{
    account_manager::AccountManager,
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, Result, account::{AccountHandle, TransactionOptions},
};

use purity::account::PurityAccountExt;
use purity::utils::setup_wallet;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut start;
    let mut duration;

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    if PathBuf::from("wallet.stronghold").exists() {
        println!("Stronghold already exists!");
    } else {
        println!("Setup stronghold");
        setup_wallet().await?;
    }

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;
    // manager.start_background_syncing(None, None).await?;
    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;
    let accounts = manager.get_accounts().await?;
    let account = manager.get_account("Alice").await?;

    account.hello();


    // Sync account to make sure account is updated with outputs from previous examples
    // Sync the account to get the outputs for the addresses
    let _ = account.sync(None).await?;

    println!("Accounts: [");
    for a in &accounts  {
        println!("  {}", a.alias().await );
        let addrs = a.addresses().await?;
        println!("  Addresses: [");
        for addr in addrs {
            println!("      {}", addr.address().to_bech32() );
        }
    }   
    println!("]");
    
    let unspent_outputs = account.unspent_outputs(None).await?;
    // println!("Unspent outputs: {unspent_outputs:#?}");
    println!("Output Ids: [");
    for o in unspent_outputs  {
        println!("{}", o.output_id)
    }   
    println!("]");
   
    println!("Total balance: {}", account.balance().await?.base_coin.total);

    let addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?;
    let addresses;
    let address;

    // if addresses_with_unspent_outputs.len() > 0 {
    //     address = addresses_with_unspent_outputs[1].address();
    // } else {
        addresses = account.generate_addresses(1, None).await?;
        address =  addresses[0].address();
    
        let faucet_response =
            request_funds_from_faucet(&env::var("FAUCET_URL").unwrap(), &address.to_bech32()).await?;
        println!("{faucet_response}");
    // }

    let printable_address = address.to_bech32(); // String::from("tst1qqe0lsnt2fk0zhvdcst9txwxm7x8dpt9vsku609gkavlvq3wutkz2jtt27n");
    println!("Generated address: {}", printable_address);
    
    let tag = "wallet-lib";
    for i in 0..2 {   
        let _ = account.sync(None).await?;
        // let s = "this is metadata";
        // let metadata = format!("{}-{}", s, i);
        // println!("{}", metadata);
        let size = 16;
        let data = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        start = Instant::now();
        
       
        let _tid = account.write_data(//write_with_wallet(
            // &account, 
            printable_address.clone(), 
            tag, 
            data, //  metadata.as_str().as_bytes().to_vec(),
            None
        ).await;
        duration = start.elapsed().as_millis();
        println!("{},{:?}",i, duration );
        // sleep(Duration::from_millis(1000));
        // account.write();
    }
    
    // Consolidate unspent outputs and print the consolidation transaction IDs
    // Set `force` to true to force the consolidation even though the `output_consolidation_threshold` isn't reached
    // let transaction = account.consolidate_outputs(true, None).await?;
    // println!("Consolidation transaction id:\n{transaction:?}\n");

    println!("end");

    Ok(())
}
