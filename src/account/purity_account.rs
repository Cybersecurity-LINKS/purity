// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

use iota_wallet::account::AccountHandle;
// use iota_wallet::account::AliasOutputOptions;

use std::{time::{Duration, SystemTime, UNIX_EPOCH, Instant}, io::SeekFrom};
use anyhow::{Context, Ok};
use async_trait::async_trait;

use iota_client::{
    block::{output::{
        feature::{TagFeature, MetadataFeature, SenderFeature},
        unlock_condition::{ 
            AddressUnlockCondition, 
            ExpirationUnlockCondition,
            UnlockCondition,
            TimelockUnlockCondition, StateControllerAddressUnlockCondition, GovernorAddressUnlockCondition
        },
        BasicOutputBuilder, Feature, OutputId, AliasOutput, RentStructure, AliasId, AliasOutputBuilder,
       
    }, BlockId},
    utils::request_funds_from_faucet,
    block::{address::{Address, self}},
    secret::{mnemonic::MnemonicSecretManager,SecretManager},
    Client, 
    node_api::indexer::query_parameters::QueryParameter, Error, Result
};

// use iota_wallet::account::operations::transaction::high_level::create_alias::AliasOutputOptionsDto;

#[async_trait]
pub trait PurityAccountExt {
    fn hello(&self);
    async fn write_data(
        &self,
        address: String,
        tag: &str, 
        metadata: Vec<u8>,
        expiration: Option<u32>
    ) -> anyhow::Result<OutputId>;

    async fn write_alias_data(
        &self,
        address: String,
        tag: Vec<u8>, 
        metadata: Vec<u8>,
        alias_id: Option<AliasId>,
    ) -> anyhow::Result<String>;
}

#[async_trait]
impl PurityAccountExt for AccountHandle {
    fn hello(&self) {
        println!("Hello Extension!");
    }

    async fn write_data(
        &self,
        address: String,
        tag: &str, 
        metadata: Vec<u8>,
        expiration: Option<u32>
    ) -> anyhow::Result<OutputId> {
        log::info!("Start write_data");
        let write_data_start_time = Instant::now();
        
        let timelock = (SystemTime::now() + Duration::from_secs(60*60))
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs()
            .try_into()
            .unwrap();
        // Send native tokens together with the required storage deposit
        let rent_structure = self.client().get_rent_structure().await?;
    
        let output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)?
            .add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?))
            .add_feature(Feature::Metadata(MetadataFeature::new(metadata)?))
            // .add_feature(Feature::Sender(SenderFeature::new(
            //     Address::try_from_bech32(&address)?.1,
            // )))
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                Address::try_from_bech32(&address)?.1,
            )))
            .add_unlock_condition(UnlockCondition::Timelock(TimelockUnlockCondition::new(timelock)?))
            .finish_output(self.client().get_token_supply().await?)?;

        let outputs = vec![
            output
        ];
    
        // Send back with custom provided input
        // let custom_input = &account.unspent_outputs(None).await?;
        
        // println!("{:?}", custom_input );
    
        // let unspent_outputs = account.unspent_outputs(None).await?;
        // println!("Unspent outputs: {unspent_outputs:#?}");
    
        // println!("Output Ids: [");
        // for o in unspent_outputs  {
        //     println!("{}", o.output_id)
        // }   
        // println!("]");
    
    
        let mut options = None; 
        // if custom_input.len() != 0 {
        //     options = Some(TransactionOptions {
        //         custom_inputs: Some(vec![custom_input[0].output_id]),
        //         ..Default::default()
        //     });
        // }
    
    
    
        //let transaction = account.send(outputs, options).await?;
        let return_value = match self.send(outputs, options).await {
            anyhow::Result::Ok(t) => {
                // Save the transaction in a variable
                           
                let _ = self
                    .retry_transaction_until_included(&t.transaction_id, None, None)
                    .await;
                println!("Block on Explorer: {}/block/{}", std::env::var("EXPLORER_URL").unwrap(), t.block_id.expect("no block created yet"));
                Ok(OutputId::new(t.transaction_id, 0 as u16)?)  // TODO: fragmentation will require something else
            } 
            Err(err) => {
                // Print the error message and throw an exception
                log::warn!("Error sending transaction: {}", err);
                anyhow::bail!(err)
                //panic!("Transaction send failed");
            }
        };
           
        log::info!("Finished write_data in {:.2?}", write_data_start_time.elapsed());
        println!("Finished write_data in {:.2?}", write_data_start_time.elapsed());
        let _ = self.sync(None).await?;
        return_value
    }

    async fn write_alias_data(
        &self,
        _address: String,
        _tag: Vec<u8>, 
        _metadata: Vec<u8>,
        alias_id: Option<AliasId>,
    ) -> anyhow::Result<String> {

        // TODO: alias_options
        // let alias_options = AliasOutputOptions {
        //     address: None,
        //     immutable_metadata: Some(b"some immutable alias metadata".to_vec()),
        //     metadata: Some(b"some alias metadata".to_vec()),
        //     state_metadata: Some(b"some alias state metadata".to_vec()),
        // };
        
        
        if let Some(_a_id) = alias_id {
            // let alias_output = self.get_alias_output(alias_id).await?; // TODO: retrieve output
            // TODO: update output
            // self.send() // TODO: publish updated output
        } else {
            // TODO: create alias output for the first time 
            let transaction = self.create_alias_output(None, None).await?;
            println!(
                "Block sent: {}",
                transaction.block_id.expect("no block created yet")
            );
        };

        let _ = self.sync(None).await?;
        Ok("return".to_string()) // TODO: return alias id (useful for the first time creation)
    }
}