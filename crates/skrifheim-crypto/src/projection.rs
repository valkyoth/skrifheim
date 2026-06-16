use skrifheim_core::{Result, SkrifheimError};

use crate::{EncryptionDomain, EncryptionDomainPurpose};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionSurface {
    SecondaryIndex,
    GraphIndex,
    SearchIndex,
    VectorIndex,
    ColumnarProjection,
    CacheFile,
    CompactionTemporaryFile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectionEncryptionPolicy {
    surface: ProjectionSurface,
    domain: EncryptionDomain,
}

impl ProjectionEncryptionPolicy {
    pub fn new(surface: ProjectionSurface, domain: EncryptionDomain) -> Result<Self> {
        if domain.purpose() != EncryptionDomainPurpose::Projection {
            return Err(SkrifheimError::InvalidProjectionPolicy);
        }
        Ok(Self { surface, domain })
    }

    pub fn secondary_index(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::SecondaryIndex, domain)
    }

    pub fn graph_index(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::GraphIndex, domain)
    }

    pub fn search_index(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::SearchIndex, domain)
    }

    pub fn vector_index(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::VectorIndex, domain)
    }

    pub fn columnar_projection(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::ColumnarProjection, domain)
    }

    pub fn cache_file(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::CacheFile, domain)
    }

    pub fn compaction_temporary_file(domain: EncryptionDomain) -> Result<Self> {
        Self::new(ProjectionSurface::CompactionTemporaryFile, domain)
    }

    #[must_use]
    pub const fn surface(self) -> ProjectionSurface {
        self.surface
    }

    #[must_use]
    pub const fn domain(self) -> EncryptionDomain {
        self.domain
    }

    #[must_use]
    pub const fn requires_encryption_at_rest(self) -> bool {
        true
    }

    #[must_use]
    pub const fn allows_plaintext_temporary_files(self) -> bool {
        false
    }

    #[must_use]
    pub fn is_domain_compatible_with(self, other: Self) -> bool {
        self.domain.is_merge_compatible_with(other.domain)
    }

    #[must_use]
    pub fn merge_with(self, other: Self) -> Option<Self> {
        if self.surface == other.surface && self.is_domain_compatible_with(other) {
            Some(self)
        } else {
            None
        }
    }
}
