//! Type-safe wrappers for domain identifiers.
//!
//! This module provides newtype wrappers to prevent mixing up different kinds
//! of identifiers (file names, configuration names, IDs, etc.) and make the API
//! more type-safe and self-documenting.

use std::fmt;
use std::ops::Deref;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Metadata associated with a filler model.
///
/// Only `transformations` is meaningful to the build itself; every other key in
/// the file is arbitrary, project-defined metadata. This type is used for
/// *serialisation* (e.g. by `migrate`); loading parses files generically into a
/// free-form map so that any user-defined field is preserved (see
/// `ProjectManager::filler_metadata`).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FillerMetadata {
    pub transformations: Option<IndexMap<EnvelopeName, Option<String>>>,
}

/// The reserved metadata key that carries per-envelope transformations. Every
/// other key in a filler's `.metadata` file is treated as free-form metadata.
pub const TRANSFORMATIONS_KEY: &str = "transformations";

/// Defines a string-backed newtype with the standard `new`, `Deref<Target =
/// str>`, `Display`, `From<String>` and `From<&str>` impls shared by every
/// identifier type below. Extra derives (e.g. `Ord`) can be appended after
/// the type name.
macro_rules! string_newtype {
    ($(#[$meta:meta])* $name:ident $(, derive($($extra:ident),+ $(,)?))?) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize $(, $($extra),+)?)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "`.")]
            pub fn new(name: impl Into<String>) -> Self {
                Self(name.into())
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.into())
            }
        }
    };
}

string_newtype!(
    /// File stem name (filename without extension).
    ///
    /// Example: `FileName::new("vacuum_vessel")`
    FileName
);

string_newtype!(
    /// Filler model name.
    ///
    /// Example: `FillerName::new("universe_101")`
    FillerName,
    derive(PartialOrd, Ord)
);

string_newtype!(
    /// Envelope name.
    ///
    /// Example: `EnvelopeName::new("main_vessel")`
    EnvelopeName
);

/// A filler's file is always named `<filler_name>.mcnp` by convention — the
/// filler name *is* the file stem — so this conversion is exact, not a lookup.
impl From<&FillerName> for FileName {
    fn from(filler_name: &FillerName) -> Self {
        Self(filler_name.0.clone())
    }
}

/// The inverse of `FileName::from(&FillerName)` — see its doc comment. Only
/// meaningful for a `FileName` that is known to name a filler.
impl From<&FileName> for FillerName {
    fn from(file_name: &FileName) -> Self {
        Self(file_name.0.clone())
    }
}

/// Universe ID in MCNP models.
///
/// Example: `UniverseId::new(101)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct UniverseId(u32);

impl UniverseId {
    /// Creates a new universe ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for UniverseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for UniverseId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_id() {
        let id = UniverseId::new(101);
        assert_eq!(id.to_string(), "101");
    }

    #[test]
    fn test_newtype_prevents_mixing() {
        let filler = FillerName::new("universe_101");
        let envelope = EnvelopeName::new("universe_101");

        // This would not compile if we tried to compare them:
        // assert_eq!(filler, envelope); // Type error!

        // But we can compare same types:
        assert_eq!(filler, FillerName::new("universe_101"));
        assert_eq!(envelope, EnvelopeName::new("universe_101"));
    }

    #[test]
    fn test_from_conversions() {
        // Test FillerName -> FileName
        let filler = FillerName::new("universe_101");
        let file: FileName = (&filler).into();
        assert_eq!(&*file, "universe_101");

        // Test &str -> FileName
        let file: FileName = "test".into();
        assert_eq!(&*file, "test");
    }
}
