use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use ring::rand;
use ring::signature::{self, KeyPair};
use serde::{Deserialize, Serialize};
/// Represents an Ed25519 key pair
///
/// An instance of `Keys` contains a `private_key` field and
/// a `public_key` field for the purpose of generating and verifying
/// digital signatures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keys {
    // A base64-encoded Ed25519 key pair,
    private_key: String,
    // A public key, encoded as base64 to facilitate serialization and sharing.
    public_key: String,
}

impl Keys {
    /// Generates an Ed25519 key pair, encodes the keys as base64 and returns
    /// them within a new instance of `Keys`.
    pub fn new_key_pair() -> Self {
        let rng = rand::SystemRandom::new();

        // Generate a PKCS#8 representation of the new key pair
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("[!] error: unable to generate agent keys\n");

        // Construct the key pair from the bytes representation
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("[!] error: unable to generate agent keys\n");

        // Encode the key pair as base64
        let private_key = general_purpose::STANDARD.encode(pkcs8_bytes.as_ref());

        // Derive the public key from the key pair and encode it as base64
        let public_key = general_purpose::STANDARD.encode(key_pair.public_key().as_ref());

        Keys {
            private_key,
            public_key,
        }
    }

    /// Returns the keypair's public key.
    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }

    /// Decodes a String `data` from base64 into a vector of bytes.
    fn base64_to_bytes(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(data)
    }

    /// Verifies if a digital signature `signature` is a valid signature of `message` by the
    /// owner of the private key that corresponds to `public_key`. Returns Ok(()) if the signature
    /// is valid.
    pub fn verify(message: &[u8], signature: &[u8], public_key: &str) -> anyhow::Result<()> {
        let public_key_bytes = Self::base64_to_bytes(public_key)
            .context("[!] error: unable to decode public key; cannot verify message signature\n")?;

        let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);

        public_key
            .verify(message, signature)
            .map_err(|_| anyhow::anyhow!("[!] error: not a valid signature of the message"))
    }

    /// Generates a digital signature of a byte slice `data` using `self.private_key`.
    pub fn sign(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let private_key_bytes = general_purpose::STANDARD.decode(&self.private_key)?;

        let key_pair =
            signature::Ed25519KeyPair::from_pkcs8(&private_key_bytes.as_ref()).map_err(|e| {
                anyhow::anyhow!(
                    "[!] error: unable to sign message; failed to reconstruct key pair - {}",
                    e
                )
            })?;

        let signature = key_pair.sign(data);

        Ok(signature.as_ref().to_vec())
    }
}

// ******************************************************************************************
// ************************************* UNIT TESTS *****************************************
// ******************************************************************************************

#[cfg(test)]
mod tests {
    use super::*;

    // Test if the generated key pair can be decoded from base64 and used
    // to generate valid signatures
    #[test]
    fn test_key_pair_signing() {
        // Generate a new keypair and store it base64-encoded in the `Keys` struct
        let keys = Keys::new_key_pair();

        // Decode the key from base64 to bytes
        let private_key_bytes = general_purpose::STANDARD
            .decode(keys.private_key)
            .expect("[!] error: failed to decode private key from base64");

        let public_key_bytes = general_purpose::STANDARD
            .decode(keys.public_key)
            .expect("[!] error: failed to decode public key from base64");

        // Reconstruct the key pair from the pkcs8 bytes representation
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(&private_key_bytes.as_ref())
            .expect("[!] error: unable to construct key pair");

        // Reconstruct the public key from the public key bytes
        let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);

        let message = b"Tis but a scratch!";

        // Generate a signature of `message`
        let sig = key_pair.sign(message);

        // Verify if `sig` is a valid signature of `message`, i.e, verifies
        // if the signature was generated by the private key that matches `public_key`
        public_key
            .verify(message, sig.as_ref())
            .expect("[!] error: not a valid signature of the message");
    }

    // Test if the keys are unique, i.e, they are not using the same source of entropy
    #[test]
    fn test_key_pair_collision() {
        let keys1 = Keys::new_key_pair();
        let keys2 = Keys::new_key_pair();

        assert_ne!(keys1.private_key, keys2.private_key);
        assert_ne!(keys1.public_key, keys2.public_key);
    }
}
