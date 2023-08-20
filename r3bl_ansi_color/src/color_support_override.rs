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

use std::sync::atomic::{AtomicI8, Ordering};

/// Global [ColorSupport] override.
static mut COLOR_SUPPORT_OVERRIDE_VALUE: AtomicI8 =
    AtomicI8::new(ColorSupportOverride::NotSet as i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupportOverride {
    Ansi256,
    Truecolor,
    NotSet,
}

mod convert_between_color_support_override_and_i8 {
    impl From<i8> for super::ColorSupportOverride {
        fn from(value: i8) -> Self {
            match value {
                1 => super::ColorSupportOverride::Ansi256,
                2 => super::ColorSupportOverride::Truecolor,
                _ => super::ColorSupportOverride::NotSet,
            }
        }
    }

    impl From<super::ColorSupportOverride> for i8 {
        fn from(value: super::ColorSupportOverride) -> Self {
            match value {
                super::ColorSupportOverride::Ansi256 => 1,
                super::ColorSupportOverride::Truecolor => 2,
                _ => -1,
            }
        }
    }
}

pub fn color_support_override_set(value: ColorSupportOverride) {
    unsafe {
        COLOR_SUPPORT_OVERRIDE_VALUE.store(value.into(), Ordering::SeqCst);
    };
}

pub fn color_support_override_get() -> ColorSupportOverride {
    unsafe { COLOR_SUPPORT_OVERRIDE_VALUE.load(Ordering::SeqCst).into() }
}
