use std::fs;

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

use crate::{SegmentFileError, SegmentFileWriter, SegmentWriteOptions, cleanup_staged_segments};

use super::helpers::{SegmentResult, temp_path, wal_to_segment_error};

#[test]
fn published_durability_error_does_not_disclose_storage_path() -> SegmentResult<()> {
    let path = temp_path("tenant-secret-compartment").map_err(wal_to_segment_error)?;
    let parent = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .ok_or_else(|| SegmentFileError::Io(std::io::Error::other("missing temp parent")))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| SegmentFileError::Io(std::io::Error::other("missing temp file name")))?;
    let error = SegmentFileError::PublishedDurabilityUnknown {
        source: std::io::Error::other("fsync failed"),
    };
    let display = error.to_string();
    let debug = format!("{error:?}");

    assert!(!display.contains(file_name));
    assert!(!display.contains(parent));
    assert!(!debug.contains(file_name));
    assert!(!debug.contains(parent));
    assert!(std::error::Error::source(&error).is_some());
    Ok(())
}

#[cfg(unix)]
#[test]
fn cleanup_staged_segments_removes_owned_strict_candidates_only() -> SegmentResult<()> {
    let dir = temp_path("segment-cleanup-dir").map_err(wal_to_segment_error)?;
    fs::create_dir(&dir)?;
    let staged = dir.join(".target.seg.skrifheim-stage-0123456789abcdef-1");
    let ignored = dir.join(".target.seg.skrifheim-stage-nothex-1");
    let no_leading_dot = dir.join("important.skrifheim-stage-0123456789abcdef-1");
    let empty_target = dir.join(".skrifheim-stage-0123456789abcdef-1");
    let uppercase_nonce = dir.join(".target.seg.skrifheim-stage-ABCDEF0123456789-1");
    let noncanonical_counter = dir.join(".target.seg.skrifheim-stage-0123456789abcdef-0001");
    fs::write(&staged, [])?;
    fs::set_permissions(&staged, fs::Permissions::from_mode(0o600))?;
    fs::write(&ignored, [])?;
    fs::set_permissions(&ignored, fs::Permissions::from_mode(0o600))?;
    fs::write(&no_leading_dot, [])?;
    fs::set_permissions(&no_leading_dot, fs::Permissions::from_mode(0o600))?;
    fs::write(&empty_target, [])?;
    fs::set_permissions(&empty_target, fs::Permissions::from_mode(0o600))?;
    fs::write(&uppercase_nonce, [])?;
    fs::set_permissions(&uppercase_nonce, fs::Permissions::from_mode(0o600))?;
    fs::write(&noncanonical_counter, [])?;
    fs::set_permissions(&noncanonical_counter, fs::Permissions::from_mode(0o600))?;

    assert_eq!(cleanup_staged_segments(&dir)?, 1);
    assert!(!staged.exists());
    assert!(ignored.exists());
    assert!(no_leading_dot.exists());
    assert!(empty_target.exists());
    assert!(uppercase_nonce.exists());
    assert!(noncanonical_counter.exists());

    fs::remove_file(ignored)?;
    fs::remove_file(no_leading_dot)?;
    fs::remove_file(empty_target)?;
    fs::remove_file(uppercase_nonce)?;
    fs::remove_file(noncanonical_counter)?;
    fs::remove_dir(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn cleanup_staged_segments_preserves_published_staging_like_segment_name() -> SegmentResult<()> {
    let dir = temp_path("segment-cleanup-published-dir").map_err(wal_to_segment_error)?;
    fs::create_dir(&dir)?;
    let published = dir.join("published.skrifheim-stage-0123456789abcdef-1");
    fs::write(&published, [])?;
    fs::set_permissions(&published, fs::Permissions::from_mode(0o600))?;

    assert_eq!(cleanup_staged_segments(&dir)?, 0);
    assert!(published.exists());

    fs::remove_file(published)?;
    fs::remove_dir(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_writer_rejects_reserved_staging_namespace_target() -> SegmentResult<()> {
    let dir = temp_path("segment-reserved-target-dir").map_err(wal_to_segment_error)?;
    fs::create_dir(&dir)?;
    let reserved_target = dir.join(".published.skrifheim-stage-0123456789abcdef-1");
    let result = SegmentFileWriter::create(&reserved_target, SegmentWriteOptions::new(false));

    assert!(matches!(result, Err(SegmentFileError::Io(_))));
    assert!(!reserved_target.exists());

    fs::remove_dir(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn cleanup_staged_segments_rejects_symlink_candidates() -> SegmentResult<()> {
    let dir = temp_path("segment-cleanup-symlink-dir").map_err(wal_to_segment_error)?;
    fs::create_dir(&dir)?;
    let target = dir.join("target");
    let staged_link = dir.join(".target.seg.skrifheim-stage-0123456789abcdef-2");
    fs::write(&target, [])?;
    symlink(&target, &staged_link)?;

    assert!(matches!(
        cleanup_staged_segments(&dir),
        Err(SegmentFileError::Io(_))
    ));
    assert!(staged_link.exists());

    fs::remove_file(staged_link)?;
    fs::remove_file(target)?;
    fs::remove_dir(dir)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn cleanup_staged_segments_rejects_symlink_directory() -> SegmentResult<()> {
    let target_dir = temp_path("segment-cleanup-symlink-target").map_err(wal_to_segment_error)?;
    let link_dir = temp_path("segment-cleanup-symlink-link").map_err(wal_to_segment_error)?;
    fs::create_dir(&target_dir)?;
    symlink(&target_dir, &link_dir)?;

    assert!(matches!(
        cleanup_staged_segments(&link_dir),
        Err(SegmentFileError::Io(_))
    ));

    fs::remove_file(link_dir)?;
    fs::remove_dir(target_dir)?;
    Ok(())
}
