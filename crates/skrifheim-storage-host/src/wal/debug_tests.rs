use super::*;

#[test]
fn frame_debug_is_fixed_size_even_for_maximum_body() -> Result<()> {
    for size in [1, skrifheim_storage::WAL_FRAME_BODY_MAX_BYTES as usize] {
        let body = vec![b'S'; size];
        let frame = WalFileFrame {
            header: crate::tests::helpers::header(123456, &body)?,
            encrypted_body: body,
        };
        assert_eq!(format!("{frame:?}"), "WalFileFrame(<redacted>)");
        assert_eq!(format!("{frame:#?}"), "WalFileFrame(<redacted>)");
    }
    Ok(())
}
