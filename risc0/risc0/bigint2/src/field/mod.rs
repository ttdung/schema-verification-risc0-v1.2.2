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

#[cfg(test)]
mod tests;

use include_bytes_aligned::include_bytes_aligned;

use crate::ffi::{sys_bigint2_3, sys_bigint2_4};
use crate::WORD_SIZE;

pub const FIELD_256_WIDTH_WORDS: usize = 256 / (WORD_SIZE * 8);

const MODADD_256_BLOB: &[u8] = include_bytes_aligned!(4, "modadd_256.blob");
const MODINV_256_BLOB: &[u8] = include_bytes_aligned!(4, "modinv_256.blob");
const MODMUL_256_BLOB: &[u8] = include_bytes_aligned!(4, "modmul_256.blob");
const MODSUB_256_BLOB: &[u8] = include_bytes_aligned!(4, "modsub_256.blob");

// These "unchecked" modular arithmetic operations provide no guarantee that `result < modulus`
// This can be acceptable when computing internal results during a series of finite field
// operations, but will not work for other use cases (e.g. comparing to a hash value).

pub fn modadd_256_unchecked(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    unsafe {
        sys_bigint2_4(
            MODADD_256_BLOB.as_ptr(),
            lhs.as_ptr() as *const u32,
            rhs.as_ptr() as *const u32,
            modulus.as_ptr() as *const u32,
            result.as_mut_ptr() as *mut u32,
        );
    }
}

pub fn modinv_256_unchecked(
    inp: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    unsafe {
        sys_bigint2_3(
            MODINV_256_BLOB.as_ptr(),
            inp.as_ptr() as *const u32,
            modulus.as_ptr() as *const u32,
            result.as_mut_ptr() as *mut u32,
        );
    }
}

pub fn modmul_256_unchecked(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    unsafe {
        sys_bigint2_4(
            MODMUL_256_BLOB.as_ptr(),
            lhs.as_ptr() as *const u32,
            rhs.as_ptr() as *const u32,
            modulus.as_ptr() as *const u32,
            result.as_mut_ptr() as *mut u32,
        );
    }
}

pub fn modsub_256_unchecked(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    unsafe {
        sys_bigint2_4(
            MODSUB_256_BLOB.as_ptr(),
            lhs.as_ptr() as *const u32,
            rhs.as_ptr() as *const u32,
            modulus.as_ptr() as *const u32,
            result.as_mut_ptr() as *mut u32,
        );
    }
}

// These "checked" versions verify that `result < modulus`

pub fn modadd_256(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    modadd_256_unchecked(&lhs, &rhs, &modulus, result);

    // An honest host will always return a result less than the modulus. A dishonest prover can
    // sometimes return a result greater than the modulus, so enforce that we're in the honest case.
    assert!(crate::is_less(&result, &modulus));
}

/// Compute the multiplicative inverse of `inp` mod `modulus`
///
/// The result is returned in `result`. Note that this can only compute the inverse if a
/// multiplicative inverse actually exists. Otherwise, no proof can be generated (and the executor
/// will panic).
///
/// Most notably, this means `inp` cannot be zero ("no division by zero"), but the full rule is that
/// `inp` and `modulus` must be relatively prime. When `modulus` is prime, this just means that
/// `inp` must not be a multiple of `modulus`, but when `modulus` is composite there are more cases
/// with no solution, e.g. `8` does not have an inverse mod `12`.
pub fn modinv_256(
    inp: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    modinv_256_unchecked(&inp, &modulus, result);

    // An honest host will always return a result less than the modulus. A dishonest prover can
    // sometimes return a result greater than the modulus, so enforce that we're in the honest case.
    assert!(crate::is_less(&result, &modulus));
}

pub fn modmul_256(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    modmul_256_unchecked(&lhs, &rhs, &modulus, result);

    // An honest host will always return a result less than the modulus. A dishonest prover can
    // sometimes return a result greater than the modulus, so enforce that we're in the honest case.
    assert!(crate::is_less(&result, &modulus));
}

pub fn modsub_256(
    lhs: &[u32; FIELD_256_WIDTH_WORDS],
    rhs: &[u32; FIELD_256_WIDTH_WORDS],
    modulus: &[u32; FIELD_256_WIDTH_WORDS],
    result: &mut [u32; FIELD_256_WIDTH_WORDS],
) {
    modsub_256_unchecked(&lhs, &rhs, &modulus, result);

    // An honest host will always return a result less than the modulus. A dishonest prover can
    // sometimes return a result greater than the modulus, so enforce that we're in the honest case.
    assert!(crate::is_less(&result, &modulus));
}
