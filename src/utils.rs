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

use std::path::PathBuf;
use anyhow::Context;
use iota_sdk::client::Client;
use iota_sdk::client::node_api::indexer::query_parameters::QueryParameter;
use iota_sdk::crypto::keys::bip39::Mnemonic;
use iota_sdk::types::api::core::response::OutputWithMetadataResponse;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::types::block::output::dto::OutputDto;
use iota_sdk::types::block::output::feature::dto::FeatureDto;

use iota_sdk::client::stronghold::StrongholdAdapter;
use iota_sdk::client::constants::SHIMMER_COIN_TYPE;
use iota_sdk::client::secret::{stronghold::StrongholdSecretManager, SecretManager};

use iota_sdk::Wallet;
use iota_sdk::wallet::{ClientOptions, Result, Account};

pub async fn setup_secret_manager() -> Result<StrongholdAdapter> {

    // Setup Stronghold secret_manager
    let secret_manager = StrongholdSecretManager::builder()
        .password(std::env::var("STRONGHOLD_PASSWORD").unwrap())
        .build(std::env::var("STRONGHOLD_SNAPSHOT_PATH").unwrap())?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = Mnemonic::from(std::env::var("MNEMONIC").unwrap());

    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic).await?;

    Ok(secret_manager)
}

pub async fn setup_wallet(secret_manager: StrongholdAdapter) -> Result<Wallet> {

    // Create the wallet with the secret_manager and client options
    let client_options = ClientOptions::new().with_node(&std::env::var("NODE_URL").unwrap())?;

    // Create the wallet
    let wallet = Wallet::builder()
        .with_secret_manager(SecretManager::Stronghold(secret_manager))
        .with_storage_path(&std::env::var("WALLET_DB_PATH").unwrap())
        .with_client_options(client_options)
        .with_coin_type(SHIMMER_COIN_TYPE)
        .finish()
        .await?;

    Ok(wallet)
}

pub async fn create_or_recover_wallet() -> Result<Wallet> {

    let wallet = if PathBuf::from(&std::env::var("WALLET_DB_PATH").unwrap()).exists() {
        log::info!("Recovering wallet...");
        let wallet = Wallet::builder()
        .with_storage_path(&std::env::var("WALLET_DB_PATH").unwrap())
        .finish()
        .await?;

        wallet
        .set_stronghold_password(std::env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;
        
        Ok(wallet)
    } else {
        log::info!("Creating wallet...");
        let secret_manager = setup_secret_manager().await?;
        setup_wallet(secret_manager).await
    };

    wallet
}

pub fn get_metadata(o: OutputWithMetadataResponse) -> anyhow::Result<Vec<u8>> {
    match o.output {
        OutputDto::Basic(b) => { 

            for f in b.features {
                if let FeatureDto::Metadata(m) = f {
                    return Ok( hex::decode(&m.data[2..]).unwrap() );               
                }
            }
            anyhow::bail!("No FeatureDto::Metadata") 
        }
        _ => { 
            anyhow::bail!("No OutputDto of type Basic")
        }
    }
}

// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Requests funds from the faucet for the given `address`.
pub async fn request_faucet_funds(client: &Client, address: &Bech32Address, faucet_endpoint: &str) -> anyhow::Result<()> {
    iota_sdk::client::request_funds_from_faucet(faucet_endpoint, &address).await?;

    tokio::time::timeout(std::time::Duration::from_secs(45), async {
        loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let balance = get_address_balance(client, &address)
            .await
            .context("failed to get address balance")?;
        if balance > 0 {
            break;
        }
        }
        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("maximum timeout exceeded")??;

    Ok(())
}
  
/// Returns the balance of the given Bech32-encoded `address`.
pub async fn get_address_balance(client: &Client, address: &Bech32Address) -> anyhow::Result<u64> {
    let output_ids = client
        .basic_output_ids(vec![
        QueryParameter::Address(address.to_owned()),
        QueryParameter::HasExpiration(false),
        QueryParameter::HasTimelock(false),
        QueryParameter::HasStorageDepositReturn(false),
        ])
        .await?;

    let outputs = client.get_outputs(&output_ids).await?;

    let mut total_amount = 0;
    for output_response in outputs {
        total_amount += output_response.output().amount();
    }

    Ok(total_amount)
}

pub async fn print_accounts(wallet: &Wallet) -> Result<()> {
    let accounts = wallet.get_accounts().await?;
    println!("Accounts:");
    for account in accounts {
        let details = account.details().await;
        println!("- {}", details.alias());
    }
    Ok(())
}

pub async fn print_addresses(account: &Account) -> Result<()> {
    let addresses = account.addresses().await?;
    println!("{}'s addresses:", account.alias().await);
    for address in addresses {
        println!("- {}", address.address());
    }
    Ok(())
}

pub async fn sync_print_balance(account: &Account, full_report: bool) -> Result<()> {
    let alias = account.alias().await;
    let now = tokio::time::Instant::now();
    let balance = account.sync(None).await?;
    println!("{alias}'s account synced in: {:.2?}", now.elapsed());
    if full_report {
        println!("{alias}'s balance:\n{balance:#?}");
    } else {
        println!("{alias}'s coin balance:\n{:#?}", balance.base_coin());
    }
    Ok(())
}

pub async fn print_addresses_with_funds(account: &Account) -> Result<()> {
    let addresses_with_unspent_outputs = account.addresses_with_unspent_outputs().await?;
    println!(
        "{}'s addresses with funds/assets: {}",
        account.alias().await,
        addresses_with_unspent_outputs.len()
    );
    for address_with_unspent_outputs in addresses_with_unspent_outputs {
        println!("- {}", address_with_unspent_outputs.address());
        println!("  Output Ids:");
        for output_id in address_with_unspent_outputs.output_ids() {
            println!("  {}", output_id);
        }
    }
    Ok(())
}