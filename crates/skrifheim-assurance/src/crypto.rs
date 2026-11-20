use skrifheim::{core::TenantId, crypto::*, crypto_provider::*};

// Private deterministic entropy for synthetic fixtures only. This crate is unpublished.
struct FixtureEntropy(u8);
impl EntropySource for FixtureEntropy {
    fn fill(&mut self, bytes: &mut [u8]) -> CryptoResult<()> {
        bytes.fill(self.0);
        self.0 = self
            .0
            .checked_add(1)
            .ok_or(CryptoError::EntropyUnavailable)?;
        Ok(())
    }
}

fn context(tenant: u128, key: u128, epoch: u64, operation: u8) -> CryptoResult<AeadContext> {
    AeadContext::new(
        KeyId::from_u128(key).ok_or(CryptoError::InvalidInput)?,
        CryptoEpoch::new(epoch),
        EncryptionDomain::wal(
            TenantId::from_u128(tenant).ok_or(CryptoError::InvalidInput)?,
            None,
            None,
        ),
        [operation; 32],
        CryptoTranscript::new(TranscriptKind::Wal),
    )
}

pub fn envelope(data: &[u8]) {
    assert_eq!(envelope_inner(data), Ok(()));
}
fn envelope_inner(data: &[u8]) -> CryptoResult<()> {
    let Ok(envelope) = AeadEnvelope::decode(data) else {
        return Ok(());
    };
    assert_eq!(envelope.encode(), data);
    let original = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut FixtureEntropy(3), &original)?;
    let engine = AeadEngine::new(RustCryptoProvider, FixtureEntropy(10));
    let known = crate::seeds::envelope();
    let opened = engine.open(&key, &original, &envelope);
    if data == known {
        assert!(opened.is_ok());
    }
    if opened.is_err() {
        return Ok(());
    }
    let mut damaged = data.to_vec();
    let last = damaged.len() - 1;
    damaged[last] ^= 1;
    assert!(
        engine
            .open(&key, &original, &AeadEnvelope::decode(&damaged)?)
            .is_err()
    );
    for changed in [
        context(2, 2, 1, 1)?,
        context(1, 3, 1, 1)?,
        context(1, 2, 2, 1)?,
        context(1, 2, 1, 2)?,
    ] {
        assert!(engine.open(&key, &changed, &envelope).is_err());
    }
    Ok(())
}

pub fn transcript(data: &[u8]) {
    let mut transcript = CryptoTranscript::new(TranscriptKind::Content);
    let mut expected = b"skrifheim/transcript/v1\0\x01".to_vec();
    let mut offset = 0;
    let mut last = 0;
    let mut fields = 0;
    while let Some(prefix) = data.get(offset..offset + 3) {
        let tag = u16::from_le_bytes([prefix[0], prefix[1]]);
        let end = (offset + 3 + usize::from(prefix[2])).min(data.len());
        let value = &data[offset + 3..end];
        let allowed = tag > last
            && fields < TRANSCRIPT_MAX_FIELDS
            && expected.len() + 6 + value.len() <= TRANSCRIPT_MAX_BYTES;
        assert_eq!(transcript.field(tag, value).is_ok(), allowed);
        if allowed {
            expected.extend_from_slice(&tag.to_le_bytes());
            expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected.extend_from_slice(value);
            last = tag;
            fields += 1;
        }
        assert_eq!(transcript.as_bytes(), expected);
        offset = end;
    }
}

#[cfg(test)]
pub(crate) fn golden_seal() -> CryptoResult<alloc::vec::Vec<u8>> {
    let context = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut FixtureEntropy(3), &context)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, FixtureEntropy(10));
    engine
        .seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(b"private data".to_vec())?,
        )
        .map(|value| value.encode())
}
