// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example create_wallet --release
// In this example we will create a new wallet
// Rename `.env.example` to `.env` first

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

use mylib::channel::{
    write_with_wallet
};

async fn setup_wallet() -> Result<()> {
    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
    .password(&env::var("STRONGHOLD_PASSWORD").unwrap())
    .build(PathBuf::from("wallet.stronghold"))?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap();

    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic).await?;

    // Create the account manager with the secret_manager and client options
    let client_options = ClientOptions::new().with_node(&env::var("NODE_URL").unwrap())?;

    let manager = AccountManager::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    // Create a new account
    let _account = manager
        .create_account()
        .with_alias("Alice".to_string())
        .finish()
        .await?;

    println!("Generated a new account");
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
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
    
        // let faucet_response =
        //     request_funds_from_faucet(&env::var("FAUCET_URL").unwrap(), &address.to_bech32()).await?;
        // println!("{faucet_response}");
    // }

    let printable_address = address.to_bech32(); // String::from("tst1qqe0lsnt2fk0zhvdcst9txwxm7x8dpt9vsku609gkavlvq3wutkz2jtt27n");
    println!("Generated address: {}", printable_address);
    
    

    let tag = "wallet-lib";
    for i in 0..100 {   
        let _ = account.sync(None).await?;
        // let s = "this is metadata";
        // let metadata = format!("{}-{}", s, i);
        // println!("{}", metadata);
        let size = 16;
        let data = (0..size).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        start = Instant::now();
        
       
        let _tid = write_with_wallet(
            &account, 
            printable_address.clone(), 
            tag, 
            data, //  metadata.as_str().as_bytes().to_vec(),
            None
        ).await;
        duration = start.elapsed().as_millis();
        println!("{},{:?}",i, duration );
        // sleep(Duration::from_millis(1000));
    }
    
    // Consolidate unspent outputs and print the consolidation transaction IDs
    // Set `force` to true to force the consolidation even though the `output_consolidation_threshold` isn't reached
    // let transaction = account.consolidate_outputs(true, None).await?;
    // println!("Consolidation transaction id:\n{transaction:?}\n");

    println!("end");

    Ok(())
}