// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// use serde_json::json;
// use jsonschema::{Draft, JSONSchema};
// use json_validate_core::Outputs;
// use alloy::sol_types::SolValue;

use json_validate_methods::{CHECK_SCHEMA_ELF,CHECK_SCHEMA_ID};
// use risc0_zkvm::{default_prover, ExecutorEnv};
use risc0_zkvm::{compute_image_id,default_prover, ExecutorEnv, ProverOpts, VerifierContext, InnerReceipt, sha::Digestible};
use std::fs::File;
use std::io::Write;
use anyhow::{Result, bail, Context};
use alloy_sol_types::SolValue;
// use std::env;
// use std::time::Instant;

fn main() {
    // let data = "{\"name1\": \"John Doe\",\"age\": 23}";
    // let data = include_str!("../res/data_complex_obj.json");
    // let schema = include_str!("../res/schema_complex_obj.json");

    // let data = include_str!("../res/data_array.json");
    // let schema = include_str!("../res/schema_array.json");

    let data = include_str!("../res/data.json");
    let schema = include_str!("../res/schema.json");
    // let args: Vec<String> = env::args().collect();
    // let filename = &args[1];

    // if filename.len() == 0 {
    //     eprintln!("Error NO input file:");
    // }
    // let data = include_str!(filename);
    // println!("input {}", filename);

    // let contents = fs::read_to_string(filename)
    // .expect("Should have been able to read the file");

    // let outputs = check_schema(data, schema);
    // println!();
    // println!("validate schema result {}", outputs);

    // let _ = benchmark_prove(data, schema);
    let _ = check_schema(data, schema);
}


fn check_schema(data: &str, schema: &str) -> Result<()> {
    let input = (data, schema);
    println!("data {}", data);
    println!("schema {}", schema);

    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // // Obtain the default prover.
    let prover = default_prover();

    // // Produce a receipt by proving the specified ELF binary.
    // let receipt = prover.prove(env, CHECK_SCHEMA_ELF).unwrap().receipt;


    let receipt = prover.prove_with_ctx(
        env,
        &VerifierContext::default(),
        CHECK_SCHEMA_ELF,
        &ProverOpts::groth16(),
    )?
    .receipt;

    receipt.verify(CHECK_SCHEMA_ID).unwrap();

    // Encode the seal with the selector.
    let seal = encode_seal(&receipt)?;

    // let seal_hex_string = vec_to_hex_string(&seal);
    println!("seal hex_string: {}", hex::encode(seal));

        
    // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();

    // Decode Journal: Upon receiving the proof, the application decodes the journal to extract
    // the verified number. This ensures that the number being submitted to the blockchain matches
    // the number that was verified off-chain.


    println!("journal: {}", hex::encode(journal.clone()));

    let x = Vec::<u8>::abi_decode(&journal, true).context("decoding journal data")?;
    
    println!("journal abi_decode: {}", hex::encode(x));

    // Compute the Image ID
    let image_id = hex::encode(compute_image_id(CHECK_SCHEMA_ELF)?);

    println!("Image ID: {}", image_id);

    // Dump receipe using serde
    let receipt_json = serde_json::to_string_pretty(&receipt).unwrap();

    // Write the JSON string to a file
    let mut file = File::create("./res/receipt_groth16.json").expect("failed to create file");
    file.write_all(receipt_json.as_bytes()).expect("failed to write");

    // println!("Data written to file successfully.");

    // receipt.journal.decode.unwrap();
    Ok(())
}

// fn vec_to_hex_string(vec: &[u8]) -> String { 
//     let mut hex_string = String::from("0x"); 
//     for byte in vec { 
//         hex_string.push_str(&format!("{:02x}", byte)); 
//     } 
//     hex_string 
// }

pub fn encode_seal(receipt: &risc0_zkvm::Receipt) -> Result<Vec<u8>> {
    let seal = match receipt.inner.clone() {
        InnerReceipt::Fake(receipt) => {
            let seal = receipt.claim.digest().as_bytes().to_vec();
            let selector = &[0u8; 4];
            // Create a new vector with the capacity to hold both selector and seal
            let mut selector_seal = Vec::with_capacity(selector.len() + seal.len());
            selector_seal.extend_from_slice(selector);
            selector_seal.extend_from_slice(&seal);
            selector_seal
        }
        InnerReceipt::Groth16(receipt) => {
            let selector = &receipt.verifier_parameters.as_bytes()[..4];
            // Create a new vector with the capacity to hold both selector and seal
            let mut selector_seal = Vec::with_capacity(selector.len() + receipt.seal.len());
            selector_seal.extend_from_slice(selector);
            selector_seal.extend_from_slice(receipt.seal.as_ref());
            selector_seal
        }
        _ => bail!("Unsupported receipt type"),
    };
    Ok(seal)
}

/*
fn benchmark_prove(data: &str, schema: &str) ->Result<()>{
    // start benchmarks
    const ITER: usize = 1;
    let mut benches = Vec::new();
    let mut benches_verify = Vec::new();

    let input = (data, schema);
    for _ in 0..ITER {
        // Obtain the default prover.
        let prover = default_prover();

        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        let before = Instant::now();
        // Produce a receipt by proving the specified ELF binary.
        // let receipt  = prover.prove(env, CHECK_SCHEMA_ELF).unwrap().receipt;
        let receipt = prover.prove_with_ctx(
            env,
            &VerifierContext::default(),
            CHECK_SCHEMA_ELF,
            &ProverOpts::groth16(),
        )?
        .receipt;
        // println!("\n###### Time: {:.2?}", before.elapsed());
        benches.push(before.elapsed());

        let before_verify = Instant::now();
        receipt.verify(CHECK_SCHEMA_ID).unwrap();
        benches_verify.push(before_verify.elapsed());
    }

    println!("\n-------- BENCHMARK VERIFY ---------");
    for bench in benches_verify {
        println!("{:.2?}", bench);
    }
    println!("\n---------------------------");
    

    println!("\n-------- BENCHMARK ---------");
    for bench in benches {
        println!("{:.2?}", bench);
    }
    println!("\n---------------------------");
    // end benchmarks

    Ok(())
}
*/
#[cfg(test)]
mod tests {
    use crate::check_schema;
    #[test]
    fn success_case() {
        let data = include_str!("../res/data.json");
        let schema = include_str!("../res/schema.json");

        let outputs = check_schema(data, schema);
        assert_eq!(
            outputs.result, 1,
            "The input data is not satisfy the schema"
        );
    }
    #[test]
    fn fail_case() {
        let data = include_str!("../res/data_failcase.json");
        let schema = include_str!("../res/schema.json");

        let outputs = check_schema(data, schema);
        assert_eq!(outputs.result, 0, "The input data is satisfy the schema");
    }
}
