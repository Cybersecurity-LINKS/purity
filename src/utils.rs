// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

use std::thread::sleep;
use std::{env, path::PathBuf};
use std::time::{Instant};
use iota_client::api_types::core::response::OutputWithMetadataResponse;
use iota_client::block::output::dto::OutputDto;
use iota_client::block::output::feature::dto::FeatureDto;
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

pub async fn setup_wallet() -> Result<()> {
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

pub fn get_metadata(o: OutputWithMetadataResponse) -> anyhow::Result<Vec<u8>> {
    match o.output {
        OutputDto::Basic(b) => { 

            for f in b.features {
                if let FeatureDto::Metadata(m) = f {
                    return Ok( hex::decode(&m.data[2..]).unwrap() );               
                }
            }
            return anyhow::bail!("No FeatureDto::Metadata"); 
        }
        _ => { 
            anyhow::bail!("No OutputDto of type Basic")
        }
    }
}