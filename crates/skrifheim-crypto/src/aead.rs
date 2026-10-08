//! Generic envelope v1, independent of future WAL/block/segment framing.
use crate::{CryptoEpoch, CryptoTranscript, EncryptionDomain, KeyId};
use alloc::vec::Vec;
use core::fmt;
use sanitization::wipe::WipeOnDrop;

mod envelope;
pub use envelope::AeadEnvelope;

pub const AEAD_PLAINTEXT_MAX_BYTES: usize = 64 * 1024;
pub const AEAD_TAG_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CryptoError {
    InvalidInput,
    AuthenticationFailed,
    EntropyUnavailable,
    ProviderFailure,
}
pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum AeadSuite {
    XChaCha20Poly1305 = 1,
}

#[derive(Clone, Copy, Eq, PartialEq)]
/// Only the engine can construct nonce tokens for an admitted provider.
/// ```compile_fail
/// use skrifheim_crypto::PublicNonce;
/// let chosen_nonce = PublicNonce([0; 24]);
/// ```
pub struct PublicNonce(pub(crate) [u8; 24]);
impl PublicNonce {
    pub const fn as_bytes(&self) -> &[u8; 24] {
        &self.0
    }
}
impl fmt::Debug for PublicNonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PublicNonce(<redacted>)")
    }
}

/// Only admitted entropy providers belong in a production composition.
/// Deterministic implementations belong exclusively under cfg(test).
pub trait EntropySource {
    fn fill(&mut self, destination: &mut [u8]) -> CryptoResult<()>;
}

/// Trusted expected context resolved by the caller, not from an untrusted envelope.
pub struct AeadContext {
    key_id: KeyId,
    epoch: CryptoEpoch,
    domain: EncryptionDomain,
    operation: [u8; 32],
    transcript: CryptoTranscript,
}
impl AeadContext {
    pub fn new(
        key_id: KeyId,
        epoch: CryptoEpoch,
        domain: EncryptionDomain,
        operation: [u8; 32],
        transcript: CryptoTranscript,
    ) -> CryptoResult<Self> {
        if epoch.get() == 0 || operation == [0; 32] {
            return Err(CryptoError::InvalidInput);
        }
        Ok(Self {
            key_id,
            epoch,
            domain,
            operation,
            transcript,
        })
    }
    pub const fn key_id(&self) -> KeyId {
        self.key_id
    }
    pub const fn epoch(&self) -> CryptoEpoch {
        self.epoch
    }
    pub const fn domain(&self) -> EncryptionDomain {
        self.domain
    }

    fn associated_data(&self, envelope: &AeadEnvelope) -> WipeOnDrop<Vec<u8>> {
        let mut bytes = Vec::with_capacity(256 + self.transcript.as_bytes().len());
        bytes.extend_from_slice(b"skrifheim/aead/v1\0");
        bytes.extend_from_slice(&envelope.header());
        bytes.extend_from_slice(&self.key_id.get().to_le_bytes());
        bytes.extend_from_slice(&self.epoch.get().to_le_bytes());
        bytes.extend_from_slice(&self.domain.canonical_bytes());
        bytes.extend_from_slice(&self.operation);
        bytes.extend_from_slice(
            &(envelope.ciphertext.len() as u64 - AEAD_TAG_BYTES as u64).to_le_bytes(),
        );
        bytes.extend_from_slice(&(self.transcript.as_bytes().len() as u64).to_le_bytes());
        bytes.extend_from_slice(self.transcript.as_bytes());
        WipeOnDrop::new(bytes)
    }
}
impl fmt::Debug for AeadContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AeadContext(<redacted>)")
    }
}

/// Provider keys and plaintext stay in provider-owned purpose-specific buffers.
/// Provider implementations are trusted code, never tenant-supplied plugins.
pub trait AeadProvider {
    type Key;
    type Plaintext;
    fn plaintext_len(&self, plaintext: &Self::Plaintext) -> usize;
    fn seal(
        &self,
        key: &Self::Key,
        context: &AeadContext,
        nonce: PublicNonce,
        aad: &[u8],
        plaintext: Self::Plaintext,
    ) -> CryptoResult<Vec<u8>>;
    fn open(
        &self,
        key: &Self::Key,
        context: &AeadContext,
        nonce: PublicNonce,
        aad: &[u8],
        ciphertext: &[u8],
    ) -> CryptoResult<Self::Plaintext>;
}

/// Can be created only by a successful provider authentication operation.
/// This is cryptographic authenticity, not authorization or replay freshness.
pub struct AuthenticatedPlaintext<P: AeadProvider> {
    value: P::Plaintext,
}
impl<P: AeadProvider> fmt::Debug for AuthenticatedPlaintext<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticatedPlaintext(<redacted>)")
    }
}

pub struct AeadEngine<P, E> {
    provider: P,
    entropy: E,
}
impl<P: AeadProvider, E: EntropySource> AeadEngine<P, E> {
    pub const fn new(provider: P, entropy: E) -> Self {
        Self { provider, entropy }
    }

    pub fn seal(
        &mut self,
        key: &P::Key,
        context: &AeadContext,
        plaintext: P::Plaintext,
    ) -> CryptoResult<AeadEnvelope> {
        let length = self.provider.plaintext_len(&plaintext);
        if length > AEAD_PLAINTEXT_MAX_BYTES {
            return Err(CryptoError::InvalidInput);
        }
        let mut nonce = PublicNonce([0; 24]);
        self.entropy.fill(&mut nonce.0)?;
        let mut envelope = AeadEnvelope {
            nonce,
            ciphertext: alloc::vec![0; length + AEAD_TAG_BYTES],
        };
        let aad = context.associated_data(&envelope);
        envelope.ciphertext =
            aad.with_secret(|aad| self.provider.seal(key, context, nonce, aad, plaintext))?;
        if envelope.ciphertext.len() != length + AEAD_TAG_BYTES {
            return Err(CryptoError::ProviderFailure);
        }
        Ok(envelope)
    }

    pub fn open(
        &self,
        key: &P::Key,
        context: &AeadContext,
        envelope: &AeadEnvelope,
    ) -> CryptoResult<AuthenticatedPlaintext<P>> {
        let aad = context.associated_data(envelope);
        let value = aad.with_secret(|aad| {
            self.provider
                .open(key, context, envelope.nonce, aad, &envelope.ciphertext)
        })?;
        if self
            .provider
            .plaintext_len(&value)
            .checked_add(AEAD_TAG_BYTES)
            != Some(envelope.ciphertext.len())
        {
            return Err(CryptoError::ProviderFailure);
        }
        Ok(AuthenticatedPlaintext { value })
    }

    /// Transfers authenticated plaintext directly into a new protected envelope.
    /// Policy authorization must precede this cryptographic operation.
    pub fn reseal(
        &mut self,
        key: &P::Key,
        context: &AeadContext,
        plaintext: AuthenticatedPlaintext<P>,
    ) -> CryptoResult<AeadEnvelope> {
        self.seal(key, context, plaintext.value)
    }
}
