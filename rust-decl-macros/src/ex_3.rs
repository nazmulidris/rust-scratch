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

#[macro_export]
macro_rules! join_fmt {
    (
        write_into: $acc:expr,
        delim: $sep:expr,
        items: $collection:expr,
        each: $item:ident,
        format_item: $($format:tt)*
    ) => {{
        use std::fmt::Write as _;

        let mut iter = $collection.iter();
        // First item.
        if let Some($item) = iter.next() {
            _ = write!($acc, $($format)*);
        }

        // Remaining items.
        for $item in iter {
            _ = write!($acc, "{}", $sep);
            _ = write!($acc, $($format)*);
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn custom_item_format() {
        let items = ["one", "two", "three"];
        let mut acc = String::new();
        join_fmt!(
            write_into: acc,
            delim: "\n",
            items: items,
            each: item,
            format_item: "> '{item}' <"
        );

        println!("{}", acc);
    }

    #[test]
    fn expected_syn() {
        let items = ["one", "two", "three"];
        let expected = items.join(", ");

        let mut acc = String::new();

        join_fmt!(
            write_into: acc,
            delim: ", ",
            items: items,
            each: item,
            format_item: "{item}"
        );

        println!("{:#?}", acc);
        assert_eq!(acc, expected);
    }
}
