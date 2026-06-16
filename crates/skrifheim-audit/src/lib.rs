#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use core::fmt;

use skrifheim_core::{
    ActorId, AdminToolId, AiWorkerId, AttestationEvidenceId, AuditEventId, BackupAgentId, DeviceId,
    FactId, NodeId, PluginId, PolicyId, ReplicaId, Result, ServiceId, SkrifheimError, TenantId,
    Timestamp, TxId, UserId, WorkloadId, WorldId,
};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, EncryptionDomainPurpose, SignatureSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityKind {
    Actor,
    User,
    Device,
    Workload,
    Service,
    Node,
    Replica,
    Plugin,
    AiWorker,
    BackupAgent,
    AdminTool,
}

#[derive(Clone, Copy)]
pub enum AuditIdentity {
    Actor(ActorId),
    User(UserId),
    Device(DeviceId),
    Workload(WorkloadId),
    Service(ServiceId),
    Node(NodeId),
    Replica(ReplicaId),
    Plugin(PluginId),
    AiWorker(AiWorkerId),
    BackupAgent(BackupAgentId),
    AdminTool(AdminToolId),
}

impl AuditIdentity {
    #[must_use]
    pub const fn kind(self) -> IdentityKind {
        match self {
            Self::Actor(_) => IdentityKind::Actor,
            Self::User(_) => IdentityKind::User,
            Self::Device(_) => IdentityKind::Device,
            Self::Workload(_) => IdentityKind::Workload,
            Self::Service(_) => IdentityKind::Service,
            Self::Node(_) => IdentityKind::Node,
            Self::Replica(_) => IdentityKind::Replica,
            Self::Plugin(_) => IdentityKind::Plugin,
            Self::AiWorker(_) => IdentityKind::AiWorker,
            Self::BackupAgent(_) => IdentityKind::BackupAgent,
            Self::AdminTool(_) => IdentityKind::AdminTool,
        }
    }
}

impl fmt::Debug for AuditIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditIdentity")
            .field("kind", &self.kind())
            .field("id", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct AttestationEvidenceRef {
    evidence_id: AttestationEvidenceId,
    issuer: AuditIdentity,
    captured_at: Timestamp,
    expires_at: Option<Timestamp>,
}

impl AttestationEvidenceRef {
    pub const fn new(
        evidence_id: AttestationEvidenceId,
        issuer: AuditIdentity,
        captured_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Result<Self> {
        if let Some(expires_at) = expires_at
            && expires_at.get() <= captured_at.get()
        {
            return Err(SkrifheimError::InvalidAttestationEvidence);
        }
        Ok(Self {
            evidence_id,
            issuer,
            captured_at,
            expires_at,
        })
    }

    #[must_use]
    pub const fn evidence_id(self) -> AttestationEvidenceId {
        self.evidence_id
    }

    #[must_use]
    pub const fn issuer(self) -> AuditIdentity {
        self.issuer
    }

    #[must_use]
    pub const fn is_current_at(self, timestamp: Timestamp) -> bool {
        timestamp.get() >= self.captured_at.get()
            && match self.expires_at {
                Some(expires_at) => timestamp.get() < expires_at.get(),
                None => true,
            }
    }
}

impl fmt::Debug for AttestationEvidenceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttestationEvidenceRef")
            .field("evidence_id", &"<redacted>")
            .field("issuer", &self.issuer)
            .field("time_bounds", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct DeviceAuditContext {
    device_id: DeviceId,
    attestation: Option<AttestationEvidenceRef>,
}

impl DeviceAuditContext {
    #[must_use]
    pub const fn new(device_id: DeviceId, attestation: Option<AttestationEvidenceRef>) -> Self {
        Self {
            device_id,
            attestation,
        }
    }

    #[must_use]
    pub const fn has_attestation(self) -> bool {
        self.attestation.is_some()
    }

    #[must_use]
    pub const fn device_id(self) -> DeviceId {
        self.device_id
    }
}

impl fmt::Debug for DeviceAuditContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceAuditContext")
            .field("device_id", &"<redacted>")
            .field("has_attestation", &self.has_attestation())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct WorkloadAuditContext {
    workload_id: WorkloadId,
    attestation: Option<AttestationEvidenceRef>,
}

impl WorkloadAuditContext {
    #[must_use]
    pub const fn new(workload_id: WorkloadId, attestation: Option<AttestationEvidenceRef>) -> Self {
        Self {
            workload_id,
            attestation,
        }
    }

    #[must_use]
    pub const fn has_attestation(self) -> bool {
        self.attestation.is_some()
    }

    #[must_use]
    pub const fn workload_id(self) -> WorkloadId {
        self.workload_id
    }
}

impl fmt::Debug for WorkloadAuditContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkloadAuditContext")
            .field("workload_id", &"<redacted>")
            .field("has_attestation", &self.has_attestation())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakGlassJustification {
    LifeSafety,
    IncidentContainment,
    CourtOrRegulatorOrder,
    TenantRecovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditEventKind {
    FactWrite,
    FactRead,
    PolicyDecision,
    WorldFork,
    WorldPromotion,
    KeyLifecycle,
    ProjectionBuild,
    BreakGlass(BreakGlassJustification),
}

pub struct AuditEventInput {
    pub event_id: AuditEventId,
    pub tenant_id: TenantId,
    pub occurred_at: Timestamp,
    pub actor: Option<AuditIdentity>,
    pub device: Option<DeviceAuditContext>,
    pub workload: Option<WorkloadAuditContext>,
    pub kind: AuditEventKind,
    pub world_id: Option<WorldId>,
    pub fact_id: Option<FactId>,
    pub tx_id: Option<TxId>,
    pub policy_id: Option<PolicyId>,
    pub crypto_epoch: CryptoEpoch,
}

pub struct AuditEvent {
    event_id: AuditEventId,
    tenant_id: TenantId,
    occurred_at: Timestamp,
    actor: AuditIdentity,
    device: Option<DeviceAuditContext>,
    workload: Option<WorkloadAuditContext>,
    kind: AuditEventKind,
    world_id: Option<WorldId>,
    fact_id: Option<FactId>,
    tx_id: Option<TxId>,
    policy_id: Option<PolicyId>,
    crypto_epoch: CryptoEpoch,
}

impl AuditEvent {
    pub fn new(input: AuditEventInput) -> Result<Self> {
        let actor = input.actor.ok_or(SkrifheimError::MissingAuditActor)?;
        if matches!(input.kind, AuditEventKind::BreakGlass(_))
            && (input.device.is_none() || input.workload.is_none())
        {
            return Err(SkrifheimError::InvalidAuditEvent);
        }
        Ok(Self {
            event_id: input.event_id,
            tenant_id: input.tenant_id,
            occurred_at: input.occurred_at,
            actor,
            device: input.device,
            workload: input.workload,
            kind: input.kind,
            world_id: input.world_id,
            fact_id: input.fact_id,
            tx_id: input.tx_id,
            policy_id: input.policy_id,
            crypto_epoch: input.crypto_epoch,
        })
    }

    #[must_use]
    pub const fn event_id(&self) -> AuditEventId {
        self.event_id
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    #[must_use]
    pub const fn actor_kind(&self) -> IdentityKind {
        self.actor.kind()
    }

    #[must_use]
    pub const fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }

    #[must_use]
    pub const fn kind(&self) -> AuditEventKind {
        self.kind
    }

    #[must_use]
    pub const fn has_device_attestation(&self) -> bool {
        match self.device {
            Some(device) => device.has_attestation(),
            None => false,
        }
    }

    #[must_use]
    pub const fn has_workload_attestation(&self) -> bool {
        match self.workload {
            Some(workload) => workload.has_attestation(),
            None => false,
        }
    }

    #[must_use]
    pub const fn has_world_target(&self) -> bool {
        self.world_id.is_some()
    }

    #[must_use]
    pub const fn has_fact_target(&self) -> bool {
        self.fact_id.is_some()
    }

    #[must_use]
    pub const fn has_transaction_target(&self) -> bool {
        self.tx_id.is_some()
    }

    #[must_use]
    pub const fn has_policy_target(&self) -> bool {
        self.policy_id.is_some()
    }

    #[must_use]
    pub const fn crypto_epoch(&self) -> CryptoEpoch {
        self.crypto_epoch
    }
}

impl fmt::Debug for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditEvent")
            .field("event_id", &"<redacted>")
            .field("tenant_id", &"<redacted>")
            .field("occurred_at", &"<redacted>")
            .field("actor", &self.actor)
            .field("device", &"<redacted>")
            .field("workload", &"<redacted>")
            .field("kind", &self.kind)
            .field("targets", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .finish()
    }
}

pub struct AuditLogProtection {
    domain: EncryptionDomain,
    signatures: SignatureSet,
    crypto_epoch: CryptoEpoch,
}

impl AuditLogProtection {
    pub fn new(
        domain: EncryptionDomain,
        signatures: SignatureSet,
        crypto_epoch: CryptoEpoch,
    ) -> Result<Self> {
        if domain.purpose() != EncryptionDomainPurpose::AuditLog {
            return Err(SkrifheimError::InvalidAuditProtection);
        }
        Ok(Self {
            domain,
            signatures,
            crypto_epoch,
        })
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.domain.tenant_id()
    }

    #[must_use]
    pub fn signature_count(&self) -> usize {
        self.signatures.envelopes().len()
    }

    #[must_use]
    pub const fn crypto_epoch(&self) -> CryptoEpoch {
        self.crypto_epoch
    }
}

impl fmt::Debug for AuditLogProtection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditLogProtection")
            .field("domain", &"<redacted>")
            .field("signature_count", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .finish()
    }
}

pub struct AuditRecord {
    event: AuditEvent,
    protection: AuditLogProtection,
}

impl AuditRecord {
    pub fn new(event: AuditEvent, protection: AuditLogProtection) -> Result<Self> {
        if event.tenant_id().get() != protection.tenant_id().get() {
            return Err(SkrifheimError::InvalidAuditProtection);
        }
        Ok(Self { event, protection })
    }

    #[must_use]
    pub const fn event(&self) -> &AuditEvent {
        &self.event
    }

    #[must_use]
    pub const fn protection(&self) -> &AuditLogProtection {
        &self.protection
    }
}

impl fmt::Debug for AuditRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuditRecord")
            .field("event", &self.event)
            .field("protection", &self.protection)
            .finish()
    }
}

#[cfg(test)]
mod tests;
