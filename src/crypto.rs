use pqc_kyber::encapsulate;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroize;

/// Encrypted payload with nonce prepended for transmission
pub struct EncryptedPayload {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

impl EncryptedPayload {
    /// Serialize to bytes: [12-byte nonce][ciphertext...]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + self.ciphertext.len());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext);
        out
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 { return None; }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&data[..12]);
        Some(Self { nonce, ciphertext: data[12..].to_vec() })
    }
}

pub struct QuantumSession {
    secret: Vec<u8>, // Shared secret from Kyber-768
}

impl Drop for QuantumSession {
    fn drop(&mut self) {
        self.secret.zeroize(); // Level 11: Wipe session entropy from RAM
    }
}

impl QuantumSession {
    /// Sender initiates the handshake by encapsulating a secret for the receiver's Public Key
    pub fn initiate(pk_bytes: &[u8]) -> Result<(Vec<u8>, Self), &'static str> {
        let mut rng = OsRng;
        let (ct, ss) = encapsulate(pk_bytes, &mut rng).map_err(|_| "Encapsulation failed")?;
        Ok((ct.to_vec(), Self { secret: ss.to_vec() }))
    }
    
    /// Create session from existing shared secret (for receiver)
    pub fn from_secret(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    /// Encrypts the file payload using the derived quantum secret with random nonce
    /// Returns EncryptedPayload containing nonce + ciphertext for transmission
    pub fn encrypt_payload(&self, data: &[u8]) -> EncryptedPayload {
        let key = Key::<Aes256Gcm>::from_slice(&self.secret);
        let cipher = Aes256Gcm::new(key);
        
        // Generate cryptographically secure random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data).expect("Encryption failed");
        EncryptedPayload { nonce: nonce_bytes, ciphertext }
    }
    
    /// Decrypts payload using the shared secret
    pub fn decrypt_payload(&self, encrypted: &EncryptedPayload) -> Result<Vec<u8>, &'static str> {
        let key = Key::<Aes256Gcm>::from_slice(&self.secret);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        cipher.decrypt(nonce, encrypted.ciphertext.as_slice())
            .map_err(|_| "Decryption failed")
    }
}
