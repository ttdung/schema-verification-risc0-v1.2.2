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
use json_validate_methods::CHECK_SCHEMA_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};
use bencher::{benchmark_main, benchmark_group, Bencher};

fn bench_prove(b: &mut Bencher) {
    let data = include_str!("../res/data.json");
    let schema = include_str!("../res/schema.json");

    let input = (data, schema);
    
    // Obtain the default prover.
    let prover = default_prover();

    b.iter(|| {
        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        // Produce a receipt by proving the specified ELF binary.
        let _receipt = prover.prove(env, CHECK_SCHEMA_ELF).unwrap().receipt;
    });

    // receipt.journal.decode().unwrap()
}

benchmark_group!(
    prove,
    bench_prove
);

benchmark_main!(prove);
