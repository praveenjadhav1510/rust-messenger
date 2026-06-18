use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::Rng;

pub struct KeyPair {
    pub private_key_hex: String,
    pub public_key_hex: String,
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn generate_keypair() -> KeyPair {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let signing_key = SigningKey::from_bytes(&bytes);
    let verifying_key = VerifyingKey::from(&signing_key);

    KeyPair {
        private_key_hex: bytes_to_hex(&signing_key.to_bytes()),
        public_key_hex: bytes_to_hex(verifying_key.as_bytes()),
    }
}
