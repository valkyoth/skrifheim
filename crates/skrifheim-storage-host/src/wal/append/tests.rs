use super::*;
use crate::tests::helpers::header;

struct FaultSink {
    bytes: Vec<u8>,
    fail_after: usize,
    fail_flush: bool,
    fail_sync: bool,
}
impl Write for FaultSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let n = bytes
            .len()
            .min(self.fail_after.saturating_sub(self.bytes.len()));
        if n == 0 {
            return Err(io::Error::other("injected write failure"));
        }
        self.bytes.extend_from_slice(&bytes[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}
impl WalSink for FaultSink {
    fn sync_all(&mut self) -> io::Result<()> {
        if self.fail_sync {
            Err(io::Error::other("injected sync failure"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn failures_at_every_byte_flush_and_sync_poison_writer() -> Result<()> {
    let body = [7; 4];
    let header = header(101, &body)?;
    let length = WAL_FRAME_HEADER_BYTES + body.len();
    for point in 0..length + 2 {
        let mut sink = FaultSink {
            bytes: Vec::new(),
            fail_after: point,
            fail_flush: point == length,
            fail_sync: point == length + 1,
        };
        let mut state = AppendState::new(0);
        assert!(matches!(
            state.append(&mut sink, DurabilityMode::SyncAll, &header, &body),
            Err(WalFileError::Ambiguous { .. })
        ));
        assert!(state.is_poisoned());
        let count = sink.bytes.len();
        assert!(matches!(
            state.append(&mut sink, DurabilityMode::SyncAll, &header, &body),
            Err(WalFileError::Poisoned)
        ));
        assert_eq!(sink.bytes.len(), count);
    }
    Ok(())
}

#[test]
fn successful_receipts_distinguish_buffered_and_durable_ranges() -> Result<()> {
    let body = [7; 4];
    let header = header(102, &body)?;
    let mut sink = FaultSink {
        bytes: Vec::new(),
        fail_after: usize::MAX,
        fail_flush: false,
        fail_sync: false,
    };
    let mut state = AppendState::new(1000);
    let first = state.append(&mut sink, DurabilityMode::Buffered, &header, &body)?;
    let second = state.append(&mut sink, DurabilityMode::SyncAll, &header, &body)?;
    assert!(matches!(first, WalAppendOutcome::Buffered(r) if r.start() == 1000 && r.end() == 1124));
    assert!(matches!(second, WalAppendOutcome::Durable(r) if r.start() == 1124 && r.end() == 1248));
    assert_eq!(
        first.encode_evidence(),
        [
            1, 0, 232, 3, 0, 0, 0, 0, 0, 0, 100, 4, 0, 0, 0, 0, 0, 0, 102, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(
        second.encode_evidence(),
        [
            1, 1, 100, 4, 0, 0, 0, 0, 0, 0, 224, 4, 0, 0, 0, 0, 0, 0, 102, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0
        ]
    );
    Ok(())
}
