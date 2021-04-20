use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "failed to read /proc/mounts: {}", err),
        }
    }
}

pub fn mountpaths() -> Result<Vec<String>, Error> {
    let mounts = fs::read_to_string("/proc/mounts").map_err(|err| Error::IoError(err))?;
    let mut mountpaths = Vec::new();
    for mount in mounts.split('\n') {
        if mount.starts_with('#') {
            continue;
        }
        let mut it = mount.split(&[' ', '\t'][..]);
        let fs = it.next();
        if let Some(mountpath) = it.next() {
            mountpaths.push(mountpath.replace("\\040", " ").replace("\\011", "\t"));
        }
    }
    Ok(mountpaths)
}
