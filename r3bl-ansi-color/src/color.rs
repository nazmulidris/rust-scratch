/*
 *   Copyright (c) 2023 R3BL LLC
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

pub trait ColorTransform {
    /// Returns a [RgbColor] representation of the `self` color.
    fn as_rgb(&self) -> RgbColor;

    /// Returns the index of a color in 256-color ANSI palette approximating the `self`
    /// color.
    fn as_ansi256(&self) -> Ansi256Color;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

mod rgb_color_impl {
    use crate::{ansi256_from_rgb, Ansi256Color};

    use super::ColorTransform;
    use super::RgbColor;

    impl ColorTransform for RgbColor {
        fn as_rgb(&self) -> RgbColor {
            *self
        }

        fn as_ansi256(&self) -> Ansi256Color {
            ansi256_from_rgb(*self)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ansi256Color {
    pub index: u8,
}

mod ansi_color_impl {
    use crate::{Ansi256Color, ColorTransform, RgbColor, ANSI_COLOR_PALETTE};

    impl ColorTransform for Ansi256Color {
        fn as_rgb(&self) -> RgbColor {
            let index = self.index as usize;
            ANSI_COLOR_PALETTE[index].into()
        }

        fn as_ansi256(&self) -> Ansi256Color {
            *self
        }
    }
}
