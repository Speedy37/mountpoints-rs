use crate::MountInfo;
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::mem::MaybeUninit;
use std::os::raw::c_int;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    StatError(c_int),
    NulError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "failed to read /proc/mounts: {}", err),
            Error::StatError(err) => write!(f, "statvfs failed: {}", err),
            Error::NulError => write!(f, "mount path contains NUL"),
        }
    }
}

fn _mounts(
    mut cb: impl FnMut(String, bool, Option<&str>) -> Result<(), Error>,
) -> Result<(), Error> {
    let mounts = fs::read_to_string("/proc/mounts").map_err(|err| Error::IoError(err))?;
    for mount in mounts.split('\n') {
        if mount.starts_with('#') {
            continue;
        }
        let mut it = mount.split(&[' ', '\t'][..]);
        let _fsname = it.next();
        if let Some(mountpath) = it.next() {
            let fstype = it.next();
            let dummy = match fstype.unwrap_or("") {
                "autofs" | "proc" | "subfs" | "debugfs" | "devpts" | "fusectl" | "mqueue"
                | "rpc_pipefs" | "sysfs" | "devfs" | "kernfs" | "ignore" => true,
                _ => false,
            };
            let path = mountpath.replace("\\040", " ").replace("\\011", "\t");
            cb(path, dummy, fstype)?;
        }
    }
    Ok(())
}

pub fn mountinfos() -> Result<Vec<MountInfo>, Error> {
    let mut mountinfos = Vec::new();
    _mounts(|path, dummy, fstype| {
        let cpath = CString::new(path.as_str()).map_err(|_| Error::NulError)?;
        let mut stat = MaybeUninit::<libc::statvfs>::zeroed();
        let r = unsafe { libc::statvfs(cpath.as_ptr(), stat.as_mut_ptr()) };
        if r != 0 {
            return Err(Error::StatError(unsafe { *libc::__errno_location() }));
        }
        let stat = unsafe { stat.assume_init() };
        mountinfos.push(MountInfo {
            path,
            avail: Some(stat.f_bavail.saturating_mul(u64::from(stat.f_bsize))),
            free: Some(stat.f_bfree.saturating_mul(u64::from(stat.f_bsize))),
            size: Some(stat.f_blocks.saturating_mul(u64::from(stat.f_frsize))),
            name: None,
            format: fstype.map(|s| s.to_string()),
            readonly: Some((stat.f_flag & libc::ST_RDONLY) == libc::ST_RDONLY),
            dummy,
            __priv: (),
        });
        Ok(())
    })?;
    Ok(mountinfos)
}

pub fn mountpaths() -> Result<Vec<String>, Error> {
    let mut mountpaths = Vec::new();
    _mounts(|mountpath, _, _| {
        mountpaths.push(mountpath);
        Ok(())
    })?;
    Ok(mountpaths)
}
