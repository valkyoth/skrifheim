use alloc::{string::String, vec::Vec};
use core::{array, fmt};

use crate::{Result, SkrifheimError};
use subtle::ConstantTimeEq;

pub const POLICY_TOKEN_MAX_BYTES: usize = 128;
pub const POLICY_TOKEN_SET_MAX_ITEMS: usize = 64;
const NOOP_POLICY_TOKEN: PolicyTokenSlot = PolicyTokenSlot::empty();
const INVALID_POLICY_TOKEN_NEEDLE: PolicyTokenSlot = PolicyTokenSlot::invalid_needle();

#[derive(Clone)]
pub struct PolicyTokenSet {
    slots: [PolicyTokenSlot; POLICY_TOKEN_SET_MAX_ITEMS],
    len: usize,
}

impl PolicyTokenSet {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            slots: [NOOP_POLICY_TOKEN; POLICY_TOKEN_SET_MAX_ITEMS],
            len: 0,
        }
    }

    pub fn new(values: Vec<String>) -> Result<Self> {
        if values.len() > POLICY_TOKEN_SET_MAX_ITEMS {
            return Err(SkrifheimError::InvalidSecurityToken);
        }

        let mut values = values
            .into_iter()
            .map(canonical_policy_token)
            .collect::<Result<Vec<_>>>()?;
        values.sort();
        values.dedup();

        let mut slots = [NOOP_POLICY_TOKEN; POLICY_TOKEN_SET_MAX_ITEMS];
        let len = values.len();
        for (index, value) in values.iter().enumerate() {
            slots[index] = PolicyTokenSlot::from_canonical(value);
        }

        Ok(Self { slots, len })
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub fn contains(&self, needle: &str) -> bool {
        contains_policy_token_ct(self, needle)
    }

    pub fn union(&self, other: &Self) -> Result<Self> {
        let mut merged = Self::empty();
        merged.extend_from(self)?;
        merged.extend_from(other)?;
        merged.sort_present_slots();
        Ok(merged)
    }

    /// Compares canonical token-set structure with ordinary equality.
    ///
    /// This is not constant-time. Access-control and policy decisions must use
    /// `contains_policy_token_ct` or `contains_policy_token_slot_ct` instead.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.len == other.len
            && self
                .slots
                .iter()
                .zip(other.slots.iter())
                .all(|(left, right)| left.structurally_equal(*right))
    }

    #[must_use]
    pub const fn slot(&self, index: usize) -> Option<PolicyTokenSlot> {
        if index < POLICY_TOKEN_SET_MAX_ITEMS {
            Some(self.slots[index])
        } else {
            None
        }
    }

    #[must_use]
    pub fn slots(&self) -> &[PolicyTokenSlot; POLICY_TOKEN_SET_MAX_ITEMS] {
        &self.slots
    }

    fn extend_from(&mut self, other: &Self) -> Result<()> {
        for slot in other.slots {
            if slot.present_mask() == 1 && !contains_policy_token_slot_ct(self, slot) {
                if self.len == POLICY_TOKEN_SET_MAX_ITEMS {
                    return Err(SkrifheimError::InvalidSecurityToken);
                }
                self.slots[self.len] = slot;
                self.len += 1;
            }
        }
        Ok(())
    }

    fn sort_present_slots(&mut self) {
        let mut index = 1;
        while index < self.len {
            let slot = self.slots[index];
            let mut insert_at = index;
            while insert_at > 0 && slot_sorts_after(self.slots[insert_at - 1], slot) {
                self.slots[insert_at] = self.slots[insert_at - 1];
                insert_at -= 1;
            }
            self.slots[insert_at] = slot;
            index += 1;
        }
    }
}

impl fmt::Debug for PolicyTokenSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyTokenSet")
            .field("len", &"<redacted>")
            .field("capacity", &POLICY_TOKEN_SET_MAX_ITEMS)
            .field("tokens", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct PolicyTokenSlot {
    bytes: [u8; POLICY_TOKEN_MAX_BYTES],
    len: usize,
    present: u8,
}

impl PolicyTokenSlot {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            bytes: [0; POLICY_TOKEN_MAX_BYTES],
            len: 0,
            present: 0,
        }
    }

    #[must_use]
    pub const fn invalid_needle() -> Self {
        Self {
            bytes: [0xff; POLICY_TOKEN_MAX_BYTES],
            len: POLICY_TOKEN_MAX_BYTES,
            present: 1,
        }
    }

    fn from_canonical(value: &str) -> Self {
        // Private constructor: callers must pass `canonical_policy_token`
        // output, which enforces the fixed slot size before copying.
        let mut bytes = [0; POLICY_TOKEN_MAX_BYTES];
        let len = value.len().min(POLICY_TOKEN_MAX_BYTES);
        bytes[..len].copy_from_slice(&value.as_bytes()[..len]);
        Self {
            bytes,
            len,
            present: 1,
        }
    }

    #[must_use]
    pub const fn present_mask(self) -> u8 {
        self.present
    }

    /// Compares token-slot structure with ordinary equality.
    ///
    /// This is not constant-time and must not be used for access-control
    /// decisions.
    #[must_use]
    pub fn structurally_equal(self, other: Self) -> bool {
        self.len == other.len && self.present == other.present && self.bytes == other.bytes
    }
}

impl fmt::Debug for PolicyTokenSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.present == 0 {
            return f.write_str("PolicyTokenSlot(<empty>)");
        }
        f.debug_struct("PolicyTokenSlot")
            .field("len", &"<redacted>")
            .field("token", &"<redacted>")
            .finish()
    }
}

pub fn canonical_policy_set(values: Vec<String>) -> Result<PolicyTokenSet> {
    PolicyTokenSet::new(values)
}

pub fn canonical_policy_token(mut value: String) -> Result<String> {
    if !is_valid_policy_token(&value) {
        return Err(SkrifheimError::InvalidSecurityToken);
    }
    value.make_ascii_uppercase();
    Ok(value)
}

#[must_use]
pub fn contains_policy_token_ct(tokens: &PolicyTokenSet, needle: &str) -> bool {
    // Needle canonicalization is not constant-time; only the fixed-slot
    // comparison below is constant-shape.
    let needle = match canonical_policy_token(String::from(needle)) {
        Ok(needle) => PolicyTokenSlot::from_canonical(&needle),
        Err(_) => INVALID_POLICY_TOKEN_NEEDLE,
    };
    contains_policy_token_slot_ct(tokens, needle)
}

#[must_use]
pub fn contains_policy_token_slot_ct(tokens: &PolicyTokenSet, needle: PolicyTokenSlot) -> bool {
    let mut found = 0_u8;
    for token in tokens.slots() {
        found |= token.present_mask() & policy_token_eq_ct(*token, needle);
    }
    found == 1
}

fn is_valid_policy_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= POLICY_TOKEN_MAX_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

fn policy_token_eq_ct(left: PolicyTokenSlot, right: PolicyTokenSlot) -> u8 {
    let left_len = (left.len as u16).to_le_bytes();
    let right_len = (right.len as u16).to_le_bytes();
    (left_len.ct_eq(&right_len) & left.bytes.ct_eq(&right.bytes)).unwrap_u8()
}

fn slot_sorts_after(left: PolicyTokenSlot, right: PolicyTokenSlot) -> bool {
    // Structural canonicalization only. This comparator is intentionally not
    // constant-time and must not be used for policy authorization decisions.
    let mut index = 0;
    while index < POLICY_TOKEN_MAX_BYTES {
        let left_byte = left.bytes[index];
        let right_byte = right.bytes[index];
        if left_byte != right_byte {
            return left_byte > right_byte;
        }
        index += 1;
    }
    left.len > right.len
}

impl Default for PolicyTokenSet {
    fn default() -> Self {
        Self {
            slots: array::from_fn(|_| NOOP_POLICY_TOKEN),
            len: 0,
        }
    }
}
