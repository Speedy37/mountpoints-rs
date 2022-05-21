use crate::MountInfo;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_int};

#[allow(non_camel_case_types)]
type uid_t = u32;

#[repr(C)]
struct fsid_t {
    val: [i32; 2],
}

const MFSTYPENAMELEN: usize = 16;
const MAXPATHLEN: usize = 1024;
const MNT_NOWAIT: c_int = 2;
const MNT_RDONLY: u32 = 1;

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
    #[link_name = "\u{1}_getfsstat$INODE64"]
    fn getfsstat(buf: *mut statfs64, bufsize: c_int, flags: c_int) -> c_int;
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

fn _mounts(mut cb: impl FnMut(&statfs64, String) -> Result<(), Error>) -> Result<(), Error> {
    let mut n: i32 = unsafe { getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT) };
    let mut mntbuf = Vec::<statfs64>::new();
    if n > 0 {
        mntbuf.resize_with(n as usize, || unsafe { std::mem::zeroed() });
        let bufsize = mntbuf.len() * std::mem::size_of::<statfs64>();
        n = unsafe { getfsstat(mntbuf.as_mut_ptr(), bufsize as c_int, MNT_NOWAIT) };
        if n >= 0 {
            mntbuf.truncate(n as usize);
        }
    }
    if n < 0 {
        return Err(Error::GetMntInfo64(unsafe { *libc::__error() }));
    }
    for p in &mntbuf {
        let mountpath = unsafe { CStr::from_ptr(p.f_mntonname.as_ptr() as *const c_char) };
        cb(p, mountpath.to_str().map_err(|_| Error::Utf8Error)?.into())?;
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

pub fn mountpaths() -> Result<Vec<String>, Error> {
    let mut mountpaths = Vec::new();
    _mounts(|_, mountpath| {
        mountpaths.push(mountpath);
        Ok(())
    })?;
    Ok(mountpaths)
}
