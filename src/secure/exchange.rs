use anyhow::{Result, anyhow};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

pub fn encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .encrypt(Nonce::from_slice(nonce), plaintext)
        .map_err(|e| anyhow!("Encryption failed: {:?}", e))
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [42u8; 32];
        let nonce = [7u8; 12];
        let message = b"Hello, secure world!";

        let ciphertext = encrypt(&key, &nonce, message).unwrap();
        assert_ne!(ciphertext, message);

        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, message);

        let wrong_nonce = [8u8; 12];
        assert!(decrypt(&key, &wrong_nonce, &ciphertext).is_err());

        let wrong_key = [43u8; 32];
        assert!(decrypt(&wrong_key, &nonce, &ciphertext).is_err());
    }
}
