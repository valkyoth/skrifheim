#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{FactId, Result, SkrifheimError, WorldId};

#[cfg(test)]
mod tests;

pub const WORLD_NAME_MAX_BYTES: usize = 256;

const WORLD_ID_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const WORLD_ID_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldMetadata {
    id: WorldId,
    name: String,
    kind: WorldKind,
    parent: Option<WorldId>,
    depth: u32,
}

impl WorldMetadata {
    pub fn root(name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        let name = validate_world_name(name.into())?;
        let id = derive_world_id(None, 0, &name, kind)?;
        Ok(Self {
            id,
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
        let id = derive_world_id(Some(parent.id), depth, &name, kind)?;
        if id == parent.id {
            return Err(SkrifheimError::InvalidWorldIdentity);
        }
        Ok(Self {
            id,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    metadata: WorldMetadata,
    added_facts: Vec<FactId>,
    hidden_facts: Vec<FactId>,
}

impl World {
    pub fn root(name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        Ok(Self {
            metadata: WorldMetadata::root(name, kind)?,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        })
    }

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

    pub fn add_fact(&mut self, fact_id: FactId) {
        if !self.added_facts.contains(&fact_id) {
            self.added_facts.push(fact_id);
        }
    }

    pub fn hide_fact(&mut self, fact_id: FactId) {
        if !self.hidden_facts.contains(&fact_id) {
            self.hidden_facts.push(fact_id);
        }
    }
}

fn validate_world_name(name: String) -> Result<String> {
    if name.is_empty() || name.len() > WORLD_NAME_MAX_BYTES {
        return Err(SkrifheimError::InvalidWorldName);
    }
    if !name.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || byte == b' '
            || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
    }) {
        return Err(SkrifheimError::InvalidWorldName);
    }
    Ok(name)
}

fn derive_world_id(
    parent: Option<WorldId>,
    depth: u32,
    name: &str,
    kind: WorldKind,
) -> Result<WorldId> {
    let mut state = WORLD_ID_OFFSET;
    state = hash_bytes(state, b"skrifheim/world/v1");
    state = hash_bytes(state, &[kind.identity_tag()]);
    state = hash_bytes(state, &depth.to_le_bytes());
    match parent {
        Some(parent) => {
            state = hash_bytes(state, b"child");
            state = hash_bytes(state, &parent.get().to_le_bytes());
        }
        None => {
            state = hash_bytes(state, b"root");
        }
    }
    state = hash_bytes(state, name.as_bytes());
    WorldId::from_u128(state | 1).ok_or(SkrifheimError::InvalidWorldIdentity)
}

fn hash_bytes(mut state: u128, bytes: &[u8]) -> u128 {
    for byte in bytes {
        state ^= u128::from(*byte);
        state = state.wrapping_mul(WORLD_ID_PRIME);
    }
    state
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldDiff {
    pub from: WorldId,
    pub to: WorldId,
    pub added: Vec<FactId>,
    pub hidden: Vec<FactId>,
}

impl WorldDiff {
    pub fn between(from: &World, to: &World) -> Result<Self> {
        if from.id() != to.id() && to.parent() != Some(from.id()) {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(Self {
            from: from.id(),
            to: to.id(),
            added: to
                .added_facts
                .iter()
                .filter(|fact_id| !from.added_facts.contains(fact_id))
                .copied()
                .collect(),
            hidden: to
                .hidden_facts
                .iter()
                .filter(|fact_id| !from.hidden_facts.contains(fact_id))
                .copied()
                .collect(),
        })
    }
}
