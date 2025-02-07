// Copyright 2025 RISC Zero, Inc.
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

use risc0_bigint2_methods::{EC_ADD_ELF, EC_DOUBLE_ELF, EC_MUL_ELF};
use risc0_zkvm::{
    get_prover_server, ExecutorEnv, ExecutorImpl, ExitCode, ProverOpts, VerifierContext,
};
use std::time::Instant;
use test_log::test;

use crate::ec::secp256k1::SECP256K1_PRIME;

#[test]
fn ec_add_basic() {
    let lhs: Option<[[u32; 8]; 2]> = Some([
        [
            0x16f81798, 0x59f2815b, 0x2dce28d9, 0x029bfcdb, 0xce870b07, 0x55a06295, 0xf9dcbbac,
            0x79be667e,
        ],
        [
            0xfb10d4b8, 0x9c47d08f, 0xa6855419, 0xfd17b448, 0x0e1108a8, 0x5da4fbfc, 0x26a3c465,
            0x483ada77,
        ],
    ]);

    let rhs: Option<[[u32; 8]; 2]> = Some([
        [
            0xac04dc3f, 0x9465e6a4, 0xf46d2dad, 0x5d5ac4b6, 0xad2c0db6, 0xa7c06f71, 0xe335abc9,
            0x0f66dc33,
        ],
        [
            0xd3f64d1c, 0x50650be0, 0x2a8577b0, 0xb701323c, 0x95565b00, 0x6dddd83d, 0x398fcd2c,
            0x83641fc5,
        ],
    ]);

    let expected = [
        [
            0x3db079e0, 0xd4ad0ff5, 0xdd0da7e2, 0x4faad0a4, 0x85894785, 0x280d6b36, 0xe8ab292d,
            0xa901b0db,
        ],
        [
            0x47298a9d, 0x01d0e60e, 0xa6b063b3, 0x716bc5e0, 0x61e7ae64, 0xaf6f04dc, 0x834f1a61,
            0x3f27e7e1,
        ],
    ];

    let env = ExecutorEnv::builder()
        .write(&(lhs, rhs))
        .unwrap()
        .build()
        .unwrap();
    let now = Instant::now();
    let session = ExecutorImpl::from_elf(env, EC_ADD_ELF)
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(session.exit_code, ExitCode::Halted(0));

    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    let elapsed = now.elapsed();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        Some(expected)
    );
    tracing::info!("Runtime: {}", elapsed.as_millis());
    tracing::info!("User cycles: {}", prove_info.stats.user_cycles);
}

#[test]
fn ec_double_basic() {
    let point: Option<[[u32; 8]; 2]> = Some([
        [
            0x16F81798, 0x59F2815B, 0x2DCE28D9, 0x029BFCDB, 0xCE870B07, 0x55A06295, 0xF9DCBBAC,
            0x79BE667E,
        ],
        [
            0xFB10D4B8, 0x9C47D08F, 0xA6855419, 0xFD17B448, 0x0E1108A8, 0x5DA4FBFC, 0x26A3C465,
            0x483ADA77,
        ],
    ]);

    let expected: [[u32; 8]; 2] = [
        [
            0x5C709EE5, 0xABAC09B9, 0x8CEF3CA7, 0x5C778E4B, 0x95C07CD8, 0x3045406E, 0x41ED7D6D,
            0xC6047F94,
        ],
        [
            0x50CFE52A, 0x236431A9, 0x3266D0E1, 0xF7F63265, 0x466CEAEE, 0xA3C58419, 0xA63DC339,
            0x1AE168FE,
        ],
    ];

    let env = ExecutorEnv::builder()
        .write(&point)
        .unwrap()
        .build()
        .unwrap();
    let now = Instant::now();
    let session = ExecutorImpl::from_elf(env, EC_DOUBLE_ELF)
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(session.exit_code, ExitCode::Halted(0));

    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    let elapsed = now.elapsed();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        Some(expected)
    );
    tracing::info!("Runtime: {}", elapsed.as_millis());
    tracing::info!("User cycles: {}", prove_info.stats.user_cycles);
}

#[test]
fn ec_mul() {
    let env = ExecutorEnv::builder().build().unwrap();
    let now = Instant::now();
    let session = ExecutorImpl::from_elf(env, EC_MUL_ELF)
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(session.exit_code, ExitCode::Halted(0));

    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    let elapsed = now.elapsed();
    tracing::info!("Runtime: {}", elapsed.as_millis());
    tracing::info!("User cycles: {}", prove_info.stats.user_cycles);
}

#[test]
fn ec_add_point_plus_identity() {
    let point: Option<[[u32; 8]; 2]> = Some([
        [
            0x16f81798, 0x59f2815b, 0x2dce28d9, 0x029bfcdb, 0xce870b07, 0x55a06295, 0xf9dcbbac,
            0x79be667e,
        ],
        [
            0xfb10d4b8, 0x9c47d08f, 0xa6855419, 0xfd17b448, 0x0e1108a8, 0x5da4fbfc, 0x26a3c465,
            0x483ada77,
        ],
    ]);
    let identity: Option<[[u32; 8]; 2]> = None;

    let env = ExecutorEnv::builder()
        .write(&(point, identity))
        .unwrap()
        .build()
        .unwrap();
    let session = ExecutorImpl::from_elf(env, EC_ADD_ELF)
        .unwrap()
        .run()
        .unwrap();
    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        point
    );
}

#[test]
fn ec_add_identity_plus_point() {
    let point: Option<[[u32; 8]; 2]> = Some([
        [
            0x16f81798, 0x59f2815b, 0x2dce28d9, 0x029bfcdb, 0xce870b07, 0x55a06295, 0xf9dcbbac,
            0x79be667e,
        ],
        [
            0xfb10d4b8, 0x9c47d08f, 0xa6855419, 0xfd17b448, 0x0e1108a8, 0x5da4fbfc, 0x26a3c465,
            0x483ada77,
        ],
    ]);
    let identity: Option<[[u32; 8]; 2]> = None;

    let env = ExecutorEnv::builder()
        .write(&(identity, point))
        .unwrap()
        .build()
        .unwrap();
    let session = ExecutorImpl::from_elf(env, EC_ADD_ELF)
        .unwrap()
        .run()
        .unwrap();
    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        point
    );
}

#[test]
fn ec_add_point_plus_negative() {
    let point: Option<[[u32; 8]; 2]> = Some([
        [
            0x16f81798, 0x59f2815b, 0x2dce28d9, 0x029bfcdb, 0xce870b07, 0x55a06295, 0xf9dcbbac,
            0x79be667e,
        ],
        [
            0xfb10d4b8, 0x9c47d08f, 0xa6855419, 0xfd17b448, 0x0e1108a8, 0x5da4fbfc, 0x26a3c465,
            0x483ada77,
        ],
    ]);
    let x = point.unwrap()[0];
    let neg_y = {
        let mut y = point.unwrap()[1];
        for i in 0..8 {
            // Ignoring carries for this test case.
            y[i] = SECP256K1_PRIME[i] - y[i];
        }
        y
    };
    let neg_point: Option<[[u32; 8]; 2]> = Some([x, neg_y]);

    let env = ExecutorEnv::builder()
        .write(&(point, neg_point))
        .unwrap()
        .build()
        .unwrap();
    let session = ExecutorImpl::from_elf(env, EC_ADD_ELF)
        .unwrap()
        .run()
        .unwrap();
    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        None
    );
}

#[test]
fn ec_double_identity() {
    let identity: Option<[[u32; 8]; 2]> = None;

    let env = ExecutorEnv::builder()
        .write(&identity)
        .unwrap()
        .build()
        .unwrap();
    let session = ExecutorImpl::from_elf(env, EC_DOUBLE_ELF)
        .unwrap()
        .run()
        .unwrap();
    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        None
    );
}

#[test]
fn ec_double_point_with_zero_y() {
    let point_with_zero_y: Option<[[u32; 8]; 2]> = Some([
        [
            0x16f81798, 0x59f2815b, 0x2dce28d9, 0x029bfcdb, 0xce870b07, 0x55a06295, 0xf9dcbbac,
            0x79be667e,
        ],
        [0; 8], // y = 0
    ]);

    let env = ExecutorEnv::builder()
        .write(&point_with_zero_y)
        .unwrap()
        .build()
        .unwrap();
    let session = ExecutorImpl::from_elf(env, EC_DOUBLE_ELF)
        .unwrap()
        .run()
        .unwrap();
    let prover = get_prover_server(&ProverOpts::fast()).unwrap();
    let prove_info = prover
        .prove_session(&VerifierContext::default(), &session)
        .unwrap();
    assert_eq!(
        prove_info
            .receipt
            .journal
            .decode::<Option<[[u32; 8]; 2]>>()
            .unwrap(),
        None
    );
}
