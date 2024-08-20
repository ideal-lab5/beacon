/*
 * Copyright 2024 by Ideal Labs, LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#![allow(missing_docs)]

use subxt::{
    client::OnlineClient,
    config::SubstrateConfig,
    backend::rpc::{RpcClient, RpcParams},
};
use subxt::ext::codec::Encode;
use subxt::utils::{AccountId32, MultiAddress};
use subxt_signer::sr25519::dev;
use subxt::runtime_api::Payload;
use subxt::config::polkadot::PolkadotExtrinsicParamsBuilder as Params;
// use subxt::config::substrate::BlakeTwo256;

use beefy::{known_payloads, Payload as BeefyPayload, Commitment, VersionedFinalityProof};
use sp_core::{Bytes, Decode, blake2_256};

use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
// use ark_ff::UniformRand;
// use rand_core::OsRng;

use w3f_bls::{
    EngineBLS, Message, TinyBLS377, SerializableToBytes,
    double::{DoublePublicKey, DoubleSignature},
};

use std::env;

#[cfg(not(feature = "contract"))]
use crate::etf::randomness_beacon::calls::types::WritePulse;

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod etf {}

pub type BlockNumber = u32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎲 Ideal Network Relayer: initializing");
    let args: Vec<String> = env::args().collect();
    println!("⏳ Attempting connection to {:?}", args[1]);
    let rpc_client = RpcClient::from_url(args[1].clone()).await?;
    // let rpc_client = RpcClient::from_url("ws://localhost:9944").await?;
    // let rpc_client = RpcClient::from_url("wss://etf1.idealabs.network:443").await?;
    println!("🔗 RPC Client: connection established");
    run::<TinyBLS377>(rpc_client).await?;
    Ok(())
}

/// subscribe to justifications and interpolate signatures/aggregate proofs
/// then call the SCLC
async fn run<E: EngineBLS>(
    rpc_client: RpcClient,
) -> Result<(), Box<dyn std::error::Error>> {

    let client = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
    println!("🔍 Subscribing to ETF justifications...");

    // fetch the round public key from BEEFY runtime storage
    let round_key_query = subxt::dynamic::storage("Etf", "RoundPublic", ());
    let result = client
        .storage()
        .at_latest()
        .await?
        .fetch(&round_key_query)
        .await?;
    let round_pubkey_bytes = result.unwrap().as_type::<Vec<u8>>()?;
    // The ibe public key (in G2)
    let rk = DoublePublicKey::<E>::deserialize_compressed(
        &round_pubkey_bytes[..]
    ).unwrap();

    println!("🔑 Successfully retrieved the round public key.");
    
    let mut justification_subscription = rpc_client.subscribe::<Bytes>(
        "beefy_subscribeJustifications", 
        RpcParams::new(), 
        "beefy_unsubscribeJustifications"
    ).await?;

    while let Some(Ok(justification)) = justification_subscription.next().await {
        let recv_finality_proof: VersionedFinalityProof<u32, sp_application_crypto::bls377::Signature> =
            Decode::decode(&mut &justification[..]).unwrap(); 
        match recv_finality_proof {
            VersionedFinalityProof::V1(signed_commitment) => {
                let best_block_number = signed_commitment.commitment.block_number;
                // run every 10 blocks
                // if best_block_number % 10 == 0 {
                    // TODO: this is a single validator setup, so no interpolation
                    let sigs = signed_commitment.signatures;
                    // let sig = interpolate(sigs);
                    let primary = sigs[0].unwrap();
                    match DoubleSignature::<E>::from_bytes(&primary.to_raw()) {
                        Ok(sig) => {
                            let validator_set_id = get_validator_set_id(client.clone()).await?;
                            let payload = BeefyPayload::from_single_entry(known_payloads::ETF_SIGNATURE, Vec::new());
                            let commitment = Commitment { 
                                payload, 
                                block_number: best_block_number, 
                                validator_set_id,
                            };
                            if sig.verify(&Message::new(b"", &commitment.encode()), &rk) {
                                // call the contract here to send the signature
                                // we want to pass the interpolated signature to the contract
                                // so lets serialize it as bytes here?
                                let mut sig_bytes = Vec::new();
                                sig.serialize_compressed(&mut sig_bytes).unwrap();
                                let call_tx = publish(best_block_number, &sig_bytes);
                                
                                // let tx_params = Params::new()
                                    // .tip(1_000)
                                    // .mortal(current_block.header(), 32)
                                    // .build();


                                // let balance_transfer_tx = polkadot::tx().balances().transfer_allow_death(dest, 10_000);
                                println!("Submitting beacon pulse at block # {:?}", best_block_number);
                                let _ = client.tx().sign_and_submit_then_watch_default(&call_tx, &dev::alice())
                                    .await?
                                    .wait_for_finalized_success()
                                    .await?;
                            }
                        },
                        Err(_) => {
                            panic!("TODO: proper error handling: couldn't recover sig");
                        },
                    }
                // };
            }
        }
    }
    Ok(())
}

fn publish(
    best_block_number: BlockNumber, 
    sig_bytes: &[u8]
) -> subxt::tx::Payload<WritePulse> {
    etf::tx().randomness_beacon().write_pulse(
        vec![sig_bytes.to_vec()],
        best_block_number,
    )
}

/// construct the encoded commitment for the round in which block_number h
async fn get_validator_set_id(
    client: OnlineClient<SubstrateConfig>,
) -> Result<u64, Box<dyn std::error::Error>>  {
    let epoch_index_query = subxt::dynamic::storage("Beefy", "ValidatorSetId", ());
    let result = client.storage()
        .at_latest()
        .await?
        .fetch(&epoch_index_query)
        .await?;
    let epoch_index = result.unwrap().as_type::<u64>()?;
    
    Ok(epoch_index)
}