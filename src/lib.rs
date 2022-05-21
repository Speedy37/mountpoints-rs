#[cfg(target_os = "linux")]
mod linux;
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux as sys;
#[cfg(target_os = "macos")]
use macos as sys;
#[cfg(target_os = "windows")]
use windows as sys;

#[derive(Debug, Clone)]
pub struct MountInfo {
    /// Mount path
    pub path: String,
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

pub use sys::{mountinfos, mountpaths, Error};
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mountpaths_works() {
        assert!(mountpaths().unwrap().len() > 0);
        for mountpath in mountpaths().unwrap() {
            eprintln!("{}", mountpath);
        }
    }
    #[test]
    fn mountinfosworks() {
        assert!(mountinfos().unwrap().len() > 0);
        for mountinfo in mountinfos().unwrap() {
            eprintln!("{:?}", mountinfo);
        }
    }
}
