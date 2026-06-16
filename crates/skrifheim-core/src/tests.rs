use super::*;
use alloc::vec;

#[test]
fn open_time_range_contains_later_time() -> Result<()> {
    let range = TimeRange::new(Timestamp(10), None)?;
    assert!(range.contains(Timestamp(11)));
    Ok(())
}

#[test]
fn closed_time_range_excludes_end() -> Result<()> {
    let range = TimeRange::new(Timestamp(10), Some(Timestamp(20)))?;
    assert!(range.contains(Timestamp(19)));
    assert!(!range.contains(Timestamp(20)));
    Ok(())
}

#[test]
fn time_range_rejects_inverted_bounds() {
    for end in [Timestamp(10), Timestamp(20)] {
        assert_eq!(
            TimeRange::new(Timestamp(20), Some(end)),
            Err(SkrifheimError::InvalidTimeRange)
        );
    }
}

#[test]
fn higher_classification_dominates_lower_classification() {
    assert!(Classification::Secret.dominates(Classification::Restricted));
    assert!(!Classification::Internal.dominates(Classification::Secret));
}

#[test]
fn ids_reject_zero_values() {
    assert_eq!(TenantId::from_u128(0), None);
    assert_eq!(TenantId::from_u128(1).map(TenantId::get), Some(1));
}

#[test]
fn security_label_canonicalizes_sets() -> Result<()> {
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("eu-command"), String::from("EU-COMMAND")],
        vec![String::from("eu")],
    )?;
    assert_eq!(label.compartments().len(), 1);
    assert!(label.compartments().contains("EU-COMMAND"));
    assert!(label.releasable_to().contains("EU"));
    Ok(())
}

#[test]
fn security_label_rejects_oversized_policy_token_sets() {
    let mut compartments = Vec::new();
    for _ in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
        compartments.push(String::from("C"));
    }
    assert!(matches!(
        SecurityLabel::new(Classification::Secret, compartments, Vec::new()),
        Err(SkrifheimError::InvalidSecurityToken)
    ));
}

#[test]
fn security_label_rejects_unicode_homograph_tokens() {
    assert!(matches!(
        SecurityLabel::new(
            Classification::Secret,
            vec![String::from("ЕU-COMMAND")],
            Vec::new(),
        ),
        Err(SkrifheimError::InvalidSecurityToken)
    ));
}

#[test]
fn constant_time_policy_token_lookup_is_case_insensitive() -> Result<()> {
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("eu-command")],
        Vec::new(),
    )?;
    assert!(contains_policy_token_ct(label.compartments(), "EU-COMMAND"));
    assert!(!contains_policy_token_ct(
        label.compartments(),
        "EU-COMMAND-X"
    ));
    Ok(())
}

#[test]
fn constant_time_policy_token_lookup_rejects_malformed_needles() -> Result<()> {
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("eu-command")],
        Vec::new(),
    )?;
    assert!(!contains_policy_token_ct(
        label.compartments(),
        "EU COMMAND"
    ));
    Ok(())
}

#[test]
fn constant_time_policy_token_lookup_scans_full_token_sets() -> Result<()> {
    let mut tokens = Vec::new();
    for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
        tokens.push(alloc::format!("TOKEN-{index}"));
    }
    let tokens = PolicyTokenSet::new(tokens)?;
    assert!(contains_policy_token_ct(&tokens, "TOKEN-63"));
    assert!(!contains_policy_token_ct(&tokens, "TOKEN-64"));
    Ok(())
}

#[test]
fn policy_token_slot_access_is_bounded() -> Result<()> {
    let tokens = PolicyTokenSet::new(vec![String::from("TOKEN-0")])?;

    assert!(tokens.slot(0).is_some());
    assert!(tokens.slot(POLICY_TOKEN_SET_MAX_ITEMS).is_none());
    Ok(())
}

#[test]
fn policy_token_union_deduplicates_without_exposing_tokens() -> Result<()> {
    let left = PolicyTokenSet::new(vec![String::from("eu"), String::from("se")])?;
    let right = PolicyTokenSet::new(vec![String::from("SE"), String::from("no")])?;
    let merged = left.union(&right)?;
    let reversed = right.union(&left)?;

    assert_eq!(merged.len(), 3);
    assert!(merged.contains("EU"));
    assert!(merged.contains("SE"));
    assert!(merged.contains("NO"));
    assert!(merged.structurally_equal(&reversed));
    Ok(())
}

#[test]
fn policy_token_union_rejects_overflow() -> Result<()> {
    let mut left = Vec::new();
    for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
        left.push(alloc::format!("TOKEN-{index}"));
    }
    let left = PolicyTokenSet::new(left)?;
    let right = PolicyTokenSet::new(vec![String::from("EXTRA")])?;

    assert!(matches!(
        left.union(&right),
        Err(SkrifheimError::InvalidSecurityToken)
    ));
    Ok(())
}

#[test]
fn debug_redacts_security_labels_policy_tokens_and_values() -> Result<()> {
    let label = SecurityLabel::new(
        Classification::TopSecret,
        vec![String::from("EU-COMMAND")],
        vec![String::from("EU")],
    )?;
    let token_set_debug = alloc::format!("{:?}", label.compartments());
    let label_debug = alloc::format!("{label:?}");
    let text_debug = alloc::format!("{:?}", Value::Text(String::from("classified payload")));
    let bytes_debug = alloc::format!("{:?}", Value::Bytes(vec![1, 2, 3]));

    assert!(!token_set_debug.contains("EU-COMMAND"));
    assert!(!token_set_debug.contains("\"EU\""));
    assert!(!token_set_debug.contains("len: 1"));
    assert!(!label_debug.contains("EU-COMMAND"));
    assert!(!label_debug.contains("\"EU\""));
    assert!(!label_debug.contains("TopSecret"));
    assert!(!label_debug.contains("compartment_count: 1"));
    assert!(!label_debug.contains("releasable_to_count: 1"));
    assert!(!text_debug.contains("classified payload"));
    assert!(!bytes_debug.contains("[1, 2, 3]"));
    assert!(text_debug.contains("<redacted>"));
    assert!(bytes_debug.contains("<redacted>"));
    Ok(())
}

#[test]
fn public_message_hides_storage_header_detail() {
    let error = SkrifheimError::InvalidStorageHeader(String::from("segment magic mismatch"));
    assert_eq!(
        alloc::format!("{error}"),
        "invalid storage header: segment magic mismatch"
    );
    assert_eq!(error.public_message(), "invalid storage data");
}
