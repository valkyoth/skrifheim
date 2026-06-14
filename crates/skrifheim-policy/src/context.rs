use alloc::{string::String, vec::Vec};
use skrifheim_core::{
    Classification, DeviceId, PolicyTokenSet, Result, WorkloadId, canonical_policy_set,
};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
