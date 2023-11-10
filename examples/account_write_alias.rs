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

//! cargo run --bin test-alias

use std::{env, path::PathBuf};
use dotenv::dotenv;

use iota_sdk::Wallet;
use iota_sdk::client::{request_funds_from_faucet, Client};
use purity::account::PurityAccountExt;
use purity::utils::{create_or_recover_wallet, print_accounts, print_addresses, sync_print_balance, print_addresses_with_funds, request_faucet_funds};

extern crate pretty_env_logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    pretty_env_logger::init();

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    // Create a client
    let client = Client::builder().with_node(&env::var("NODE_URL").unwrap())?.finish().await?;
    // Create the wallet
    let wallet = create_or_recover_wallet().await?;
    // wallet.start_background_syncing(None, None).await?;
    
    // Create a new account
    let account = wallet.get_or_create_account("Alice").await?;
    account.hello();

    print_accounts(&wallet).await?;
    print_addresses(&account).await?;

    // Sync account to make sure account is updated with outputs from previous examples
    // Sync the account to get the outputs for the addresses
    // Change to `true` to print the full balance report
    sync_print_balance(&account, false).await?;
    print_addresses_with_funds(&account).await?;

    let addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?;
    println!("Addresses with unspend outputs: \n{:?}", addresses_with_unspent_outputs);

    let address = &account.generate_ed25519_addresses(1, None).await?[0];
    println!("Generated address: {}", address.address());

    request_faucet_funds(&client, address.address(), &env::var("FAUCET_URL").unwrap()).await?;
    
    let random_tag = (0..10).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    let random_metadata = (0..10).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    let _ = account.write_alias_data(address.address(), random_tag, random_metadata, None).await;
    println!("end");

    Ok(())
}
