use std::{
    fs::{self, File},
    io,
    path::Path,
};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use super::{SegmentFileError, SegmentResult, is_strict_staged_segment_file_name};

/// Removes owned stale segment staging files from a database directory.
///
/// This is a startup or maintenance operation. It must not run concurrently
/// with active segment writers in the same directory.
pub fn cleanup_staged_segments(dir: impl AsRef<Path>) -> SegmentResult<usize> {
    let dir = dir.as_ref();
    let dir_metadata = fs::symlink_metadata(dir)?;
    if dir_metadata.file_type().is_symlink() || !dir_metadata.is_dir() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup path must be a non-symlink directory",
        )));
    }
    let mut removed = 0_usize;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !is_strict_staged_segment_file_name(file_name) {
            continue;
        }
        let path = entry.path();
        validate_staged_cleanup_candidate(dir_metadata.uid(), &path)?;
        fs::remove_file(&path)?;
        removed += 1;
    }
    if removed != 0 {
        File::open(dir)?.sync_all()?;
    }
    Ok(removed)
}

fn validate_staged_cleanup_candidate(owner_uid: u32, path: &Path) -> SegmentResult<()> {
    let symlink_metadata = fs::symlink_metadata(path)?;
    if symlink_metadata.file_type().is_symlink() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup refuses symlink candidates",
        )));
    }
    if !symlink_metadata.is_file() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup refuses non-file candidates",
        )));
    }
    if symlink_metadata.uid() != owner_uid {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "segment staging cleanup refuses files with unexpected owner",
        )));
    }
    if symlink_metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "segment staging cleanup refuses files with unexpected permissions",
        )));
    }
    Ok(())
}
