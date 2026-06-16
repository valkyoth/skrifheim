use super::*;
use alloc::{vec, vec::Vec};

#[test]
fn secret_bytes_wraps_and_exposes_only_through_closure() -> Result<()> {
    let secret = SecretBytes::from_slice(b"tenant-key-material")?;

    assert_eq!(secret.len(), 19);
    assert!(!secret.is_empty());
    assert_eq!(secret.with_secret(|bytes| bytes[0]), b't');
    assert_eq!(
        secret.try_with_secret::<_, ()>(|bytes| Ok(bytes.len())),
        Ok(19)
    );
    Ok(())
}

#[test]
fn secret_bytes_rejects_empty_and_oversized_inputs() {
    assert!(matches!(
        SecretBytes::from_slice(&[]),
        Err(SkrifheimError::InvalidSecretMaterial)
    ));
    assert!(matches!(
        SecretBytes::from_slice(&vec![7; SECRET_VALUE_MAX_BYTES + 1]),
        Err(SkrifheimError::InvalidSecretMaterial)
    ));
}

#[test]
fn secret_bytes_rejects_excess_vec_capacity() {
    let mut bytes = Vec::with_capacity(SECRET_VALUE_MAX_BYTES + 1);
    bytes.extend_from_slice(b"small-secret");

    assert!(matches!(
        SecretBytes::from_vec(bytes),
        Err(SkrifheimError::InvalidSecretMaterial)
    ));
}

#[test]
fn secret_bytes_debug_redacts_contents_and_size() -> Result<()> {
    let secret = SecretBytes::from_slice(b"do-not-log-this-secret")?;
    let debug = alloc::format!("{secret:?}");

    assert!(debug.contains("SecretBytes"));
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("do-not-log"));
    assert!(!debug.contains("22"));
    Ok(())
}

#[test]
fn secret_bytes_can_be_cleared_explicitly() -> Result<()> {
    let mut secret = SecretBytes::from_slice(b"clear-me")?;
    secret.clear_secret();

    assert_eq!(secret.len(), 0);
    assert!(secret.is_empty());
    assert_eq!(secret.with_secret(|bytes| bytes.len()), 0);
    Ok(())
}

#[test]
fn secret_error_messages_do_not_include_secret_material() {
    let error = SkrifheimError::InvalidSecretMaterial;

    assert!(!alloc::format!("{error}").contains("tenant-key-material"));
    assert_eq!(error.public_message(), "invalid secret material");
}
