use super::*;
use alloc::{string::String, vec, vec::Vec};

fn ed25519_envelope(index: usize) -> Result<SignatureEnvelope> {
    SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch::new(1),
        alloc::format!("key-{index}"),
        vec![1; ED25519_SIG_BYTES],
    )
}

#[test]
fn empty_signature_set_is_rejected() {
    assert!(matches!(
        SignatureSet::new(Vec::new()),
        Err(SkrifheimError::EmptySignatureSet)
    ));
}

#[test]
fn signature_set_count_is_bounded() -> Result<()> {
    let mut signatures = Vec::new();
    for index in 0..=MAX_SIGNATURES_PER_SET {
        signatures.push(ed25519_envelope(index)?);
    }
    assert!(matches!(
        SignatureSet::new(signatures),
        Err(SkrifheimError::TooManySignatures)
    ));
    Ok(())
}

#[test]
fn signature_set_rejects_duplicate_signers() -> Result<()> {
    let first = SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch::new(1),
        "same-key",
        vec![1; ED25519_SIG_BYTES],
    )?;
    let second = SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch::new(2),
        "same-key",
        vec![2; ED25519_SIG_BYTES],
    )?;
    assert!(matches!(
        SignatureSet::new(vec![first, second]),
        Err(SkrifheimError::DuplicateSignatureSigner)
    ));
    Ok(())
}

#[test]
fn debug_redacts_signature_bytes_and_set_contents() -> Result<()> {
    let envelope = SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch::new(1),
        "debug-key",
        vec![255; ED25519_SIG_BYTES],
    )?;
    let envelope_debug = alloc::format!("{envelope:?}");
    assert!(envelope_debug.contains("signature_bytes"));
    assert!(!envelope_debug.contains("Ed25519"));
    assert!(!envelope_debug.contains("debug-key"));
    assert!(!envelope_debug.contains("CryptoEpoch"));
    assert!(!envelope_debug.contains("64"));
    assert!(!envelope_debug.contains("[255"));

    let set = SignatureSet::new(vec![envelope])?;
    let set_debug = alloc::format!("{set:?}");
    assert!(set_debug.contains("count"));
    assert!(!set_debug.contains("debug-key"));
    assert!(!set_debug.contains("[255"));
    Ok(())
}

#[test]
fn empty_signature_bytes_are_rejected() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch::new(1), "k", Vec::new()),
        Err(SkrifheimError::EmptySignatureSet)
    ));
}

#[test]
fn signature_key_ids_are_bounded() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch::new(1), "", vec![1; 64]),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id length out of range"
        ))
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch::new(1),
            String::from_utf8(vec![b'k'; KEY_ID_MAX_BYTES + 1]).unwrap_or_else(|_| String::new()),
            vec![1; 64]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id length out of range"
        ))
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch::new(1),
            "key with spaces",
            vec![1; 64]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id contains invalid characters"
        ))
    ));
}

#[test]
fn ed25519_signature_length_is_enforced() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch::new(1), "k", vec![1; 63]),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch::new(1), "k", vec![1; 64]).is_ok()
    );
}

#[test]
fn variable_signature_length_is_bounded() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch::new(1),
            "k",
            vec![1; ED25519_SIG_BYTES - 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch::new(1),
            "k",
            vec![1; MAX_VARIABLE_SIGNATURE_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch::new(1),
            "k",
            vec![1; ED25519_SIG_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch::new(1),
            "k",
            vec![1; ED25519_SIG_BYTES],
        )
        .is_ok()
    );
}

#[test]
fn variable_signature_ceiling_is_enforced_before_algorithm_lengths() {
    assert!(matches!(
        validate_signature_envelope_parts(
            &AlgorithmId::Ed25519,
            &vec![1; MAX_VARIABLE_SIGNATURE_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
}

#[test]
fn crypto_epoch_is_opaque_and_readable() {
    let epoch = CryptoEpoch::new(9);
    assert_eq!(epoch.get(), 9);
}

#[test]
fn hybrid_signature_length_requires_component_minimums() {
    let algorithm = AlgorithmId::HybridClassicalPq {
        classical: String::from("ED25519"),
        post_quantum: String::from("ML-DSA-65"),
    };
    let min_len = ED25519_SIG_BYTES + ML_DSA_65_SIG_BYTES;

    assert!(matches!(
        SignatureEnvelope::new(
            algorithm.clone(),
            CryptoEpoch::new(1),
            "k",
            vec![1; min_len - 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            algorithm.clone(),
            CryptoEpoch::new(1),
            "k",
            vec![1; min_len + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(SignatureEnvelope::new(algorithm, CryptoEpoch::new(1), "k", vec![1; min_len]).is_ok());
}

#[test]
fn blake3_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Blake3, CryptoEpoch::new(1), "k", vec![1; 32]),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm is not valid for signatures"
        ))
    ));
}

#[test]
fn empty_named_algorithm_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::new()),
            CryptoEpoch::new(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is empty or too long"
        ))
    ));
}

#[test]
fn unapproved_named_algorithm_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("DEBUG-SKIP")),
            CryptoEpoch::new(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures"
        ))
    ));
}

#[test]
fn hybrid_algorithm_names_are_validated() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::HybridClassicalPq {
                classical: String::from(""),
                post_quantum: String::from("ML-DSA-65"),
            },
            CryptoEpoch::new(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is empty or too long"
        ))
    ));
}

#[test]
fn hybrid_algorithm_components_must_be_approved() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::HybridClassicalPq {
                classical: String::from("ED25519"),
                post_quantum: String::from("DEBUG-SKIP"),
            },
            CryptoEpoch::new(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures"
        ))
    ));
}
