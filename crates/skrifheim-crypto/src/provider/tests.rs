use super::*;

#[test]
fn transcript_fields_are_unambiguous_ordered_and_bounded() -> Result<()> {
    let mut one = CryptoTranscript::new(TranscriptKind::Content);
    one.field(1, b"ab")?;
    one.field(2, b"c")?;
    let mut two = CryptoTranscript::new(TranscriptKind::Content);
    two.field(1, b"a")?;
    two.field(2, b"bc")?;
    assert_ne!(one.as_bytes(), two.as_bytes());
    let original = one.as_bytes().to_vec();
    assert!(one.field(2, b"duplicate").is_err());
    assert!(one.field(1, b"out of order").is_err());
    assert!(one.field(3, &alloc::vec![0; TRANSCRIPT_MAX_BYTES]).is_err());
    assert_eq!(one.as_bytes(), original);
    let mut full = CryptoTranscript::new(TranscriptKind::Content);
    for tag in 1..=TRANSCRIPT_MAX_FIELDS {
        full.field(tag, b"")?;
    }
    assert!(full.field(TRANSCRIPT_MAX_FIELDS + 1, b"").is_err());
    Ok(())
}

#[test]
fn transcript_golden_encoding() -> Result<()> {
    let mut transcript = CryptoTranscript::new(TranscriptKind::Manifest);
    transcript.field(1, b"ab")?;
    assert_eq!(
        transcript.as_bytes(),
        b"skrifheim/transcript/v1\0\x02\x01\0\x02\0\0\0ab"
    );
    Ok(())
}

#[test]
fn wrapper_rejects_wrong_context_and_provider_downgrade() -> Result<()> {
    struct WrongProvider;
    impl DigestProvider for WrongProvider {
        fn compute(&self, _: DigestStrength, _: &CryptoTranscript) -> Result<DigestValue> {
            DigestValue::new(DigestStrength::Sha3_256, &[1; 32])
        }
    }
    assert!(
        ContentDigest::compute(
            &WrongProvider,
            DigestPolicy::HIGH_SECURITY,
            &CryptoTranscript::new(TranscriptKind::Manifest)
        )
        .is_err()
    );
    assert!(
        ContentDigest::compute(
            &WrongProvider,
            DigestPolicy::MILITARY,
            &CryptoTranscript::new(TranscriptKind::Content)
        )
        .is_err()
    );
    Ok(())
}
