use crate::MountInfo;
use std::ffi::{CStr, OsStr};
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

const MNT_RDONLY: u32 = libc::MNT_RDONLY as u32;

#[derive(Debug)]
pub enum Error {
    GetMntInfo64(c_int),
    Utf8Error,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GetMntInfo64(err) => write!(f, "getmntinfo64 failed: {}", err),
            Error::Utf8Error => write!(f, "invalid utf8 format"),
        }
    }
}

fn _mounts(mut cb: impl FnMut(&libc::statfs, PathBuf) -> Result<(), Error>) -> Result<(), Error> {
    let mut n: i32 = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
    let mut mntbuf = Vec::<libc::statfs>::new();
    if n > 0 {
        mntbuf.resize_with(n as usize, || unsafe { std::mem::zeroed() });
        let bufsize = mntbuf.len() * std::mem::size_of::<libc::statfs>();
        n = unsafe { libc::getfsstat(mntbuf.as_mut_ptr(), bufsize as c_int, libc::MNT_NOWAIT) };
        if n >= 0 {
            mntbuf.truncate(n as usize);
        }
    }
    if n < 0 {
        return Err(Error::GetMntInfo64(unsafe { *libc::__error() }));
    }
    for p in &mntbuf {
        let mountpath = OsStr::from_bytes(
            unsafe { CStr::from_ptr(p.f_mntonname.as_ptr() as *const c_char) }.to_bytes(),
        );
        cb(p, PathBuf::from(mountpath))?;
    }
    Ok(())
}

pub fn mountinfos() -> Result<Vec<MountInfo>, Error> {
    let mut mountinfos = Vec::new();
    _mounts(|stat, path| {
        mountinfos.push(MountInfo {
            path,
            avail: Some(stat.f_bavail.saturating_mul(u64::from(stat.f_bsize))),
            free: Some(stat.f_bfree.saturating_mul(u64::from(stat.f_bsize))),
            size: Some(stat.f_blocks.saturating_mul(u64::from(stat.f_bsize))),
            name: None,
            format: Some(
                unsafe { CStr::from_ptr(stat.f_fstypename.as_ptr() as *const c_char) }
                    .to_str()
                    .map_err(|_| Error::Utf8Error)?
                    .into(),
            ),
            readonly: Some((stat.f_flags & MNT_RDONLY) == MNT_RDONLY),
            dummy: false,
            __priv: (),
        });
        Ok(())
    })?;
    Ok(mountinfos)
}

pub fn mountpaths() -> Result<Vec<PathBuf>, Error> {
    let mut mountpaths = Vec::new();
    _mounts(|_, mountpath| {
        mountpaths.push(mountpath);
        Ok(())
    })?;
    Ok(mountpaths)
}
