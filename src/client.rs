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

use iota_wallet::account::AccountHandle;

use crate::NODE_URL;
use crate::FAUCET_URL;



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