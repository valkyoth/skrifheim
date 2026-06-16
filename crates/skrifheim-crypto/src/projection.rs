use core::fmt;

use skrifheim_core::{Result, SkrifheimError};

use crate::{EncryptionDomain, EncryptionDomainPurpose};

#[derive(Clone, Copy)]
pub enum ProjectionSurface {
    SecondaryIndex,
    GraphIndex,
    SearchIndex,
    VectorIndex,
    ColumnarProjection,
    CacheFile,
    CompactionTemporaryFile,
}

#[derive(Clone, Copy)]
pub struct ProjectionEncryptionPolicy {
    surface: ProjectionSurface,
    domain: EncryptionDomain,
}

impl ProjectionEncryptionPolicy {
    pub fn new(surface: ProjectionSurface, domain: EncryptionDomain) -> Result<Self> {
        if domain.purpose() != EncryptionDomainPurpose::Projection
            || domain.classification_level().is_none()
        {
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
        projection_surface_eq(self.surface, other.surface)
            && self.domain.is_merge_compatible_with(other.domain)
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
        projection_surface_eq(self.surface, other.surface)
            && self.domain.structurally_equal(&other.domain)
    }
}

const fn projection_surface_eq(left: ProjectionSurface, right: ProjectionSurface) -> bool {
    matches!(
        (left, right),
        (
            ProjectionSurface::SecondaryIndex,
            ProjectionSurface::SecondaryIndex
        ) | (ProjectionSurface::GraphIndex, ProjectionSurface::GraphIndex)
            | (
                ProjectionSurface::SearchIndex,
                ProjectionSurface::SearchIndex
            )
            | (
                ProjectionSurface::VectorIndex,
                ProjectionSurface::VectorIndex
            )
            | (
                ProjectionSurface::ColumnarProjection,
                ProjectionSurface::ColumnarProjection
            )
            | (ProjectionSurface::CacheFile, ProjectionSurface::CacheFile)
            | (
                ProjectionSurface::CompactionTemporaryFile,
                ProjectionSurface::CompactionTemporaryFile
            )
    )
}

impl fmt::Debug for ProjectionSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectionSurface(<redacted>)")
    }
}

impl fmt::Debug for ProjectionEncryptionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectionEncryptionPolicy")
            .field("surface", &"<redacted>")
            .field("domain", &"<redacted>")
            .finish()
    }
}
