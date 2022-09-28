//! # mountpoints - List mount points (windows, linux, macos)
//!
//! ## Example
//!
//! ```rust
//! use mountpoints::mountpaths;
//!
//! fn main() {
//!     for mountpath in mountpaths().unwrap() {
//!         println!("{}", mountpath.display());
//!     }
//! }
//! ```
//!
//! **Windows output:**
//!
//! ```log
//! C:\
//! C:\MyLittleMountPoint
//! D:\
//! ```
//!
//! **Linux output:**
//!
//! ```log
//! /mnt/wsl
//! /init
//! /dev
//! /dev/pts
//! /run
//! /run/lock
//! /run/shm
//! /run/user
//! /proc/sys/fs/binfmt_misc
//! /sys/fs/cgroup
//! /sys/fs/cgroup/unified
//! /mnt/c
//! /mnt/d
//! ```
//!
//! **Macos output:**
//!
//! ```log
//! /
//! /dev
//! /System/Volumes/Data
//! /private/var/vm
//! /System/Volumes/Data/home
//! /Volumes/VMware Shared Folders
//! ```

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::path::PathBuf;

#[cfg(target_os = "linux")]
use linux as sys;
#[cfg(target_os = "macos")]
use macos as sys;
#[cfg(target_os = "windows")]
use windows as sys;

#[derive(Debug, Clone)]
pub struct MountInfo {
    /// Mount path
    pub path: PathBuf,
    /// Available bytes to current user
    pub avail: Option<u64>,
    /// Free bytes
    pub free: Option<u64>,
    /// Size in bytes
    pub size: Option<u64>,
    /// Name
    pub name: Option<String>,
    /// Format (NTFS, FAT, ext4, ...)
    pub format: Option<String>,
    /// Read only
    pub readonly: Option<bool>,
    /// True if this mount point is likely to not be important
    pub dummy: bool,
    __priv: (),
}

#[derive(Debug)]
pub enum Error {
    WindowsUtf16Error,
    WindowsVolumeIterError(u32),
    WindowsMountIterError(u32),
    LinuxIoError(std::io::Error),
    LinuxPathParseError,
    MacOsGetfsstatError(i32),
    MacOsUtf8Error,
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WindowsUtf16Error => f.write_str("invalid utf16 path"),
            Error::WindowsVolumeIterError(code) => {
                write!(f, "unable to get list of volumes: {}", code)
            }
            Error::WindowsMountIterError(code) => {
                write!(f, "unable to get list of mounts: {}", code)
            }
            Error::LinuxIoError(err) => write!(f, "failed to read /proc/mounts: {}", err),
            Error::LinuxPathParseError => write!(f, "failed to parse path"),
            Error::MacOsGetfsstatError(err) => write!(f, "getfsstat failed: {}", err),
            Error::MacOsUtf8Error => write!(f, "invalid utf8 format"),
        }
    }
}

pub fn mountinfos() -> Result<Vec<MountInfo>, Error> {
    sys::mountinfos()
}
pub fn mountpaths() -> Result<Vec<PathBuf>, Error> {
    sys::mountpaths()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn root() -> &'static Path {
        if cfg!(target_os = "windows") {
            Path::new("C:\\")
        } else {
            Path::new("/")
        }
    }

    #[test]
    fn mountpaths_works() {
        let paths = mountpaths().unwrap();
        assert!(paths.len() > 0);
        assert!(paths.iter().any(|p| p == root()));

        for mountpath in &paths {
            eprintln!("{:?}", mountpath);
        }
    }
    #[test]
    fn mountinfosworks() {
        let infos = mountinfos().unwrap();
        assert!(infos.len() > 0);
        assert!(infos.iter().any(|i| if i.path == root() {
            assert!(i.size.unwrap_or_default() > 1024 * 1024); // > 1Mb
            assert!(i.avail.unwrap_or_default() < i.size.unwrap_or_default());
            assert!(i.free.unwrap_or_default() < i.size.unwrap_or_default());
            true
        } else {
            false
        }));
        for mountinfo in &infos {
            eprintln!("{:?}", mountinfo);
        }
    }
}
