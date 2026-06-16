#![no_std]
#![forbid(unsafe_code)]

pub use skrifheim_audit as audit;
pub use skrifheim_core as core;
pub use skrifheim_crypto as crypto;
pub use skrifheim_fact as fact;
pub use skrifheim_policy as policy;
pub use skrifheim_query as query;
pub use skrifheim_storage as storage;
pub use skrifheim_world as world;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildInfo {
    pub database_name: &'static str,
    pub version: &'static str,
}

#[must_use]
pub const fn build_info() -> BuildInfo {
    BuildInfo {
        database_name: "skrifheim",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_uses_required_database_name() {
        assert_eq!(build_info().database_name, "skrifheim");
    }
}
