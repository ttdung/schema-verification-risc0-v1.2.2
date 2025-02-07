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
// use json_validate_methods::{CHECK_SCHEMA_ELF,CHECK_SCHEMA_ID};
use risc0_zkvm::{Receipt};
use std::fs::File;
use std::io::Read;
pub struct Outputs {
    pub result: u32,
}
fn main() {

    // Read the JSON string back from the file
    let mut file = File::open("../json-validate/res/receipt_groth16.json").expect("failed to open");
    let mut receipt_json = String::new();
    file.read_to_string(&mut receipt_json).expect("failed to read");
  
    let new_hash_id: [u32; 8] = [3159902488, 1754129237, 2872742036, 2719751631, 866932760, 1147298780, 535036495, 1127565503];

    let receipt = serde_json::from_str::<Receipt>(&receipt_json).unwrap();
    let flag = receipt.verify(new_hash_id).unwrap();

    let output:u32 = receipt.journal.decode().unwrap();

    println!("Output {}", output);
    println!("Flag {:?}", flag)
}

/* 
fn benchmark_prove(data: &str, schema: &str) {
    // start benchmarks
    const ITER: usize = 3;
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
        let receipt  = prover.prove(env, CHECK_SCHEMA_ELF).unwrap().receipt;
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

}
*/
