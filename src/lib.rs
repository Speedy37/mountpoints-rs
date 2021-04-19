#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{mount_points, Error};
#[cfg(target_os = "macos")]
pub use macos::{mount_points, Error};
#[cfg(target_os = "windows")]
pub use windows::{mount_points, Error};

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for mount_point in mount_points().unwrap() {
            eprintln!("{}", mount_point.display());
        }
    }
}
