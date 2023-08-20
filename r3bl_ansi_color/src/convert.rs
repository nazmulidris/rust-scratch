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

//! More info:
//! - <https://tintin.mudhalla.net/info/256color/>
//! - <https://github.com/Qix-/color-convert/>
//! - <https://talyian.github.io/ansicolors/>

use crate::{Ansi256Color, RgbColor};

pub fn convert_rgb_into_ansi256(rgb_color: RgbColor) -> Ansi256Color {
    let gray_index = ANSI256_FROM_GRAY[luminance(rgb_color) as usize];
    let gray_distance = {
        let gray_color: RgbColor = ANSI_COLOR_PALETTE[gray_index as usize].into();
        distance(rgb_color, gray_color)
    };

    let (cube_index, cube_rgb) = cube_index(rgb_color);

    if distance(rgb_color, cube_rgb) < gray_distance {
        Ansi256Color { index: cube_index }
    } else {
        Ansi256Color { index: gray_index }
    }
}

mod cube_index_impl {
    use crate::RgbColor;

    pub fn cube_index(rgb_color: RgbColor) -> (/* cube_index */ u8, /* cube_rgb */ RgbColor) {
        let RgbColor {
            red: r,
            green: g,
            blue: b,
        } = rgb_color;
        let r = cube_index_red(r);
        let g = cube_index_green(g);
        let b = cube_index_blue(b);

        let cube_index = r.0 + g.0 + b.0;
        let cube_rgb = r.1 + g.1 + b.1;
        let cube_rgb: RgbColor = cube_rgb.into();

        return (cube_index, cube_rgb);

        /// These functions approximate a pure by a color in the 6×6×6 color cube. For
        /// example, `cube_index_red(r)` approximates an `rgb(r, 0, 0)` color.
        mod inner {
            pub fn cube_index_red(v: u8) -> (u8, u32) {
                let (i, v) = cube_thresholds(v, 38, 115, 155, 196, 235);
                (i * 36 + 16, v << 16)
            }

            pub fn cube_index_green(v: u8) -> (u8, u32) {
                let (i, v) = cube_thresholds(v, 36, 116, 154, 195, 235);
                (i * 6, v << 8)
            }

            pub fn cube_index_blue(v: u8) -> (u8, u32) {
                cube_thresholds(v, 35, 115, 155, 195, 235)
            }

            #[rustfmt::skip]
            pub fn cube_thresholds(v: u8, a: u8, b: u8, c: u8, d: u8, e: u8) -> (u8, u32) {
                if      v < a { (0,   0) }
                else if v < b { (1,  95) }
                else if v < c { (2, 135) }
                else if v < d { (3, 175) }
                else if v < e { (4, 215) }
                else          { (5, 255) }
            }
        }
        pub use inner::*;
    }

    /// Returns luminance of given RGB color.
    pub fn luminance(rgb: RgbColor) -> u8 {
        let RgbColor {
            red: r,
            green: g,
            blue: b,
        } = rgb;
        let number = r as f32 * r as f32 * 0.2126729f32
            + g as f32 * g as f32 * 0.7151521f32
            + b as f32 * b as f32 * 0.0721750f32;
        number.sqrt() as u8
    }

    /// Calculates distance between two colors `d(x, x) = 0` and `d(x, y) < d(x, z)` implies
    /// `x` being closer to `y` than to `z`.
    /// - More info: <https://www.compuphase.com/cmetric.htm>.
    pub fn distance(this: RgbColor, other: RgbColor) -> u32 {
        let RgbColor {
            red: x_red,
            green: x_green,
            blue: x_blue,
        } = this;
        let RgbColor {
            red: y_red,
            green: y_green,
            blue: y_blue,
        } = other;

        let red_sum = (x_red as i32) + (y_red as i32);
        let red = (x_red as i32) - (y_red as i32);
        let green = (x_green as i32) - (y_green as i32);
        let blue = (x_blue as i32) - (y_blue as i32);
        let distance =
            (1024 + red_sum) * red * red + 2048 * green * green + (1534 - red_sum) * blue * blue;
        distance as u32
    }
}
pub use cube_index_impl::*;

use self::constants::{ANSI256_FROM_GRAY, ANSI_COLOR_PALETTE};

mod convert_between_rgb_and_u32 {
    use crate::RgbColor;

    impl From<RgbColor> for u32 {
        fn from(rgb: RgbColor) -> Self {
            let RgbColor {
                red: r,
                green: g,
                blue: b,
            } = rgb;
            ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
        }
    }

    impl From<u32> for RgbColor {
        fn from(rgb: u32) -> Self {
            RgbColor {
                red: (rgb >> 16) as u8,
                green: (rgb >> 8) as u8,
                blue: rgb as u8,
            }
        }
    }
}

pub mod constants {
    /// Lookup table for approximate of shades of gray.
    #[rustfmt::skip]
    pub static ANSI256_FROM_GRAY: [u8; 256] = [
        16,  16,  16,  16,  16, 232, 232, 232,
        232, 232, 232, 232, 232, 232, 233, 233,
        233, 233, 233, 233, 233, 233, 233, 233,
        234, 234, 234, 234, 234, 234, 234, 234,
        234, 234, 235, 235, 235, 235, 235, 235,
        235, 235, 235, 235, 236, 236, 236, 236,
        236, 236, 236, 236, 236, 236, 237, 237,
        237, 237, 237, 237, 237, 237, 237, 237,
        238, 238, 238, 238, 238, 238, 238, 238,
        238, 238, 239, 239, 239, 239, 239, 239,
        239, 239, 239, 239, 240, 240, 240, 240,
        240, 240, 240, 240,  59,  59,  59,  59,
        59, 241, 241, 241, 241, 241, 241, 241,
        242, 242, 242, 242, 242, 242, 242, 242,
        242, 242, 243, 243, 243, 243, 243, 243,
        243, 243, 243, 244, 244, 244, 244, 244,
        244, 244, 244, 244, 102, 102, 102, 102,
        102, 245, 245, 245, 245, 245, 245, 246,
        246, 246, 246, 246, 246, 246, 246, 246,
        246, 247, 247, 247, 247, 247, 247, 247,
        247, 247, 247, 248, 248, 248, 248, 248,
        248, 248, 248, 248, 145, 145, 145, 145,
        145, 249, 249, 249, 249, 249, 249, 250,
        250, 250, 250, 250, 250, 250, 250, 250,
        250, 251, 251, 251, 251, 251, 251, 251,
        251, 251, 251, 252, 252, 252, 252, 252,
        252, 252, 252, 252, 188, 188, 188, 188,
        188, 253, 253, 253, 253, 253, 253, 254,
        254, 254, 254, 254, 254, 254, 254, 254,
        254, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 231,
        231, 231, 231, 231, 231, 231, 231, 231,
    ];

    /// ANSI Color Palette.
    #[rustfmt::skip]
    pub static ANSI_COLOR_PALETTE: [u32; 256] = [
        // The 16 system colors as used by default by xterm.
        0x000000, 0xcd0000, 0x00cd00, 0xcdcd00,
        0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5,
        0x7f7f7f, 0xff0000, 0x00ff00, 0xffff00,
        0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,

        // 6×6×6 cube.
        // On each axis, the six indices map to [0, 95, 135, 175, 215, 255] RGB component
        // values.
        0x000000, 0x00005f, 0x000087, 0x0000af,
        0x0000d7, 0x0000ff, 0x005f00, 0x005f5f,
        0x005f87, 0x005faf, 0x005fd7, 0x005fff,
        0x008700, 0x00875f, 0x008787, 0x0087af,
        0x0087d7, 0x0087ff, 0x00af00, 0x00af5f,
        0x00af87, 0x00afaf, 0x00afd7, 0x00afff,
        0x00d700, 0x00d75f, 0x00d787, 0x00d7af,
        0x00d7d7, 0x00d7ff, 0x00ff00, 0x00ff5f,
        0x00ff87, 0x00ffaf, 0x00ffd7, 0x00ffff,
        0x5f0000, 0x5f005f, 0x5f0087, 0x5f00af,
        0x5f00d7, 0x5f00ff, 0x5f5f00, 0x5f5f5f,
        0x5f5f87, 0x5f5faf, 0x5f5fd7, 0x5f5fff,
        0x5f8700, 0x5f875f, 0x5f8787, 0x5f87af,
        0x5f87d7, 0x5f87ff, 0x5faf00, 0x5faf5f,
        0x5faf87, 0x5fafaf, 0x5fafd7, 0x5fafff,
        0x5fd700, 0x5fd75f, 0x5fd787, 0x5fd7af,
        0x5fd7d7, 0x5fd7ff, 0x5fff00, 0x5fff5f,
        0x5fff87, 0x5fffaf, 0x5fffd7, 0x5fffff,
        0x870000, 0x87005f, 0x870087, 0x8700af,
        0x8700d7, 0x8700ff, 0x875f00, 0x875f5f,
        0x875f87, 0x875faf, 0x875fd7, 0x875fff,
        0x878700, 0x87875f, 0x878787, 0x8787af,
        0x8787d7, 0x8787ff, 0x87af00, 0x87af5f,
        0x87af87, 0x87afaf, 0x87afd7, 0x87afff,
        0x87d700, 0x87d75f, 0x87d787, 0x87d7af,
        0x87d7d7, 0x87d7ff, 0x87ff00, 0x87ff5f,
        0x87ff87, 0x87ffaf, 0x87ffd7, 0x87ffff,
        0xaf0000, 0xaf005f, 0xaf0087, 0xaf00af,
        0xaf00d7, 0xaf00ff, 0xaf5f00, 0xaf5f5f,
        0xaf5f87, 0xaf5faf, 0xaf5fd7, 0xaf5fff,
        0xaf8700, 0xaf875f, 0xaf8787, 0xaf87af,
        0xaf87d7, 0xaf87ff, 0xafaf00, 0xafaf5f,
        0xafaf87, 0xafafaf, 0xafafd7, 0xafafff,
        0xafd700, 0xafd75f, 0xafd787, 0xafd7af,
        0xafd7d7, 0xafd7ff, 0xafff00, 0xafff5f,
        0xafff87, 0xafffaf, 0xafffd7, 0xafffff,
        0xd70000, 0xd7005f, 0xd70087, 0xd700af,
        0xd700d7, 0xd700ff, 0xd75f00, 0xd75f5f,
        0xd75f87, 0xd75faf, 0xd75fd7, 0xd75fff,
        0xd78700, 0xd7875f, 0xd78787, 0xd787af,
        0xd787d7, 0xd787ff, 0xd7af00, 0xd7af5f,
        0xd7af87, 0xd7afaf, 0xd7afd7, 0xd7afff,
        0xd7d700, 0xd7d75f, 0xd7d787, 0xd7d7af,
        0xd7d7d7, 0xd7d7ff, 0xd7ff00, 0xd7ff5f,
        0xd7ff87, 0xd7ffaf, 0xd7ffd7, 0xd7ffff,
        0xff0000, 0xff005f, 0xff0087, 0xff00af,
        0xff00d7, 0xff00ff, 0xff5f00, 0xff5f5f,
        0xff5f87, 0xff5faf, 0xff5fd7, 0xff5fff,
        0xff8700, 0xff875f, 0xff8787, 0xff87af,
        0xff87d7, 0xff87ff, 0xffaf00, 0xffaf5f,
        0xffaf87, 0xffafaf, 0xffafd7, 0xffafff,
        0xffd700, 0xffd75f, 0xffd787, 0xffd7af,
        0xffd7d7, 0xffd7ff, 0xffff00, 0xffff5f,
        0xffff87, 0xffffaf, 0xffffd7, 0xffffff,

        // Grayscale ramp. This is calculated as (index - 232) * 10 + 8 and repeated for each
        // RGB component.
        0x080808, 0x121212, 0x1c1c1c, 0x262626,
        0x303030, 0x3a3a3a, 0x444444, 0x4e4e4e,
        0x585858, 0x626262, 0x6c6c6c, 0x767676,
        0x808080, 0x8a8a8a, 0x949494, 0x9e9e9e,
        0xa8a8a8, 0xb2b2b2, 0xbcbcbc, 0xc6c6c6,
        0xd0d0d0, 0xdadada, 0xe4e4e4, 0xeeeeee,
    ];
}

#[cfg(test)]
mod tests {
    use crate::TransformColor;
    use crate::{Ansi256Color, RgbColor};
    use pretty_assertions::assert_eq;

    #[test]
    fn convert_ansi265_into_rgb() {
        assert_eq!(
            Ansi256Color { index: 16 }.as_rgb(),
            RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }
        );

        assert_eq!(
            Ansi256Color { index: 67 }.as_rgb(),
            RgbColor {
                red: 95,
                green: 135,
                blue: 175,
            }
        );

        assert_eq!(
            Ansi256Color { index: 150 }.as_rgb(),
            RgbColor {
                red: 175,
                green: 215,
                blue: 135
            }
        );

        assert_eq!(
            Ansi256Color { index: 231 }.as_rgb(),
            RgbColor {
                red: 255,
                green: 255,
                blue: 255,
            }
        );

        assert_eq!(
            Ansi256Color { index: 255 }.as_rgb(),
            RgbColor {
                red: 238,
                green: 238,
                blue: 238,
            }
        );
    }

    #[test]
    fn test_convert_rgb_into_ansi256() {
        use crate::TransformColor;

        assert_eq!(
            RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }
            .as_ansi256()
            .index,
            16
        );

        assert_eq!(
            RgbColor {
                red: 1,
                green: 1,
                blue: 1,
            }
            .as_ansi256()
            .index,
            16
        );

        assert_eq!(
            RgbColor {
                red: 0,
                green: 1,
                blue: 2,
            }
            .as_ansi256()
            .index,
            16
        );

        assert_eq!(
            RgbColor {
                red: 95,
                green: 135,
                blue: 175,
            }
            .as_ansi256()
            .index,
            67
        );

        assert_eq!(
            RgbColor {
                red: 255,
                green: 255,
                blue: 255,
            }
            .as_ansi256()
            .index,
            231
        );
    }
}
