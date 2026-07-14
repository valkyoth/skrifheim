use core::num::NonZeroU128;

use skrifheim_core::{Result, SkrifheimError, TenantId};

use crate::{CryptoEpoch, key_hierarchy::is_valid_parent};

macro_rules! nonzero_key_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(NonZeroU128);

        impl $name {
            #[must_use]
            pub const fn new(value: NonZeroU128) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn from_u128(value: u128) -> Option<Self> {
                match NonZeroU128::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u128 {
                self.0.get()
            }
        }
    };
}

nonzero_key_id!(KeyId);
nonzero_key_id!(DeploymentKeyId);
nonzero_key_id!(RegionKeyId);
nonzero_key_id!(CompartmentKeyId);
nonzero_key_id!(SegmentKeyId);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyScope {
    RootTrust,
    Deployment {
        deployment_id: DeploymentKeyId,
    },
    Region {
        deployment_id: DeploymentKeyId,
        region_id: RegionKeyId,
    },
    Tenant {
        deployment_id: DeploymentKeyId,
        region_id: RegionKeyId,
        tenant_id: TenantId,
    },
    Compartment {
        tenant_id: TenantId,
        compartment_id: CompartmentKeyId,
    },
    Segment {
        tenant_id: TenantId,
        compartment_id: CompartmentKeyId,
        segment_id: SegmentKeyId,
    },
    Data {
        tenant_id: TenantId,
        compartment_id: CompartmentKeyId,
        segment_id: SegmentKeyId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyLifecycleState {
    Created,
    Active,
    Rotating,
    Retired,
    Compromised,
    Quarantined,
    Destroyed,
    CryptoErased,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyErasureReason {
    Rotation,
    Expiration,
    Compromise,
    OperatorApproved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KeyLifecycleEventSequence(u64);

impl KeyLifecycleEventSequence {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyErasureMetadata {
    key_id: KeyId,
    scope: KeyScope,
    epoch: CryptoEpoch,
    lifecycle_event_sequence: KeyLifecycleEventSequence,
    reason: KeyErasureReason,
}

impl KeyErasureMetadata {
    #[must_use]
    pub const fn key_id(self) -> KeyId {
        self.key_id
    }

    #[must_use]
    pub const fn scope(self) -> KeyScope {
        self.scope
    }

    #[must_use]
    pub const fn epoch(self) -> CryptoEpoch {
        self.epoch
    }

    #[must_use]
    pub const fn lifecycle_event_sequence(self) -> KeyLifecycleEventSequence {
        self.lifecycle_event_sequence
    }

    #[must_use]
    pub const fn reason(self) -> KeyErasureReason {
        self.reason
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyRotationPreflight {
    current_key: KeyId,
    candidate_key: KeyId,
    scope: KeyScope,
    from_epoch: CryptoEpoch,
    to_epoch: CryptoEpoch,
}

impl KeyRotationPreflight {
    #[must_use]
    pub const fn current_key(self) -> KeyId {
        self.current_key
    }

    #[must_use]
    pub const fn candidate_key(self) -> KeyId {
        self.candidate_key
    }

    #[must_use]
    pub const fn scope(self) -> KeyScope {
        self.scope
    }

    #[must_use]
    pub const fn from_epoch(self) -> CryptoEpoch {
        self.from_epoch
    }

    #[must_use]
    pub const fn to_epoch(self) -> CryptoEpoch {
        self.to_epoch
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyMetadata {
    key_id: KeyId,
    parent: Option<KeyId>,
    scope: KeyScope,
    epoch: CryptoEpoch,
    lifecycle_event_sequence: KeyLifecycleEventSequence,
    lifecycle: KeyLifecycleState,
    erasure: Option<KeyErasureMetadata>,
}

impl KeyMetadata {
    pub const fn new(
        key_id: KeyId,
        parent: Option<KeyId>,
        scope: KeyScope,
        epoch: CryptoEpoch,
    ) -> Self {
        Self {
            key_id,
            parent,
            scope,
            epoch,
            lifecycle_event_sequence: KeyLifecycleEventSequence::initial(),
            lifecycle: KeyLifecycleState::Created,
            erasure: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn reconstruct(
        key_id: KeyId,
        parent: Option<KeyId>,
        scope: KeyScope,
        epoch: CryptoEpoch,
        lifecycle_event_sequence: KeyLifecycleEventSequence,
        lifecycle: KeyLifecycleState,
        erasure: Option<KeyErasureMetadata>,
    ) -> Result<Self> {
        let metadata = Self {
            key_id,
            parent,
            scope,
            epoch,
            lifecycle_event_sequence,
            lifecycle,
            erasure,
        };
        metadata.validate_invariants()?;
        Ok(metadata)
    }

    #[cfg(test)]
    pub(crate) const fn with_lifecycle_for_test(
        key_id: KeyId,
        parent: Option<KeyId>,
        scope: KeyScope,
        epoch: CryptoEpoch,
        lifecycle: KeyLifecycleState,
    ) -> Self {
        Self {
            key_id,
            parent,
            scope,
            epoch,
            lifecycle_event_sequence: KeyLifecycleEventSequence::new(2),
            lifecycle,
            erasure: None,
        }
    }

    #[must_use]
    pub const fn key_id(&self) -> KeyId {
        self.key_id
    }

    #[must_use]
    pub const fn parent(&self) -> Option<KeyId> {
        self.parent
    }

    #[must_use]
    pub const fn scope(&self) -> KeyScope {
        self.scope
    }

    #[must_use]
    pub const fn epoch(&self) -> CryptoEpoch {
        self.epoch
    }

    #[must_use]
    pub const fn lifecycle_event_sequence(&self) -> KeyLifecycleEventSequence {
        self.lifecycle_event_sequence
    }

    #[must_use]
    pub const fn lifecycle(&self) -> KeyLifecycleState {
        self.lifecycle
    }

    #[must_use]
    pub const fn erasure(&self) -> Option<KeyErasureMetadata> {
        self.erasure
    }

    pub fn validate_invariants(&self) -> Result<()> {
        let sequence = self.lifecycle_event_sequence.get();
        if sequence == 0 {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        if self.lifecycle == KeyLifecycleState::Created && sequence != 1 {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        if self.lifecycle != KeyLifecycleState::Created && sequence == 1 {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        if !self.erasure_metadata_matches_state() {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        Ok(())
    }

    pub fn validate_parent(&self, parent: Option<&Self>) -> Result<()> {
        match (self.scope, parent) {
            (KeyScope::RootTrust, None) => {
                if self.parent.is_none() {
                    Ok(())
                } else {
                    Err(SkrifheimError::InvalidKeyHierarchy)
                }
            }
            (KeyScope::RootTrust, Some(_)) => Err(SkrifheimError::InvalidKeyHierarchy),
            (_, None) => Err(SkrifheimError::InvalidKeyHierarchy),
            (scope, Some(parent)) => {
                if self.parent != Some(parent.key_id) || self.key_id == parent.key_id {
                    return Err(SkrifheimError::InvalidKeyHierarchy);
                }
                if is_unsafe_parent_lifecycle(parent.lifecycle) {
                    return Err(SkrifheimError::InvalidKeyHierarchy);
                }
                if is_valid_parent(scope, parent.scope) {
                    Ok(())
                } else {
                    Err(SkrifheimError::InvalidKeyHierarchy)
                }
            }
        }
    }

    pub fn transition(
        &self,
        next_lifecycle: KeyLifecycleState,
        next_epoch: CryptoEpoch,
    ) -> Result<Self> {
        if !is_valid_lifecycle_transition(self.lifecycle, next_lifecycle) {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        if requires_crypto_epoch_advancement(self.lifecycle, next_lifecycle) {
            if next_epoch.get() <= self.epoch.get() {
                return Err(SkrifheimError::InvalidKeyLifecycle);
            }
        } else if next_epoch != self.epoch {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        let lifecycle_event_sequence = next_lifecycle_event_sequence(self.lifecycle_event_sequence)
            .ok_or(SkrifheimError::InvalidKeyLifecycle)?;
        Ok(Self {
            key_id: self.key_id,
            parent: self.parent,
            scope: self.scope,
            epoch: next_epoch,
            lifecycle_event_sequence,
            lifecycle: next_lifecycle,
            erasure: self.erasure,
        })
    }

    pub fn rotation_preflight(&self, candidate: &Self) -> Result<KeyRotationPreflight> {
        if self.key_id == candidate.key_id
            || self.parent != candidate.parent
            || self.scope != candidate.scope
            || self.epoch.get() >= candidate.epoch.get()
            || self.lifecycle != KeyLifecycleState::Active
            || candidate.lifecycle != KeyLifecycleState::Created
        {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        Ok(KeyRotationPreflight {
            current_key: self.key_id,
            candidate_key: candidate.key_id,
            scope: self.scope,
            from_epoch: self.epoch,
            to_epoch: candidate.epoch,
        })
    }

    pub fn crypto_erase(&self, reason: KeyErasureReason) -> Result<Self> {
        if !is_valid_lifecycle_transition(self.lifecycle, KeyLifecycleState::CryptoErased) {
            return Err(SkrifheimError::InvalidKeyLifecycle);
        }
        let lifecycle_event_sequence = next_lifecycle_event_sequence(self.lifecycle_event_sequence)
            .ok_or(SkrifheimError::InvalidKeyLifecycle)?;
        Ok(Self {
            key_id: self.key_id,
            parent: self.parent,
            scope: self.scope,
            epoch: self.epoch,
            lifecycle_event_sequence,
            lifecycle: KeyLifecycleState::CryptoErased,
            erasure: Some(KeyErasureMetadata {
                key_id: self.key_id,
                scope: self.scope,
                epoch: self.epoch,
                lifecycle_event_sequence,
                reason,
            }),
        })
    }

    fn erasure_metadata_matches_state(&self) -> bool {
        match (self.lifecycle, self.erasure) {
            (KeyLifecycleState::CryptoErased, Some(erasure)) => {
                erasure.key_id == self.key_id
                    && erasure.scope == self.scope
                    && erasure.epoch == self.epoch
                    && erasure.lifecycle_event_sequence == self.lifecycle_event_sequence
            }
            (KeyLifecycleState::CryptoErased, None) | (_, Some(_)) => false,
            (_, None) => true,
        }
    }
}

fn is_unsafe_parent_lifecycle(lifecycle: KeyLifecycleState) -> bool {
    matches!(
        lifecycle,
        KeyLifecycleState::Compromised
            | KeyLifecycleState::Quarantined
            | KeyLifecycleState::Destroyed
            | KeyLifecycleState::CryptoErased
    )
}

fn is_valid_lifecycle_transition(current: KeyLifecycleState, next: KeyLifecycleState) -> bool {
    matches!(
        (current, next),
        (KeyLifecycleState::Created, KeyLifecycleState::Active)
            | (KeyLifecycleState::Created, KeyLifecycleState::Compromised)
            | (KeyLifecycleState::Created, KeyLifecycleState::Quarantined)
            | (KeyLifecycleState::Created, KeyLifecycleState::Destroyed)
            | (KeyLifecycleState::Active, KeyLifecycleState::Rotating)
            | (KeyLifecycleState::Active, KeyLifecycleState::Retired)
            | (KeyLifecycleState::Active, KeyLifecycleState::Compromised)
            | (KeyLifecycleState::Active, KeyLifecycleState::Quarantined)
            | (KeyLifecycleState::Active, KeyLifecycleState::Destroyed)
            | (KeyLifecycleState::Rotating, KeyLifecycleState::Retired)
            | (KeyLifecycleState::Rotating, KeyLifecycleState::Compromised)
            | (KeyLifecycleState::Rotating, KeyLifecycleState::Quarantined)
            | (KeyLifecycleState::Rotating, KeyLifecycleState::Destroyed)
            | (KeyLifecycleState::Retired, KeyLifecycleState::Compromised)
            | (KeyLifecycleState::Retired, KeyLifecycleState::Quarantined)
            | (KeyLifecycleState::Retired, KeyLifecycleState::Destroyed)
            | (
                KeyLifecycleState::Compromised,
                KeyLifecycleState::Quarantined
            )
            | (KeyLifecycleState::Compromised, KeyLifecycleState::Destroyed)
            | (
                KeyLifecycleState::Quarantined,
                KeyLifecycleState::Compromised
            )
            | (KeyLifecycleState::Quarantined, KeyLifecycleState::Destroyed)
            | (
                KeyLifecycleState::Destroyed,
                KeyLifecycleState::CryptoErased
            )
    )
}

fn requires_crypto_epoch_advancement(current: KeyLifecycleState, next: KeyLifecycleState) -> bool {
    matches!(
        (current, next),
        (KeyLifecycleState::Created, KeyLifecycleState::Active)
            | (KeyLifecycleState::Active, KeyLifecycleState::Rotating)
    )
}

fn next_lifecycle_event_sequence(
    current: KeyLifecycleEventSequence,
) -> Option<KeyLifecycleEventSequence> {
    current
        .get()
        .checked_add(1)
        .map(KeyLifecycleEventSequence::new)
}
