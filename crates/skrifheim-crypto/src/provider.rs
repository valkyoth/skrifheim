//! Provider-independent, versioned digest transcripts. Providers are trusted
//! implementations admitted at composition time, never selected by input data.
use crate::{
    ContentDigest, DigestPolicy, DigestStrength, DigestValue, ManifestDigest, WorldIdentityDigest,
};
use alloc::vec::Vec;
use core::fmt;
use sanitization::SecureSanitize;
use skrifheim_core::{Result, SkrifheimError};

pub const TRANSCRIPT_MAX_BYTES: usize = 1024 * 1024;
pub const TRANSCRIPT_MAX_FIELDS: u16 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TranscriptKind {
    Content = 1,
    Manifest = 2,
    WorldIdentity = 3,
    Wal = 4,
    Block = 5,
    Segment = 6,
    Backup = 7,
    Export = 8,
    Projection = 9,
    AiArtifact = 10,
    Audit = 11,
}

/// Length-delimited, ordered fields; field tags must strictly increase.
/// These transcripts are internal sensitive metadata, not public outer headers.
pub struct CryptoTranscript {
    kind: TranscriptKind,
    bytes: Vec<u8>,
    last_tag: u16,
    fields: u16,
}

impl fmt::Debug for CryptoTranscript {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CryptoTranscript(<redacted>)")
    }
}

impl CryptoTranscript {
    pub fn new(kind: TranscriptKind) -> Self {
        let mut bytes = Vec::from(&b"skrifheim/transcript/v1\0"[..]);
        bytes.push(kind as u8);
        Self {
            kind,
            bytes,
            last_tag: 0,
            fields: 0,
        }
    }

    pub fn field(&mut self, tag: u16, value: &[u8]) -> Result<()> {
        let size = self
            .bytes
            .len()
            .checked_add(6)
            .and_then(|n| n.checked_add(value.len()))
            .ok_or(SkrifheimError::InvalidDigest)?;
        if tag <= self.last_tag
            || self.fields == TRANSCRIPT_MAX_FIELDS
            || size > TRANSCRIPT_MAX_BYTES
        {
            return Err(SkrifheimError::InvalidDigest);
        }
        if size > self.bytes.capacity() {
            let mut replacement = Vec::with_capacity(size);
            replacement.extend_from_slice(&self.bytes);
            self.bytes.secure_sanitize();
            self.bytes = replacement;
        }
        self.bytes.extend_from_slice(&tag.to_le_bytes());
        self.bytes
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.bytes.extend_from_slice(value);
        self.last_tag = tag;
        self.fields += 1;
        Ok(())
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn kind(&self) -> TranscriptKind {
        self.kind
    }
}

impl Drop for CryptoTranscript {
    fn drop(&mut self) {
        self.bytes.secure_sanitize();
    }
}

/// Trusted provider boundary: no algorithm-library types enter the core API.
/// Implementations must return exactly the requested algorithm and output size.
pub trait DigestProvider {
    fn compute(
        &self,
        strength: DigestStrength,
        transcript: &CryptoTranscript,
    ) -> Result<DigestValue>;
}

/// A successful comparison against computation by the admitted provider.
/// This proves bytes under a transcript, not policy authority or freshness.
pub struct VerifiedDigestProof {
    value: DigestValue,
    kind: TranscriptKind,
}
impl VerifiedDigestProof {
    pub fn digest_bytes(&self) -> &[u8] {
        self.value.as_bytes()
    }
    pub const fn kind(&self) -> TranscriptKind {
        self.kind
    }
}
impl fmt::Debug for VerifiedDigestProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerifiedDigestProof(<redacted>)")
    }
}

macro_rules! compute_digest {
    ($name:ident, $kind:ident) => {
        impl $name {
            pub fn verify(
                &self,
                provider: &impl DigestProvider,
                policy: DigestPolicy,
                transcript: &CryptoTranscript,
            ) -> Result<VerifiedDigestProof> {
                let actual = Self::compute(provider, policy, transcript)?;
                if !self.structurally_equal_ct(&actual) {
                    return Err(SkrifheimError::InvalidDigest);
                }
                Ok(VerifiedDigestProof {
                    value: DigestValue::new(actual.strength(), actual.digest_bytes())?,
                    kind: transcript.kind(),
                })
            }
            pub fn compute(
                provider: &impl DigestProvider,
                policy: DigestPolicy,
                transcript: &CryptoTranscript,
            ) -> Result<Self> {
                if transcript.kind() != TranscriptKind::$kind {
                    return Err(SkrifheimError::InvalidDigest);
                }
                let digest = provider.compute(policy.strength(), transcript)?;
                if digest.strength() != policy.strength() {
                    return Err(SkrifheimError::InvalidDigest);
                }
                Self::new(policy, digest.as_bytes())
            }
        }
    };
}
compute_digest!(ContentDigest, Content);
compute_digest!(ManifestDigest, Manifest);
compute_digest!(WorldIdentityDigest, WorldIdentity);

#[cfg(test)]
mod tests;
