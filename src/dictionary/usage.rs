use std::{convert::Infallible, fmt::Display, str::FromStr};

/// The intended usage of the dictionary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictionaryUsage {
    /// Default value
    #[default]
    Unknown = 0,
    /// A built-in dictionary
    BuiltIn = 1,
    /// A contributed extension
    Extension = 2,
    /// A local custom dictionary
    Custom = 3,
    /// A user dictionary
    ///
    /// This usage type may be used to identify whether a dictionary
    /// should be opened as read/write and as user dictionary by the
    /// Layered virtual dictionary.
    User = 4,
    /// A exclusion user dictionary
    ///
    /// This usage type may be used to identify whether a dictionary
    /// should be opened as read/write and as the exclusion dictionary
    /// by the Layered virtual dictionary.
    ExcludeList = 5,
}

impl From<u8> for DictionaryUsage {
    fn from(value: u8) -> Self {
        match value {
            1 => DictionaryUsage::BuiltIn,
            2 => DictionaryUsage::Extension,
            3 => DictionaryUsage::Custom,
            4 => DictionaryUsage::User,
            5 => DictionaryUsage::ExcludeList,
            _ => DictionaryUsage::Unknown,
        }
    }
}

impl Display for DictionaryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictionaryUsage::Unknown => f.write_str("unknown"),
            DictionaryUsage::BuiltIn => f.write_str("built-in"),
            DictionaryUsage::Extension => f.write_str("extension"),
            DictionaryUsage::Custom => f.write_str("custom"),
            DictionaryUsage::User => f.write_str("user"),
            DictionaryUsage::ExcludeList => f.write_str("exclude-list"),
        }
    }
}

impl FromStr for DictionaryUsage {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "unknown" => DictionaryUsage::Unknown,
            "built-in" => DictionaryUsage::BuiltIn,
            "extension" => DictionaryUsage::Extension,
            "custom" => DictionaryUsage::Custom,
            "user" => DictionaryUsage::User,
            "exclude-list" => DictionaryUsage::ExcludeList,
            _ => DictionaryUsage::Unknown,
        })
    }
}
