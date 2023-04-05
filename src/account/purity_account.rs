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
    ) -> anyhow::Result<String>;

    async fn write_alias_data(
        &self,
        address: String,
        tag: Vec<u8>, 
        metadata: Vec<u8>,
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
    ) -> anyhow::Result<String> {
        let mut start;
        let mut duration;

        let timelock = (SystemTime::now() + Duration::from_secs(30))
            .duration_since(UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs()
            .try_into()
            .unwrap();
        start = Instant::now();
        // Send native tokens together with the required storage deposit
        let rent_structure = self.client().get_rent_structure().await?;
        duration = start.elapsed().as_millis();
        println!("Time elapsed in client.get_rent_structure() is: {:?}", duration );

        start = Instant::now();   
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
        duration = start.elapsed().as_millis();
        println!("Time elapsed in BasicOutputBuilder is: {:?}", duration );

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
    
        let mut transaction = None;
        start = Instant::now();   
        let return_value = match self.send(outputs, options).await {
            core::result::Result::Ok(t) => {
                duration = start.elapsed().as_millis();
                println!("Time elapsed in account.send() is: {:?}", duration );
                // Save the transaction in a variable
                transaction = Some(t);       
                
                println!("prova" );

                
                start = Instant::now();   
                self
                    .retry_transaction_until_included(&transaction.clone().unwrap().transaction_id, None, Some(1))
                    .await?;
                
                duration = start.elapsed().as_millis();
                println!("Time elapsed in account.retry_transaction_until_included() is: {:?}", duration );

                println!("Block on Explorer: {}/block/{}\n\n", std::env::var("EXPLORER_URL").unwrap(), transaction.clone().unwrap().block_id.expect("no block created yet"));
                transaction.unwrap().transaction_id.to_string()
            }
            Err(e) => {
                // Print the error message and throw an exception
                eprintln!("Error sending transaction: {}", e);
                e.to_string()
                //panic!("Transaction send failed");
            }
        };
    
        // println!(
        // "Transaction: {} Block sent: {}/api/core/v2/blocks/{}",
        // transaction.transaction_id,
        // &env::var("NODE_URL").unwrap(),
        // transaction.block_id.expect("no block created yet")
        // );
        println!("End write {}", return_value);
        Ok(return_value)
    }

    async fn write_alias_data(
        &self,
        address: String,
        tag: Vec<u8>, 
        metadata: Vec<u8>,
    ) -> anyhow::Result<String> {

        // let alias_options = AliasOutputOptions {
        //     address: None,
        //     immutable_metadata: Some(b"some immutable alias metadata".to_vec()),
        //     metadata: Some(b"some alias metadata".to_vec()),
        //     state_metadata: Some(b"some alias state metadata".to_vec()),
        // };
    
        let transaction = self.create_alias_output(None, None).await?;
        println!(
            "Block sent: {}",
            transaction.block_id.expect("no block created yet")
        );

        Ok("return".to_string())
    }
}
