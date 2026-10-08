use crate::RustCryptoProvider;
use alloc::vec::Vec;
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{AeadInOut, KeyInit},
};
use core::cell::Cell;
use core::fmt;
use sanitization::wipe::WipeOnDrop;
use skrifheim_crypto::{
    AEAD_PLAINTEXT_MAX_BYTES, AEAD_TAG_BYTES, AeadContext, AeadProvider, CryptoEpoch, CryptoError,
    CryptoResult, EncryptionDomain, EntropySource, KeyId, PublicNonce,
};
use subtle::ConstantTimeEq;

/// No public secret-retention callback is available.
/// ```compile_fail
/// use skrifheim_crypto_rustcrypto::SecretKeyBuffer;
/// fn retain(key: &SecretKeyBuffer) {
///     key.with_secret(|bytes| bytes.to_vec());
/// }
/// ```
pub struct SecretKeyBuffer {
    seals: Cell<u64>,
    bytes: WipeOnDrop<[u8; 32]>,
    key_id: KeyId,
    epoch: CryptoEpoch,
    domain: EncryptionDomain,
}
impl SecretKeyBuffer {
    pub fn generate(entropy: &mut impl EntropySource, context: &AeadContext) -> CryptoResult<Self> {
        let mut bytes = WipeOnDrop::new([0; 32]);
        bytes.with_secret_mut(|bytes| entropy.fill(bytes))?;
        Ok(Self {
            seals: Cell::new(0),
            bytes,
            key_id: context.key_id(),
            epoch: context.epoch(),
            domain: context.domain(),
        })
    }
    fn require_context(&self, context: &AeadContext) -> CryptoResult<()> {
        let id_ok = self
            .key_id
            .get()
            .to_le_bytes()
            .ct_eq(&context.key_id().get().to_le_bytes())
            .unwrap_u8();
        let epoch_ok = self
            .epoch
            .get()
            .to_le_bytes()
            .ct_eq(&context.epoch().get().to_le_bytes())
            .unwrap_u8();
        let domain_ok = self.domain.structurally_equal_ct(&context.domain()) as u8;
        if id_ok & epoch_ok & domain_ok == 0 {
            return Err(CryptoError::AuthenticationFailed);
        }
        Ok(())
    }
}

/// Owned scrubbed plaintext; no public secret-retention callback or byte export.
pub struct SensitivePlaintextBuffer(WipeOnDrop<Vec<u8>>);
impl SensitivePlaintextBuffer {
    pub fn from_vec(bytes: Vec<u8>) -> CryptoResult<Self> {
        let guarded = WipeOnDrop::new(bytes);
        if guarded.with_secret(|bytes| {
            bytes.len() > AEAD_PLAINTEXT_MAX_BYTES || bytes.capacity() > AEAD_PLAINTEXT_MAX_BYTES
        }) {
            return Err(CryptoError::InvalidInput);
        }
        Ok(Self(guarded))
    }
}
impl fmt::Debug for SecretKeyBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKeyBuffer(<redacted>)")
    }
}
impl fmt::Debug for SensitivePlaintextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SensitivePlaintextBuffer(<redacted>)")
    }
}

impl AeadProvider for RustCryptoProvider {
    type Key = SecretKeyBuffer;
    type Plaintext = SensitivePlaintextBuffer;

    fn plaintext_len(&self, plaintext: &Self::Plaintext) -> usize {
        plaintext.0.with_secret(Vec::len)
    }

    fn seal(
        &self,
        key: &Self::Key,
        context: &AeadContext,
        nonce: PublicNonce,
        aad: &[u8],
        mut plaintext: Self::Plaintext,
    ) -> CryptoResult<Vec<u8>> {
        key.require_context(context)?;
        if key.seals.get() >= (1_u64 << 32) {
            return Err(CryptoError::ProviderFailure);
        }
        key.seals.set(key.seals.get() + 1);
        encrypt(key, nonce.as_bytes(), aad, &mut plaintext)
    }

    fn open(
        &self,
        key: &Self::Key,
        context: &AeadContext,
        nonce: PublicNonce,
        aad: &[u8],
        ciphertext: &[u8],
    ) -> CryptoResult<Self::Plaintext> {
        key.require_context(context)?;
        decrypt(key, nonce.as_bytes(), aad, ciphertext)
    }
}

fn encrypt(
    key: &SecretKeyBuffer,
    nonce: &[u8; 24],
    aad: &[u8],
    plaintext: &mut SensitivePlaintextBuffer,
) -> CryptoResult<Vec<u8>> {
    key.bytes.with_secret(|bytes| {
        let cipher =
            XChaCha20Poly1305::new_from_slice(bytes).map_err(|_| CryptoError::ProviderFailure)?;
        plaintext.0.with_secret_mut(|bytes| {
            let tag = cipher
                .encrypt_inout_detached(nonce.into(), aad, bytes.as_mut_slice().into())
                .map_err(|_| CryptoError::ProviderFailure)?;
            // Only authenticated ciphertext leaves the clearing owner.
            bytes.extend_from_slice(&tag);
            Ok(core::mem::take(bytes))
        })
    })
}

fn decrypt(
    key: &SecretKeyBuffer,
    nonce: &[u8; 24],
    aad: &[u8],
    ciphertext: &[u8],
) -> CryptoResult<SensitivePlaintextBuffer> {
    if ciphertext.len() < AEAD_TAG_BYTES
        || ciphertext.len() > AEAD_PLAINTEXT_MAX_BYTES + AEAD_TAG_BYTES
    {
        return Err(CryptoError::InvalidInput);
    }
    let (body, tag) = ciphertext.split_at(ciphertext.len() - AEAD_TAG_BYTES);
    let mut output = WipeOnDrop::new(body.to_vec());
    key.bytes.with_secret(|bytes| {
        let cipher =
            XChaCha20Poly1305::new_from_slice(bytes).map_err(|_| CryptoError::ProviderFailure)?;
        let tag = tag
            .try_into()
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        output.with_secret_mut(|bytes| {
            cipher
                .decrypt_inout_detached(nonce.into(), aad, bytes.as_mut_slice().into(), &tag)
                .map_err(|_| CryptoError::AuthenticationFailed)
        })
    })?;
    Ok(SensitivePlaintextBuffer(output))
}

#[cfg(test)]
mod tests;
