# Elegant Constructor DSL Pattern in Rust

A pedagogical guide and reference implementation of the **Elegant Constructor DSL
Pattern** in Rust.

---

## 1. Motivation: The Constructor Problem in Rust

Constructing structs that accept multiple optional or unordered configuration values in
Rust often presents ergonomic challenges. Consider an `EditorBuffer` that optionally
accepts a file extension (for syntax highlighting) and a file path:

### Anti-Pattern 1: Positional `Option` Soup

```rust,ignore
let buffer = EditorBuffer::new(Some("md"), None);
let buffer = EditorBuffer::new(None, Some("src/main.rs"));
let buffer = EditorBuffer::new(None, None);
```

**Problems**:

- High cognitive load: Callers must remember which position corresponds to which argument.
- Stringly-typed confusion: Both parameters are `Option<&str>`, making accidental swaps
  silent bugs.

### Anti-Pattern 2: The Heavyweight Builder Pattern

```rust,ignore
let buffer = EditorBufferBuilder::default()
    .file_extension("md")
    .file_path("src/main.rs")
    .build();
```

**Problems**:

- High ceremony: Requires creating and maintaining separate builder structs, methods, and
  error-handling paths for simple data structures.
- Clunky for 1 or 2 parameters: Constructing a default buffer or one with a single
  property requires 3 to 4 lines of boilerplate.

---

## 2. The Solution: Elegant Constructor DSL

The **Elegant Constructor DSL Pattern** combines `impl Into<Config>` with operator
overloading (`Add`, `AddAssign`) on lightweight tokens to provide:

- **Progressive disclosure**: Callers provide only what they care about (`()`, a single
  token, or a combined expression).
- **Disambiguation at compile time**: Newtype/unit tokens prevent parameter swap bugs.
- **Order independence**: `TokenA + TokenB` and `TokenB + TokenA` produce identical
  results.
- **Zero runtime overhead**: Compiles directly into simple struct instantiations.

```rust,ignore
// 1. Default (no options)
let buffer = EditorBuffer::new_empty(());

// 2. Single option (file extension only)
let buffer = EditorBuffer::new_empty(FileExtensionToken("md"));

// 3. Single option (file path only)
let buffer = EditorBuffer::new_empty(FilePathToken("src/main.rs"));

// 4. Multiple options in any order using `+`
let buffer = EditorBuffer::new_empty(FileExtensionToken("rs") + FilePathToken("src/main.rs"));
let buffer = EditorBuffer::new_empty(FilePathToken("src/main.rs") + FileExtensionToken("rs"));
```

---

## 3. The 3-Tier Architecture

The pattern separates concerns across three distinct layers:

```
+-------------------------------------------------------------+
| Tier 1: Constructor DSL Tokens (Call Site Ergonomics)       |
| FileExtensionToken(&str), FilePathToken(&str), Bold, Italic |
+-------------------------------------------------------------+
                              |
                              v (via From / Add)
+-------------------------------------------------------------+
| Tier 2: Canonical Config Struct (Intermediate Aggregator)   |
| EditorBufferConfig<'a>, StyleConfig                         |
| Stores native types (Option<&'a str>, Option<T>)            |
+-------------------------------------------------------------+
                              |
                              v (via into())
+-------------------------------------------------------------+
| Tier 3: Target Storage Struct (Owned Final Data)            |
| EditorBuffer, Style                                         |
+-------------------------------------------------------------+
```

1. **Tier 1: Constructor DSL Tokens**: Single-purpose newtypes (e.g.
   `FileExtensionToken(&'a str)`) or unit structs (e.g. `Bold`, `Italic`). They exist
   solely to disambiguate parameters and enable operator overloading at call sites.
2. **Tier 2: Canonical Config Struct**: A transient intermediate aggregator (e.g.
   `EditorBufferConfig<'a>`). It stores raw `Option<&'a str>` values rather than wrapping
   the tokens, keeping memory layout flat and field access direct.
3. **Tier 3: Target Storage Struct**: The final data structure (e.g. `EditorBuffer`) that
   owns its storage fields (`Option<String>`). Its constructor takes `impl Into<Config>`.

---

## 4. Algebraic Mechanics of the `+` Operator

The exact `Add` implementations required depend on whether the DSL supports **2 tokens**
or **3+ tokens**.

### The 2-Token Model (Minimal & YAGNI)

When an entity only ever supports 2 tokens (such as `FileExtensionToken` and
`FilePathToken`):

- Callers can at most combine two tokens: `TokenA + TokenB` or `TokenB + TokenA`.
- Under the hood, both evaluate to `Config::from(a) + Config::from(b)`.
- This requires only:
    1. `From<()>` for Config (unit default)
    2. `From<TokenA>` and `From<TokenB>` for Config
    3. `Add<TokenB> for TokenA` and `Add<TokenA> for TokenB`
    4. `Add<Config> for Config` (Config + Config merge)
- **Why `Config + Token` and `Token + Config` are not needed**: There is no 3rd token to
  chain. Implementing `Config + Token` or `Token + Config` would be dead code. Adhering to
  YAGNI keeps the codebase lean.

See [`src/two_tokens.rs`](src/two_tokens.rs) for the complete implementation.

---

### The 3+ Token Model (Full Algebraic Completeness)

When an entity supports 3 or more tokens (such as `Bold`, `Italic`, and `Dim`):

- Callers can chain 3 or more tokens in a single expression:
    ```rust,ignore
    let style = Bold + Italic + Dim;
    ```
- Because Rust's `+` operator is **left-associative**, Rust parses this as:
    ```rust,ignore
    ((Bold + Italic) + Dim)
    ```
- Stage 1: `(Bold + Italic)` executes `Add<Italic> for Bold`, producing a `StyleConfig`.
- Stage 2: `(StyleConfig) + Dim` now has `StyleConfig` on the left-hand side. This
  **requires** `Add<Dim> for StyleConfig` (`Config + Token`).
- If parenthesized on the right:
    ```rust,ignore
    Bold + (Italic + Dim)
    ```
    `(Italic + Dim)` produces a `StyleConfig`, resulting in `Bold + StyleConfig`, which
    **requires** `Add<StyleConfig> for Bold` (`Token + Config`).
- Adding `AddAssign` (`+=`) enables convenient in-place mutation on config instances.

See [`src/three_tokens.rs`](src/three_tokens.rs) for the complete implementation.

---

## 5. Real-World Case Studies in `r3bl-open-core`

The `r3bl-open-core` project applies this pattern across multiple domains:

| Module               | Location                                                   | Token Count     | Approach                                                                       |
| :------------------- | :--------------------------------------------------------- | :-------------- | :----------------------------------------------------------------------------- |
| `EditorBufferConfig` | `tui/src/tui/editor/editor_buffer/buffer_config_struct.rs` | 2 Tokens        | Minimal 2-token model (`From` + `Token+Token` + `Config+Config`)               |
| `TuiStyleAttribs`    | `tui/src/core/tui_style/tui_style_attribs.rs`              | 9 Tokens        | Full algebraic completeness via declarative macro for all 9 unit struct tokens |
| `PtySessionConfig`   | `tui/src/core/pty/pty_session/pty_session_builder.rs`      | 5 Enum Variants | Single-enum token model with `Config + Option` chaining                        |
| `TracingConfig`      | `tui/src/core/log/log_public_api.rs`                       | 3 Input Types   | Minimal `From<Token>` + `Config + Config` model                                |

---

## 6. Running Tests

To run the test suite for this crate:

```bash
cargo test
```
