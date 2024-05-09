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
use subxt::config::polkadot::PolkadotExtrinsicParamsBuilder as Params;
// use subxt::config::substrate::BlakeTwo256;

use beefy::{known_payloads, Payload, Commitment, VersionedFinalityProof};
use sp_core::{Bytes, Decode, blake2_256};

use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
// use ark_ff::UniformRand;
// use rand_core::OsRng;

use w3f_bls::{
    EngineBLS, Message, TinyBLS377, SerializableToBytes,
    double::{DoublePublicKey, DoubleSignature},
};

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod etf {}

pub const CONTRACT_ADDRESS: &str = "0x9a4527db158b9ee6f2458152cd09698bd95584fbf4a906b8fa10b057a1133418";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎲 Ideal Network Relayer: initializing");
    let rpc_client = RpcClient::from_url("ws://localhost:9944").await?;
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
                if best_block_number % 5 == 0 {
                    // TODO: this is a single validator setup, so no interpo0ion
                    let sigs = signed_commitment.signatures;
                    // let sig = interpolate(sigs);
                    let primary = sigs[0].unwrap();
                    match DoubleSignature::<E>::from_bytes(&primary.to_raw()) {
                        Ok(sig) => {
                            let validator_set_id = get_validator_set_id(client.clone()).await?;
                            let payload = Payload::from_single_entry(known_payloads::ETF_SIGNATURE, Vec::new());
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
                                let sig_hex = array_bytes::bytes2hex("0x", sig_bytes);

                                // build the call_data
                                let mut call_data = Vec::<u8>::new();
                                call_data.append(&mut (&blake2_256("write_block".as_bytes())[0..4]).to_vec());
                                call_data.append(&mut scale::Encode::encode(&(
                                    best_block_number,
                                    sig_hex
                                )));
                            
                                let pubkey: [u8;32] = array_bytes::hex2bytes_unchecked(CONTRACT_ADDRESS)
                                    .try_into().expect("The contract address must be valid.");
                                let call_tx = etf::tx().contracts().call(
                                    MultiAddress::Id(AccountId32::from(pubkey)),
                                    0, // value
                                    etf::runtime_types::sp_weights::weight_v2::Weight {
                                        ref_time: 1_000_000_000,
                                        proof_size: u64::MAX / 2,
                                    }, // gas_limit
                                    None, // storage_deposit_limit
                                    call_data,
                                );
                                
                                let tx_params = Params::new()
                                    // .tip(1_000)
                                    // .mortal(current_block.header(), 32)
                                    .build();

                                println!("Submitting transactions for block # {:?}", best_block_number);
                                let _ = client.tx().sign_and_submit(&call_tx, &dev::alice(), tx_params).await;
                            }
                        },
                        Err(_) => {
                            panic!("TODO: proper error handling: couldn't recover sig");
                        },
                    }
                };
            }
        }
    }
    Ok(())
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