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

use std::{time::Instant, env};
use anyhow::{Context, Ok};

use iota_sdk::{
    types::block::{
        address::Bech32Address,
        output::{
            feature::{TagFeature, MetadataFeature, SenderFeature},
            unlock_condition::{AddressUnlockCondition, ExpirationUnlockCondition, UnlockCondition},
            BasicOutputBuilder, Feature, OutputId, OutputWithMetadata, 
        }, 
        BlockId
    },
    client::{ 
        Client, 
        utils::request_funds_from_faucet, 
        secret::SecretManager,
        node_api::indexer::query_parameters::QueryParameter, api::GetAddressesOptions
    }
};

use crate::utils::request_faucet_funds;

pub async fn setup_with_client() -> anyhow::Result<(SecretManager, Client, Bech32Address)> {
    let mut start;
    let mut duration;
    
    println!("IOTA channel tests\n\n");

    start = Instant::now();
    let client = Client::builder().with_node(&env::var("NODE_URL").unwrap())?.finish().await?;
    duration = start.elapsed().as_millis();
    println!("Time elapsed in Client::builder() is: {:?}", duration );

    let secret_manager = SecretManager::try_from_mnemonic(std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap())?;

    start = Instant::now();
    let token_supply = client.get_token_supply().await?;
    duration = start.elapsed().as_millis();
    println!("Time elapsed in client.get_token_supply() is: {:?}", duration );
    println!("Token supply: {token_supply}\n\n");

    start = Instant::now();
    let addresses = secret_manager
        .generate_ed25519_addresses(GetAddressesOptions::from_client(&client).await?)
        .await?;
    let address = addresses[0];
    duration = start.elapsed().as_millis();
    println!("Time elapsed in client.get_addresses() is: {:?}", duration );

    println!("Address: {address}");
    if token_supply < 1000 {
        start = Instant::now();
        request_faucet_funds(&client, &address, &env::var("FAUCET_URL").unwrap()).await?;
        duration = start.elapsed().as_millis();
        println!("Time elapsed in client.get_addresses() is: {:?}", duration );
    }
    Ok((secret_manager, client, address))
}

pub async fn write_with_client(
    secret_manager: &mut SecretManager,
    client: &Client, 
    address: Bech32Address,
    tag: &str, 
    metadata: &str,
    expiration: Option<u32>
) -> anyhow::Result<BlockId> {

    // metadata = build_payload()

    let mut start;
    let mut duration;

    let rent_structure = client.get_rent_structure().await?;
    let token_supply = client.get_token_supply().await?;
    
    start = Instant::now();

    let output;
    match expiration {
        Some(e) => { 
            output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)
                .add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?))
                .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?))
                .add_feature(Feature::Sender(SenderFeature::new(address)))
                .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(address, e)?))
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                .finish_output(token_supply)?;}, 
        None => { 
            output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)
                .add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?))
                .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?))
                .add_feature(Feature::Sender(SenderFeature::new(address)))
                .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                .finish_output(token_supply)?;
        }
    }

    duration = start.elapsed().as_millis();
    println!("Time elapsed with BasicOutputBuilder is: {:?}", duration );
    
    let outputs = vec![
        output
    ];

    start = Instant::now();
    let block = client
        .build_block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;
    duration = start.elapsed().as_millis();
    println!("Time elapsed in client.block() is: {:?}", duration );

        
    // println!("{block:#?}");

    println!("Transaction sent: {}/api/core/v2/blocks/{}", &env::var("NODE_URL").unwrap(), block.id());
    println!("Block metadata: {}/api/core/v2/blocks/{}/metadata", &env::var("NODE_URL").unwrap(), block.id());
    println!("Block on Explorer: {}/block/{}\n\n",
        std::env::var("EXPLORER_URL").unwrap(),
        block.id()
    );

    println!("Block id: {}", block.id());
        

    Ok(block.id()) // return outputid da salvare nello storage
}

pub async fn read_by_tag(
    client: &Client, 
    tag: &str,
) -> anyhow::Result<Vec<OutputId>> {

    let output_ids = client
    .basic_output_ids(vec![
        QueryParameter::Tag(format!("0x{}",hex::encode(tag))),

    ])
    .await
    .context("failed to retrieve output ids")?
    .items;
    
    Ok(output_ids)
}


pub async fn read_outputs(
    client: &Client, 
    output_ids: Vec<OutputId>,
) -> anyhow::Result<Vec<OutputWithMetadata>> {

    // Get the outputs by their IDs.
    let outputs_responses = client
        .get_outputs(&output_ids)
        .await
        .context("failed to get outputs")?;
    // println!("Basic outputs: {outputs_responses:#?}");

    Ok(outputs_responses)
}

pub async fn read(
    client: &Client, 
    tag: &str,
    address: Bech32Address,
) -> anyhow::Result<Vec<OutputId>> {

    // start = Instant::now();
    let output_ids = client
        .basic_output_ids(vec![
            QueryParameter::Address(address),
            QueryParameter::Tag(format!("0x{}",hex::encode(tag))),
            // QueryParameter::HasExpiration(false),git sta
            // QueryParameter::HasTimelock(false),
            // QueryParameter::HasStorageDepositReturn(false),
        ])
        .await
        .context("failed to retrieve output ids")?
        .items;
    // println!("Address output IDs {output_ids:#?}");
    
    // Get the outputs by their IDs.
    // start = Instant::now();
    // let outputs_responses = client
    //     .get_outputs(output_ids)
    //     .await
    //     .context("failed to get outputs")?;
    // duration = start.elapsed().as_millis();
    // println!("Time elapsed in client.get_outputs() is: {:?}", duration );
    // println!("Basic outputs: {outputs_responses:#?}");
    
    // println!("Block Ids: [");
    // for o in outputs_responses  {
    //     println!("is_spent:{}, {}", o.metadata.is_spent, o.metadata.block_id );
    //     match o.output {
    //         OutputDto::Basic(b) => { 
    //             let mut s = String::new();
    //             for f in b.features {
    //                 if let FeatureDto::Metadata(m) = f {
    //                     let bytes = hex::decode(&m.data[2..]).unwrap();
    //                     // s = String::from_utf8(bytes).unwrap();
    //                     // println!("{:?}", bytes);
    //                 }
    //             }
    //              println!("{:?}", s);                    
    //         }
    //         _ => { println!("No basic");}
    //     }
    // }   
    // println!("]");
    Ok(output_ids)
}

// ESEMPIO di Output
//
// {
//     OutputWithMetadataResponse { 
//         metadata: 
//             OutputMetadataDto { 
//             block_id: "0x15afd28049463ece9c67e5fbf87caf923d1a162613672aeb883158cf8fa27642", 
//             transaction_id: "0xe914c6c420b14ce0a8a51758fe67b8a8928e73967ec53b83e8e1890e24fd1422", 
//             output_index: 0, 
//             is_spent: false,
//             milestone_index_spent: None,
//             milestone_timestamp_spent: None,
//             transaction_id_spent: None, 
//             milestone_index_booked: 4102206, 
//             milestone_timestamp_booked: 1678697677, 
//             ledger_index: 4120081 
//             }, 
//         output: 
//             Basic(BasicOutputDto { 
//                 kind: 3, 
//                 amount: "45700", 
//                 native_tokens: [], 
//                 unlock_conditions: [Address(AddressUnlockConditionDto { kind: 0, address: Ed25519(Ed25519AddressDto { kind: 0, pub_key_hash: "0x4e61a19a38edec73b62e96a76d469cce0ebd9953bb73ed98aae20cfbc3af6204" }) })], 
//                 features: [
//                     Metadata(MetadataFeatureDto { kind: 2, data: "0x74686973206973206d65746164617461" }), 
//                     Tag(TagFeatureDto { kind: 3, tag: "0x77616c6c65742d6c6962" })
//                 ] 
//         }) 
//     }
// }