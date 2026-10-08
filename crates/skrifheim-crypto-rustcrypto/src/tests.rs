use super::*;
use skrifheim_crypto::{
    ContentDigest, DigestPolicy, ManifestDigest, TranscriptKind, WorldIdentityDigest,
};

fn bytes(hex: &str) -> [u8; 64] {
    let mut result = [0; 64];
    for (i, pair) in hex.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let nibble = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
        result[i] = nibble(pair[0]) * 16 + nibble(pair[1]);
    }
    result
}

#[test]
fn fips202_empty_message_known_answers() -> Result<()> {
    for (strength, hex) in [
        (
            DigestStrength::Sha3_256,
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
        ),
        (
            DigestStrength::Sha3_384,
            "0c63a75b845e4f7d01107d852e4c2485c51a50aaaa94fc61995e71bbee983a2ac3713831264adb47fb6bd1e058d5f004",
        ),
        (
            DigestStrength::Sha3_512,
            "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26",
        ),
        (
            DigestStrength::Shake256_256,
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f",
        ),
        (
            DigestStrength::Shake256_512,
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762fd75dc4ddd8c0f200cb05019d67b592f6fc821c49479ab48640292eacb3b7c4be",
        ),
    ] {
        assert_eq!(
            digest(strength, b"")?.as_bytes(),
            &bytes(hex)[..strength.output_bytes()]
        );
    }
    Ok(())
}

#[test]
fn transcript_digest_golden_vectors_match_independent_hashlib() -> Result<()> {
    let mut transcript = CryptoTranscript::new(TranscriptKind::Content);
    transcript.field(1, b"abc")?;
    for (strength, hex) in [
        (
            DigestStrength::Sha3_256,
            "649909dc9d6900f2c4c74ffdd3120fed910f5550f45fa615a01a5af375d3a295",
        ),
        (
            DigestStrength::Sha3_384,
            "e5bbedbbd5ff8e726deaab523aa4a52a8e304c7233d2ad3d3c0e718153b1689ebd4ac4de83097deb1f94497ee8c03134",
        ),
        (
            DigestStrength::Sha3_512,
            "3796d6bb2d9aa9a13bc71a5360ea6742db12e91b92dcbb097dc81be7f272260459d03b6a8e0c4c6c3eab089bf73c563400cb5ab65de0d0b8f6724455696050d2",
        ),
        (
            DigestStrength::Shake256_256,
            "7066b56349ca2152fe10743c406207b183a3d63f3fdb35ae8009077197722a0d",
        ),
        (
            DigestStrength::Shake256_512,
            "7066b56349ca2152fe10743c406207b183a3d63f3fdb35ae8009077197722a0df09921f8f0f383ea50a33912772c2de47b121a7f6f1325e14b76b7a2bbde040d",
        ),
    ] {
        let actual = ContentDigest::compute(
            &RustCryptoProvider,
            DigestPolicy::new(strength),
            &transcript,
        )?;
        assert_eq!(
            actual.digest_bytes(),
            &bytes(hex)[..strength.output_bytes()]
        );
    }
    Ok(())
}

#[test]
fn all_profiles_compute_typed_domain_separated_digests() -> Result<()> {
    for strength in [
        DigestStrength::Sha3_256,
        DigestStrength::Sha3_384,
        DigestStrength::Sha3_512,
        DigestStrength::Shake256_256,
        DigestStrength::Shake256_512,
    ] {
        let policy = DigestPolicy::new(strength);
        let mut content = CryptoTranscript::new(TranscriptKind::Content);
        let mut manifest = CryptoTranscript::new(TranscriptKind::Manifest);
        let mut world = CryptoTranscript::new(TranscriptKind::WorldIdentity);
        for transcript in [&mut content, &mut manifest, &mut world] {
            transcript.field(1, b"same bytes")?;
        }
        let a = ContentDigest::compute(&RustCryptoProvider, policy, &content)?;
        let b = ManifestDigest::compute(&RustCryptoProvider, policy, &manifest)?;
        let c = WorldIdentityDigest::compute(&RustCryptoProvider, policy, &world)?;
        assert_eq!(a.digest_bytes().len(), strength.output_bytes());
        assert_ne!(a.digest_bytes(), b.digest_bytes());
        assert_ne!(a.digest_bytes(), c.digest_bytes());
        assert_ne!(b.digest_bytes(), c.digest_bytes());
        let proof = a.verify(&RustCryptoProvider, policy, &content)?;
        assert_eq!(proof.digest_bytes(), a.digest_bytes());
        assert_eq!(proof.kind(), TranscriptKind::Content);
        let forged = ContentDigest::new(policy, &alloc::vec![0; strength.output_bytes()])?;
        assert!(
            forged
                .verify(&RustCryptoProvider, policy, &content)
                .is_err()
        );
    }
    Ok(())
}
