use std::fmt;
use std::path::PathBuf;

// GetDriveTypeW
// GetDiskFreeSpaceExW

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

pub fn mountpaths() -> Result<Vec<PathBuf>, Error> {
    use winapi::shared::winerror::{ERROR_MORE_DATA, ERROR_NO_MORE_FILES};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::fileapi::GetVolumePathNamesForVolumeNameW;
    use winapi::um::fileapi::{FindFirstVolumeW, FindNextVolumeW, FindVolumeClose};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;

    let mut mountpaths = Vec::new();
    const MAX_PATH: usize = 32768;
    let mut name = [0u16; MAX_PATH];
    let handle = unsafe { FindFirstVolumeW(name.as_mut_ptr(), name.len() as u32) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::VolumeIterError(unsafe { GetLastError() }));
    }
    loop {
        let mut names_vec;
        let mut names = [0u16; 1];
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
        if ok == 0 && unsafe { GetLastError() } == ERROR_MORE_DATA {
            names_vec = vec![0u16; len as usize];
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
            return Err(Error::MountIterError(unsafe { GetLastError() }));
        }

        for mount_pointw in slice.split(|&c| c == 0).take_while(|s| !s.is_empty()) {
            let mountpath = String::from_utf16(mount_pointw).map_err(|_| Error::Utf16Error)?;
            mountpaths.push(mountpath.into());
        }

        let more = unsafe { FindNextVolumeW(handle, name.as_mut_ptr(), name.len() as u32) };
        if more == 0 {
            let err = unsafe { GetLastError() };
            if err == ERROR_NO_MORE_FILES {
                break;
            } else {
                return Err(Error::VolumeIterError(err));
            }
        }
    }
    if unsafe { FindVolumeClose(handle) } == 0 {
        return Err(Error::VolumeIterError(unsafe { GetLastError() }));
    }

    Ok(mountpaths)
}
