use super::helpers::*;
use crate::{WalAppendOptions, WalFileReader, WalFileWriter};
use skrifheim_core::{TenantId, WorldId};
use skrifheim_crypto::{EncryptionDomain, RegionKeyId};
use skrifheim_storage::{BodyChecksum, WalFrameHeader, WalFrameHeaderInput};
use std::fs;

#[test]
fn writer_rejects_foreign_domain_on_append_status_and_reopen() -> WalResult<()> {
    let expected = wal_domain()?;
    let domains = [
        EncryptionDomain::wal(id(TenantId::from_u128(9))?, None, None),
        EncryptionDomain::wal(tenant()?, None, None),
        EncryptionDomain::wal(
            tenant()?,
            Some(id(RegionKeyId::from_u128(9))?),
            Some(id(WorldId::from_u128(3))?),
        ),
        EncryptionDomain::wal(
            tenant()?,
            Some(id(RegionKeyId::from_u128(2))?),
            Some(id(WorldId::from_u128(9))?),
        ),
    ];
    let body = [1];
    let batch = header(10, &body)?;
    for other in domains {
        let path = temp_path("writer-domain")?;
        let foreign = WalFrameHeader::new(WalFrameHeaderInput {
            record_kind: batch.record_kind(),
            tenant_id: other.tenant_id(),
            tx_id: batch.tx_id(),
            encryption_key_id: batch.encryption_key_id(),
            crypto_epoch: batch.crypto_epoch(),
            encryption_domain: other,
            encrypted_body_len: 1,
            body_crc64: BodyChecksum::Present(batch.body_crc64()),
        })?;
        let mut writer = WalFileWriter::open_append(&path, expected, WalAppendOptions::default())?;
        // Reject foreign writes even before the file's first frame establishes a domain.
        for populated in [false, true] {
            if populated {
                let _outcome = writer.append_transaction_once(&batch, &body)?;
            }
            let before = fs::read(&path)?;
            assert!(writer.append_frame(&foreign, &body).is_err());
            assert!(writer.append_transaction_once(&foreign, &body).is_err());
            assert!(writer.transaction_status(other, batch.tx_id()).is_err());
            assert_eq!(fs::read(&path)?, before);
            assert!(!writer.is_poisoned());
        }
        drop(writer);
        let before = fs::read(&path)?;
        assert!(WalFileWriter::open_append(&path, other, WalAppendOptions::default()).is_err());
        assert_eq!(fs::read(&path)?, before);
        // A pre-existing mixed file must not be accepted under either domain.
        let mut mixed = before;
        mixed.extend_from_slice(&foreign.encode());
        mixed.extend_from_slice(&body);
        fs::write(&path, &mixed)?;
        for domain in [expected, other] {
            assert!(
                WalFileWriter::open_append(&path, domain, WalAppendOptions::default()).is_err()
            );
            let mut reader = WalFileReader::open(&path, domain)?;
            loop {
                match reader.next_frame() {
                    Ok(Some(_)) => {}
                    Err(_) => break,
                    Ok(None) => return Err(crate::WalFileError::PartialFrame),
                }
            }
        }
        assert_eq!(fs::read(&path)?, mixed);
        fs::remove_file(path)?;
    }
    Ok(())
}
