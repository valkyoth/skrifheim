use super::*;
const HEADER_BYTES: usize = 44;
const MAGIC: &[u8; 8] = b"SKRIFENC";

/// Minimal public framing. Key identity/domain/context are supplied separately
/// by trusted key resolution and are authenticated, not serialized here.
pub struct AeadEnvelope {
    pub(super) nonce: PublicNonce,
    pub(super) ciphertext: Vec<u8>,
}
impl AeadEnvelope {
    pub fn decode(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < HEADER_BYTES + AEAD_TAG_BYTES
            || bytes.len() > HEADER_BYTES + AEAD_TAG_BYTES + AEAD_PLAINTEXT_MAX_BYTES
            || &bytes[..8] != MAGIC
            || bytes[8..10] != 1_u16.to_le_bytes()
            || bytes[10..12] != (AeadSuite::XChaCha20Poly1305 as u16).to_le_bytes()
        {
            return Err(CryptoError::InvalidInput);
        }
        let mut length = [0; 8];
        length.copy_from_slice(&bytes[36..44]);
        if u64::from_le_bytes(length) != (bytes.len() - HEADER_BYTES) as u64 {
            return Err(CryptoError::InvalidInput);
        }
        let mut nonce = PublicNonce([0; 24]);
        nonce.0.copy_from_slice(&bytes[12..36]);
        Ok(Self {
            nonce,
            ciphertext: bytes[HEADER_BYTES..].to_vec(),
        })
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = self.header().to_vec();
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }
    pub(super) fn header(&self) -> [u8; HEADER_BYTES] {
        let mut bytes = [0; HEADER_BYTES];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..10].copy_from_slice(&1_u16.to_le_bytes());
        bytes[10..12].copy_from_slice(&(AeadSuite::XChaCha20Poly1305 as u16).to_le_bytes());
        bytes[12..36].copy_from_slice(&self.nonce.0);
        bytes[36..44].copy_from_slice(&(self.ciphertext.len() as u64).to_le_bytes());
        bytes
    }
}
impl fmt::Debug for AeadEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AeadEnvelope(<redacted>)")
    }
}
