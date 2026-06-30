use core::fmt;

use skrifheim_core::{Result, SkrifheimError, TenantId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, EncryptionDomainPurpose, SignatureSet};

use crate::AuditEvent;

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
