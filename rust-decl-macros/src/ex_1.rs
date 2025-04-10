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

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct R(pub u8);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct G(pub u8);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct B(pub u8);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Color(pub R, pub G, pub B);

#[macro_export]
macro_rules! color {
    (pink) => {{
        use $crate::{B, Color, G, R};
        Color(R(195), G(106), B(138))
    }};

    (lizard_green) => {{
        use $crate::{B, Color, G, R};
        Color(R(20), G(244), B(0))
    }};

    ($r:expr, $g:expr, $b:expr) => {{
        use $crate::{B, Color, G, R};
        Color($r, $g, $b)
    }};
}

#[test]
fn newtypes_works() {
    let r = R(1);
    let g = G(2);
    let b = B(3);
    assert_eq!(r.0, 1);
    assert_eq!(g.0, 2);
    assert_eq!(b.0, 3);
}

#[test]
fn expected_syntax_rgb() {
    {
        let r /* ident */ = R(1);
        let b /* ident */  = B(3);

        let c = color!(r, G(2) /* expr */, b);
        assert_eq!(c.0, R(1));
        assert_eq!(c.1, G(2));
        assert_eq!(c.2, B(3));
    }
}

#[test]
fn expected_syntax_named_color() {
    {
        let c = color!(pink);
        assert_eq!(c.0, R(195));
        assert_eq!(c.1, G(106));
        assert_eq!(c.2, B(138));
    }

    {
        let c = color!(lizard_green);
        assert_eq!(c.0, R(20));
        assert_eq!(c.1, G(244));
        assert_eq!(c.2, B(0));
    }
}
