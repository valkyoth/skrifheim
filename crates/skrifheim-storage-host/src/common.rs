use std::{fs::File, io, path::Path};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[cfg(not(unix))]
compile_error!(
    "skrifheim-storage-host requires a supported Unix target for O_NOFOLLOW, owner-only file permissions, and parent-directory fsync; no hardened fallback exists for this platform"
);

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) const O_NOFOLLOW_FLAG: i32 = 0o400000;

#[cfg(any(target_os = "illumos", target_os = "solaris"))]
pub(crate) const O_NOFOLLOW_FLAG: i32 = 0x20000;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub(crate) const O_NOFOLLOW_FLAG: i32 = 0x0100;

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
compile_error!(
    "O_NOFOLLOW is not defined for this Unix platform; add an explicit constant or disable WAL file I/O for this target"
);

const _: () = assert!(
    usize::BITS >= 32,
    "skrifheim-storage-host requires at least a 32-bit address space"
);

#[cfg(unix)]
pub(crate) fn add_no_follow(options: &mut std::fs::OpenOptions) {
    options.custom_flags(O_NOFOLLOW_FLAG);
}

#[cfg(unix)]
pub(crate) fn require_explicit_parent(path: &Path) -> io::Result<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must include an explicit parent directory for durable fsync",
            )
        })
}

#[cfg(unix)]
pub(crate) fn fsync_parent_dir(path: &Path) -> io::Result<()> {
    let parent = require_explicit_parent(path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
