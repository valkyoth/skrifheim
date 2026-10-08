#![no_std]
#![forbid(unsafe_code)]
//! OS entropy only. No userspace PRNG state, seed fallback or deterministic
//! production feature. Unsupported targets fail to build through getrandom.
use skrifheim_crypto::{CryptoError, CryptoResult, EntropySource};

pub struct OsEntropy;
impl EntropySource for OsEntropy {
    fn fill(&mut self, destination: &mut [u8]) -> CryptoResult<()> {
        getrandom::fill(destination).map_err(|_| CryptoError::EntropyUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_provider_instances_read_os_entropy() -> CryptoResult<()> {
        let mut first = [0; 32];
        let mut second = [0; 32];
        OsEntropy.fill(&mut first)?;
        OsEntropy.fill(&mut second)?;
        assert_ne!(first, second);
        Ok(())
    }
}
