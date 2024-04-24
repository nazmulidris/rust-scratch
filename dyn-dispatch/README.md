# dyn-dispatch

## Related links:

- https://github.com/users/nazmulidris/projects/3/views/1?pane=issue&itemId=60464160
- https://www.youtube.com/watch?v=xcygqF5LVmM&t=1162s
- https://www.youtube.com/watch?v=ZQNbyna2O04
- https://gemini.google.com/app/157980ca7d9b588c

## Flow

When it comes to polymorphism in Rust, which means that you want to be "vague" about what
arguments a function can receive or what values it can return, there are roughly two
approaches:

1. Generics and monomorphization, with static dispatch:
  - Choose concrete types at compile time
  - Example of `Error` trait using `impl` and syntactic desugaring of `impl` (receive)

2. Trait objects (or slices):
  - Choose types at runtime, via pointers to traits
  - `?Sized`
  - dynamic dispatch, vtables,
  - pointer indirection via `Box` or `Arc` (to deal with `?Sized`), and smart pointers
  - Example of adding different concrete types to a vec (via trait pointers)
  - Vtable
    - diagram: https://developerlife.com/assets/rust-container-cheat-sheet.svg
    - info: https://developerlife.com/2022/03/12/rust-redux/#of-things-and-their-managers

## This table provides a roadmap to the code in this repo

| static  | dynamic |
| ------- | ------- |
| receive | receive |
| return  | return  |

## Research links

- Vtable diagram:
  https://developerlife.com/2022/03/12/rust-redux/#of-things-and-their-managers

- Here are 2 videos that show the difference between `dyn T` and `impl T` and also
  explains the whole trait system concisely!
  - https://www.youtube.com/watch?v=ZQNbyna2O04
  - https://www.youtube.com/watch?v=oLuqAG-kGS4

- Here is another video that's great about monomorphization, generics, static dispatch,
  and dynamic dispatch: https://www.youtube.com/watch?v=xcygqF5LVmM&t=1162s

- Here's a good video that reviews `dyn T` and generics:
  https://www.youtube.com/watch?v=3biW5NkNnrk

- Here's a great blog post on `Sized`:
  https://github.com/pretzelhammer/rust-blog/blob/master/posts/sizedness-in-rust.md

- Good book on `dyn` and `Box`:
https://rust-unofficial.github.io/too-many-lists/index.html

- This is the difference between `Arc` and `Box` for the purposes of sizing a dyn trait
  object: https://gemini.google.com/app/157980ca7d9b588c