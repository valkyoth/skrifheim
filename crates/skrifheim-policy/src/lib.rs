#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use skrifheim_core::{
    AccessDeniedReason, Classification, DeviceId, Result, SecurityLabel, SkrifheimError,
    WorkloadId, canonical_policy_set, contains_policy_token_ct,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectContext {
    clearance: Classification,
    compartments: BTreeSet<String>,
    releasable_to: BTreeSet<String>,
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
    pub const fn compartments(&self) -> &BTreeSet<String> {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &BTreeSet<String> {
        &self.releasable_to
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceContext {
    device_id: DeviceId,
    clearance: Classification,
    compartments: BTreeSet<String>,
    releasable_to: BTreeSet<String>,
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
    pub const fn compartments(&self) -> &BTreeSet<String> {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &BTreeSet<String> {
        &self.releasable_to
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkloadContext {
    workload_id: WorkloadId,
    clearance: Classification,
    compartments: BTreeSet<String>,
    releasable_to: BTreeSet<String>,
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
    pub const fn compartments(&self) -> &BTreeSet<String> {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &BTreeSet<String> {
        &self.releasable_to
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannerDecision {
    Allow,
    Redact { reason: AccessDeniedReason },
    Reject { reason: AccessDeniedReason },
}

#[must_use]
pub fn evaluate_read(authority: &AuthorityContext, label: &SecurityLabel) -> PlannerDecision {
    if !authority
        .subject
        .clearance
        .dominates(label.classification())
        || !authority.device.clearance.dominates(label.classification())
        || !authority
            .workload
            .clearance
            .dominates(label.classification())
    {
        return PlannerDecision::Reject {
            reason: AccessDeniedReason::new(),
        };
    }

    let mut compartment_allowed = 1_u8;
    for compartment in label.compartments() {
        compartment_allowed &=
            contains_policy_token_ct(&authority.subject.compartments, compartment) as u8;
        compartment_allowed &=
            contains_policy_token_ct(&authority.device.compartments, compartment) as u8;
        compartment_allowed &=
            contains_policy_token_ct(&authority.workload.compartments, compartment) as u8;
    }

    let mut releasability_allowed = 1_u8;
    for releasability in label.releasable_to() {
        releasability_allowed &=
            contains_policy_token_ct(&authority.subject.releasable_to, releasability) as u8;
        releasability_allowed &=
            contains_policy_token_ct(&authority.device.releasable_to, releasability) as u8;
        releasability_allowed &=
            contains_policy_token_ct(&authority.workload.releasable_to, releasability) as u8;
    }

    if compartment_allowed == 0 {
        return PlannerDecision::Reject {
            reason: AccessDeniedReason::new(),
        };
    }

    if releasability_allowed == 0 {
        return PlannerDecision::Redact {
            reason: AccessDeniedReason::new(),
        };
    }

    PlannerDecision::Allow
}

pub fn require_allowed(decision: PlannerDecision) -> Result<()> {
    match decision {
        PlannerDecision::Allow => Ok(()),
        PlannerDecision::Redact { reason } | PlannerDecision::Reject { reason } => {
            Err(SkrifheimError::PolicyDenied(reason))
        }
    }
}
