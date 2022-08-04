use crate::MountInfo;
use std::ffi::OsString;
use std::fmt;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use winapi::shared::winerror;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::GetVolumePathNamesForVolumeNameW;
use winapi::um::fileapi::{FindFirstVolumeW, FindNextVolumeW, FindVolumeClose};
use winapi::um::fileapi::{GetDiskFreeSpaceExW, GetVolumeInformationW};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winnt::{FILE_READ_ONLY_VOLUME, ULARGE_INTEGER};

const MAX_PATH: usize = 32768;

#[derive(Debug)]
pub enum Error {
    Utf16Error,
    VolumeIterError(u32),
    MountIterError(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Utf16Error => f.write_str("invalid utf16 path"),
            Error::VolumeIterError(code) => write!(f, "unable to get list of volumes: {}", code),
            Error::MountIterError(code) => write!(f, "unable to get list of mounts: {}", code),
        }
    }
}

fn _mounts(mut cb: impl FnMut(&[u16], PathBuf) -> Result<(), Error>) -> Result<(), Error> {
    let mut ret = Ok(());
    let mut name = [0u16; MAX_PATH];
    let mut names_vec = Vec::<u16>::new();
    let handle = unsafe { FindFirstVolumeW(name.as_mut_ptr(), name.len() as u32) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::VolumeIterError(unsafe { GetLastError() }));
    }
    loop {
        let mut names = [0u16; MAX_PATH];
        let mut len = names.len() as u32;
        let mut ok = unsafe {
            GetVolumePathNamesForVolumeNameW(
                name.as_ptr(),
                names.as_mut_ptr(),
                names.len() as u32,
                &mut len,
            )
        };
        let mut slice = &names[..];
        if ok == 0 && unsafe { GetLastError() } == winerror::ERROR_MORE_DATA {
            names_vec.clear();
            names_vec.resize(len as usize, 0);
            ok = unsafe {
                GetVolumePathNamesForVolumeNameW(
                    name.as_ptr(),
                    names_vec.as_mut_slice().as_mut_ptr(),
                    names_vec.len() as u32,
                    &mut len,
                )
            };
            slice = names_vec.as_slice();
        }
        if ok == 0 {
            let err = unsafe { GetLastError() };
            if err != winerror::ERROR_FILE_NOT_FOUND {
                ret = Err(Error::MountIterError(err));
                break;
            }
        } else {
            for mountpathw in slice.split(|&c| c == 0).take_while(|s| !s.is_empty()) {
                let mountpath = PathBuf::from(OsString::from_wide(mountpathw));
                cb(mountpathw, mountpath)?;
            }
        }

        let more = unsafe { FindNextVolumeW(handle, name.as_mut_ptr(), name.len() as u32) };
        if more == 0 {
            let err = unsafe { GetLastError() };
            if err != winerror::ERROR_NO_MORE_FILES {
                ret = Err(Error::VolumeIterError(err));
            }
            break;
        }
    }
    unsafe { FindVolumeClose(handle) };

    ret
}

#[allow(non_snake_case)]
pub fn mountinfos() -> Result<Vec<MountInfo>, Error> {
    let mut mountinfos = Vec::new();
    _mounts(|pathw, path| {
        let mut info = MountInfo {
            path,
            avail: None,
            free: None,
            size: None,
            name: None,
            format: None,
            readonly: None,
            dummy: true,
            __priv: (),
        };
        // sizes
        {
            let mut lpFreeBytesAvailableToCaller = ULARGE_INTEGER::default();
            let mut lpTotalNumberOfBytes = ULARGE_INTEGER::default();
            let mut lpTotalNumberOfFreeBytes = ULARGE_INTEGER::default();
            let ok = unsafe {
                GetDiskFreeSpaceExW(
                    pathw.as_ptr(),
                    &mut lpFreeBytesAvailableToCaller,
                    &mut lpTotalNumberOfBytes,
                    &mut lpTotalNumberOfFreeBytes,
                )
            };
            if ok != 0 {
                info.avail = Some(unsafe { *lpFreeBytesAvailableToCaller.QuadPart() });
                info.free = Some(unsafe { *lpTotalNumberOfFreeBytes.QuadPart() });
                info.size = Some(unsafe { *lpTotalNumberOfBytes.QuadPart() });
                info.dummy = false;
            }
        }
        // name
        {
            let mut name = [0u16; MAX_PATH];
            let mut fsname = [0u16; MAX_PATH];
            let mut flags = 0;
            let mut maxlen = 0;
            let ok = unsafe {
                GetVolumeInformationW(
                    pathw.as_ptr(),
                    name.as_mut_ptr(),
                    name.len() as u32,
                    std::ptr::null_mut(),
                    &mut maxlen,
                    &mut flags,
                    fsname.as_mut_ptr(),
                    fsname.len() as u32,
                )
            };
            if ok != 0 {
                if let Some(slice) = name.split(|&c| c == 0).next() {
                    info.name = Some(String::from_utf16(slice).map_err(|_| Error::Utf16Error)?);
                }
                if let Some(slice) = fsname.split(|&c| c == 0).next() {
                    info.format = Some(String::from_utf16(slice).map_err(|_| Error::Utf16Error)?);
                }
                info.readonly = Some((flags & FILE_READ_ONLY_VOLUME) == FILE_READ_ONLY_VOLUME);
                info.dummy = false;
            }
        }

        mountinfos.push(info);
        Ok(())
    })?;
    Ok(mountinfos)
}

pub fn mountpaths() -> Result<Vec<PathBuf>, Error> {
    let mut mountpaths = Vec::new();
    _mounts(|_, path| {
        mountpaths.push(path);
        Ok(())
    })?;
    Ok(mountpaths)
}
