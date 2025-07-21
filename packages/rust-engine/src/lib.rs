use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use ethereum_types::{H160, U256};
use std::collections::HashMap;

// A simplified Transaction struct, deserialized from JSON sent by Node.js.
// We rename "from" to "sender" because "from" is a Rust keyword.
#[derive(Deserialize, Serialize, Clone, Debug)]
struct Transaction {
    hash: String,
    #[serde(rename = "from")]
    sender: H160,
    to: Option<H160>,
    value: U256,
    data: String,
    #[serde(rename = "gasPrice")]
    gas_price: U256,
}

// The output struct representing a detected attack.
#[derive(Serialize, Debug)]
struct SandwichAttack {
    victim_tx_hash: String,
    attacker: H160,
    frontrun_tx_hash: String,
    backrun_tx_hash: String,
}

// A well-known address for the Uniswap V2 Router.
const UNISWAP_V2_ROUTER: H160 = H160([
    0x7a, 0x25, 0x09, 0x56, 0x80, 0x8f, 0x5c, 0x3d, 0x7c, 0x48,
    0x53, 0x74, 0x0a, 0x6d, 0x7e, 0x44, 0x4e, 0x9a, 0xce, 0xd8
]);

// This is the exported function that will be called from Node.js.
// It accepts a JSON string of a transaction batch and returns a JSON string of detected attacks.
#[wasm_bindgen]
pub fn detect_mev(tx_batch_json: &str) -> String {
    // Improves debugging by logging Rust panics to the browser console.
    console_error_panic_hook::set_once();

    let transactions: Vec<Transaction> = match serde_json::from_str(tx_batch_json) {
        Ok(txs) => txs,
        Err(_) => return "[]".to_string(), // Return empty array on parsing error
    };

    let mut detected_attacks = Vec::new();
    // Use a HashMap for faster lookups of transactions by their properties.
    let tx_map: HashMap<String, Transaction> = transactions
        .iter()
        .map(|tx| (tx.hash.clone(), tx.clone()))
        .collect();

    // This is a simplified triple loop. For production, you'd optimize this by
    // pre-sorting and indexing transactions by the assets they interact with.
    for potential_victim in &transactions {
        // Condition 1: Is this a swap on Uniswap V2?
        if potential_victim.to != Some(UNISWAP_V2_ROUTER) {
            continue;
        }

        for potential_frontrun in &transactions {
            // Condition 2: Is this a potential front-run?
            // Same destination, higher gas price, different sender.
            if potential_frontrun.to != Some(UNISWAP_V2_ROUTER) ||
               potential_frontrun.sender == potential_victim.sender ||
               potential_frontrun.gas_price <= potential_victim.gas_price {
                continue;
            }

            for potential_backrun in &transactions {
                // Condition 3: Is this a potential back-run?
                // Sender must match the front-runner.
                // Gas price must be lower than the victim's to execute after.
                if potential_backrun.sender != potential_frontrun.sender ||
                   potential_backrun.gas_price >= potential_victim.gas_price {
                    continue;
                }

                // Heuristic: A sandwich involves trading the same assets.
                // This is a simplified check. A robust solution would decode the
                // 'data' field to extract the exact token path.
                if a_b_a_path_matches(&potential_frontrun.data, &potential_backrun.data) {
                    detected_attacks.push(SandwichAttack {
                        victim_tx_hash: potential_victim.hash.clone(),
                        attacker: potential_frontrun.sender,
                        frontrun_tx_hash: potential_frontrun.hash.clone(),
                        backrun_tx_hash: potential_backrun.hash.clone(),
                    });
                }
            }
        }
    }

    serde_json::to_string(&detected_attacks).unwrap_or_else(|_| "[]".to_string())
}

// A simple heuristic to check if a front-run and back-run form an A->B->A trade pattern.
// e.g., Front-run: ETH->TOKEN_X, Back-run: TOKEN_X->ETH
fn a_b_a_path_matches(frontrun_data: &str, backrun_data: &str) -> bool {
    // A production implementation requires a proper ABI decoder.
    // Here we make a simplifying assumption: the token paths are at the end of the calldata.
    if frontrun_data.len() < 74 || backrun_data.len() < 74 {
        return false;
    }
    // Extract last two tokens in path for frontrun
    let front_token_a = &frontrun_data[frontrun_data.len() - 128..frontrun_data.len() - 64];
    let front_token_b = &frontrun_data[frontrun_data.len() - 64..];
    // Extract last two tokens in path for backrun
    let back_token_a = &backrun_data[backrun_data.len() - 128..backrun_data.len() - 64];
    let back_token_b = &backrun_data[backrun_data.len() - 64..];

    // Check if path is reversed: front(A->B) and back(B->A)
    front_token_a == back_token_b && front_token_b == back_token_a
}
