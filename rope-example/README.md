# Zed Technical Interview

The goal of this technical interview is to build a simplified version of a rope, a data structure we
use in Zed to efficiently represent text.

A rope is a collection of text chunks that respects the following invariants:

- All text chunks must be shorter than `BASE * 2`
- All text chunks, except the last one, must be longer than `BASE`

In `src/lib.rs`, we included the skeleton of a `Rope` data structure.

## Step 1: `Rope::push_str`

The `Rope::push_str` method takes a piece of text as its input and adds it to the end of the current
`Rope` instance without violating the above invariants.

Test drive the implementation, asserting that the new rope has the expected text overall and that no
chunks in the rope violate the invariant. To start with, assume we're working exclusively with ASCII
strings.

## Step 2: `Rope::append` (time permitting)

The `Rope::append` method takes another `Rope` as its input and appends its chunks to the end of the
current `Rope` instance without violating the above invariants.

Test drive the implementation, asserting that the new rope has the expected text overall and that no
chunks in the rope violate the invariant.

## Step 3: Questions

- What's worth considering when selecting a value for `BASE`?
- What if we wanted to support ropes containing arbitrary UTF-8? What would need to change about the
  code we wrote?
