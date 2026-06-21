use alloc::format;

use super::*;

#[test]
fn digest_strengths_report_expected_lengths_and_quantum_margin() {
    assert_eq!(DigestStrength::Sha3_256.output_bytes(), 32);
    assert_eq!(DigestStrength::Sha3_384.output_bytes(), 48);
    assert_eq!(DigestStrength::Sha3_512.output_bytes(), 64);
    assert_eq!(DigestStrength::Shake256_256.output_bytes(), 32);
    assert_eq!(DigestStrength::Shake256_512.output_bytes(), 64);
    assert_eq!(
        DigestStrength::Sha3_256.quantum_preimage_security_bits(),
        128
    );
    assert_eq!(
        DigestStrength::Sha3_384.quantum_preimage_security_bits(),
        192
    );
    assert_eq!(
        DigestStrength::Sha3_512.quantum_preimage_security_bits(),
        256
    );
    assert!(DigestStrength::Sha3_512.is_military_profile());
    assert!(DigestStrength::Shake256_512.is_military_profile());
}

#[test]
fn digest_values_require_matching_policy_length() -> Result<()> {
    let high = DigestPolicy::HIGH_SECURITY;
    let long = DigestPolicy::LONG_HORIZON;
    let military = DigestPolicy::MILITARY;

    assert!(WorldIdentityDigest::new(high, &[1; 32]).is_ok());
    assert!(ContentDigest::new(long, &[2; 48]).is_ok());
    assert!(ManifestDigest::new(military, &[3; 64]).is_ok());
    assert!(matches!(
        WorldIdentityDigest::new(high, &[1; 31]),
        Err(SkrifheimError::InvalidDigest)
    ));
    assert!(matches!(
        ContentDigest::new(long, &[2; 64]),
        Err(SkrifheimError::InvalidDigest)
    ));
    Ok(())
}

#[test]
fn digest_policy_mismatch_is_rejected() -> Result<()> {
    let digest = ManifestDigest::new(DigestPolicy::MILITARY, &[9; 64])?;

    assert!(digest.require_policy(DigestPolicy::MILITARY).is_ok());
    assert_eq!(
        digest.require_policy(DigestPolicy::HIGH_SECURITY),
        Err(SkrifheimError::InvalidDigest)
    );
    Ok(())
}

#[test]
fn digest_constant_time_comparison_checks_strength_and_bytes() -> Result<()> {
    let left = WorldIdentityDigest::new(DigestPolicy::HIGH_SECURITY, &[7; 32])?;
    let same = WorldIdentityDigest::new(DigestPolicy::HIGH_SECURITY, &[7; 32])?;
    let different_bytes = WorldIdentityDigest::new(DigestPolicy::HIGH_SECURITY, &[8; 32])?;
    let different_strength =
        WorldIdentityDigest::new(DigestPolicy::new(DigestStrength::Shake256_256), &[7; 32])?;

    assert!(left.structurally_equal_ct(&same));
    assert!(!left.structurally_equal_ct(&different_bytes));
    assert!(!left.structurally_equal_ct(&different_strength));
    Ok(())
}

#[test]
fn digest_debug_redacts_digest_bytes() -> Result<()> {
    let digest = WorldIdentityDigest::new(DigestPolicy::HIGH_SECURITY, &[7; 32])?;
    let debug = format!("{digest:?}");

    assert!(debug.contains("strength: \"<redacted>\""));
    assert!(debug.contains("digest: \"<redacted>\""));
    assert!(!debug.contains('7'));
    assert!(!debug.contains("Sha3"));
    Ok(())
}
