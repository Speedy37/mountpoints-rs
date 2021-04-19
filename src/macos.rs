use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;

#[allow(non_camel_case_types)]
type uid_t = u32;

#[repr(C)]
struct fsid_t {
    val: [i32; 2],
}

const MFSTYPENAMELEN: usize = 16;
const MAXPATHLEN: usize = 1024;
const MNT_NOWAIT: c_int = 2;

#[repr(C)]
struct statfs64 {
    /// fundamental file system block size
    f_bsize: u32,
    /// optimal transfer block size
    f_iosize: i32,
    /// total data blocks in file system
    f_blocks: u64,
    /// free blocks in fs
    f_bfree: u64,
    /// free blocks avail to non-superuser
    f_bavail: u64,
    /// total file nodes in file system
    f_files: u64,
    /// free file nodes in fs
    f_ffree: u64,
    /// file system id
    f_fsid: fsid_t,
    /// user that mounted the filesystem    
    f_owner: uid_t,
    /// type of filesystem
    f_type: u32,
    /// copy of mount exported flags
    f_flags: u32,
    /// fs sub-type (flavor)
    f_fssubtype: u32,
    /// fs type name
    f_fstypename: [u8; MFSTYPENAMELEN],
    /// directory on which mounted
    f_mntonname: [u8; MAXPATHLEN],
    /// mounted filesystem
    f_mntfromname: [u8; MAXPATHLEN],
    /// For future use  
    f_reserved: [u32; 8],
}

extern "C" {
    fn getmntinfo64(mntbufp: *mut *const statfs64, flags: c_int) -> c_int;
}

#[derive(Debug)]
pub enum Error {
    GetMntInfo64(c_int),
    Utf8Error,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::GetMntInfo64(err) => write!(f, "getmntinfo64 failed: {}", err),
            Error::Utf8Error => write!(f, "invalid utf8 path"),
        }
    }
}

pub fn mount_points() -> Result<Vec<PathBuf>, Error> {
    let mut mntbuf: *const statfs64 = std::ptr::null_mut();
    let mut n = unsafe { getmntinfo64(&mut mntbuf, MNT_NOWAIT) };
    if n <= 0 {
        return Err(Error::GetMntInfo64(unsafe { *libc::__error() }));
    }

    let mut mount_points = Vec::with_capacity(n as usize);
    while n > 0 {
        let p: &statfs64 = unsafe { &*mntbuf };
        let mount_point = unsafe { CStr::from_ptr(p.f_mntonname.as_ptr() as *const c_char) };
        mount_points.push(mount_point.to_str().map_err(|_| Error::Utf8Error)?.into());
        n -= 1;
    }

    Ok(mount_points)
}
