use base64::{engine::general_purpose, Engine as _};
use ring::rand;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};

/// Represents an Ed25519 key pair
///
/// An instance of `Keys` contains a `private_key` field and
/// a `public_key` field for the purpose of generating and verifying
/// digital signatures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keys {
    private_key: String,
    pub public_key: String,
}

impl Keys {
    pub fn new_keypair() -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes =
            Ed25519KeyPair::generate_pkcs8(&rng).expect("error: unable to generate agent keys\n");
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("error: unable to generate agent keys\n");

        let private_key = general_purpose::STANDARD.encode(pkcs8_bytes.as_ref());
        let public_key = general_purpose::STANDARD.encode(key_pair.public_key().as_ref());

        Keys {
            private_key,
            public_key,
        }
    }
}
