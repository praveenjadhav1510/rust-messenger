use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub fn generate_ephemeral_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(&mut OsRng);
    let public = PublicKey::from(&secret);
    (secret, public)
}

pub fn derive_shared_secret(secret: EphemeralSecret, their_public: &PublicKey) -> [u8; 32] {
    let shared = secret.diffie_hellman(their_public);
    *shared.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange() {
        let (alice_secret, alice_public) = generate_ephemeral_keypair();
        let (bob_secret, bob_public) = generate_ephemeral_keypair();

        let secret_alice = derive_shared_secret(alice_secret, &bob_public);
        let secret_bob = derive_shared_secret(bob_secret, &alice_public);

        assert_eq!(secret_alice, secret_bob);
    }
}
