//! Type-safe wrappers for domain identifiers.
//!
//! This module provides newtype wrappers to prevent mixing up different kinds
//! of identifiers (file names, configuration names, IDs, etc.) and make the API
//! more type-safe and self-documenting.

use std::fmt;
use std::ops::Deref;

use indexmap::IndexMap;
use migjorn::Model;
use serde::{Deserialize, Serialize};

/// Metadata associated with a filler model, describing how it should be placed.
#[derive(Debug, Serialize, Deserialize)]
pub struct FillerMetadata {
    pub transformations: Option<IndexMap<EnvelopeName, Option<String>>>,
}

/// File stem name (filename without extension).
///
/// Example: `FileName::new("vacuum_vessel")`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FileName(String);

impl FileName {
    /// Creates a new file name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Deref for FileName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FileName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FileName {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&FillerName> for FileName {
    fn from(filler_name: &FillerName) -> Self {
        Self(filler_name.0.clone())
    }
}

/// Filler model name.
///
/// Example: `FillerName::new("universe_101")`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FillerName(String);

impl FillerName {
    /// Creates a new filler name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Deref for FillerName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FillerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FillerName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FillerName {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&FileName> for FillerName {
    fn from(file_name: &FileName) -> Self {
        Self(file_name.0.clone())
    }
}

impl From<&Model> for FillerName {
    fn from(model: &Model) -> Self {
        model
            .path
            .file_stem()
            .map(|s| FillerName::new(s.to_string_lossy().to_string()))
            .unwrap_or_else(|| FillerName::new("<invalid path>".to_string()))
    }
}

/// Envelope name.
///
/// Example: `EnvelopeName::new("main_vessel")`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct EnvelopeName(String);

impl EnvelopeName {
    /// Creates a new envelope name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Deref for EnvelopeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for EnvelopeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EnvelopeName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EnvelopeName {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

/// Universe ID in MCNP models.
///
/// Example: `UniverseId::new(101)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// Cell ID in MCNP models.
///
/// Example: `CellId::new(10001)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId(u32);

impl CellId {
    /// Creates a new cell ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for CellId {
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
    fn test_cell_id_ordering() {
        let id1 = CellId::new(100);
        let id2 = CellId::new(200);
        assert!(id1 < id2);
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
