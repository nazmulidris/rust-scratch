//! # 3+ Token Constructor DSL with Full Algebraic Completeness
//!
//! This module demonstrates the **Elegant Constructor DSL Pattern** when an entity has
//! 3 or more configuration tokens that can be composed and chained:
//! 1. [`Bold`]: Unit struct token enabling bold style.
//! 2. [`Italic`]: Unit struct token enabling italic style.
//! 3. [`Dim`]: Unit struct token enabling dim style.
//!
//! ## Key Insights: Why `Config + Token` and `Token + Config` are Essential Here
//!
//! Rust's binary addition operator `+` is left-associative. When writing:
//!
//! ```rust,ignore
//! let style = Bold + Italic + Dim;
//! ```
//!
//! Rust evaluates the expression in two stages:
//! 1. `(Bold + Italic)`: Evaluates first using `Add<Italic> for Bold`, producing a [`StyleConfig`].
//! 2. `(StyleConfig) + Dim`: Evaluates second. Because the left side is now [`StyleConfig`],
//!    Rust requires an implementation of `Add<Dim> for StyleConfig` (`Config + Token`).
//!
//! If parenthesized explicitly on the right:
//!
//! ```rust,ignore
//! let style = Bold + (Italic + Dim);
//! ```
//!
//! `(Italic + Dim)` produces a [`StyleConfig`], making the addition `Bold + StyleConfig` (`Token + Config`).
//!
//! Implementing both ensures full algebraic completeness across arbitrary chains and groupings.
//!
//! # Examples
//!
//! ```rust
//! use elegant_constructor_dsl::three_tokens::{Bold, Dim, Italic, Style, StyleConfig};
//!
//! // 1. Unit type `()`: default empty style.
//! let style = Style::new(());
//! assert!(!style.bold && !style.italic && !style.dim);
//!
//! // 2. Single token: only one attribute enabled.
//! let style = Style::new(Bold);
//! assert!(style.bold && !style.italic && !style.dim);
//!
//! // 3. 2 tokens: combined with `+`.
//! let style_a = Style::new(Bold + Italic);
//! let style_b = Style::new(Italic + Bold);
//! assert_eq!(style_a, style_b);
//!
//! // 4. 3 tokens: chained in any order.
//! let style_1 = Style::new(Bold + Italic + Dim);
//! let style_2 = Style::new(Dim + Bold + Italic);
//! let style_3 = Style::new(Italic + Dim + Bold);
//! assert_eq!(style_1, style_2);
//! assert_eq!(style_2, style_3);
//!
//! // 5. Mutation with `+=`.
//! let mut config = StyleConfig::from(Bold);
//! config += Italic;
//! config += Dim;
//! assert_eq!(Style::new(config), style_1);
//! ```

use std::ops::{Add, AddAssign};

/// Canonical configuration struct holding resolved style attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StyleConfig {
    pub bold: Option<Bold>,
    pub italic: Option<Italic>,
    pub dim: Option<Dim>,
}

/// Unit struct token for bold styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bold;

/// Unit struct token for italic styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Italic;

/// Unit struct token for dim styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dim;

/// Target struct representing the final style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
}

impl Style {
    /// Creates a new [`Style`] from a unit `()`, individual tokens, or chained tokens.
    #[must_use]
    pub fn new(arg_config: impl Into<StyleConfig>) -> Self {
        let config: StyleConfig = arg_config.into();
        Self {
            bold: config.bold.is_some(),
            italic: config.italic.is_some(),
            dim: config.dim.is_some(),
        }
    }
}

// -----------------------------------------------------------------------------
// DSL Implementations
// -----------------------------------------------------------------------------

// 1. From conversions for unit `()` and individual tokens.

impl From<()> for StyleConfig {
    fn from((): ()) -> Self {
        Self::default()
    }
}

impl From<Bold> for StyleConfig {
    fn from(val: Bold) -> Self {
        Self {
            bold: Some(val),
            ..Default::default()
        }
    }
}

impl From<Italic> for StyleConfig {
    fn from(val: Italic) -> Self {
        Self {
            italic: Some(val),
            ..Default::default()
        }
    }
}

impl From<Dim> for StyleConfig {
    fn from(val: Dim) -> Self {
        Self {
            dim: Some(val),
            ..Default::default()
        }
    }
}

// 2. Config + Config.

impl Add for StyleConfig {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            bold: self.bold.or(rhs.bold),
            italic: self.italic.or(rhs.italic),
            dim: self.dim.or(rhs.dim),
        }
    }
}

// 3. Pairwise Token + Token additions.

impl Add<Italic> for Bold {
    type Output = StyleConfig;
    fn add(self, rhs: Italic) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

impl Add<Dim> for Bold {
    type Output = StyleConfig;
    fn add(self, rhs: Dim) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

impl Add<Bold> for Italic {
    type Output = StyleConfig;
    fn add(self, rhs: Bold) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

impl Add<Dim> for Italic {
    type Output = StyleConfig;
    fn add(self, rhs: Dim) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

impl Add<Bold> for Dim {
    type Output = StyleConfig;
    fn add(self, rhs: Bold) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

impl Add<Italic> for Dim {
    type Output = StyleConfig;
    fn add(self, rhs: Italic) -> Self::Output {
        StyleConfig::from(self) + StyleConfig::from(rhs)
    }
}

// 4. Config + Token (required for left-associative chaining: (A + B) + C).

impl Add<Bold> for StyleConfig {
    type Output = StyleConfig;
    fn add(mut self, rhs: Bold) -> Self::Output {
        self.bold = Some(rhs);
        self
    }
}

impl Add<Italic> for StyleConfig {
    type Output = StyleConfig;
    fn add(mut self, rhs: Italic) -> Self::Output {
        self.italic = Some(rhs);
        self
    }
}

impl Add<Dim> for StyleConfig {
    type Output = StyleConfig;
    fn add(mut self, rhs: Dim) -> Self::Output {
        self.dim = Some(rhs);
        self
    }
}

// 5. Token + Config (required for right-associative grouping: A + (B + C)).

impl Add<StyleConfig> for Bold {
    type Output = StyleConfig;
    fn add(self, mut rhs: StyleConfig) -> Self::Output {
        rhs.bold = Some(self);
        rhs
    }
}

impl Add<StyleConfig> for Italic {
    type Output = StyleConfig;
    fn add(self, mut rhs: StyleConfig) -> Self::Output {
        rhs.italic = Some(self);
        rhs
    }
}

impl Add<StyleConfig> for Dim {
    type Output = StyleConfig;
    fn add(self, mut rhs: StyleConfig) -> Self::Output {
        rhs.dim = Some(self);
        rhs
    }
}

// 6. In-place mutation with += (AddAssign).

impl AddAssign<Bold> for StyleConfig {
    fn add_assign(&mut self, rhs: Bold) {
        self.bold = Some(rhs);
    }
}

impl AddAssign<Italic> for StyleConfig {
    fn add_assign(&mut self, rhs: Italic) {
        self.italic = Some(rhs);
    }
}

impl AddAssign<Dim> for StyleConfig {
    fn add_assign(&mut self, rhs: Dim) {
        self.dim = Some(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_and_default_style() {
        let style = Style::new(());
        assert_eq!(
            style,
            Style {
                bold: false,
                italic: false,
                dim: false,
            }
        );
    }

    #[test]
    fn test_single_token_styles() {
        assert_eq!(
            Style::new(Bold),
            Style {
                bold: true,
                italic: false,
                dim: false,
            }
        );
        assert_eq!(
            Style::new(Italic),
            Style {
                bold: false,
                italic: true,
                dim: false,
            }
        );
        assert_eq!(
            Style::new(Dim),
            Style {
                bold: false,
                italic: false,
                dim: true,
            }
        );
    }

    #[test]
    fn test_two_token_commutative_styles() {
        let bold_italic_a = Bold + Italic;
        let bold_italic_b = Italic + Bold;
        assert_eq!(bold_italic_a, bold_italic_b);

        let style_a = Style::new(bold_italic_a);
        let style_b = Style::new(bold_italic_b);
        assert_eq!(
            style_a,
            Style {
                bold: true,
                italic: true,
                dim: false,
            }
        );
        assert_eq!(style_a, style_b);
    }

    #[test]
    fn test_three_token_chaining_and_permutations() {
        // Left-associative chains: ((A + B) + C) -> Config + Token.
        let chain_1 = Bold + Italic + Dim;
        let chain_2 = Dim + Bold + Italic;
        let chain_3 = Italic + Dim + Bold;

        let expected_config = StyleConfig {
            bold: Some(Bold),
            italic: Some(Italic),
            dim: Some(Dim),
        };

        assert_eq!(chain_1, expected_config);
        assert_eq!(chain_2, expected_config);
        assert_eq!(chain_3, expected_config);

        let style_1 = Style::new(chain_1);
        let style_2 = Style::new(chain_2);
        let style_3 = Style::new(chain_3);

        let expected_style = Style {
            bold: true,
            italic: true,
            dim: true,
        };

        assert_eq!(style_1, expected_style);
        assert_eq!(style_2, expected_style);
        assert_eq!(style_3, expected_style);

        // Right-associative grouped chain: A + (B + C) -> Token + Config.
        let grouped = Bold + (Italic + Dim);
        assert_eq!(grouped, expected_config);
        assert_eq!(Style::new(grouped), expected_style);
    }

    #[test]
    fn test_in_place_add_assign() {
        let mut config = StyleConfig::default();
        config += Bold;
        config += Italic;
        config += Dim;

        assert_eq!(
            config,
            StyleConfig {
                bold: Some(Bold),
                italic: Some(Italic),
                dim: Some(Dim),
            }
        );
        assert_eq!(
            Style::new(config),
            Style {
                bold: true,
                italic: true,
                dim: true,
            }
        );
    }
}
