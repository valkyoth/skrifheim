use super::*;
use skrifheim::{crypto::*, crypto_provider::RustCryptoProvider, storage::*};

#[test]
fn every_selected_target_executes_nonzero_corpus_cases() -> Result<(), &'static str> {
    for target in TARGETS {
        assert!(smoke(target)? >= 3);
    }
    assert!(smoke("unknown").is_err());
    assert!(exercise("wal_header", &alloc::vec![0;MAX_INPUT+1]).is_err());
    Ok(())
}

#[test]
fn public_wire_fixtures_are_accepted_and_exact() -> skrifheim::core::Result<()> {
    let wal = WalFrameHeader::parse(&seeds::wal())?;
    assert_eq!(wal.encode().as_slice(), seeds::wal());
    assert_eq!(wal.tx_id().get(), 1);
    let header = SegmentHeader::parse(&seeds::segment())?;
    let footer = SegmentFooter::parse(&seeds::footer())?;
    assert_eq!(header.encode().as_slice(), seeds::segment());
    assert_eq!(footer.encode().as_slice(), seeds::footer());
    footer.validate_against_header(&header)?;
    let mut other = seeds::footer();
    other[16] = 2;
    assert!(
        SegmentFooter::parse(&other)?
            .validate_against_header(&header)
            .is_err()
    );
    assert!(!segment_file_length_matches(1, 512));
    assert!(segment_file_length_matches(1, 513));
    assert!(!segment_file_length_matches(u64::MAX, u64::MAX));
    Ok(())
}

#[test]
fn public_crypto_consumer_matches_independent_vectors() -> skrifheim::core::Result<()> {
    assert_eq!(crypto::golden_seal(), Ok(seeds::envelope()));
    crypto::envelope(&seeds::envelope());
    let mut transcript = CryptoTranscript::new(TranscriptKind::Content);
    transcript.field(1, b"abc")?;
    assert_eq!(
        transcript.as_bytes(),
        b"skrifheim/transcript/v1\0\x01\x01\0\x03\0\0\0abc"
    );
    let digest = ContentDigest::compute(
        &RustCryptoProvider,
        DigestPolicy::new(DigestStrength::Sha3_256),
        &transcript,
    )?;
    assert_eq!(
        digest.digest_bytes(),
        &[
            0x64, 0x99, 0x09, 0xdc, 0x9d, 0x69, 0x00, 0xf2, 0xc4, 0xc7, 0x4f, 0xfd, 0xd3, 0x12,
            0x0f, 0xed, 0x91, 0x0f, 0x55, 0x50, 0xf4, 0x5f, 0xa6, 0x15, 0xa0, 0x1a, 0x5a, 0xf3,
            0x75, 0xd3, 0xa2, 0x95,
        ][..]
    );
    Ok(())
}

#[test]
fn transcript_field_and_byte_budgets_are_atomic() -> skrifheim::core::Result<()> {
    let mut transcript = CryptoTranscript::new(TranscriptKind::Content);
    for tag in 1..=256 {
        transcript.field(tag, b"")?;
    }
    let before = transcript.as_bytes().to_vec();
    assert!(transcript.field(257, b"").is_err());
    assert_eq!(transcript.as_bytes(), before);
    let mut transcript = CryptoTranscript::new(TranscriptKind::Content);
    let body = alloc::vec![0;TRANSCRIPT_MAX_BYTES-transcript.as_bytes().len()-6];
    transcript.field(1, &body)?;
    let before = transcript.as_bytes().to_vec();
    assert!(transcript.field(2, b"").is_err());
    assert_eq!(transcript.as_bytes(), before);
    Ok(())
}
