//! Distinct registries prevent cross-protocol algorithm confusion.
//! Uninhabited registries deliberately admit no implementation yet.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KdfSuite {}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyWrappingSuite {}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuorumProofSuite {}

/// Structural signature identities only. No verifier is admitted by this enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SigningSuite {
    Ed25519,
    MlDsa65,
    SlhDsaSha2S128s,
    Ed25519MlDsa65,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuiteLifecycle {
    ActiveWrite,
    ReadOnly,
    Quarantined,
    EmergencyRejected,
    Erased,
}
impl SuiteLifecycle {
    pub const fn permits_write(self) -> bool {
        matches!(self, Self::ActiveWrite)
    }
    pub const fn permits_read(self) -> bool {
        matches!(self, Self::ActiveWrite | Self::ReadOnly)
    }
}
