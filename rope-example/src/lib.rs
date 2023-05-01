/// [      CHUNK (first)    ,     CHUNK     ,    CHUNK (last)       ] : ROPE
///     <-- 1..2 * BASE -->                   <-- 0..2 * BASE-->
///         (<= BASE * 2)
pub struct Rope {
    chunks: Vec<String>,
}

const BASE: usize = 4;

impl Rope {
    pub fn push_str(&mut self, text: &str) {
        // Early return if the entire rope need to be re-chunked to fit the text.
        if helpers::try_fit_input_into_first_chunk_and_rechunk_rope_if_it_doesnt(self, text) {
            return;
        }

        // Early return if the last chunk (which can also be the first chunk) can fit the text.
        if helpers::try_fit_input_into_last_chunk(self, text) {
            return;
        }

        helpers::chunk_text_into_rope(self, text);
    }

    pub fn append(&mut self, other: &Self) {
        let lhs_chunks = &self.chunks;
        let rhs_chunks = &other.chunks;
        let new_text = format!("{}{}", lhs_chunks.join(""), rhs_chunks.join(""));
        let mut new_rope = Rope { chunks: vec![] };
        new_rope.push_str(&new_text);
        self.chunks = new_rope.chunks;
    }
}

pub mod helpers {
    use super::*;

    pub fn try_fit_input_into_first_chunk_and_rechunk_rope_if_it_doesnt(
        rope: &mut Rope,
        text: &str,
    ) -> bool {
        let first_chunk = rope.chunks.first_mut();
        if let Some(first_chunk) = first_chunk {
            let space_avail_in_first_chunk = first_chunk.len() < BASE * 2;
            let text_wont_fit_in_first_chunk = first_chunk.len() + text.len() > BASE * 2;

            if space_avail_in_first_chunk && text_wont_fit_in_first_chunk {
                // Make a new rope w/ the first chunk + the input.
                let new_text = format!("{}{}", first_chunk, text);
                let mut new_rope = Rope { chunks: vec![] };
                chunk_text_into_rope(&mut new_rope, &new_text);
                rope.chunks = new_rope.chunks;
                return true;
            }
        }
        false
    }

    pub fn chunk_text_into_rope(rope: &mut Rope, text: &str) {
        let mut input = text;
        while !input.is_empty() {
            // Try to fit the input into a chunk.
            let (remainder, chunk) = helpers::try_fit_given_input_into_double_base(input);
            rope.chunks.push(chunk.to_string());
            input = remainder;
        }
    }

    pub fn try_fit_input_into_last_chunk(rope: &mut Rope, input: &str) -> bool {
        let last_chunk = rope.chunks.last_mut();
        if let Some(last_chunk) = last_chunk {
            if (last_chunk.len() + input.len()) <= BASE * 2 {
                last_chunk.push_str(input);
                return true;
            }
        }
        false
    }

    pub fn try_fit_given_input_into_double_base(
        input: &str,
    ) -> (/* remainder */ &str, /* output */ &str) {
        let max_per_chunk = BASE * 2;

        if input.len() <= max_per_chunk {
            return ("", input);
        }

        let (chunk, remainder) = input.split_at(max_per_chunk);

        (remainder, chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helpers::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_helper_fn() {
        // happy path.
        {
            let text = "aaaabbbb";
            let (remainder, output_chunk) = try_fit_given_input_into_double_base(text);
            assert_eq!(remainder, "");
            assert_eq!(output_chunk, "aaaabbbb");
        }

        // happy path.
        {
            let text = "aaaabbbbc";
            let (remainder, output_chunk) = try_fit_given_input_into_double_base(text);
            assert_eq!(remainder, "c");
            assert_eq!(output_chunk, "aaaabbbb");
        }

        // unhappy path.
        {
            let text = "aa";
            let (remainder, output_chunk) = try_fit_given_input_into_double_base(text);
            assert_eq!(remainder, "");
            assert_eq!(output_chunk, "aa");
        }
    }

    #[test]
    fn insert_one_chunk_at_end_empty_rope() {
        // Chunk size is 8. Works the same for size [0..=8].
        {
            let mut rope = Rope { chunks: vec![] };
            rope.push_str("aaaabbbb");

            assert_eq!(rope.chunks, vec!["aaaabbbb"]);
        }

        // Chunk size is 9.
        {
            let mut rope = Rope { chunks: vec![] };
            rope.push_str("aaaabbbbc");
            assert_eq!(rope.chunks, vec!["aaaabbbb", "c"]);
        }
    }

    #[test]
    fn insert_one_chunk_at_end_rope_with_1_chunk() {
        // happy path
        {
            let mut rope = Rope {
                chunks: vec!["333".to_string()],
            };
            rope.push_str("aaaa");
            assert_eq!(rope.chunks, vec!["333aaaa"]);
        }

        // unhappy path
        {
            let mut rope = Rope {
                chunks: vec!["333".to_string()],
            };
            rope.push_str("aaaabbbbc");
            // "333a aaab bbbc"
            //  0..3
            assert_eq!(rope.chunks, vec!["333aaaab", "bbbc"]);
        }
    }

    #[test]
    fn insert_one_chunk_at_end_rope_with_2_chunks() {}
}
