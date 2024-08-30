/*
 *   Copyright (c) 2024 Nazmul Idris
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

/// Let's define that `Sub` is a subtype of Super (ie `Sub : Super`).
/// - What this is suggesting to us is that the set of requirements that `Super` defines
///   are completely satisfied by `Sub`.
/// - `Sub` may then have more requirements.
/// - That is, `Sub` > `Super`.
///
/// Replacing this with lifetimes, `'long : 'short` if and only if
/// - `'long` defines a region of code that completely contains `'short`.
/// - `'long` may define a region larger than `'short`, but that still fits our
///   definition.
/// - That is, `'long` > `'short`.
///
/// More info:
/// - <https://doc.rust-lang.org/nomicon/subtyping.html>
#[rustfmt::skip]
#[test]
fn subtyping() {
    /// - Since: `&'a T` is covariant over `'a`, we are allowed to perform subtyping.
    /// - And: `&'static str` is a subtype of `&'short str`.
    /// - And since:
    ///   ```text
    ///   'static : 'short
    ///      ↑       ↑
    ///    Sub     Super
    ///   ```
    ///
    /// |                 | `'a`     | `T` |
    /// |-----------------|----------|-----|
    /// | `&'a T`         | C        | C   |
    /// | `&'a mut T`     | C        | I   |
    fn debug<'a, T: std::fmt::Display + ?Sized>(a: &'a T, b: &'a T) {
        println!("a: {}, b: {}", a, b);
    }

    let hello: &'static str = "hello";

    // 'short {
    {
        let world = "world".to_string();
        debug(
            /*&'static*/ hello,
            &/*'short*/  world
        );
        // Why does this work?
        // 1) `&'static str` : `&'short str`
        //       ↑                ↑
        //     Subtype          Super type
        // 2) `hello` silently downgrades from `&'static str` into `&'short str`
    }
    // }
}

/// More info:
/// - <https://doc.rust-lang.org/nomicon/subtyping.html>
#[rustfmt::skip]
#[test]
fn variance() {
    /// 1. Take a mutable reference and a value and overwrite the **referent** with it.
    /// 2. It clearly says in its signature the referent and the value must be the
    ///    **exact** same type.
    ///
    /// - `&mut T` is invariant over `T`, meaning,
    /// - `&mut &'long T` is **NOT** a subtype of `&'short T`,
    /// - even if
    ///   ```text
    ///   'long : 'short
    ///      ↑       ↑
    ///    Sub     Super
    ///   ```
    ///
    /// |                 | `'a`     | `T` |
    /// |-----------------|----------|-----|
    /// | `&'a T`         | C        | C   |
    /// | `&'a mut T`     | C        | I   |
    fn assign<'a, T>(reference: &'a mut T, value: T) {
        *reference = value;
    }

    let mut hello: &'static str = "hello";

    // 'short {
    {
        let world = "world".to_string();
        /* 🧨 does not compile! Due to invariance, the 2 args are different types! */
        // assign(
        //     &mut/*&'static*/ hello,
        //     &/*'short*/      world
        // );
        // `&mut T` is invariant over `T`, meaning, these are incompatible:
        //
        // 1. 1st arg: `&mut &'static str`, which is `&mut T` where `T = &'static str`.
        // 2. 2nd arg: `&'short str`, and it is expecting `T = &'static str`. This `T`
        //    does not match!
        //
        // This means that:
        // - `&mut &'static str` cannot be a subtype of `&'short str`,
        // - even if `'static` **is** a subtype of `'short`.
    }
    // }
}
