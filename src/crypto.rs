use pqc_kyber::{keypair, encapsulate, decapsulate};
use aes_gcm::{Aes256Gcm, Key, KeyInit};
use rand::thread_rng;

pub struct QuantumSession {
    pub shared_secret: [u8; 32],
}

impl QuantumSession {
    pub fn new_client_handshake(public_key: &[u8]) -> (Vec<u8>, Self) {
        let mut rng = thread_rng();
        let (ciphertext, shared_secret) = encapsulate(public_key, &mut rng)
            .expect("Lattice handshake failed");
        (ciphertext.to_vec(), Self { shared_secret })
    }
}
