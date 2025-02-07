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
use serde_json::json;
use jsonschema::{Draft, JSONSchema};
// use json_validate_core::Outputs;
use risc0_zkvm::{
    guest::env,
};
use alloy_sol_types::SolValue;

fn main() {
    let datastr : String = env::read();

    let d : serde_json::Value  = serde_json::from_str(&datastr).unwrap();
    // let s : serde_json::Value  = serde_json::from_str(&schemastr).unwrap();

    let data = json!(&d);
    // let schema = json!(&s);

    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    });

    // Compile the schema
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("A valid schema");

    // // Validate the data against the schema
    let result = compiled_schema.validate(&data);

    let mut rs: Vec<u8> = vec![0; 1];

    let number = match result {
        Err(_) => rs[0] = 0,
        Ok(_) => rs[0] = 1
    };

    assert_eq!(rs[0], 1, "{}", format!("json is not valid {:?}", data));
    
    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    // env::commit_slice(jsonstr.abi_encode().as_slice());
    env::commit_slice(rs.abi_encode().as_slice());
}
