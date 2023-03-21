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
                TimelockUnlockCondition
            },
            BasicOutputBuilder, Feature, OutputId,
           
        }, BlockId},
        utils::request_funds_from_faucet,
        block::{address::Address},
        secret::{mnemonic::MnemonicSecretManager,SecretManager},
        Client, 
        node_api::indexer::query_parameters::QueryParameter
    };

    use iota_wallet::{
        account::{AccountHandle}
    };


    pub static NODE_URL: &str = "https://api.testnet.shimmer.network";
    pub static FAUCET_URL: &str = "https://faucet.testnet.shimmer.network/api/enqueue";
    // let NODE_URL: String = std::env::var("NODE_URL").unwrap();
    // let FAUCET_URL: String = std::env::var("FAUCET_URL").unwrap();

    pub async fn setup_with_client() -> anyhow::Result<(SecretManager, Client, Address)> {
        let mut start;
        let mut duration;
    
        println!("IOTA channel tests\n\n");

        start = Instant::now();
        let client = Client::builder().with_node(&NODE_URL)?.finish()?;
        duration = start.elapsed().as_millis();
        println!("Time elapsed in Client::builder() is: {:?}", duration );

        let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(
            &std::env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC").unwrap(),
        )?);

        start = Instant::now();
        let token_supply = client.get_token_supply().await?;
        duration = start.elapsed().as_millis();
        println!("Time elapsed in client.get_token_supply() is: {:?}", duration );
        println!("Token supply: {token_supply}\n\n");

        start = Instant::now();
        let address = client.get_addresses(&secret_manager).with_range(0..1).get_raw().await?[0];
        duration = start.elapsed().as_millis();
        println!("Time elapsed in client.get_addresses() is: {:?}", duration );
        let printable_address = address.to_bech32(client.get_bech32_hrp().await?);

        println!("Address: {printable_address}");
        if token_supply < 1000 {
            start = Instant::now();
            request_funds_from_faucet(&FAUCET_URL, &printable_address).await?;
            duration = start.elapsed().as_millis();
            println!("Time elapsed in client.get_addresses() is: {:?}", duration );
        }
        Ok((secret_manager, client, address))
    }

    pub async fn write_with_client(
        secret_manager: &mut SecretManager,
        client: &Client, 
        address: Address,
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
                output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)?
                    .add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?))
                    .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?))
                    .add_feature(Feature::Sender(SenderFeature::new(address)))
                    .add_unlock_condition(UnlockCondition::Expiration(ExpirationUnlockCondition::new(address, e)?))
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                    .finish_output(token_supply)?;}, 
            None => { 
                output = BasicOutputBuilder::new_with_minimum_storage_deposit(rent_structure)?
                    .add_feature(Feature::Tag(TagFeature::new(tag.as_bytes().to_vec())?))
                    .add_feature(Feature::Metadata(MetadataFeature::new(metadata.as_bytes().to_vec())?))
                    .add_feature(Feature::Sender(SenderFeature::new(address)))
                    .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
                    .finish_output(token_supply)?;}
        }

        duration = start.elapsed().as_millis();
        println!("Time elapsed with BasicOutputBuilder is: {:?}", duration );
        
        let outputs = vec![
            output
        ];
    
        start = Instant::now();
        let block = client
            .block()
            .with_secret_manager(&secret_manager)
            .with_outputs(outputs)?
            .finish()
            .await?;
        duration = start.elapsed().as_millis();
        println!("Time elapsed in client.block() is: {:?}", duration );
    
            
        // println!("{block:#?}");
    
        println!("Transaction sent: {NODE_URL}/api/core/v2/blocks/{}", block.id());
        println!("Block metadata: {NODE_URL}/api/core/v2/blocks/{}/metadata", block.id());
        println!("Block on Explorer: {}/block/{}\n\n",
            std::env::var("EXPLORER_URL").unwrap(),
            block.id()
        );
    
        println!("Block id: {}", block.id());
            

        Ok(block.id()) // return outputid da salvare nello storage
    }

    pub async fn write_with_wallet(
        account: &AccountHandle,
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
        let rent_structure = account.client().get_rent_structure().await?;
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
            .finish_output(account.client().get_token_supply().await?)?;
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
        let return_value = match account.send(outputs, options).await {
            core::result::Result::Ok(t) => {
                duration = start.elapsed().as_millis();
                println!("Time elapsed in account.send() is: {:?}", duration );
                // Save the transaction in a variable
                transaction = Some(t);       
                
                println!("prova" );

                
                start = Instant::now();   
                account
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
    

    pub async fn read(
        client: &Client, 
        tag: &str
    ) -> anyhow::Result<()> {

        // Get output IDs of basic outputs that can be controlled by this address without further unlock constraints.
        // start = Instant::now();
        let output_ids = client
            .basic_output_ids(vec![
                //QueryParameter::Address(printable_address),
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
        let outputs_responses = client
            .get_outputs(output_ids)
            .await
            .context("failed to get outputs")?;
        // duration = start.elapsed().as_millis();
        // println!("Time elapsed in client.get_outputs() is: {:?}", duration );
        // println!("Basic outputs: {outputs_responses:#?}");
        
        println!("Block Ids: [");
        for o in outputs_responses  {
            println!("is_spent:{}, {}", o.metadata.is_spent, o.metadata.block_id );
            // match o.output {
            //     OutputDto::Basic(b) => { 
            //         let mut s = String::new();
            //         for f in b.features {
            //             if let FeatureDto::Metadata(m) = f {
            //                 let bytes = hex::decode(&m.data[2..]).unwrap();
            //                 // s = String::from_utf8(bytes).unwrap();
            //                 println!("{:?}", bytes);
            //             }
            //         }
            //         println!("{:?}", s);
                    
            //     }
            //     _ => { println!("No basic");}
            // }
        }   
        println!("]");
        Ok(())
    }

    pub async fn read_addr(
        client: &Client, 
        tag: &str,
        address: &str
    ) -> anyhow::Result<Vec<OutputId>> {

        // Get output IDs of basic outputs that can be controlled by this address without further unlock constraints.
        // start = Instant::now();
        let output_ids = client
            .basic_output_ids(vec![
                QueryParameter::Address(String::from(address)),
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