use alloc::{string::String, vec::Vec};
use core::hint::black_box;
use core::{array, fmt};

use crate::{Result, SkrifheimError};

pub const POLICY_TOKEN_MAX_BYTES: usize = 128;
pub const POLICY_TOKEN_SET_MAX_ITEMS: usize = 64;
const NOOP_POLICY_TOKEN: PolicyTokenSlot = PolicyTokenSlot::empty();
const INVALID_POLICY_TOKEN_NEEDLE: PolicyTokenSlot = PolicyTokenSlot::invalid_needle();

#[derive(Clone, Eq, PartialEq)]
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
}

impl fmt::Debug for PolicyTokenSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyTokenSet")
            .field("len", &self.len)
            .field("capacity", &POLICY_TOKEN_SET_MAX_ITEMS)
            .field("tokens", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
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
        let mut bytes = [0; POLICY_TOKEN_MAX_BYTES];
        for (index, byte) in value.bytes().enumerate() {
            bytes[index] = byte;
        }
        Self {
            bytes,
            len: value.len(),
            present: 1,
        }
    }

    #[must_use]
    pub const fn present_mask(self) -> u8 {
        self.present
    }
}

impl fmt::Debug for PolicyTokenSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.present == 0 {
            return f.write_str("PolicyTokenSlot(<empty>)");
        }
        f.debug_struct("PolicyTokenSlot")
            .field("len", &self.len)
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
    let left = black_box(left);
    let right = black_box(right);
    let mut diff = black_box(left.len ^ right.len);
    let mut index = 0;
    while index < POLICY_TOKEN_MAX_BYTES {
        diff |= usize::from(black_box(left.bytes[index] ^ right.bytes[index]));
        index += 1;
    }
    black_box(usize_is_zero(black_box(diff)))
}

fn usize_is_zero(value: usize) -> u8 {
    let folded = value | value.wrapping_neg();
    let top_bit = folded >> (usize::BITS - 1);
    (top_bit ^ 1) as u8
}

impl Default for PolicyTokenSet {
    fn default() -> Self {
        Self {
            slots: array::from_fn(|_| NOOP_POLICY_TOKEN),
            len: 0,
        }
    }
}
