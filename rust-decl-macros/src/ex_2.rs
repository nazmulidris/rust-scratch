/*
 *   Copyright (c) 2025 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use crate::Color;

#[derive(Debug, PartialEq, Default)]
pub struct Style {
    pub bold: Option<Bold>,
    pub italic: Option<Italic>,
    pub underline: Option<Underline>,
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
}

#[derive(Debug, PartialEq)]
pub struct Bold(pub bool);
#[derive(Debug, PartialEq)]
pub struct Italic(pub bool);
#[derive(Debug, PartialEq)]
pub struct Underline(pub bool);

/// Main macro for setting up a new style.
#[macro_export]
macro_rules! new_style {
    ($($all:tt)*) => {{
        #[allow(unused_mut)]
        let mut style = $crate::Style::default();
        $crate::apply_style!(style, $($all)*);
        style
    }};
}

/// Helper macro that's called recusively, to apply various attributes to the style
#[macro_export]
macro_rules! apply_style {
    ($style:ident, bold $($rem:tt)*) => {{
        $style.bold = Some($crate::Bold(true));
        $crate::apply_style!($style, $($rem)*);
    }};

    ($style:ident, italic $($rem:tt)*) => {{
        $style.italic = Some($crate::Italic(true));
        $crate::apply_style!($style, $($rem)*);
    }};

    ($style:ident, underline $($rem:tt)*) => {{
        $style.underline = Some($crate::Underline(true));
        $crate::apply_style!($style, $($rem)*);
    }};

    ($style:ident, fg_color: $c:block $($rem:tt)*) => {{
        $style.fg_color = Some($c);
        $crate::apply_style!($style, $($rem)*);
    }};

    ($style:ident, bg_color: $c:block $($rem:tt)*) => {{
        $style.bg_color = Some($c);
        $crate::apply_style!($style, $($rem)*);
    }};

    // Base case to stop recursing, when no more tokens are left.
    ($style:ident,) => {};
}

#[cfg(test)]
mod tests {
    use crate::{Bold, Italic, Underline, color};

    #[test]
    fn expected_syntax() {
        let pink = color!(pink);
        let lizard_green = color!(lizard_green);

        {
            let style = new_style!(
                bold italic underline fg_color: {pink} bg_color: {lizard_green}
            );
            assert_eq!(style.bold, Some(Bold(true)));
            assert_eq!(style.italic, Some(Italic(true)));
            assert_eq!(style.underline, Some(Underline(true)));
            assert_eq!(style.fg_color, Some(pink));
            assert_eq!(style.bg_color, Some(lizard_green));
        }

        {
            let style = new_style!(
                bold fg_color: {pink}
            );
            assert_eq!(style.bold, Some(Bold(true)));
            assert_eq!(style.italic, None);
            assert_eq!(style.underline, None);
            assert_eq!(style.fg_color, Some(pink));
            assert_eq!(style.bg_color, None);
        }
    }
}
