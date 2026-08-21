// Copyright (c) 2026 R3BL LLC. Licensed under Apache License, Version 2.0.

//! # Method Overloading in Rust via Generic Trait Parameterization
//!
//! This crate demonstrates:
//! 1. Why Rust does not allow duplicate inherent methods on structs.
//! 2. How multiple traits with identical method names can be implemented on a single struct.
//! 3. How to resolve trait name collisions using Fully Qualified Syntax.
//! 4. How to emulate true compile-time method overloading using Generic Trait Parameterization
//!    (Ad-hoc Polymorphism).

// -----------------------------------------------------------------------------------------
// Section 1 & 2: BlogPost, HtmlExport, and JsonExport
// -----------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlogPost {
    pub title: String,
    pub body: String,
    pub categories: Vec<String>,
}

impl BlogPost {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        categories: Vec<String>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            categories,
        }
    }
}

pub trait HtmlExport {
    fn export(&self) -> String;
}

pub trait JsonExport {
    fn export(&self) -> String;
}

impl HtmlExport for BlogPost {
    fn export(&self) -> String {
        let cats = self.categories.join(", ");
        format!(
            "<article><h1>{}</h1><p>{}</p><footer>Categories: {}</footer></article>",
            self.title, self.body, cats
        )
    }
}

impl JsonExport for BlogPost {
    fn export(&self) -> String {
        let cats = self
            .categories
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

// -----------------------------------------------------------------------------------------
// Section 3: Generic Trait Parameterization (Method Overloading on BlogPost)
// -----------------------------------------------------------------------------------------

/// Extension trait enabling overloaded `.append(...)` on [`BlogPost`].
pub trait AppendExt<Item> {
    fn append(&mut self, item: Item);
}

/// Overload 1: Append text slice to the blog post body.
impl AppendExt<&str> for BlogPost {
    fn append(&mut self, text: &str) {
        self.body.push_str(text);
    }
}

/// Overload 2: Append a single category tag (`String`) to categories.
impl AppendExt<String> for BlogPost {
    fn append(&mut self, category: String) {
        self.categories.push(category);
    }
}

/// Overload 3: Append a list of category tags (`Vec<String>`) to categories.
impl AppendExt<Vec<String>> for BlogPost {
    fn append(&mut self, mut categories: Vec<String>) {
        self.categories.append(&mut categories);
    }
}

// -----------------------------------------------------------------------------------------
// Section 4: Query Overloading on BlogStore
// -----------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slug<'a>(pub &'a str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category<'a>(pub &'a str);

#[derive(Debug, Default)]
pub struct BlogStore {
    posts: Vec<(u64, String, BlogPost)>, // (id, slug, post)
}

impl BlogStore {
    pub fn add(&mut self, id: u64, slug: impl Into<String>, post: BlogPost) {
        self.posts.push((id, slug.into(), post));
    }
}

/// Generic search extension trait overloaded across different query key types.
pub trait FindExt<'a, Key> {
    type Output;
    fn find(&'a self, key: Key) -> Self::Output;
}

/// Overload 1: Find post by unique `PostId`.
impl<'a> FindExt<'a, PostId> for BlogStore {
    type Output = Option<&'a BlogPost>;

    fn find(&'a self, key: PostId) -> Self::Output {
        self.posts
            .iter()
            .find(|(id, _, _)| *id == key.0)
            .map(|(_, _, post)| post)
    }
}

/// Overload 2: Find post by unique `Slug`.
impl<'a, 'b> FindExt<'a, Slug<'b>> for BlogStore {
    type Output = Option<&'a BlogPost>;

    fn find(&'a self, key: Slug<'b>) -> Self::Output {
        self.posts
            .iter()
            .find(|(_, slug, _)| slug.as_str() == key.0)
            .map(|(_, _, post)| post)
    }
}

/// Overload 3: Find all posts matching a `Category`.
impl<'a, 'b> FindExt<'a, Category<'b>> for BlogStore {
    type Output = Vec<&'a BlogPost>;

    fn find(&'a self, key: Category<'b>) -> Self::Output {
        self.posts
            .iter()
            .filter(|(_, _, post)| post.categories.iter().any(|c| c == key.0))
            .map(|(_, _, post)| post)
            .collect()
    }
}

// -----------------------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_name_collision_and_disambiguation() {
        let post = BlogPost::new(
            "Rust Overloading",
            "Traits allow ad-hoc polymorphism.",
            vec!["rust".into(), "architecture".into()],
        );

        // Disambiguate with fully-qualified syntax:
        let html = HtmlExport::export(&post);
        let json = JsonExport::export(&post);

        assert!(html.starts_with("<article><h1>Rust Overloading</h1>"));
        assert!(json.starts_with(r#"{"title":"Rust Overloading""#));
    }

    #[test]
    fn test_overloaded_append_on_blog_post() {
        let mut post = BlogPost::new("Rust Tips", "Hello", vec!["rust".into()]);

        // Overload 1: &str -> appends to body
        post.append(" world!");
        assert_eq!(post.body, "Hello world!");

        // Overload 2: String -> adds single category
        post.append("guide".to_string());
        assert_eq!(post.categories, vec!["rust", "guide"]);

        // Overload 3: Vec<String> -> adds multiple categories
        post.append(vec!["tui".into(), "patterns".into()]);
        assert_eq!(post.categories, vec!["rust", "guide", "tui", "patterns"]);
    }

    #[test]
    fn test_overloaded_find_on_blog_store() {
        let mut store = BlogStore::default();
        let post1 = BlogPost::new("Post 1", "Body 1", vec!["rust".into()]);
        let post2 = BlogPost::new("Post 2", "Body 2", vec!["rust".into(), "tui".into()]);

        store.add(1, "post-1", post1);
        store.add(2, "post-2", post2);

        // Find by PostId -> Option<&BlogPost>
        let by_id = store.find(PostId(1));
        assert_eq!(by_id.map(|p| p.title.as_str()), Some("Post 1"));

        // Find by Slug -> Option<&BlogPost>
        let by_slug = store.find(Slug("post-2"));
        assert_eq!(by_slug.map(|p| p.title.as_str()), Some("Post 2"));

        // Find by Category -> Vec<&BlogPost>
        let by_cat = store.find(Category("tui"));
        assert_eq!(by_cat.len(), 1);
        assert_eq!(by_cat[0].title, "Post 2");

        let all_rust = store.find(Category("rust"));
        assert_eq!(all_rust.len(), 2);
    }
}
