use crate::error::ErrorKind;
use rust_embed::RustEmbed;
use serde_derive::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
}

impl Version {
    pub fn from_client_exe(version: &str) -> Version {
        let parts: Vec<_> = version.split(",").collect();
        assert!(parts.len() == 4);
        Version {
            major: parts[0].parse::<u32>().unwrap(),
            minor: parts[1].parse::<u32>().unwrap(),
            patch: parts[2].parse::<u32>().unwrap(),
            build: parts[3].parse::<u32>().unwrap(),
        }
    }

    pub fn to_path(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn is_at_least(&self, other: &Version) -> bool {
        if self.major > other.major {
            true
        } else if self.major < other.major {
            false
        } else if self.minor > other.minor {
            true
        } else if self.minor < other.minor {
            false
        } else if self.patch >= other.patch {
            true
        } else {
            false
        }
    }
}

#[derive(RustEmbed)]
#[folder = "../addons/"]
struct Embedded;

pub struct Datafiles {
    base_path: PathBuf,
}

impl Datafiles {
    pub fn new(base_path: PathBuf) -> Result<Datafiles, ErrorKind> {
        Ok(Datafiles {
            base_path
        })
    }

    pub fn get(&self, path: &str) -> Result<Cow<'static, [u8]>, ErrorKind> {
        let mut p = self.base_path.clone();
        p.push(path);
        if !p.exists() {
            if let Some(x) = Embedded::get(&path) {
                return Ok(x.data);
            }
            return Err(ErrorKind::DatafileNotFound {
                path: path.to_string(),
            });
        }
        Ok(Cow::from(std::fs::read(p).unwrap()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_older_newer(older: Version, newer: Version) {
        assert!(newer.is_at_least(&older));
        assert!(newer.is_at_least(&newer));
        assert!(!older.is_at_least(&newer));
    }

    #[test]
    fn different_patch() {
        let older = Version::from_client_exe("0,10,9,0");
        let newer = Version::from_client_exe("0,10,10,0");
        assert_older_newer(older, newer);
    }

    #[test]
    fn different_minor() {
        let older = Version::from_client_exe("0,10,9,0");
        let newer = Version::from_client_exe("0,11,0,0");
        assert_older_newer(older, newer);
    }

    #[test]
    fn different_major() {
        let older = Version::from_client_exe("0,11,5,0");
        let newer = Version::from_client_exe("1,0,0,0");
        assert_older_newer(older, newer);
    }
}
