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

pub fn mount_points() -> Result<Vec<PathBuf>, Error> {
    let mounts = fs::read_to_string("/proc/mounts").map_err(|err| Error::IoError(err))?;
    let mut mount_points = Vec::new();
    for mount in mounts.split('\n') {
        if mount.starts_with('#') {
            continue;
        }
        let mut it = mount.split(&[' ', '\t'][..]);
        let fs = it.next();
        if let Some(mount_point) = it.next() {
            mount_points.push(
                mount_point
                    .replace("\\040", " ")
                    .replace("\\011", "\t")
                    .into(),
            );
        }
    }
    Ok(mount_points)
}
