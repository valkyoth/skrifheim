use alloc::{string::String, vec::Vec};
use core::fmt;
use skrifheim_core::{
    Classification, DeviceId, PolicyTokenSet, Result, WorkloadId, canonical_policy_set,
};

#[derive(Clone)]
pub struct SubjectContext {
    clearance: Classification,
    compartments: PolicyTokenSet,
    releasable_to: PolicyTokenSet,
}

impl SubjectContext {
    pub fn new(
        clearance: Classification,
        compartments: Vec<String>,
        releasable_to: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            clearance,
            compartments: canonical_policy_set(compartments)?,
            releasable_to: canonical_policy_set(releasable_to)?,
        })
    }

    #[must_use]
    pub const fn clearance(&self) -> Classification {
        self.clearance
    }

    #[must_use]
    pub const fn compartments(&self) -> &PolicyTokenSet {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &PolicyTokenSet {
        &self.releasable_to
    }
}

impl fmt::Debug for SubjectContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubjectContext")
            .field("clearance", &"<redacted>")
            .field("compartments", &"<redacted>")
            .field("releasable_to", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct DeviceContext {
    device_id: DeviceId,
    clearance: Classification,
    compartments: PolicyTokenSet,
    releasable_to: PolicyTokenSet,
}

impl DeviceContext {
    pub fn new(
        device_id: DeviceId,
        clearance: Classification,
        compartments: Vec<String>,
        releasable_to: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            device_id,
            clearance,
            compartments: canonical_policy_set(compartments)?,
            releasable_to: canonical_policy_set(releasable_to)?,
        })
    }

    #[must_use]
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    #[must_use]
    pub const fn clearance(&self) -> Classification {
        self.clearance
    }

    #[must_use]
    pub const fn compartments(&self) -> &PolicyTokenSet {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &PolicyTokenSet {
        &self.releasable_to
    }
}

impl fmt::Debug for DeviceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceContext")
            .field("device_id", &"<redacted>")
            .field("clearance", &"<redacted>")
            .field("compartments", &"<redacted>")
            .field("releasable_to", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct WorkloadContext {
    workload_id: WorkloadId,
    clearance: Classification,
    compartments: PolicyTokenSet,
    releasable_to: PolicyTokenSet,
}

impl WorkloadContext {
    pub fn new(
        workload_id: WorkloadId,
        clearance: Classification,
        compartments: Vec<String>,
        releasable_to: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            workload_id,
            clearance,
            compartments: canonical_policy_set(compartments)?,
            releasable_to: canonical_policy_set(releasable_to)?,
        })
    }

    #[must_use]
    pub const fn workload_id(&self) -> WorkloadId {
        self.workload_id
    }

    #[must_use]
    pub const fn clearance(&self) -> Classification {
        self.clearance
    }

    #[must_use]
    pub const fn compartments(&self) -> &PolicyTokenSet {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &PolicyTokenSet {
        &self.releasable_to
    }
}

impl fmt::Debug for WorkloadContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkloadContext")
            .field("workload_id", &"<redacted>")
            .field("clearance", &"<redacted>")
            .field("compartments", &"<redacted>")
            .field("releasable_to", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct AuthorityContext {
    subject: SubjectContext,
    device: DeviceContext,
    workload: WorkloadContext,
}

impl AuthorityContext {
    #[must_use]
    pub const fn new(
        subject: SubjectContext,
        device: DeviceContext,
        workload: WorkloadContext,
    ) -> Self {
        Self {
            subject,
            device,
            workload,
        }
    }

    #[must_use]
    pub const fn subject(&self) -> &SubjectContext {
        &self.subject
    }

    #[must_use]
    pub const fn device(&self) -> &DeviceContext {
        &self.device
    }

    #[must_use]
    pub const fn workload(&self) -> &WorkloadContext {
        &self.workload
    }
}

impl fmt::Debug for AuthorityContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthorityContext")
            .field("subject", &"<redacted>")
            .field("device", &"<redacted>")
            .field("workload", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, string::String, vec};

    #[test]
    fn debug_redacts_authority_context_metadata() -> Result<()> {
        let subject = SubjectContext::new(
            Classification::TopSecret,
            vec![String::from("SECRET-COMPARTMENT")],
            vec![String::from("EU")],
        )?;
        let device = DeviceContext::new(
            DeviceId::from_u128(42).ok_or(skrifheim_core::SkrifheimError::InvalidIdentifier)?,
            Classification::TopSecret,
            vec![String::from("SECRET-COMPARTMENT")],
            vec![String::from("EU")],
        )?;
        let workload = WorkloadContext::new(
            WorkloadId::from_u128(43).ok_or(skrifheim_core::SkrifheimError::InvalidIdentifier)?,
            Classification::TopSecret,
            vec![String::from("SECRET-COMPARTMENT")],
            vec![String::from("EU")],
        )?;
        let authority = AuthorityContext::new(subject, device, workload);

        for debug in [
            format!("{:?}", authority.subject()),
            format!("{:?}", authority.device()),
            format!("{:?}", authority.workload()),
            format!("{authority:?}"),
        ] {
            assert!(!debug.contains("TopSecret"));
            assert!(!debug.contains("SECRET-COMPARTMENT"));
            assert!(!debug.contains("EU"));
            assert!(!debug.contains("42"));
            assert!(!debug.contains("43"));
        }
        Ok(())
    }
}
