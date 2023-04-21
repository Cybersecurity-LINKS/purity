// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

use std::{env, path::PathBuf};
use iota_client::{api_types::core::response::OutputWithMetadataResponse, stronghold::StrongholdAdapter};
use iota_client::block::output::dto::OutputDto;
use iota_client::block::output::feature::dto::FeatureDto;


use iota_wallet::{
    iota_client::constants::SHIMMER_COIN_TYPE,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    ClientOptions, Result, account_manager::AccountManager,
};

pub async fn setup_secret_manager(password: &str, path: &str, mnemonic: &str) -> Result<StrongholdAdapter> {
    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
    .password(password)
    .build(PathBuf::from(path))?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic.to_string()).await?;

    Ok(secret_manager)
}

pub async fn setup_wallet(password: &str, path: &str, mnemonic: &str) -> Result<()> {
    // Setup Stronghold secret_manager
    let mut secret_manager = StrongholdSecretManager::builder()
    .password(password)
    .build(PathBuf::from(path))?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    // The mnemonic only needs to be stored the first time
    secret_manager.store_mnemonic(mnemonic.to_string()).await?;

    // Create the account manager with the secret_manager and client options
    let client_options = ClientOptions::new()
    // .with_local_pow(false)
    .with_node(&env::var("NODE_URL")
    .unwrap())?;

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

    // println!("Generated a new account");
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
            anyhow::bail!("No FeatureDto::Metadata") 
        }
        _ => { 
            anyhow::bail!("No OutputDto of type Basic")
        }
    }
}