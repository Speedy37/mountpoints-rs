use crate::MountInfo;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::unix::prelude::OsStrExt;
use std::os::unix::prelude::OsStringExt;
use std::path::PathBuf;

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
    mut cb: impl FnMut(PathBuf, bool, Option<&str>) -> Result<(), Error>,
) -> Result<(), Error> {
    let mounts = fs::read("/proc/mounts").map_err(|err| Error::IoError(err))?;
    for mount in mounts.split(|b| *b == b'\n') {
        if mount.starts_with(b"#") {
            continue;
        }
        let mut it = mount.split(|b| *b == b' ' || *b == b'\t');
        let _fsname = it.next();
        if let Some(mountpath) = it.next() {
            let fstype = it.next().and_then(|mp| std::str::from_utf8(mp).ok());
            let dummy = match fstype.unwrap_or("") {
                "autofs" | "proc" | "subfs" | "debugfs" | "devpts" | "fusectl" | "mqueue"
                | "rpc_pipefs" | "sysfs" | "devfs" | "kernfs" | "ignore" | "configfs"
                | "binfmt_misc" | "bpf" | "pstore" | "cgroup" | "cgroup2" | "securityfs"
                | "efivarfs" => true,
                _ => false,
            };
            cb(PathBuf::from(unescape_path(mountpath)), dummy, fstype)?;
        }
    }
    Ok(())
}

pub fn mountinfos() -> Result<Vec<MountInfo>, Error> {
    let mut mountinfos = Vec::new();
    _mounts(|path, dummy, fstype| {
        let mut cpath = Vec::from(path.as_os_str().as_bytes());
        cpath.push(0);
        let mut stat = MaybeUninit::<libc::statvfs>::zeroed();
        let r = unsafe { libc::statvfs(cpath.as_ptr() as *const c_char, stat.as_mut_ptr()) };
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

pub fn mountpaths() -> Result<Vec<PathBuf>, Error> {
    let mut mountpaths = Vec::new();
    _mounts(|mountpath, _, _| {
        mountpaths.push(mountpath);
        Ok(())
    })?;
    Ok(mountpaths)
}

fn unescape_path(path: &[u8]) -> OsString {
    let mut out = vec![];
    let mut i = 0;
    loop {
        if let Some((bs_i, _)) = path.iter().enumerate().skip(i).find(|(_, b)| **b == b'\\') {
            out.extend_from_slice(&path[i..bs_i]);
            let escape =
                u8::from_str_radix(std::str::from_utf8(&path[bs_i + 1..bs_i + 4]).unwrap(), 8)
                    .unwrap();
            out.push(escape);
            i = bs_i + 4;
        } else {
            out.extend_from_slice(&path[i..]);
            break;
        }
    }
    OsString::from_vec(out)
}
