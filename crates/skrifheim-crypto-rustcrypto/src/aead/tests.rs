use super::*;
use skrifheim_core::TenantId;
use skrifheim_crypto::{AeadEngine, AeadEnvelope, CryptoTranscript, TranscriptKind};

struct TestEntropy(u8);
impl EntropySource for TestEntropy {
    fn fill(&mut self, output: &mut [u8]) -> CryptoResult<()> {
        output.fill(self.0);
        self.0 = self
            .0
            .checked_add(1)
            .ok_or(CryptoError::EntropyUnavailable)?;
        Ok(())
    }
}
struct NoEntropy;
impl EntropySource for NoEntropy {
    fn fill(&mut self, _: &mut [u8]) -> CryptoResult<()> {
        Err(CryptoError::EntropyUnavailable)
    }
}

#[test]
fn exhausted_key_stops_sealing_but_preserves_existing_reads() -> CryptoResult<()> {
    let context = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &context)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, TestEntropy(10));
    key.seals.set((1_u64 << 32) - 1);
    let envelope = engine.seal(
        &key,
        &context,
        SensitivePlaintextBuffer::from_vec(b"last permitted seal".to_vec())?,
    )?;
    assert!(matches!(
        engine.seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(b"denied".to_vec())?
        ),
        Err(CryptoError::ProviderFailure)
    ));
    assert!(engine.open(&key, &context, &envelope).is_ok());
    Ok(())
}
fn context(tenant: u128, key: u128, epoch: u64, operation: u8) -> CryptoResult<AeadContext> {
    let tenant = TenantId::from_u128(tenant).ok_or(CryptoError::InvalidInput)?;
    AeadContext::new(
        KeyId::from_u128(key).ok_or(CryptoError::InvalidInput)?,
        CryptoEpoch::new(epoch),
        EncryptionDomain::wal(tenant, None, None),
        [operation; 32],
        CryptoTranscript::new(TranscriptKind::Wal),
    )
}

#[test]
fn no_entropy_never_yields_key_or_ciphertext() -> CryptoResult<()> {
    let context = context(1, 2, 1, 1)?;
    assert!(matches!(
        SecretKeyBuffer::generate(&mut NoEntropy, &context),
        Err(CryptoError::EntropyUnavailable)
    ));
    let key = SecretKeyBuffer::generate(&mut TestEntropy(1), &context)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, NoEntropy);
    assert!(matches!(
        engine.seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(alloc::vec![1])?
        ),
        Err(CryptoError::EntropyUnavailable)
    ));
    Ok(())
}

#[test]
fn every_encoded_byte_and_truncation_is_rejected_after_tampering() -> CryptoResult<()> {
    let context = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &context)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, TestEntropy(10));
    let encoded = engine
        .seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(b"private data".to_vec())?,
        )?
        .encode();
    // Independently reproduced with libsodium on 2026-10-08; freezes envelope
    // header, nonce, context transcript and ciphertext/tag bytes together.
    let hex: alloc::string::String = encoded.iter().map(|b| alloc::format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "534b524946454e43010001000a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a1c00000000000000bd50dc9ab85e97e466d7f0f0afbc9b951db83ebb58452278be21c844"
    );
    assert!(
        engine
            .open(&key, &context, &AeadEnvelope::decode(&encoded)?)
            .is_ok()
    );
    for i in 0..encoded.len() {
        let mut tampered = encoded.clone();
        tampered[i] ^= 1;
        if let Ok(envelope) = AeadEnvelope::decode(&tampered) {
            assert!(engine.open(&key, &context, &envelope).is_err());
        }
        assert!(AeadEnvelope::decode(&encoded[..i]).is_err());
    }
    let mut extended = encoded;
    extended.push(0);
    assert!(AeadEnvelope::decode(&extended).is_err());
    Ok(())
}

#[test]
fn key_epoch_tenant_operation_and_transcript_are_bound() -> CryptoResult<()> {
    let original = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &original)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, TestEntropy(10));
    let envelope = engine.seal(
        &key,
        &original,
        SensitivePlaintextBuffer::from_vec(b"private data".to_vec())?,
    )?;
    for other in [
        context(9, 2, 1, 1)?,
        context(1, 9, 1, 1)?,
        context(1, 2, 9, 1)?,
        context(1, 2, 1, 9)?,
    ] {
        assert!(engine.open(&key, &other, &envelope).is_err());
    }
    let wrong_key = SecretKeyBuffer::generate(&mut TestEntropy(4), &original)?;
    assert!(engine.open(&wrong_key, &original, &envelope).is_err());
    let mut other = context(1, 2, 1, 1)?;
    // Use a different transcript while keeping all key-scope fields unchanged.
    let mut transcript = CryptoTranscript::new(TranscriptKind::Wal);
    transcript
        .field(1, b"wrong snapshot")
        .map_err(|_| CryptoError::InvalidInput)?;
    other = AeadContext::new(
        other.key_id(),
        other.epoch(),
        other.domain(),
        [1; 32],
        transcript,
    )?;
    assert!(engine.open(&key, &other, &envelope).is_err());
    Ok(())
}

#[test]
fn all_encryption_domains_are_isolated_even_with_identical_key_bytes() -> CryptoResult<()> {
    use skrifheim_core::{Classification, WorldId};
    use skrifheim_crypto::{CompartmentKeyId, RegionKeyId, SegmentKeyId};
    let tenant = TenantId::from_u128(1).ok_or(CryptoError::InvalidInput)?;
    let region = RegionKeyId::from_u128(2).ok_or(CryptoError::InvalidInput)?;
    let compartment = CompartmentKeyId::from_u128(3).ok_or(CryptoError::InvalidInput)?;
    let world = WorldId::from_u128(4).ok_or(CryptoError::InvalidInput)?;
    let segment = SegmentKeyId::from_u128(5).ok_or(CryptoError::InvalidInput)?;
    let domains = [
        EncryptionDomain::tenant(tenant),
        EncryptionDomain::region(tenant, region),
        EncryptionDomain::compartment(tenant, Some(region), Classification::Secret, compartment),
        EncryptionDomain::world(
            tenant,
            Some(region),
            Some(Classification::Secret),
            Some(compartment),
            world,
        ),
        EncryptionDomain::wal(tenant, Some(region), Some(world)),
        EncryptionDomain::segment(
            tenant,
            Some(region),
            Classification::Secret,
            compartment,
            segment,
        ),
        EncryptionDomain::projection(
            tenant,
            Some(region),
            Classification::Secret,
            Some(compartment),
            Some(world),
        ),
        EncryptionDomain::backup(tenant, Some(region)),
        EncryptionDomain::export_capsule(tenant, Some(region), Classification::Secret),
        EncryptionDomain::ai_artifact(
            tenant,
            Some(region),
            Classification::Secret,
            Some(compartment),
            Some(world),
        ),
        EncryptionDomain::audit_log(tenant, Some(region)),
    ];
    for (i, domain) in domains.iter().enumerate() {
        let context = AeadContext::new(
            KeyId::from_u128(2).ok_or(CryptoError::InvalidInput)?,
            CryptoEpoch::new(1),
            *domain,
            [1; 32],
            CryptoTranscript::new(TranscriptKind::Content),
        )?;
        let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &context)?;
        let mut engine = AeadEngine::new(RustCryptoProvider, TestEntropy(10));
        let envelope = engine.seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(b"data".to_vec())?,
        )?;
        for (j, other) in domains.iter().enumerate() {
            let context = AeadContext::new(
                key.key_id,
                key.epoch,
                *other,
                [1; 32],
                CryptoTranscript::new(TranscriptKind::Content),
            )?;
            let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &context)?;
            assert_eq!(engine.open(&key, &context, &envelope).is_ok(), i == j);
        }
    }
    Ok(())
}

#[test]
fn boundary_lengths_and_authenticated_reseal() -> CryptoResult<()> {
    let context = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer::generate(&mut TestEntropy(3), &context)?;
    let mut engine = AeadEngine::new(RustCryptoProvider, TestEntropy(10));
    for len in [0, 1, AEAD_PLAINTEXT_MAX_BYTES] {
        let sealed = engine.seal(
            &key,
            &context,
            SensitivePlaintextBuffer::from_vec(alloc::vec![123;len])?,
        )?;
        let authenticated = engine.open(&key, &context, &sealed)?;
        let resealed = engine.reseal(&key, &context, authenticated)?;
        assert_ne!(sealed.encode(), resealed.encode());
        assert!(engine.open(&key, &context, &resealed).is_ok());
    }
    assert!(
        SensitivePlaintextBuffer::from_vec(alloc::vec![0; AEAD_PLAINTEXT_MAX_BYTES+1]).is_err()
    );
    Ok(())
}

#[test]
fn xchacha_draft_known_answer_through_provider() -> CryptoResult<()> {
    // draft-irtf-cfrg-xchacha, Appendix A.1. Algorithm vector, not envelope AAD.
    let context = context(1, 2, 1, 1)?;
    let key = SecretKeyBuffer {
        seals: core::cell::Cell::new(0),
        bytes: WipeOnDrop::new(core::array::from_fn(|i| 0x80 + i as u8)),
        key_id: context.key_id(),
        epoch: context.epoch(),
        domain: context.domain(),
    };
    let nonce = core::array::from_fn(|i| 0x40 + i as u8);
    let aad = [
        0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    ];
    let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
    let expected = "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52ec0875924c1c7987947deafd8780acf49";
    let actual = encrypt(
        &key,
        &nonce,
        &aad,
        &mut SensitivePlaintextBuffer::from_vec(plaintext.to_vec())?,
    )?;
    let actual_hex: alloc::string::String =
        actual.iter().map(|b| alloc::format!("{b:02x}")).collect();
    assert_eq!(actual_hex, expected);
    let opened = decrypt(&key, &nonce, &aad, &actual)?;
    opened.0.with_secret(|bytes| assert_eq!(bytes, plaintext));
    Ok(())
}
