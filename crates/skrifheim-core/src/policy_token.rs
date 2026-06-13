use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::hint::black_box;

use crate::{Result, SkrifheimError};

pub const POLICY_TOKEN_MAX_BYTES: usize = 128;
pub const POLICY_TOKEN_SET_MAX_ITEMS: usize = 64;
const INVALID_POLICY_TOKEN_NEEDLE: &str = concat!(
    "SKRIFHEIM-INVALID-NEEDLE-SKRIFHEIM-INVALID-NEEDLE-",
    "SKRIFHEIM-INVALID-NEEDLE-SKRIFHEIM-INVALID-NEEDLE-",
    "SKRIFHEIM-INVALID-NEEDLE"
);

pub fn canonical_policy_set(values: Vec<String>) -> Result<BTreeSet<String>> {
    if values.len() > POLICY_TOKEN_SET_MAX_ITEMS {
        return Err(SkrifheimError::InvalidSecurityToken);
    }
    values.into_iter().map(canonical_policy_token).collect()
}

pub fn canonical_policy_token(mut value: String) -> Result<String> {
    if !is_valid_policy_token(&value) {
        return Err(SkrifheimError::InvalidSecurityToken);
    }
    value.make_ascii_uppercase();
    Ok(value)
}

#[must_use]
pub fn contains_policy_token_ct(tokens: &BTreeSet<String>, needle: &str) -> bool {
    if tokens.len() > POLICY_TOKEN_SET_MAX_ITEMS {
        return false;
    }
    let needle = match canonical_policy_token(String::from(needle)) {
        Ok(needle) => needle,
        Err(_) => String::from(INVALID_POLICY_TOKEN_NEEDLE),
    };
    let mut found = 0_u8;
    let mut tokens = tokens.iter();
    let mut index = 0;
    while index < POLICY_TOKEN_SET_MAX_ITEMS {
        let token = tokens.next();
        let present = token.is_some() as u8;
        let token = match token {
            Some(token) => token.as_str(),
            None => "SKRIFHEIM-NOOP",
        };
        found |= present & policy_token_eq_ct(token, &needle);
        index += 1;
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

fn policy_token_eq_ct(left: &str, right: &str) -> u8 {
    let left = black_box(left.as_bytes());
    let right = black_box(right.as_bytes());
    let mut diff = black_box(left.len() ^ right.len());
    let mut index = 0;
    while index < POLICY_TOKEN_MAX_BYTES {
        let left_byte = black_box(byte_at(left, black_box(index)));
        let right_byte = black_box(byte_at(right, black_box(index)));
        diff |= usize::from(left_byte ^ right_byte);
        index += 1;
    }
    black_box(usize_is_zero(black_box(diff)))
}

fn byte_at(bytes: &[u8], index: usize) -> u8 {
    if index < bytes.len() { bytes[index] } else { 0 }
}

fn usize_is_zero(value: usize) -> u8 {
    let folded = value | value.wrapping_neg();
    let top_bit = folded >> (usize::BITS - 1);
    (top_bit ^ 1) as u8
}
