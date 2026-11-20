use skrifheim::storage::*;

pub fn wal_header(data: &[u8]) {
    if let Ok(header) = WalFrameHeader::parse(data) {
        assert_eq!(data.len(), WAL_FRAME_HEADER_BYTES);
        assert!(header.validate().is_ok());
        assert_eq!(header.encode(), data);
        assert!((1..=WAL_FRAME_BODY_MAX_BYTES).contains(&header.encrypted_body_len()));
        assert!(WalFrameHeader::parse_for_domain(data, header.encryption_domain()).is_ok());
    }
}

pub fn segment_metadata(data: &[u8]) {
    if let Ok(header) = SegmentHeader::parse(data) {
        assert!(header.validate().is_ok());
        assert!((1..=SEGMENT_BODY_MAX_BYTES).contains(&header.body_len()));
        assert!(SegmentHeader::parse_for_domain(data, header.encryption_domain()).is_ok());
        let canonical = header.encode();
        assert!(SegmentHeader::parse(&canonical).is_ok());
        let footer = SegmentFooter::from_header(&header);
        assert!(footer.is_ok());
        if let Ok(footer) = footer {
            assert!(footer.validate_against_header(&header).is_ok());
        }
    }
    if let Ok(footer) = SegmentFooter::parse(data) {
        assert!(footer.validate().is_ok());
        assert!(SegmentFooter::parse(&footer.encode()).is_ok());
    }
    if data.len() == 512
        && let (Ok(header), Ok(footer)) = (
            SegmentHeader::parse(&data[..256]),
            SegmentFooter::parse(&data[256..]),
        )
    {
        let matched = footer.validate_against_header(&header).is_ok();
        if matched {
            assert!(!footer.has_legacy_unbound_kind());
            assert_eq!(header.body_len(), footer.body_len());
            assert_eq!(header.tenant_id(), footer.tenant_id());
            assert!(
                header
                    .encryption_domain()
                    .structurally_equal_ct(&footer.encryption_domain())
            );
        }
    }
}

pub fn segment_layout(data: &[u8]) {
    if let Some(prefix) = data.get(..256)
        && let Ok(header) = SegmentHeader::parse(prefix)
        && segment_file_length_matches(header.body_len(), data.len() as u64)
    {
        let end = data.len() - 256;
        assert_eq!(data[256..end].len() as u64, header.body_len());
        if let Ok(footer) = SegmentFooter::parse(&data[end..])
            && footer.validate_against_header(&header).is_ok()
        {
            assert_eq!(footer.body_len(), header.body_len());
            assert_eq!(footer.body_crc64(), header.body_crc64());
        }
    }
    if let Some(bytes) = data.get(..16) {
        let mut body = [0; 8];
        let mut file = [0; 8];
        body.copy_from_slice(&bytes[..8]);
        file.copy_from_slice(&bytes[8..]);
        let body = u64::from_le_bytes(body);
        let file = u64::from_le_bytes(file);
        let expected = body > 0
            && body <= SEGMENT_BODY_MAX_BYTES
            && u128::from(file) == u128::from(body) + 512;
        assert_eq!(segment_file_length_matches(body, file), expected);
    }
}
