#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;
use skrifheim_core::{FactId, Result, SkrifheimError, TenantId, WorldId};

mod diff;

pub use diff::{
    PromotionPreflight, RollbackPreflight, WorldConflict, WorldConflictKind, WorldDiff,
};

#[cfg(test)]
mod tests;

pub const WORLD_NAME_MAX_BYTES: usize = 256;
pub const WORLD_FACT_LIST_MAX_ITEMS: usize = 1_000_000;

const WORLD_ID_DERIVE_CONTEXT: &str = "skrifheim/world/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldKind {
    Production,
    Staging,
    UserLocal,
    AgentScratchpad,
    Simulation,
    LegalAudit,
    MissionCapsule,
}

impl WorldKind {
    const fn identity_tag(self) -> u8 {
        match self {
            Self::Production => 1,
            Self::Staging => 2,
            Self::UserLocal => 3,
            Self::AgentScratchpad => 4,
            Self::Simulation => 5,
            Self::LegalAudit => 6,
            Self::MissionCapsule => 7,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct WorldMetadata {
    id: WorldId,
    tenant_id: TenantId,
    name: String,
    kind: WorldKind,
    parent: Option<WorldId>,
    depth: u32,
}

impl fmt::Debug for WorldMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldMetadata")
            .field("id", &"<redacted>")
            .field("tenant_id", &"<redacted>")
            .field("name", &"<redacted>")
            .field("kind", &"<redacted>")
            .field("parent", &"<redacted>")
            .field("depth", &self.depth)
            .finish()
    }
}

impl WorldMetadata {
    pub fn root(tenant_id: TenantId, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        let name = validate_world_name(name.into())?;
        let id = derive_world_id(tenant_id, None, 0, &name, kind)?;
        Ok(Self {
            id,
            tenant_id,
            name,
            kind,
            parent: None,
            depth: 0,
        })
    }

    pub fn child(parent: &Self, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        let name = validate_world_name(name.into())?;
        let depth = parent
            .depth
            .checked_add(1)
            .ok_or(SkrifheimError::InvalidWorldIdentity)?;
        let id = derive_world_id(parent.tenant_id, Some(parent.id), depth, &name, kind)?;
        if id == parent.id {
            return Err(SkrifheimError::InvalidWorldIdentity);
        }
        Ok(Self {
            id,
            tenant_id: parent.tenant_id,
            name,
            kind,
            parent: Some(parent.id),
            depth,
        })
    }

    #[must_use]
    pub const fn id(&self) -> WorldId {
        self.id
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> WorldKind {
        self.kind
    }

    #[must_use]
    pub const fn parent(&self) -> Option<WorldId> {
        self.parent
    }

    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct World {
    metadata: WorldMetadata,
    added_facts: Vec<FactId>,
    hidden_facts: Vec<FactId>,
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("metadata", &"<redacted>")
            .field("added_facts", &"<redacted>")
            .field("hidden_facts", &"<redacted>")
            .finish()
    }
}

impl World {
    /// Creates a deterministic root world identity for the
    /// `(tenant_id, kind, depth, parent, name)` tuple.
    ///
    /// Repeating this call with the same arguments returns the same `WorldId`
    /// by design; storage must treat that tuple as the uniqueness key.
    pub fn root(tenant_id: TenantId, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        Ok(Self {
            metadata: WorldMetadata::root(tenant_id, name, kind)?,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        })
    }

    /// Creates a deterministic child world identity under this parent.
    ///
    /// Repeating this call with the same parent, name, and kind returns the same
    /// `WorldId` by design; it does not mint a fresh scratch branch.
    pub fn fork(&self, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        Ok(Self {
            metadata: WorldMetadata::child(&self.metadata, name, kind)?,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        })
    }

    #[must_use]
    pub const fn metadata(&self) -> &WorldMetadata {
        &self.metadata
    }

    #[must_use]
    pub const fn id(&self) -> WorldId {
        self.metadata.id()
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.metadata.tenant_id()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.metadata.name()
    }

    #[must_use]
    pub const fn kind(&self) -> WorldKind {
        self.metadata.kind()
    }

    #[must_use]
    pub const fn parent(&self) -> Option<WorldId> {
        self.metadata.parent()
    }

    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.metadata.depth()
    }

    #[must_use]
    pub fn added_facts(&self) -> &[FactId] {
        &self.added_facts
    }

    #[must_use]
    pub fn hidden_facts(&self) -> &[FactId] {
        &self.hidden_facts
    }

    pub fn add_fact(&mut self, fact_id: FactId) -> Result<()> {
        insert_fact_id(&mut self.added_facts, fact_id)
    }

    pub fn add_facts(&mut self, fact_ids: Vec<FactId>) -> Result<()> {
        merge_fact_ids(&mut self.added_facts, fact_ids, WORLD_FACT_LIST_MAX_ITEMS)
    }

    pub fn hide_fact(&mut self, fact_id: FactId) -> Result<()> {
        insert_fact_id(&mut self.hidden_facts, fact_id)
    }

    pub fn hide_facts(&mut self, fact_ids: Vec<FactId>) -> Result<()> {
        merge_fact_ids(&mut self.hidden_facts, fact_ids, WORLD_FACT_LIST_MAX_ITEMS)
    }

    pub fn diff_to_child(&self, child: &Self) -> Result<WorldDiff> {
        WorldDiff::between(self, child)
    }

    pub fn promotion_preflight(&self, candidate: &Self) -> Result<PromotionPreflight> {
        PromotionPreflight::between(self, candidate)
    }

    pub fn rollback_preflight(&self, child: &Self) -> Result<RollbackPreflight> {
        RollbackPreflight::from_child(self, child)
    }
}

fn insert_fact_id(values: &mut Vec<FactId>, fact_id: FactId) -> Result<()> {
    insert_fact_id_with_limit(values, fact_id, WORLD_FACT_LIST_MAX_ITEMS)
}

fn insert_fact_id_with_limit(
    values: &mut Vec<FactId>,
    fact_id: FactId,
    max_items: usize,
) -> Result<()> {
    match values.binary_search(&fact_id) {
        Ok(_) => Ok(()),
        Err(index) => {
            if values.len() >= max_items {
                return Err(SkrifheimError::TooManyFactLinks);
            }
            values.insert(index, fact_id);
            Ok(())
        }
    }
}

fn merge_fact_ids(
    values: &mut Vec<FactId>,
    mut incoming: Vec<FactId>,
    max_items: usize,
) -> Result<()> {
    if incoming.is_empty() {
        return Ok(());
    }
    if incoming.len() > max_items {
        return Err(SkrifheimError::TooManyFactLinks);
    }
    incoming.sort_unstable();
    incoming.dedup();

    let capacity = core::cmp::min(max_items, values.len().saturating_add(incoming.len()));
    let mut merged = Vec::with_capacity(capacity);
    let mut left = 0;
    let mut right = 0;
    while left < values.len() && right < incoming.len() {
        match values[left].cmp(&incoming[right]) {
            core::cmp::Ordering::Less => {
                push_merged_fact(&mut merged, values[left], max_items)?;
                left += 1;
            }
            core::cmp::Ordering::Equal => {
                push_merged_fact(&mut merged, values[left], max_items)?;
                left += 1;
                right += 1;
            }
            core::cmp::Ordering::Greater => {
                push_merged_fact(&mut merged, incoming[right], max_items)?;
                right += 1;
            }
        }
    }
    while left < values.len() {
        push_merged_fact(&mut merged, values[left], max_items)?;
        left += 1;
    }
    while right < incoming.len() {
        push_merged_fact(&mut merged, incoming[right], max_items)?;
        right += 1;
    }
    *values = merged;
    Ok(())
}

fn push_merged_fact(values: &mut Vec<FactId>, fact_id: FactId, max_items: usize) -> Result<()> {
    if values.len() >= max_items {
        return Err(SkrifheimError::TooManyFactLinks);
    }
    values.push(fact_id);
    Ok(())
}

fn validate_world_name(name: String) -> Result<String> {
    if name.is_empty() || name.len() > WORLD_NAME_MAX_BYTES {
        return Err(SkrifheimError::InvalidWorldName);
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(SkrifheimError::InvalidWorldName);
    }
    Ok(name)
}

fn derive_world_id(
    tenant_id: TenantId,
    parent: Option<WorldId>,
    depth: u32,
    name: &str,
    kind: WorldKind,
) -> Result<WorldId> {
    let mut hasher = blake3::Hasher::new_derive_key(WORLD_ID_DERIVE_CONTEXT);
    hash_field(&mut hasher, b"tenant", &tenant_id.get().to_le_bytes());
    hash_field(&mut hasher, b"kind", &[kind.identity_tag()]);
    hash_field(&mut hasher, b"depth", &depth.to_le_bytes());
    match parent {
        Some(parent) => {
            hash_field(&mut hasher, b"parent-kind", b"child");
            hash_field(&mut hasher, b"parent-id", &parent.get().to_le_bytes());
        }
        None => {
            hash_field(&mut hasher, b"parent-kind", b"root");
        }
    }
    hash_field(&mut hasher, b"name", name.as_bytes());
    let mut bytes = [0_u8; 16];
    // WorldId is a u128, so the deterministic identity uses 16 bytes of the
    // BLAKE3 output. The low bit is forced on only to satisfy the non-zero ID
    // type; it is not a secrecy boundary or bearer capability.
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    let id = u128::from_le_bytes(bytes) | 1;
    // INVARIANT: `id | 1` is always odd and therefore non-zero.
    WorldId::from_u128(id).ok_or(SkrifheimError::InvalidWorldIdentity)
}

fn hash_field(hasher: &mut blake3::Hasher, tag: &[u8], bytes: &[u8]) {
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
