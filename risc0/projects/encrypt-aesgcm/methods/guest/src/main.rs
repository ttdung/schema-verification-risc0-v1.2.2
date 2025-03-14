extern crate aes_gcm;
extern crate rand;

use aes_gcm::aead::{Aead, KeyInit}; //, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or `Aes128Gcm`
// use rand::RngCore;
// use hex::encode;
use sha2::{Sha256, Digest};
use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;

fn main() {

    let (key_str, aad, plaintext, nonce) : (String, String, String, Vec<u8>) = env::read();

    // println!("key_str: {}", key_str);
    // println!("plaintext: {}", plaintext);    
    // println!("nonce: {:?}", nonce);

    let ciphertext = encrypt(key_str.clone(), plaintext.clone(), nonce);

    println!("Ciphertext: {}", hex::encode(ciphertext.clone()));

    // Calculate hash256(<aad>:<document_content>)
    let input = aad.clone() + &plaintext;
    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash1 = hasher.finalize();
    
    // Calculate hash256(<encrypted_document_content>)
   // let ciphertext_vec: Vec<u8> = hex::decode(ciphertext.clone()).expect("decoeable");
    let mut input2 = aad.into_bytes();
    input2.extend(ciphertext);
    let mut hasher2 = Sha256::new();
    hasher2.update(input2);
    let hash2 = hasher2.finalize();

    println!("SHA-256 hash1: {:x}", hash1);
    println!("SHA-256 hash2: {:x}", hash2);

    let result_vec: Vec<u8> = hash1.to_vec();

    let result_vec: Vec<u8> = hash1.iter().chain(hash2.iter()).cloned().collect();

    // println!("HASH | cipherText: {}", hex::encode(&result_vec));

    env::commit_slice(result_vec.clone().abi_encode().as_slice());

}

fn encrypt(key_str: String, plaintext: String, nonce_vec : Vec<u8>) -> Vec<u8> {
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    // let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_vec);
    let ciphered_data = cipher.encrypt(&nonce, plaintext.as_bytes())
        .expect("failed to encrypt");
    // combining nonce and encrypted data together
    // for storage purpose
    let mut encrypted_data: Vec<u8> = nonce.to_vec();
    encrypted_data.extend_from_slice(&ciphered_data);
    ciphered_data
    // hex::encode(encrypted_data)
}

fn decrypt(key_str: String, encrypted_data: String) -> String {
    let encrypted_data = hex::decode(encrypted_data)
        .expect("failed to decode hex string into vec");
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let (nonce_arr, ciphered_data) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_arr);
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(nonce, ciphered_data)
        .expect("failed to decrypt data");
    String::from_utf8(plaintext)
        .expect("failed to convert vector of bytes to string")
}