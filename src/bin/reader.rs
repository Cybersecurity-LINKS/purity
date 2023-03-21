use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::collections::{BinaryHeap, HashSet};
use iota_client::block::BlockId;
use mylib::channel::{
    read, read_addr, write_with_client, setup_with_client
};

use iota_client::{
    block::address::Address,
    block::output::OutputId,
    secret::{SecretManager},
    Client
};


// async fn sync() -> crate::Result<AccountBalance> {


//     // Add the output response to the output ids, the output response is optional, because an output could be pruned
//     // and then we can't get the metadata
//     let mut spent_or_unsynced_output_metadata_map: HashMap<OutputId, Option<OutputMetadataDto>> =
//         spent_or_not_synced_output_ids.into_iter().map(|o| (o, None)).collect();

//     for output_metadata_response in spent_or_unsynced_output_metadata_responses {
//         let output_id = output_metadata_response.output_id()?;
//         spent_or_unsynced_output_metadata_map.insert(output_id, Some(output_metadata_response));
//     }

   
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();

    let tag = "wallet-lib";
    let client = Client::builder().with_node(&std::env::var("NODE_URL").unwrap())?.finish()?;
    let addr = "rms1qzr4f5uy29mc2f92xulgez9ja0jcj349yggwze9y4mm9te6yd9np5ewahuu";
    // read(&client, tag).await?;
    let mut id_set: HashSet<OutputId> = HashSet::new();
    loop {
        
        let outputs =  read_addr(&client, tag, addr).await?;
        outputs.iter().for_each(|output| {
            if !id_set.contains(&output) {
                id_set.insert(output.clone());
                println!("Output ID: {output:#?}");
            }
        });

    }



    Ok(())
}