#![no_std]
#![forbid(unsafe_code)]

//! Replaceable software provider. No final storage-encryption or memory-residue
//! assurance is implied by availability of digest computation.
extern crate alloc;
mod aead;
pub use aead::{SecretKeyBuffer, SensitivePlaintextBuffer};
use sha3::{
    Digest, Sha3_256, Sha3_384, Sha3_512,
    digest::{ExtendableOutput, Update, XofReader},
};
use shake::Shake256;
use skrifheim_core::Result;
use skrifheim_crypto::{CryptoTranscript, DigestProvider, DigestStrength, DigestValue};

pub struct RustCryptoProvider;

impl DigestProvider for RustCryptoProvider {
    fn compute(
        &self,
        strength: DigestStrength,
        transcript: &CryptoTranscript,
    ) -> Result<DigestValue> {
        digest(strength, transcript.as_bytes())
    }
}

fn digest(strength: DigestStrength, bytes: &[u8]) -> Result<DigestValue> {
    match strength {
        DigestStrength::Sha3_256 => DigestValue::new(strength, &Sha3_256::digest(bytes)),
        DigestStrength::Sha3_384 => DigestValue::new(strength, &Sha3_384::digest(bytes)),
        DigestStrength::Sha3_512 => DigestValue::new(strength, &Sha3_512::digest(bytes)),
        DigestStrength::Shake256_256 | DigestStrength::Shake256_512 => {
            let mut state = Shake256::default();
            state.update(bytes);
            let mut output = [0; 64];
            let output = &mut output[..strength.output_bytes()];
            state.finalize_xof().read(output);
            DigestValue::new(strength, output)
        }
    }
}

#[cfg(test)]
mod tests;
