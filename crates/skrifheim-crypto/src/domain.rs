use core::fmt;

use skrifheim_core::{Classification, TenantId, WorldId};
use subtle::ConstantTimeEq;

use crate::{CompartmentKeyId, RegionKeyId, SegmentKeyId};

const DOMAIN_FIXED_BYTES: usize = 86;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncryptionDomainPurpose {
    Tenant,
    Region,
    Classification,
    Compartment,
    World,
    Wal,
    Segment,
    Projection,
    Backup,
    ExportCapsule,
    AiArtifact,
    WasmPluginSecret,
    AuditLog,
}

#[derive(Clone, Copy)]
pub struct EncryptionDomain {
    purpose: EncryptionDomainPurpose,
    tenant_id: TenantId,
    region_id: Option<RegionKeyId>,
    classification: Option<Classification>,
    compartment_id: Option<CompartmentKeyId>,
    world_id: Option<WorldId>,
    segment_id: Option<SegmentKeyId>,
}

impl EncryptionDomain {
    #[must_use]
    pub const fn tenant(tenant_id: TenantId) -> Self {
        Self::new(
            EncryptionDomainPurpose::Tenant,
            tenant_id,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn region(tenant_id: TenantId, region_id: RegionKeyId) -> Self {
        Self::new(
            EncryptionDomainPurpose::Region,
            tenant_id,
            Some(region_id),
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn classification(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::Classification,
            tenant_id,
            region_id,
            Some(classification),
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn compartment(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
        compartment_id: CompartmentKeyId,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::Compartment,
            tenant_id,
            region_id,
            Some(classification),
            Some(compartment_id),
            None,
            None,
        )
    }

    #[must_use]
    pub const fn world(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Option<Classification>,
        compartment_id: Option<CompartmentKeyId>,
        world_id: WorldId,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::World,
            tenant_id,
            region_id,
            classification,
            compartment_id,
            Some(world_id),
            None,
        )
    }

    #[must_use]
    pub const fn wal(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        world_id: Option<WorldId>,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::Wal,
            tenant_id,
            region_id,
            None,
            None,
            world_id,
            None,
        )
    }

    #[must_use]
    pub const fn segment(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
        compartment_id: CompartmentKeyId,
        segment_id: SegmentKeyId,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::Segment,
            tenant_id,
            region_id,
            Some(classification),
            Some(compartment_id),
            None,
            Some(segment_id),
        )
    }

    #[must_use]
    pub const fn projection(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
        compartment_id: Option<CompartmentKeyId>,
        world_id: Option<WorldId>,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::Projection,
            tenant_id,
            region_id,
            Some(classification),
            compartment_id,
            world_id,
            None,
        )
    }

    #[cfg(test)]
    pub(crate) const fn projection_without_classification_for_test(tenant_id: TenantId) -> Self {
        Self::new(
            EncryptionDomainPurpose::Projection,
            tenant_id,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn backup(tenant_id: TenantId, region_id: Option<RegionKeyId>) -> Self {
        Self::new(
            EncryptionDomainPurpose::Backup,
            tenant_id,
            region_id,
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn export_capsule(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::ExportCapsule,
            tenant_id,
            region_id,
            Some(classification),
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn ai_artifact(
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Classification,
        compartment_id: Option<CompartmentKeyId>,
        world_id: Option<WorldId>,
    ) -> Self {
        Self::new(
            EncryptionDomainPurpose::AiArtifact,
            tenant_id,
            region_id,
            Some(classification),
            compartment_id,
            world_id,
            None,
        )
    }

    #[must_use]
    pub const fn wasm_plugin_secret(tenant_id: TenantId, region_id: Option<RegionKeyId>) -> Self {
        Self::new(
            EncryptionDomainPurpose::WasmPluginSecret,
            tenant_id,
            region_id,
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn audit_log(tenant_id: TenantId, region_id: Option<RegionKeyId>) -> Self {
        Self::new(
            EncryptionDomainPurpose::AuditLog,
            tenant_id,
            region_id,
            None,
            None,
            None,
            None,
        )
    }

    #[must_use]
    pub const fn purpose(self) -> EncryptionDomainPurpose {
        self.purpose
    }

    #[must_use]
    pub const fn tenant_id(self) -> TenantId {
        self.tenant_id
    }

    #[must_use]
    pub const fn region_id(self) -> Option<RegionKeyId> {
        self.region_id
    }

    #[must_use]
    pub const fn classification_level(self) -> Option<Classification> {
        self.classification
    }

    #[must_use]
    pub const fn compartment_id(self) -> Option<CompartmentKeyId> {
        self.compartment_id
    }

    #[must_use]
    pub const fn world_id(self) -> Option<WorldId> {
        self.world_id
    }

    #[must_use]
    pub const fn segment_id(self) -> Option<SegmentKeyId> {
        self.segment_id
    }

    #[must_use]
    pub fn is_merge_compatible_with(self, other: Self) -> bool {
        self.structurally_equal(&other)
    }

    #[must_use]
    pub fn merge_with(self, other: Self) -> Option<Self> {
        if self.structurally_equal(&other) {
            Some(self)
        } else {
            None
        }
    }

    /// Structural comparison only. Not constant-time and not for authorization
    /// decisions.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.purpose == other.purpose
            && self.tenant_id.get() == other.tenant_id.get()
            && option_region_eq(self.region_id, other.region_id)
            && self.classification == other.classification
            && option_compartment_eq(self.compartment_id, other.compartment_id)
            && option_world_eq(self.world_id, other.world_id)
            && option_segment_eq(self.segment_id, other.segment_id)
    }

    #[must_use]
    pub fn structurally_equal_ct(&self, other: &Self) -> bool {
        self.fixed_bytes().ct_eq(&other.fixed_bytes()).unwrap_u8() == 1
    }

    fn fixed_bytes(&self) -> [u8; DOMAIN_FIXED_BYTES] {
        let mut bytes = [0_u8; DOMAIN_FIXED_BYTES];
        bytes[0] = purpose_tag(self.purpose);
        bytes[1..17].copy_from_slice(&self.tenant_id.get().to_le_bytes());
        write_optional_u128(&mut bytes, 17, self.region_id.map(RegionKeyId::get));
        bytes[34] = self.classification.map_or(0, classification_tag);
        write_optional_u128(
            &mut bytes,
            35,
            self.compartment_id.map(CompartmentKeyId::get),
        );
        write_optional_u128(&mut bytes, 52, self.world_id.map(WorldId::get));
        write_optional_u128(&mut bytes, 69, self.segment_id.map(SegmentKeyId::get));
        bytes
    }

    const fn new(
        purpose: EncryptionDomainPurpose,
        tenant_id: TenantId,
        region_id: Option<RegionKeyId>,
        classification: Option<Classification>,
        compartment_id: Option<CompartmentKeyId>,
        world_id: Option<WorldId>,
        segment_id: Option<SegmentKeyId>,
    ) -> Self {
        Self {
            purpose,
            tenant_id,
            region_id,
            classification,
            compartment_id,
            world_id,
            segment_id,
        }
    }
}

impl fmt::Debug for EncryptionDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptionDomain")
            .field("purpose", &"<redacted>")
            .field("tenant_id", &"<redacted>")
            .field("region_id", &"<redacted>")
            .field("classification", &"<redacted>")
            .field("compartment_id", &"<redacted>")
            .field("world_id", &"<redacted>")
            .field("segment_id", &"<redacted>")
            .finish()
    }
}

const fn option_region_eq(left: Option<RegionKeyId>, right: Option<RegionKeyId>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.get() == right.get(),
        (None, None) => true,
        _ => false,
    }
}

const fn option_compartment_eq(
    left: Option<CompartmentKeyId>,
    right: Option<CompartmentKeyId>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.get() == right.get(),
        (None, None) => true,
        _ => false,
    }
}

const fn option_world_eq(left: Option<WorldId>, right: Option<WorldId>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.get() == right.get(),
        (None, None) => true,
        _ => false,
    }
}

const fn option_segment_eq(left: Option<SegmentKeyId>, right: Option<SegmentKeyId>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.get() == right.get(),
        (None, None) => true,
        _ => false,
    }
}

fn write_optional_u128(bytes: &mut [u8; DOMAIN_FIXED_BYTES], offset: usize, value: Option<u128>) {
    if let Some(value) = value {
        bytes[offset] = 1;
        bytes[offset + 1..offset + 17].copy_from_slice(&value.to_le_bytes());
    }
}

const fn purpose_tag(purpose: EncryptionDomainPurpose) -> u8 {
    match purpose {
        EncryptionDomainPurpose::Tenant => 1,
        EncryptionDomainPurpose::Region => 2,
        EncryptionDomainPurpose::Classification => 3,
        EncryptionDomainPurpose::Compartment => 4,
        EncryptionDomainPurpose::World => 5,
        EncryptionDomainPurpose::Wal => 6,
        EncryptionDomainPurpose::Segment => 7,
        EncryptionDomainPurpose::Projection => 8,
        EncryptionDomainPurpose::Backup => 9,
        EncryptionDomainPurpose::ExportCapsule => 10,
        EncryptionDomainPurpose::AiArtifact => 11,
        EncryptionDomainPurpose::WasmPluginSecret => 12,
        EncryptionDomainPurpose::AuditLog => 13,
    }
}

const fn classification_tag(classification: Classification) -> u8 {
    match classification {
        Classification::Public => 1,
        Classification::Internal => 2,
        Classification::Restricted => 3,
        Classification::Secret => 4,
        Classification::TopSecret => 5,
    }
}
