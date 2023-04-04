// Copyright (c) 2023, LINKS Foundation
// SPDX-License-Identifier: Apache-2.0
// All Rights Reserved. See LICENSE for license details.

pub mod account;
pub mod client;
pub mod utils;

pub static NODE_URL: &str = "https://api.testnet.shimmer.network";
pub static FAUCET_URL: &str = "https://faucet.testnet.shimmer.network/api/enqueue";


pub mod purity {

    use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
    use anyhow::{Context, Ok};

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

    use iota_wallet::{
        account::{AccountHandle}
    };


    pub async fn read(
        client: &Client, 
        tag: &str,
        address: Address,
    ) -> anyhow::Result<Vec<OutputId>> {

        // Get output IDs of basic outputs that can be controlled by this address without further unlock constraints.
        // start = Instant::now();
        let output_ids = client
            .basic_output_ids(vec![
                QueryParameter::Address(address.to_bech32(client.get_bech32_hrp().await?)),
                QueryParameter::Tag(format!("0x{}",hex::encode(tag))),
                // QueryParameter::HasExpiration(false),git sta
                // QueryParameter::HasTimelock(false),
                // QueryParameter::HasStorageDepositReturn(false),
            ])
            .await
            .context("failed to retrieve output ids")?;
        // duration = start.elapsed().as_millis();
        // println!("Time elapsed in client.basic_output_ids() is: {:?}", duration );
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