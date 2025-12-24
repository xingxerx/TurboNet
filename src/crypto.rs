use pqc_kyber::encapsulate;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::rngs::OsRng;

pub struct QuantumSession {
    pub shared_secret: [u8; 32],
}

impl QuantumSession {
    /// Sender initiates the handshake by encapsulating a secret for the receiver's Public Key
    pub fn initiate(pk_bytes: &[u8]) -> Result<(Vec<u8>, Self), &'static str> {
        let mut rng = OsRng;
        let (ct, ss) = encapsulate(pk_bytes, &mut rng).map_err(|_| "Encapsulation failed")?;
        Ok((ct.to_vec(), Self { shared_secret: ss }))
    }

    /// Encrypts the file payload using the derived quantum secret
    pub fn encrypt_payload(&self, data: &[u8]) -> Vec<u8> {
        let key = Key::<Aes256Gcm>::from_slice(&self.shared_secret);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(b"TURBONET_V40"); // In production, use a unique nonce
        cipher.encrypt(nonce, data).expect("Encryption failed")
    }
}
