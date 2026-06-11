use std::cmp::Ordering;

use crate::SqlitePool;
pub struct Data {
    pub pool: SqlitePool,
}
pub struct DBVersion {
    pub major: i8,
    pub minor: i8,
    pub patch: i8,
}

impl DBVersion {
    pub fn new(major: i8, minor: i8, patch: i8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
    pub fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}

impl PartialEq for DBVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl PartialOrd for DBVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major_cmp = other.major.cmp(&self.major);
        let minor_cmp = other.minor.cmp(&self.minor);
        let patch_cmp = other.patch.cmp(&self.patch);
        Some(match major_cmp {
            Ordering::Equal => match minor_cmp {
                Ordering::Equal => patch_cmp,
                _ => minor_cmp,
            },
            _ => major_cmp,
        })
    }
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
