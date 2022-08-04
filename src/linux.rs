use crate::MountInfo;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::os::unix::prelude::OsStrExt;
use std::os::unix::prelude::OsStringExt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PathParseError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "failed to read /proc/mounts: {}", err),
            Error::PathParseError => write!(f, "failed to parse path"),
        }
    }
}

fn _mounts(
    mut cb: impl FnMut(PathBuf, bool, Option<&str>) -> Result<(), Error>,
) -> Result<(), Error> {
    let mounts = fs::read("/proc/mounts").map_err(|err| Error::IoError(err))?;
    for mount in mounts.split(|b| *b == b'\n') {
        // Each filesystem is described on a separate line. Fields on each
        // line are separated by tabs or spaces. Lines starting with '#' are
        // comments. Blank lines are ignored.
        if mount.starts_with(b"#") {
            continue;
        }
        let mut it = mount
            .split(|b| *b == b' ' || *b == b'\t')
            .skip(1 /*fs_spec*/);
        if let Some(mountpath) = it.next() {
            let fs_vfstype = it.next().and_then(|mp| std::str::from_utf8(mp).ok());
            let dummy = match fs_vfstype.unwrap_or("") {
                "autofs" | "proc" | "subfs" | "debugfs" | "devpts" | "fusectl" | "mqueue"
                | "rpc_pipefs" | "sysfs" | "devfs" | "kernfs" | "ignore" | "configfs"
                | "binfmt_misc" | "bpf" | "pstore" | "cgroup" | "cgroup2" | "securityfs"
                | "efivarfs" => true,
                _ => false,
            };
            cb(unescape_path(mountpath)?.into(), dummy, fs_vfstype)?;
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
        let stat = if r == 0 {
            Some(unsafe { stat.assume_init() })
        } else {
            None
        };
        mountinfos.push(MountInfo {
            path,
            avail: stat.map(|stat| stat.f_bavail.saturating_mul(u64::from(stat.f_bsize))),
            free: stat.map(|stat| stat.f_bfree.saturating_mul(u64::from(stat.f_bsize))),
            size: stat.map(|stat| stat.f_blocks.saturating_mul(u64::from(stat.f_frsize))),
            name: None,
            format: fstype.map(|s| s.to_string()),
            readonly: stat.map(|stat| (stat.f_flag & libc::ST_RDONLY) == libc::ST_RDONLY),
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

/// unescape octal trigrams from path (ie. `\040` => ` `)
fn unescape_path(path: &[u8]) -> Result<OsString, Error> {
    let mut it = path.split(|b| *b == b'\\');
    if let (Some(left), Some(mut part)) = (it.next(), it.next()) {
        let mut vec = Vec::<u8>::new();
        vec.extend_from_slice(left);
        loop {
            if part.len() < 3 {
                return Err(Error::PathParseError);
            }
            let escaped = part
                .iter()
                .take(3)
                .try_fold(0u8, |acc, digit| match digit {
                    b'0'..=b'7' if acc < 0o40 => Ok(acc * 8 + (digit - b'0')),
                    _ => Err(Error::PathParseError),
                })?;
            vec.push(escaped);
            vec.extend_from_slice(&part[3..]);
            match it.next() {
                None => break,
                Some(p) => part = p,
            }
        }
        Ok(OsString::from_vec(vec))
    } else {
        Ok(OsStr::from_bytes(path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_path_works() {
        assert_eq!(unescape_path(b"").unwrap(), "");
        assert_eq!(
            unescape_path(b"/tmp/a\\134\\054b\\134\\134c/lower").unwrap(),
            "/tmp/a\\,b\\\\c/lower"
        );
        assert!(matches!(
            unescape_path(b"\\54ab").unwrap_err(),
            Error::PathParseError
        ));
        assert!(matches!(
            unescape_path(b"\\666").unwrap_err(),
            Error::PathParseError
        ));
        assert_eq!(
            unescape_path(b"\\000\\377").unwrap(),
            OsStr::from_bytes(b"\x00\xFF")
        );
    }
}
