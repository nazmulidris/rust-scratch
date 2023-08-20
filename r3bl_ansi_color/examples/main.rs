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

use r3bl_ansi_color::*;

fn main() {
    {
        color_support_override_set(ColorSupportOverride::Ansi256);

        let eg_1 = FormattedString {
            text: "Hello",
            foreground: Color::Rgb(0, 0, 0),
            background: Color::Rgb(1, 1, 1),
        };
        println!("ansi256 override: eg_1: {0}", eg_1);

        let eg_2 = FormattedString {
            text: "World",
            foreground: Color::Ansi256(150),
            background: Color::Rgb(1, 1, 1),
        };
        println!("ansi256 override: eg_2: {0}", eg_2);
    }

    {
        color_support_override_set(ColorSupportOverride::Truecolor);

        let eg_1 = FormattedString {
            text: "Hello",
            foreground: Color::Rgb(0, 0, 0),
            background: Color::Rgb(1, 1, 1),
        };
        println!("truecolor override: eg_1: {0}", eg_1);

        let eg_2 = FormattedString {
            text: "World",
            foreground: Color::Ansi256(150),
            background: Color::Rgb(1, 1, 1),
        };
        println!("truecolor override: eg_2: {0}", eg_2);
    }

    {
        color_support_override_set(ColorSupportOverride::NotSet);

        let eg_1 = FormattedString {
            text: "Hello",
            foreground: Color::Rgb(0, 0, 0),
            background: Color::Rgb(1, 1, 1),
        };
        println!("no override set: eg_1: {0}", eg_1);

        let eg_2 = FormattedString {
            text: "World",
            foreground: Color::Ansi256(150),
            background: Color::Rgb(1, 1, 1),
        };
        println!("no override set: eg_2: {0}", eg_2);
    }
}
