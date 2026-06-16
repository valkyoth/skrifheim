use alloc::format;

use skrifheim_core::{
    ActorId, AttestationEvidenceId, AuditEventId, DeviceId, TenantId, Timestamp, WorkloadId,
};
use skrifheim_crypto::{
    AlgorithmId, CryptoEpoch, EncryptionDomain, SignatureEnvelope, SignatureSet,
};

use super::*;

fn id<T>(value: Option<T>) -> Result<T> {
    value.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant() -> Result<TenantId> {
    id(TenantId::from_u128(1))
}

fn actor() -> Result<AuditIdentity> {
    Ok(AuditIdentity::Actor(id(ActorId::from_u128(2))?))
}

fn device_attestation() -> Result<AttestationEvidenceRef> {
    AttestationEvidenceRef::new(
        id(AttestationEvidenceId::from_u128(3))?,
        actor()?,
        Timestamp::new(10),
        Some(Timestamp::new(20)),
    )
}

fn signatures() -> Result<SignatureSet> {
    SignatureSet::new(alloc::vec![SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch::new(1),
        "audit-key",
        alloc::vec![7; 64],
    )?])
}

fn event_input() -> Result<AuditEventInput> {
    Ok(AuditEventInput {
        event_id: id(AuditEventId::from_u128(4))?,
        tenant_id: tenant()?,
        occurred_at: Timestamp::new(11),
        actor: Some(actor()?),
        device: Some(DeviceAuditContext::new(
            id(DeviceId::from_u128(5))?,
            Some(device_attestation()?),
        )),
        workload: Some(WorkloadAuditContext::new(
            id(WorkloadId::from_u128(6))?,
            Some(device_attestation()?),
        )),
        kind: AuditEventKind::PolicyDecision,
        world_id: None,
        fact_id: None,
        tx_id: None,
        policy_id: None,
        crypto_epoch: CryptoEpoch::new(1),
    })
}

#[test]
fn audit_event_requires_actor_attribution() -> Result<()> {
    let mut input = event_input()?;
    input.actor = None;

    assert!(matches!(
        AuditEvent::new(input),
        Err(SkrifheimError::MissingAuditActor)
    ));
    Ok(())
}

#[test]
fn audit_event_tracks_actor_and_attestation_presence() -> Result<()> {
    let event = AuditEvent::new(event_input()?)?;

    assert_eq!(event.actor_kind(), IdentityKind::Actor);
    assert!(event.has_device_attestation());
    assert!(event.has_workload_attestation());
    assert_eq!(event.kind(), AuditEventKind::PolicyDecision);
    Ok(())
}

#[test]
fn attestation_rejects_inverted_time_bounds() -> Result<()> {
    let error = AttestationEvidenceRef::new(
        id(AttestationEvidenceId::from_u128(7))?,
        actor()?,
        Timestamp::new(20),
        Some(Timestamp::new(20)),
    );

    assert!(matches!(
        error,
        Err(SkrifheimError::InvalidAttestationEvidence)
    ));
    Ok(())
}

#[test]
fn break_glass_requires_device_and_workload_context() -> Result<()> {
    let mut input = event_input()?;
    input.kind = AuditEventKind::BreakGlass(BreakGlassJustification::IncidentContainment);
    input.device = None;

    assert!(matches!(
        AuditEvent::new(input),
        Err(SkrifheimError::InvalidAuditEvent)
    ));
    Ok(())
}

#[test]
fn break_glass_requires_attested_device_and_workload_context() -> Result<()> {
    let mut input = event_input()?;
    input.kind = AuditEventKind::BreakGlass(BreakGlassJustification::TenantRecovery);
    input.device = Some(DeviceAuditContext::new(id(DeviceId::from_u128(5))?, None));

    assert!(matches!(
        AuditEvent::new(input),
        Err(SkrifheimError::InvalidAuditEvent)
    ));

    let mut input = event_input()?;
    input.kind = AuditEventKind::BreakGlass(BreakGlassJustification::TenantRecovery);
    input.workload = Some(WorkloadAuditContext::new(
        id(WorkloadId::from_u128(6))?,
        None,
    ));

    assert!(matches!(
        AuditEvent::new(input),
        Err(SkrifheimError::InvalidAuditEvent)
    ));
    Ok(())
}

#[test]
fn audit_event_rejects_stale_or_future_attestation() -> Result<()> {
    let mut stale = event_input()?;
    stale.occurred_at = Timestamp::new(30);
    assert!(matches!(
        AuditEvent::new(stale),
        Err(SkrifheimError::InvalidAttestationEvidence)
    ));

    let mut future = event_input()?;
    future.occurred_at = Timestamp::new(9);
    assert!(matches!(
        AuditEvent::new(future),
        Err(SkrifheimError::InvalidAttestationEvidence)
    ));
    Ok(())
}

#[test]
fn audit_log_protection_requires_audit_log_domain() -> Result<()> {
    let domain = EncryptionDomain::tenant(tenant()?);

    assert!(matches!(
        AuditLogProtection::new(domain, signatures()?, CryptoEpoch::new(1)),
        Err(SkrifheimError::InvalidAuditProtection)
    ));
    Ok(())
}

#[test]
fn audit_record_requires_matching_tenant() -> Result<()> {
    let event = AuditEvent::new(event_input()?)?;
    let other_tenant = id(TenantId::from_u128(8))?;
    let protection = AuditLogProtection::new(
        EncryptionDomain::audit_log(other_tenant, None),
        signatures()?,
        CryptoEpoch::new(1),
    )?;

    assert!(matches!(
        AuditRecord::new(event, protection),
        Err(SkrifheimError::InvalidAuditProtection)
    ));
    Ok(())
}

#[test]
fn audit_record_accepts_signed_encrypted_metadata() -> Result<()> {
    let event = AuditEvent::new(event_input()?)?;
    let protection = AuditLogProtection::new(
        EncryptionDomain::audit_log(tenant()?, None),
        signatures()?,
        CryptoEpoch::new(1),
    )?;
    let record = AuditRecord::new(event, protection)?;

    assert_eq!(record.protection().signature_count(), 1);
    assert_eq!(record.event().tenant_id().get(), tenant()?.get());
    Ok(())
}

#[test]
fn debug_output_redacts_ids_targets_and_crypto_metadata() -> Result<()> {
    let event = AuditEvent::new(event_input()?)?;
    let protection = AuditLogProtection::new(
        EncryptionDomain::audit_log(tenant()?, None),
        signatures()?,
        CryptoEpoch::new(1),
    )?;
    let debug = format!("{:?}", AuditRecord::new(event, protection)?);

    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("audit-key"));
    assert!(!debug.contains("event_id: 4"));
    assert!(!debug.contains("tenant_id: 1"));
    assert!(!debug.contains("crypto_epoch: 1"));
    assert!(!debug.contains("kind: Actor"));
    assert!(!debug.contains("BreakGlass"));
    assert!(!debug.contains("LifeSafety"));
    Ok(())
}

#[test]
fn debug_output_redacts_break_glass_kind() -> Result<()> {
    let mut input = event_input()?;
    input.kind = AuditEventKind::BreakGlass(BreakGlassJustification::LifeSafety);
    let debug = format!("{:?}", AuditEvent::new(input)?);

    assert!(debug.contains("kind: \"<redacted>\""));
    assert!(!debug.contains("BreakGlass"));
    assert!(!debug.contains("LifeSafety"));
    Ok(())
}
