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

/// Rust enforces a set of rules that govern how references are used via **lifetimes**.
///
/// Lifetimes are named regions of code that a reference must be valid for.
///
/// - For simple programs, lifetimes coincide with lexical scope.
/// - Those regions may be fairly complex, as they correspond to paths of execution in the
///   program.
/// - There may even be holes in these paths of execution, as it's possible to invalidate
///   a reference as long as it's reinitialized before it's used again.
/// - Types which contain references (or pretend to) may also be tagged with lifetimes so
///   that Rust can prevent them from being invalidated as well.
///
/// Inside a function, Rust doesn't let you explicitly name lifetimes. And each let
/// statement implicitly introduces a scope. However, once you cross the function
/// boundary, you need to start talking about lifetimes.
/// 
/// More info:
/// - <https://doc.rust-lang.org/nomicon/lifetimes.html#the-area-covered-by-a-lifetime>
/// - <https://doc.rust-lang.org/nomicon/lifetime-mismatch.html>
#[rustfmt::skip]
#[test]
fn ex_3_lifetimes_1() {
    /// 'fn is          <  'input.
    /// 'fn needs to be >= 'input.
    ///
    /// - 'fn is the lifetime of the referent. It is short.
    /// - 'input is the lifetime of the reference. It is long.
    fn try_to_make_reference_outlive_referent<'input>(param: &'input usize) -> &'input str {
        // 'fn: {
            let referent = format!("{}", param);
            let reference = &/*'fn*/referent;
            // return reference; /* 🧨 does not compile! */
            unreachable!()
        // }
    }

    fn fix_try_to_make_reference_outlive_referent<'input>(param: &'input usize) -> &'input str {
        // The string literals "zero", "one", and "many" are stored in a special section
        // of memory that is accessible throughout the entire program execution. This
        // means that these string literals are available for the entire duration of the
        // program, hence they have the 'static lifetime.
        match param {
            0 => /* &'static */ "zero",
            1 => /* &'static */ "one",
            _ => /* &'static */ "many",
        }
    }

    assert_eq!(fix_try_to_make_reference_outlive_referent(&0), "zero");
}

#[rustfmt::skip]
#[test]
fn ex_3_lifetimes_2() {
    /// Rust doesn't understand that `ref_to_first_item` is a reference to a subpath of
    /// `data`. It doesn't understand [`Vec`] at all. 🤯
    ///
    /// Here's what it sees:
    /// - `ref_to_first_item` which is `&'first data` has to live for `'first` in order to
    ///   be printed.
    /// - When we try to call push, it then sees us try to make an `&'second mut data`.
    /// - It knows that `'second` is contained within `'first`, and rejects our program
    ///   because the `&'first data` must still be alive! And we can't alias a **mutable
    ///   reference**.
    ///
    /// The lifetime system is much more coarse than the reference semantics we're
    /// actually interested in preserving.
    fn try_to_modify_referent() {
        let mut data = vec![1, 2, 3]; /* referent */
        // 'first: {
            let ref_to_first_item = &/*'first*/data[0]; /* reference */
            //   'second: {
            //        Vec::push(&/*'second*/mut data, 4); /* 🧨 won't compile */
            //    }
            println!("first_item: {}", ref_to_first_item); /* reference still in scope */
        // }
        // drop(ref_to_first_item);
    }
}
