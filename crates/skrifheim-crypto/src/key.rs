use core::num::NonZeroU128;

use skrifheim_core::{Result, SkrifheimError, TenantId};

use crate::CryptoEpoch;

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
        tenant_id: TenantId,
    },
    Compartment {
        tenant_id: TenantId,
        compartment_id: CompartmentKeyId,
    },
    Segment {
        tenant_id: TenantId,
        segment_id: SegmentKeyId,
    },
    Data {
        tenant_id: TenantId,
        segment_id: SegmentKeyId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyMetadata {
    key_id: KeyId,
    parent: Option<KeyId>,
    scope: KeyScope,
    epoch: CryptoEpoch,
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
                if is_valid_parent(scope, parent.scope) {
                    Ok(())
                } else {
                    Err(SkrifheimError::InvalidKeyHierarchy)
                }
            }
        }
    }
}

fn is_valid_parent(child: KeyScope, parent: KeyScope) -> bool {
    match (child, parent) {
        (KeyScope::Deployment { .. }, KeyScope::RootTrust) => true,
        (
            KeyScope::Region { deployment_id, .. },
            KeyScope::Deployment {
                deployment_id: parent_deployment,
            },
        ) => deployment_id == parent_deployment,
        (KeyScope::Tenant { .. }, KeyScope::Region { .. }) => true,
        (
            KeyScope::Compartment { tenant_id, .. },
            KeyScope::Tenant {
                tenant_id: parent_tenant,
            },
        ) => tenant_id == parent_tenant,
        (
            KeyScope::Segment { tenant_id, .. } | KeyScope::Data { tenant_id, .. },
            KeyScope::Compartment {
                tenant_id: parent_tenant,
                ..
            },
        ) => tenant_id == parent_tenant,
        _ => false,
    }
}
