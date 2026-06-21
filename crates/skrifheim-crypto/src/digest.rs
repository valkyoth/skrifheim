use core::fmt;

use skrifheim_core::{Result, SkrifheimError};
use subtle::ConstantTimeEq;

pub const SHA3_256_DIGEST_BYTES: usize = 32;
pub const SHA3_384_DIGEST_BYTES: usize = 48;
pub const SHA3_512_DIGEST_BYTES: usize = 64;
pub const SHAKE256_256_DIGEST_BYTES: usize = 32;
pub const SHAKE256_512_DIGEST_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestStrength {
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Shake256_256,
    Shake256_512,
}

impl DigestStrength {
    #[must_use]
    pub const fn output_bytes(self) -> usize {
        match self {
            Self::Sha3_256 => SHA3_256_DIGEST_BYTES,
            Self::Sha3_384 => SHA3_384_DIGEST_BYTES,
            Self::Sha3_512 => SHA3_512_DIGEST_BYTES,
            Self::Shake256_256 => SHAKE256_256_DIGEST_BYTES,
            Self::Shake256_512 => SHAKE256_512_DIGEST_BYTES,
        }
    }

    #[must_use]
    pub const fn quantum_preimage_security_bits(self) -> u16 {
        match self {
            Self::Sha3_256 | Self::Shake256_256 => 128,
            Self::Sha3_384 => 192,
            Self::Sha3_512 | Self::Shake256_512 => 256,
        }
    }

    #[must_use]
    pub const fn is_military_profile(self) -> bool {
        matches!(self, Self::Sha3_512 | Self::Shake256_512)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigestPolicy {
    strength: DigestStrength,
}

impl DigestPolicy {
    pub const HIGH_SECURITY: Self = Self {
        strength: DigestStrength::Sha3_256,
    };
    pub const LONG_HORIZON: Self = Self {
        strength: DigestStrength::Sha3_384,
    };
    pub const MILITARY: Self = Self {
        strength: DigestStrength::Sha3_512,
    };

    #[must_use]
    pub const fn new(strength: DigestStrength) -> Self {
        Self { strength }
    }

    #[must_use]
    pub const fn strength(self) -> DigestStrength {
        self.strength
    }

    #[must_use]
    pub const fn output_bytes(self) -> usize {
        self.strength.output_bytes()
    }
}

#[derive(Clone)]
pub enum DigestValue {
    Bytes32 {
        strength: DigestStrength,
        bytes: [u8; 32],
    },
    Bytes48 {
        strength: DigestStrength,
        bytes: [u8; 48],
    },
    Bytes64 {
        strength: DigestStrength,
        bytes: [u8; 64],
    },
}

impl fmt::Debug for DigestValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DigestValue")
            .field("strength", &"<redacted>")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl DigestValue {
    pub fn new(strength: DigestStrength, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != strength.output_bytes() {
            return Err(SkrifheimError::InvalidDigest);
        }
        match strength {
            DigestStrength::Sha3_256 | DigestStrength::Shake256_256 => {
                let mut value = [0_u8; 32];
                value.copy_from_slice(bytes);
                Ok(Self::Bytes32 {
                    strength,
                    bytes: value,
                })
            }
            DigestStrength::Sha3_384 => {
                let mut value = [0_u8; 48];
                value.copy_from_slice(bytes);
                Ok(Self::Bytes48 {
                    strength,
                    bytes: value,
                })
            }
            DigestStrength::Sha3_512 | DigestStrength::Shake256_512 => {
                let mut value = [0_u8; 64];
                value.copy_from_slice(bytes);
                Ok(Self::Bytes64 {
                    strength,
                    bytes: value,
                })
            }
        }
    }

    #[must_use]
    pub const fn strength(&self) -> DigestStrength {
        match self {
            Self::Bytes32 { strength, .. }
            | Self::Bytes48 { strength, .. }
            | Self::Bytes64 { strength, .. } => *strength,
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Bytes32 { bytes, .. } => bytes,
            Self::Bytes48 { bytes, .. } => bytes,
            Self::Bytes64 { bytes, .. } => bytes,
        }
    }

    /// Non-constant-time structural comparison for tests and data-model checks.
    ///
    /// This must not be used for authentication, authorization, manifest-root
    /// acceptance, or any other trust-boundary decision. Use
    /// [`Self::structurally_equal_ct`] for timing-sensitive equality.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.strength() == other.strength() && self.as_bytes() == other.as_bytes()
    }

    #[must_use]
    pub fn structurally_equal_ct(&self, other: &Self) -> bool {
        let left_strength = [self.strength_tag()];
        let right_strength = [other.strength_tag()];
        let left = self.fixed_bytes();
        let right = other.fixed_bytes();

        (left_strength.ct_eq(&right_strength) & left.ct_eq(&right)).unwrap_u8() == 1
    }

    const fn strength_tag(&self) -> u8 {
        match self.strength() {
            DigestStrength::Sha3_256 => 1,
            DigestStrength::Sha3_384 => 2,
            DigestStrength::Sha3_512 => 3,
            DigestStrength::Shake256_256 => 4,
            DigestStrength::Shake256_512 => 5,
        }
    }

    fn fixed_bytes(&self) -> [u8; SHA3_512_DIGEST_BYTES] {
        let mut fixed = [0_u8; SHA3_512_DIGEST_BYTES];
        fixed[..self.as_bytes().len()].copy_from_slice(self.as_bytes());
        fixed
    }
}

macro_rules! digest_wrapper {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name(DigestValue);

        impl $name {
            pub fn new(policy: DigestPolicy, bytes: &[u8]) -> Result<Self> {
                Ok(Self(DigestValue::new(policy.strength(), bytes)?))
            }

            pub fn require_policy(&self, policy: DigestPolicy) -> Result<()> {
                if self.0.strength() != policy.strength() {
                    return Err(SkrifheimError::InvalidDigest);
                }
                Ok(())
            }

            #[must_use]
            pub const fn strength(&self) -> DigestStrength {
                self.0.strength()
            }

            #[must_use]
            pub fn digest_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }

            /// Non-constant-time structural comparison for tests and metadata
            /// checks only.
            ///
            /// This must not be used for authentication, authorization,
            /// manifest-root acceptance, or other trust-boundary decisions. Use
            /// [`Self::structurally_equal_ct`] for timing-sensitive equality.
            #[must_use]
            pub fn structurally_equal(&self, other: &Self) -> bool {
                self.0.structurally_equal(&other.0)
            }

            #[must_use]
            pub fn structurally_equal_ct(&self, other: &Self) -> bool {
                self.0.structurally_equal_ct(&other.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("strength", &"<redacted>")
                    .field("digest", &"<redacted>")
                    .finish()
            }
        }
    };
}

digest_wrapper!(WorldIdentityDigest);
digest_wrapper!(ContentDigest);
digest_wrapper!(ManifestDigest);
