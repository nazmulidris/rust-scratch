//! # Minimal 2-Token Constructor DSL
//!
//! This module demonstrates the **Elegant Constructor DSL Pattern** when an entity needs
//! at most two optional configuration tokens:
//! 1. [`FileExtensionToken`]: Optional file extension (e.g. `"rs"` or `"md"`).
//! 2. [`FilePathToken`]: Optional file path (e.g. `"src/main.rs"`).
//!
//! ## Key Insights: Why `Config + Token` is Not Needed Here
//!
//! With only two tokens, callers can at most combine two tokens together:
//! `TokenA + TokenB` or `TokenB + TokenA`.
//!
//! Under the hood, both evaluate to `EditorBufferConfig::from(a) + EditorBufferConfig::from(b)`,
//! which relies exclusively on `Add<EditorBufferConfig> for EditorBufferConfig` (`Config + Config`).
//!
//! There is no 3rd token to chain, so implementing `Config + Token` or `Token + Config`
//! would be dead code. Adhering to YAGNI (You Aren't Gonna Need It) keeps the implementation
//! lean and minimal.
//!
//! # Examples
//!
//! ```rust
//! use elegant_constructor_dsl::two_tokens::{
//!     EditorBuffer, EditorBufferConfig, FileExtensionToken, FilePathToken,
//! };
//!
//! // 1. Unit type `()`: no extension, no file path.
//! let buffer = EditorBuffer::new_empty(());
//! assert_eq!(buffer.maybe_file_extension, None);
//! assert_eq!(buffer.maybe_file_path, None);
//!
//! // 2. Single token: file extension set, no file path.
//! let buffer = EditorBuffer::new_empty(FileExtensionToken("md"));
//! assert_eq!(buffer.maybe_file_extension, Some("md".to_string()));
//! assert_eq!(buffer.maybe_file_path, None);
//!
//! // 3. Single token: file path set, no extension.
//! let buffer = EditorBuffer::new_empty(FilePathToken("test.rs"));
//! assert_eq!(buffer.maybe_file_extension, None);
//! assert_eq!(buffer.maybe_file_path, Some("test.rs".to_string()));
//!
//! // 4. Binary `+` addition (either order): both extension and path set.
//! let buffer_a = EditorBuffer::new_empty(
//!     FileExtensionToken("rs") + FilePathToken("src/main.rs"),
//! );
//! let buffer_b = EditorBuffer::new_empty(
//!     FilePathToken("src/main.rs") + FileExtensionToken("rs"),
//! );
//! assert_eq!(buffer_a, buffer_b);
//! ```

use std::ops::Add;

/// Canonical configuration struct holding resolved configuration options.
///
/// This struct is a transient aggregator used during construction. It stores
/// `Option<&'a str>` directly instead of nesting newtypes, keeping memory and
/// field access simple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditorBufferConfig<'a> {
    pub maybe_file_extension: Option<&'a str>,
    pub maybe_file_path: Option<&'a str>,
}

/// Newtype constructor token representing a file extension (e.g. `"rs"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileExtensionToken<'a>(pub &'a str);

/// Newtype constructor token representing a file path (e.g. `"src/main.rs"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePathToken<'a>(pub &'a str);

/// Target struct that owns the final data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorBuffer {
    pub maybe_file_extension: Option<String>,
    pub maybe_file_path: Option<String>,
}

impl EditorBuffer {
    /// Creates a new, empty [`EditorBuffer`].
    ///
    /// Accepts unit `()`, [`FileExtensionToken`], [`FilePathToken`], `Token + Token`,
    /// or [`EditorBufferConfig`] directly.
    #[must_use]
    pub fn new_empty<'a>(arg_config: impl Into<EditorBufferConfig<'a>>) -> Self {
        let config: EditorBufferConfig<'a> = arg_config.into();
        Self {
            maybe_file_extension: config.maybe_file_extension.map(str::to_string),
            maybe_file_path: config.maybe_file_path.map(str::to_string),
        }
    }
}

// -----------------------------------------------------------------------------
// DSL Implementations
// -----------------------------------------------------------------------------

// 1. From conversions for unit `()` and single tokens.

impl From<()> for EditorBufferConfig<'_> {
    fn from((): ()) -> Self {
        Self {
            maybe_file_extension: None,
            maybe_file_path: None,
        }
    }
}

impl<'a> From<FileExtensionToken<'a>> for EditorBufferConfig<'a> {
    fn from(FileExtensionToken(ext): FileExtensionToken<'a>) -> Self {
        Self {
            maybe_file_extension: Some(ext),
            maybe_file_path: None,
        }
    }
}

impl<'a> From<FilePathToken<'a>> for EditorBufferConfig<'a> {
    fn from(FilePathToken(path): FilePathToken<'a>) -> Self {
        Self {
            maybe_file_extension: None,
            maybe_file_path: Some(path),
        }
    }
}

// 2. Token + Token (in either order).

impl<'a> Add<FilePathToken<'a>> for FileExtensionToken<'a> {
    type Output = EditorBufferConfig<'a>;

    fn add(self, rhs: FilePathToken<'a>) -> Self::Output {
        EditorBufferConfig::from(self) + EditorBufferConfig::from(rhs)
    }
}

impl<'a> Add<FileExtensionToken<'a>> for FilePathToken<'a> {
    type Output = EditorBufferConfig<'a>;

    fn add(self, rhs: FileExtensionToken<'a>) -> Self::Output {
        EditorBufferConfig::from(self) + EditorBufferConfig::from(rhs)
    }
}

// 3. Config + Config (merge two configs with Option::or).

impl Add for EditorBufferConfig<'_> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            maybe_file_extension: self.maybe_file_extension.or(rhs.maybe_file_extension),
            maybe_file_path: self.maybe_file_path.or(rhs.maybe_file_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_and_default_conversion() {
        let default_config = EditorBufferConfig::default();
        let unit_config: EditorBufferConfig = ().into();
        assert_eq!(unit_config, default_config);
        assert_eq!(unit_config.maybe_file_extension, None);
        assert_eq!(unit_config.maybe_file_path, None);

        let buffer = EditorBuffer::new_empty(());
        assert_eq!(buffer.maybe_file_extension, None);
        assert_eq!(buffer.maybe_file_path, None);
    }

    #[test]
    fn test_single_token_conversion() {
        let ext_config: EditorBufferConfig = FileExtensionToken("md").into();
        assert_eq!(
            ext_config,
            EditorBufferConfig {
                maybe_file_extension: Some("md"),
                maybe_file_path: None,
            }
        );

        let path_config: EditorBufferConfig = FilePathToken("test.rs").into();
        assert_eq!(
            path_config,
            EditorBufferConfig {
                maybe_file_extension: None,
                maybe_file_path: Some("test.rs"),
            }
        );
    }

    #[test]
    fn test_two_token_commutative_addition() {
        let combined_a = FileExtensionToken("rs") + FilePathToken("src/main.rs");
        let combined_b = FilePathToken("src/main.rs") + FileExtensionToken("rs");

        let expected = EditorBufferConfig {
            maybe_file_extension: Some("rs"),
            maybe_file_path: Some("src/main.rs"),
        };

        assert_eq!(combined_a, expected);
        assert_eq!(combined_b, expected);
        assert_eq!(combined_a, combined_b);

        let buffer_a = EditorBuffer::new_empty(combined_a);
        let buffer_b = EditorBuffer::new_empty(combined_b);
        assert_eq!(buffer_a, buffer_b);
        assert_eq!(buffer_a.maybe_file_extension, Some("rs".to_string()));
        assert_eq!(buffer_a.maybe_file_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_config_plus_config_merge() {
        let config_a: EditorBufferConfig = FileExtensionToken("rs").into();
        let config_b: EditorBufferConfig = FilePathToken("src/main.rs").into();

        let merged_ab = config_a + config_b;
        let merged_ba = config_b + config_a;

        let expected = EditorBufferConfig {
            maybe_file_extension: Some("rs"),
            maybe_file_path: Some("src/main.rs"),
        };

        assert_eq!(merged_ab, expected);
        assert_eq!(merged_ba, expected);
    }
}
