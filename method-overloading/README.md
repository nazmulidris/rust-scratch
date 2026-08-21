# Emulating Method Overloading in Rust with Generic Trait Parameterization

A pedagogical guide and reference implementation exploring how to achieve clean, type-safe, compile-time **Method Overloading** in Rust using **Generic Trait Parameterization** (Ad-hoc Polymorphism), along with a real-world case study from [R3BL Open Core (`r3bl-open-core`)](https://github.com/r3bl-org/r3bl-open-core).

---

## Table of Contents
- [1. Motivation: The Method Overloading Dilemma in Rust](#1-motivation-the-method-overloading-dilemma-in-rust)
- [2. Trait Mechanics: Identical Method Names on the Same Struct](#2-trait-mechanics-identical-method-names-on-the-same-struct)
  - [The Setup: `BlogPost` with `HtmlExport` and `JsonExport`](#the-setup-blogpost-with-htmlexport-and-jsonexport)
  - [Ambiguity Resolution: Fully Qualified Syntax](#ambiguity-resolution-fully-qualified-syntax)
- [3. The Pattern: Generic Trait Parameterization (Ad-hoc Polymorphism)](#3-the-pattern-generic-trait-parameterization-ad-hoc-polymorphism)
  - [Example A: Overloaded Mutators (`BlogPost::append`)](#example-a-overloaded-mutators-blogpostappend)
  - [Example B: Overloaded Queries (`BlogStore::find`)](#example-b-overloaded-queries-blogstorefind)
- [4. Zero-Cost Abstraction & Performance](#4-zero-cost-abstraction--performance)
- [5. Real-World Production Case Study: R3BL Open Core (`canvas_camera_ext.rs`)](#5-real-world-production-case-study-r3bl-open-core-canvas_camera_extrs)
  - [The Coordinate Panning Problem](#the-coordinate-panning-problem)
  - [The `CanvasCameraExt<InputCoord>` Solution](#the-canvascameraextinputcoord-solution)
- [6. Design Guidelines: When to Use & When to Avoid](#6-design-guidelines-when-to-use--when-to-avoid)
- [7. Running the Companion Code](#7-running-the-companion-code)

---

## 1. Motivation: The Method Overloading Dilemma in Rust

Developers coming from languages like Java, C++, or TypeScript are accustomed to **traditional method overloading**, where multiple methods share the same name but differ in their parameter lists:

```typescript
// Traditional method overloading (TypeScript / Java / C++)
class BlogPost {
    append(text: string): void;
    append(categories: string[]): void;
}
```

In Rust, this is **strictly forbidden** in inherent `impl` blocks. If you attempt to define multiple methods with the same name on a struct, the Rust compiler immediately rejects the code with error `E0592`:

```rust,compile_fail
struct BlogPost {
    title: String,
    body: String,
    categories: Vec<String>,
}

impl BlogPost {
    // ❌ COMPILE ERROR (E0592: duplicate definitions with name `append`)
    pub fn append(&mut self, text: &str) {
        self.body.push_str(text);
    }

    pub fn append(&mut self, category: String) {
        self.categories.push(category);
    }
}
```

### Why Does Rust Disallow Inherent Method Overloading?
1. **Explicit Name Resolution**: Rust avoids the cognitive ambiguity and complex overload resolution rules found in C++ (e.g., implicit type conversions, numeric promotions, ranking rules).
2. **Predictable Type Inference**: Rust's Hindley-Milner type inference engine relies on deterministic method names to infer types bidirectionally without exponential backtracking.

However, splitting methods into awkward, verbose names like `append_text`, `append_category_str`, `append_category_vec` degrades API ergonomics. Fortunately, Rust provides a powerful, type-safe alternative: **Generic Trait Parameterization**.

---

## 2. Trait Mechanics: Identical Method Names on the Same Struct

Before exploring generic parameterization, it is important to understand how Rust handles method name collisions across traits.

### The Setup: `BlogPost` with `HtmlExport` and `JsonExport`

In Rust, a struct **can implement multiple traits that share the exact same method signature**. Consider our `BlogPost` struct implementing two separate exporting traits:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlogPost {
    pub title: String,
    pub body: String,
    pub categories: Vec<String>,
}

pub trait HtmlExport {
    fn export(&self) -> String;
}

pub trait JsonExport {
    fn export(&self) -> String;
}

// ✅ Valid Rust: Both traits implement `fn export(&self) -> String`
impl HtmlExport for BlogPost {
    fn export(&self) -> String {
        format!(
            "<article><h1>{}</h1><p>{}</p><footer>Categories: {}</footer></article>",
            self.title,
            self.body,
            self.categories.join(", ")
        )
    }
}

impl JsonExport for BlogPost {
    fn export(&self) -> String {
        let cats = self.categories
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"title":"{}","body":"{}","categories":[{}]}}"#,
            self.title, self.body, cats
        )
    }
}
```

### Ambiguity Resolution: Fully Qualified Syntax

How does Rust resolve `.export()` when calling it on a `BlogPost`?

1. **Single Trait in Scope**: If only `use HtmlExport;` is in scope, `post.export()` unambiguously invokes `HtmlExport::export`.
2. **Multiple Traits in Scope**: If both `use HtmlExport;` and `use JsonExport;` are in scope, `post.export()` produces compiler error `E0034: multiple applicable items in scope`.

To resolve this ambiguity, Rust provides **Fully Qualified Syntax** (Universal Function Call Syntax):

```rust
let post = BlogPost {
    title: "Rust Overloading".into(),
    body: "Traits enable ad-hoc polymorphism.".into(),
    categories: vec!["rust".into(), "architecture".into()],
};

// Disambiguate using trait namespace or fully qualified syntax:
let html = HtmlExport::export(&post);
let json = JsonExport::export(&post);

// Or the fully explicit turbofish syntax:
let html_explicit = <BlogPost as HtmlExport>::export(&post);
let json_explicit = <BlogPost as JsonExport>::export(&post);
```

---

## 3. The Pattern: Generic Trait Parameterization (Ad-hoc Polymorphism)

By parameterizing a trait over an input type parameter (`trait MyTrait<Input>`), we can implement the trait multiple times for the **same struct** across different input types.

Because each implementation takes a distinct argument type, **Rust's compiler automatically selects the correct implementation based on the argument passed in at the call site**. No fully qualified syntax is needed!

```
+---------------------------------------------------------------+
| Call Site: post.append(argument)                              |
+---------------------------------------------------------------+
       |                                     |
       v (arg is &str)                       v (arg is Vec<String>)
+-------------------------------+     +---------------------------------+
| impl AppendExt<&str>          |     | impl AppendExt<Vec<String>>     |
| for BlogPost                  |     | for BlogPost                    |
| -> Appends to post.body       |     | -> Appends to post.categories   |
+-------------------------------+     +---------------------------------+
```

### Example A: Overloaded Mutators (`BlogPost::append`)

Here, we define a generic extension trait `AppendExt<Item>`:

```rust
/// Extension trait enabling overloaded `.append(...)` on [`BlogPost`].
pub trait AppendExt<Item> {
    fn append(&mut self, item: Item);
}

/// Overload 1: Passing `&str` appends text to the blog post body.
impl AppendExt<&str> for BlogPost {
    fn append(&mut self, text: &str) {
        self.body.push_str(text);
    }
}

/// Overload 2: Passing `String` appends a single category tag.
impl AppendExt<String> for BlogPost {
    fn append(&mut self, category: String) {
        self.categories.push(category);
    }
}

/// Overload 3: Passing `Vec<String>` appends multiple category tags.
impl AppendExt<Vec<String>> for BlogPost {
    fn append(&mut self, mut categories: Vec<String>) {
        self.categories.append(&mut categories);
    }
}
```

#### The Clean Call Site Experience:
```rust
let mut post = BlogPost::new("Rust Tips", "Hello", vec!["rust".into()]);

// 1. Appends &str to post.body
post.append(" world!");
assert_eq!(post.body, "Hello world!");

// 2. Appends String to post.categories
post.append("guide".to_string());
assert_eq!(post.categories, vec!["rust", "guide"]);

// 3. Appends Vec<String> to post.categories
post.append(vec!["tui".into(), "patterns".into()]);
assert_eq!(post.categories, vec!["rust", "guide", "tui", "patterns"]);
```

---

### Example B: Overloaded Queries (`BlogStore::find`)

We can also use newtype wrappers (Domain Primitive Pattern) to overload queries returning different result types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slug<'a>(pub &'a str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category<'a>(pub &'a str);

pub struct BlogStore {
    posts: Vec<(u64, String, BlogPost)>, // (id, slug, post)
}

/// Generic search trait overloaded by query key type.
pub trait FindExt<'a, Key> {
    type Output;
    fn find(&'a self, key: Key) -> Self::Output;
}

// Overload 1: Find by PostId -> Option<&'a BlogPost>
impl<'a> FindExt<'a, PostId> for BlogStore {
    type Output = Option<&'a BlogPost>;

    fn find(&'a self, key: PostId) -> Self::Output {
        self.posts.iter()
            .find(|(id, _, _)| *id == key.0)
            .map(|(_, _, post)| post)
    }
}

// Overload 2: Find by Slug -> Option<&'a BlogPost>
impl<'a, 'b> FindExt<'a, Slug<'b>> for BlogStore {
    type Output = Option<&'a BlogPost>;

    fn find(&'a self, key: Slug<'b>) -> Self::Output {
        self.posts.iter()
            .find(|(_, slug, _)| slug.as_str() == key.0)
            .map(|(_, _, post)| post)
    }
}

// Overload 3: Find by Category -> Vec<&'a BlogPost>
impl<'a, 'b> FindExt<'a, Category<'b>> for BlogStore {
    type Output = Vec<&'a BlogPost>;

    fn find(&'a self, key: Category<'b>) -> Self::Output {
        self.posts.iter()
            .filter(|(_, _, post)| post.categories.iter().any(|c| c == key.0))
            .map(|(_, _, post)| post)
            .collect()
    }
}
```

#### Query Call Sites:
```rust
let store = BlogStore::default();

// Lookup by ID:
let by_id: Option<&BlogPost> = store.find(PostId(1));

// Lookup by Slug:
let by_slug: Option<&BlogPost> = store.find(Slug("rust-overloading"));

// Lookup by Category (returns collection):
let by_cat: Vec<&BlogPost> = store.find(Category("rust"));
```

---

## 4. Zero-Cost Abstraction & Performance

A common concern when introducing traits is runtime overhead. In Rust, Generic Trait Parameterization incurs **zero runtime cost**:

- **Static Dispatch (Monomorphization)**: The Rust compiler analyzes the concrete type at each call site during compilation and generates direct function calls to the specific implementation.
- **No Vtables / Heap Allocations**: Unlike trait objects (`dyn Trait`), there are no vtables, indirection pointers, or runtime dynamic dispatch.
- **Inlining**: The compiler can easily inline the monomorphized method bodies directly into the calling functions.

---

## 5. Real-World Production Case Study: R3BL Open Core (`canvas_camera_ext.rs`)

In the open-source terminal framework [R3BL Open Core (`r3bl-open-core`)](https://github.com/r3bl-org/r3bl-open-core), this exact pattern is used in [`canvas_camera_ext.rs`](https://github.com/r3bl-org/r3bl-open-core/blob/main/tui/src/core/coordinates/canvas/canvas_camera_ext.rs) for camera viewport operations.

### The Coordinate Panning Problem
A terminal `Viewport` represents a 2D camera looking onto a 2D document canvas. To keep text or cursors in view, the viewport needs to:
1. **Pan horizontally**: When scrolling across columns (`CCol` -> `VPCol`).
2. **Pan vertically**: When scrolling down rows (`CRow` -> `VPRow`).
3. **Pan bidirectionally**: When positioning on 2D coordinates (`CPos` -> `VPPos`).

Without method overloading, `Viewport` would require 6 clunky methods:
- `viewport.pan_row_to_keep_coord_in_view(row)`
- `viewport.pan_col_to_keep_coord_in_view(col)`
- `viewport.pan_pos_to_keep_coord_in_view(pos)`
- `viewport.to_vp_row(row)`
- `viewport.to_vp_col(col)`
- `viewport.to_vp_pos(pos)`

### The `CanvasCameraExt<InputCoord>` Solution

Instead, `r3bl_tui` parameterizes `CanvasCameraExt` over the input coordinate type:

```rust
pub trait CanvasCameraExt<InputCoord> {
    type OutputCoord;

    /// Pan the viewport to ensure the given coordinate is visible on screen.
    fn pan_to_keep_coord_in_view(&mut self, coord: InputCoord);

    /// Convert canvas coordinate into viewport space.
    fn to_vp(&self, coord: InputCoord) -> Self::OutputCoord;
}
```

Separate implementations are provided for `CRow`, `CCol`, and `CPos`:

```rust
// 1. 1D Horizontal Camera Panning
impl CanvasCameraExt<CCol> for Viewport {
    type OutputCoord = VPCol;
    fn pan_to_keep_coord_in_view(&mut self, coord: CCol) { /* horizontal pan */ }
    fn to_vp(&self, coord: CCol) -> VPCol { /* column projection */ }
}

// 2. 1D Vertical Camera Panning
impl CanvasCameraExt<CRow> for Viewport {
    type OutputCoord = VPRow;
    fn pan_to_keep_coord_in_view(&mut self, coord: CRow) { /* vertical pan */ }
    fn to_vp(&self, coord: CRow) -> VPRow { /* row projection */ }
}

// 3. 2D Bidirectional Camera Panning
impl CanvasCameraExt<CPos> for Viewport {
    type OutputCoord = VPPos;
    fn pan_to_keep_coord_in_view(&mut self, coord: CPos) { /* 2D pan */ }
    fn to_vp(&self, coord: CPos) -> VPPos { /* 2D projection */ }
}
```

#### Production Call Sites in `r3bl_tui`:

```rust
use r3bl_tui::{
    CanvasCameraExt, Viewport, VPSize, c_col, c_pos, c_row,
    vp_col, vp_height, vp_pos, vp_row, vp_width,
};

let mut viewport = Viewport::new(c_pos(0, 0), VPSize::new((vp_width(80), vp_height(24))));

// 1D Horizontal Panning (CCol -> VPCol)
let target_col = c_col(10);
viewport.pan_to_keep_coord_in_view(target_col);
let res_col = viewport.to_vp(target_col);
assert_eq!(res_col, vp_col(10));

// 1D Vertical Panning (CRow -> VPRow)
let target_row = c_row(5);
viewport.pan_to_keep_coord_in_view(target_row);
let res_row = viewport.to_vp(target_row);
assert_eq!(res_row, vp_row(5));

// 2D Position Panning (CPos -> VPPos)
let target_pos = c_pos(15, 10);
viewport.pan_to_keep_coord_in_view(target_pos);
let res_pos = viewport.to_vp(target_pos);
assert_eq!(res_pos, vp_pos(15, 10));
```

---

## 6. Design Guidelines: When to Use & When to Avoid

### When to Use This Pattern
1. **Polymorphic Domain Primitives**: When an operation is semantically identical across different domain representations (e.g., 1D vs 2D coordinates, units of measurement, data payload types).
2. **Unified Progressive Disclosure**: When you want to present a single, clean entry point (`.draw()`, `.append()`, `.find()`, `.pan_to_keep_coord_in_view()`) without forcing callers to memorize distinct function names.
3. **Zero Runtime Overhead**: When you require compile-time monomorphization and cannot tolerate dynamic dispatch (`dyn Trait`) vtables.

### When to Avoid This Pattern
1. **Fundamentally Different Semantics**: If two operations have different side effects (e.g., `save_to_disk` vs `save_to_database`), use distinct method names to make the behavior explicit.
2. **Over-Engineering Simple Types**: If an operation only ever takes one type, an inherent method is simpler and requires no `use Trait;` import.

---

## 7. Running the Companion Code

The companion code includes a complete test suite verifying all code examples in this guide.

```bash
# Navigate to project directory
cd method-overloading

# Run all unit and integration tests
cargo test
```
