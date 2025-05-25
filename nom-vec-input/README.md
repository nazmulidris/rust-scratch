# nom-vec-input

## Use nom for "non-slice" input

This project explores how we can use something other than slices with nom. For eg, instead
of a `&str`, what if the input is `Vec<String>` or something along those lines.

- Typically you pass in a slice to a nom parser, and it can return slices or owned types.
- So the parser can allocate if it needs to modify slices of the input to generate the
  output.
- Slices are contiguous in nature, so if you want to stitch together 2 disparate slices,
  you will have to allocate.

## Understanding nom and slices

The nom parser is primarily designed to work with slices as input.

1. **Slice-Based Design**: nom is fundamentally built around parsing data from slices -
   particularly `&[u8]` for binary data and `&str` for string data.

2. **Zero-Copy Parsing**: One of nom's key strengths is its zero-copy approach, where
   parsers don't need to own or copy the data they parse. Instead, they operate directly
   on references (slices) of the input.

3. **Input Trait**: While nom's core functionality works with slices, it actually uses an
   abstraction called the `Input` trait. This allows parsers to work on different kinds of
   input data structures, but slices are the most common and natural fit.

4. **Streaming vs. Complete Input**: nom supports both "complete" parsing (where all input
   is available at once as a slice) and "streaming" parsing (where input arrives in
   chunks).

5. **Incremental Parsing**: The way nom parsers return the remaining unparsed input as a
   slice is key to its combinatorial approach, allowing parsers to be chained together.

For example, a typical nom parser function looks like:

```rs
fn my_parser(input: &str) -> IResult<&str, ParsedValue> {
    // Parse something from the input slice
    // Return the remaining input slice and the parsed value
}
```

This signature shows how nom is designed to take a slice, parse some data from it, and
return both the parsed data and the remaining slice - perfectly aligning with the
slice-based approach to parsing.

- `ex_0_0.rs` shows how to work with slices.

A typical example of nom parsing that works with string slice input is in `ex_normal.rs`. 

## Using nom with owned types like Vec

There are two approaches:

1. Implement [`nom::Input`](https://docs.rs/nom/latest/nom/trait.Input.html#tymethod.take)
   on your owned type. You have to provide nom with the ability to "take" chunks out of
   your owned type. This can be really complex, and you should only consider this if you
   absolutely need to.
   - `ex_0_1.rs` shows how to do this.
2. Convert the owned types to slices. 
   - `ex_1.rs` shows how to work with owned collection types converted to slices.
   - `ex_2.rs` shows how to work with more complex owned collection types converted to slices.

## Using complete vs streaming parser

When the data (slice or owned type) is available in a variable, then you use the complete
parsers. The only time you would use streaming parser is if you are loading something over
the network with a big delay. Note that `Iterator` is used with the complete parsers,
since it iterates over data that is already fully available in memory. It does not mean
that it is streaming.
