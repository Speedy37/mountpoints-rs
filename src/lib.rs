#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{mountpaths, Error};
#[cfg(target_os = "macos")]
pub use macos::{mountpaths, Error};
#[cfg(target_os = "windows")]
pub use windows::{mountpaths, Error};

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for mountpath in mountpaths().unwrap() {
            eprintln!("{}", mountpath.display());
        }
    }
}
